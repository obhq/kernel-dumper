#![no_std]

use self::thread::Thread;
use core::ffi::{c_char, c_int};
use korbis::offset;
use korbis::uio::UioSeg;

mod thread;

/// Implementation of [`ps4k::Kernel`] for 11.00.
pub struct Kernel {
    elf: &'static [u8],
}

impl korbis::Kernel for Kernel {
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
        seg: UioSeg,
        flags: c_int,
        mode: c_int,
    ) -> c_int;

    #[offset(0x416920)]
    unsafe fn kern_close(&self, td: *mut Self::Thread, fd: c_int) -> c_int;
}
