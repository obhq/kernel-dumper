#![no_std]

use self::version::KernelVersion;

pub use ps4k_macros::*;

pub mod elf;
pub mod version;

/// Struct to access internal kernel functions and variables.
pub struct Kernel<V> {
    version: V,
}

impl<V: KernelVersion> Kernel<V> {
    /// # Safety
    /// `base` must point to a valid address of the kernel. Behavior is undefined if format of the
    /// kernel is unknown to `V`.
    ///
    /// # Panics
    /// This function may panic if format of the kernel is unknown to `V`.
    pub unsafe fn new(base: *const u8) -> Self {
        let version = V::new(base);

        Self { version }
    }

    pub fn version(&self) -> &V {
        &self.version
    }
}
