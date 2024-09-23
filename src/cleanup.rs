use crate::device::Device;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use smallbox::{smallbox, SmallBox};

use crate::exclusive::Exclusive;

struct Resource(SmallBox<dyn Send + Sync, smallbox::space::S2>);

impl Resource {
    fn new<T: Send + Sync + 'static>(value: T) -> Self {
        let result = Self(smallbox!(value));
        debug_assert!(!result.0.is_heap());
        result
    }
}

#[derive(Debug)]
pub struct Epoch {
    number: usize,
    garbage_in: mpsc::Sender<Resource>,
    garbage_out: Exclusive<mpsc::Receiver<Resource>>,
}

impl PartialEq for Epoch {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}
impl Eq for Epoch {}
impl PartialOrd for Epoch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.number.partial_cmp(&other.number)
    }
}
impl Ord for Epoch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number.cmp(&other.number)
    }
}

impl Epoch {
    fn new(number: usize) -> Self {
        let (sender, reciever) = mpsc::channel();
        Self { number, garbage_in: sender, garbage_out: reciever.into() }
    }

    pub fn dispose(&self, resource: impl Send + Sync + 'static) {
        self.garbage_in.send(Resource::new(resource)).unwrap();
    }
}

/// Manages the lifetimes of objects that have been submitted to the device.
pub struct Cleanup {
    epoch: AtomicUsize,
    epochs: [AtomicRefCell<Arc<Epoch>>; 2],
    purgatory: AtomicRefCell<Vec<Resource>>,
    device: Device,
}

impl Cleanup {
    pub(crate) fn new(device: &Device) -> Self {
        Self {
            epoch: 0.into(),
            epochs: [
                Arc::new(Epoch::new(0)).into(),
                Arc::new(Epoch::new(1)).into(),
            ],
            purgatory: Default::default(),
            device: device.clone(),
        }
    }

    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn pin(&self) -> Arc<Epoch> {
        let mut n = self.epoch.load(Ordering::Relaxed);
        loop {
            if let Ok(epoch) = self.epochs[n % 2].try_borrow() {
                return epoch.clone();
            }
            n = n.wrapping_add(1);
        }
    }

    pub fn dispose(&self, resource: impl Send + Sync + 'static) {
        self.pin().dispose(resource);
    }

    // This function is potentially expensive (if garbage exists), maybe run in
    // the background.
    pub fn try_advance(&self, current: &Epoch) {
        let n = current.number.wrapping_add(1);
        let Ok(mut epoch) = self.epochs[n % 2].try_borrow_mut() else { return };
        let Some(epoch) = Arc::get_mut(&mut epoch) else { return };
        // No other thread can be simultaneously in the rest of the function,
        // since 'current' has a shared reference to one epoch and 'epoch' has
        // an exclusive reference to the other.
        let mut purgatory = self.purgatory.try_borrow_mut().unwrap();
        // We're ok to clear the items from epoch n - 2.
        purgatory.clear();
        for item in epoch.garbage_out.get_mut().try_iter() {
            purgatory.push(item);
        }
        epoch.number = n;
        self.epoch.store(n, Ordering::Relaxed);
    }
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        // This is ext. sync. on the queues.
        unsafe {
            (self.device.fun().device_wait_idle)(self.device.handle());
        }

        for e in &mut self.epochs {
            // This is supposed to be called when the Device is being destroyed,
            // so anything that holds an epoch reference should also hold a
            // device reference and have been dropped by now.
            Arc::get_mut(&mut e.borrow_mut())
                .unwrap()
                .garbage_out
                .get_mut()
                .try_iter()
                .for_each(drop);
        }
    }
}

impl std::fmt::Debug for Cleanup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cleanup")
            .field("epoch", &self.epoch)
            .field("epochs", &self.epochs)
            .field("device", &self.device)
            .finish()
    }
}
