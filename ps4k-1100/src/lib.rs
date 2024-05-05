#![no_std]

use self::thread::Thread;
use core::ffi::{c_char, c_int};
use ps4k::offset;

mod thread;

/// Implementation of [`ps4k::version::KernelVersion`] for 11.00.
pub struct KernelVersion {
    elf: &'static [u8],
}

impl ps4k::version::KernelVersion for KernelVersion {
    type Thread = Thread;

    unsafe fn new(base: *const u8) -> Self {
        let elf = Self::get_mapped_elf(base);

        Self { elf }
    }

    unsafe fn elf(&self) -> &'static [u8] {
        self.elf
    }

    #[offset(0xE63B0)]
    unsafe fn kern_openat(
        &self,
        td: *mut Self::Thread,
        fd: c_int,
        path: *const c_char,
        kernel: bool,
        flags: c_int,
        mode: c_int,
    ) -> c_int;
}
