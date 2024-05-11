#![no_std]

use self::thread::Thread;
use core::ffi::{c_char, c_int};
use korbis::offset;
use korbis::uio::UioSeg;

mod thread;

/// Implementation of [`ps4k::Kernel`] for 11.00.
#[derive(Clone, Copy)]
pub struct Kernel(&'static [u8]);

impl korbis::Kernel for Kernel {
    type Thread = Thread;

    unsafe fn new(base: *const u8) -> Self {
        Self(Self::get_mapped_elf(base))
    }

    unsafe fn elf(self) -> &'static [u8] {
        self.0
    }

    #[offset(0xE63B0)]
    unsafe fn kern_openat(
        self,
        td: *mut Self::Thread,
        fd: c_int,
        path: *const c_char,
        seg: UioSeg,
        flags: c_int,
        mode: c_int,
    ) -> c_int;

    #[offset(0x416920)]
    unsafe fn kern_close(self, td: *mut Self::Thread, fd: c_int) -> c_int;
}
