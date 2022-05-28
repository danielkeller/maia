use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct CleanupQueue {
    cursor: usize,
    level: u64,
    array: Arc<Box<[QueueEntry]>>,
}

#[derive(Debug)]
pub struct Cleanup {
    cursor: usize,
    level: u64,
    array: Arc<Box<[QueueEntry]>>,
}

#[derive(Debug)]
struct QueueEntry {
    guard: AtomicU64,
    value: UnsafeCell<MaybeUninit<Arc<dyn Send + Sync>>>,
}

unsafe impl Send for QueueEntry {}
unsafe impl Sync for QueueEntry {}

impl Default for QueueEntry {
    fn default() -> Self {
        Self {
            guard: AtomicU64::new(u64::MAX),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

impl CleanupQueue {
    fn new_array(capacity: usize) -> Arc<Box<[QueueEntry]>> {
        let mut vec = Vec::new();
        vec.resize_with(capacity.max(1), Default::default);
        Arc::new(vec.into_boxed_slice())
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
            unsafe { entry.value.get().write(MaybeUninit::new(value)) };
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
            drop(unsafe { entry.value.get().read().assume_init_read() });
            entry.guard.store(u64::MAX, Ordering::Release);

            cursor =
                if cursor == 0 { self.array.len() - 1 } else { cursor - 1 };
        }
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
