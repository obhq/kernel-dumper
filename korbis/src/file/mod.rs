use crate::thread::Thread;
use crate::Kernel;
use core::sync::atomic::{fence, AtomicU32, Ordering};

/// Represents `file` structure.
pub trait File: Sized {
    /// Returns `f_count` field.
    fn refcnt(&self) -> &AtomicU32;
}

/// RAII struct to decrease `file::f_count` when dropped.
pub struct OwnedFile<K: Kernel> {
    kernel: K,
    file: *mut K::File,
}

impl<K: Kernel> OwnedFile<K> {
    /// # Safety
    /// `file` cannot be null and the caller must own a strong reference to it. This method do
    /// **not** increase the reference count of this file.
    pub unsafe fn new(kernel: K, file: *mut K::File) -> Self {
        Self { kernel, file }
    }
}

impl<K: Kernel> Drop for OwnedFile<K> {
    fn drop(&mut self) {
        // See Drop implementation on Arc how this thing work.
        if unsafe { (*self.file).refcnt().fetch_sub(1, Ordering::Release) } != 1 {
            return;
        }

        fence(Ordering::Acquire);

        // The kernel itself does not check if fdrop is success so we don't need to.
        unsafe { self.kernel.fdrop(self.file, K::Thread::current()) };
    }
}
