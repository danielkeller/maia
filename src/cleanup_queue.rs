#![allow(clippy::redundant_allocation)]

use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct CleanupQueue {
    cursor: usize,
    level: u64,
    array: Arc<Vec<QueueEntry>>,
}

#[derive(Debug)]
pub struct Cleanup {
    cursor: usize,
    level: u64,
    array: Arc<Vec<QueueEntry>>,
}

/// Cleans up on drop
#[derive(Debug)]
pub struct CleanupRAII(Cleanup);

struct QueueEntry {
    guard: AtomicU64,
    value: Cell<Option<Arc<dyn Send + Sync>>>,
}

impl std::fmt::Debug for QueueEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueEntry")
            .field("guard", &self.guard)
            .finish_non_exhaustive()
    }
}

// Safety: Access to 'value' is guarded by 'guard'
unsafe impl Sync for QueueEntry {}
impl std::panic::UnwindSafe for QueueEntry {}
impl std::panic::RefUnwindSafe for QueueEntry {}

impl Default for QueueEntry {
    fn default() -> Self {
        Self {
            guard: AtomicU64::new(u64::MAX),
            value: Cell::new(None),
        }
    }
}

impl CleanupQueue {
    fn new_array(capacity: usize) -> Arc<Vec<QueueEntry>> {
        let mut vec = Vec::new();
        vec.resize_with(capacity.max(1), Default::default);
        Arc::new(vec)
    }

    pub fn new(capacity: usize) -> Self {
        Self {
            cursor: 0,
            level: 0,
            array: Self::new_array(capacity),
        }
    }
    pub fn push(&mut self, value: Arc<dyn Send + Sync>) {
        let entry = &self.array[self.cursor];
        // No need to RMW, we're the only writer
        let guard = entry.guard.load(Ordering::Acquire);
        if guard == u64::MAX {
            // Safety: We're the only writer
            entry.value.set(Some(value));
            self.cursor = (self.cursor + 1) % self.array.len();
            entry.guard.store(self.level, Ordering::Release);
        } else {
            // Full
            let mut array = Self::new_array(self.array.len() * 2);
            std::mem::swap(&mut array, &mut self.array);
            self.cursor = 0;
            self.level = 0;
            self.push(array);
            self.push(value);
        }
    }
    /// Create a new object which can clean up everything previously pushed.
    pub fn new_cleanup(&mut self) -> Cleanup {
        let cursor = if self.cursor == 0 {
            self.array.len() - 1
        } else {
            self.cursor - 1
        };
        let result = Cleanup {
            cursor,
            level: self.level,
            array: self.array.clone(),
        };
        self.level += 1;
        result
    }
    /// Prevent any resources from being freed when the CleanupQueue is dropped.
    /// cleanup() can still be called on any outstanding Cleanup, freeing the
    /// corresponding resources.
    pub fn leak(&self) {
        std::mem::forget(self.array.clone())
    }
}

impl<T: Send + Sync + 'static> std::iter::Extend<Arc<T>> for CleanupQueue {
    fn extend<I: IntoIterator<Item = Arc<T>>>(&mut self, iter: I) {
        for elem in iter {
            self.push(elem)
        }
    }
}

impl Cleanup {
    pub fn cleanup(&self) {
        let mut cursor = self.cursor;
        loop {
            let entry = &self.array[cursor];
            let guard = entry.guard.load(Ordering::Relaxed);
            if guard > self.level {
                return; //Done
            }
            // Lock the location.
            if entry
                .guard
                .compare_exchange(
                    guard,
                    u64::MAX - 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .is_err()
            {
                // We are racing another cleanup. Give up and let them do it.
                return;
            }
            // Safety: The value was written, we have exclusive access, and it
            // will not be overwritten until we set guard to u64::MAX.
            drop(entry.value.replace(None));
            entry.guard.store(u64::MAX, Ordering::Release);

            cursor =
                if cursor == 0 { self.array.len() - 1 } else { cursor - 1 };
        }
    }

    pub fn raii(self) -> CleanupRAII {
        CleanupRAII(self)
    }
}

impl Drop for CleanupRAII {
    fn drop(&mut self) {
        self.0.cleanup()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cleanup_queue() {
        let mut queue = CleanupQueue::new(10);
        for _ in 0..10_000 {
            for _ in 1..100 {
                queue.push(Arc::new(42));
            }
            let cleanup = queue.new_cleanup();
            std::thread::spawn(move || cleanup.cleanup());
            if rand::random::<u32>() % 1_000u32 == 0 {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
}
