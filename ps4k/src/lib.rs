#![no_std]

use self::elf::ProgramType;
use self::thread::Thread;
use core::ffi::{c_char, c_int};

pub use ps4k_macros::*;

pub mod elf;
pub mod thread;

/// Provides information about the PS4 kernel for a specific version.
pub trait Kernel: Send + Sync + 'static {
    type Thread: Thread;

    /// # Safety
    /// `base` must point to a valid address of the kernel. Behavior is undefined if format of the
    /// kernel is unknown.
    ///
    /// # Panics
    /// This function may panic if format of the kernel is unknown.
    unsafe fn new(base: *const u8) -> Self;

    /// Returns mapped ELF of the kernel.
    ///
    /// # Safety
    /// The returned slice can contains `PF_W` programs. That mean the memory covered by this slice
    /// can mutate at any time. The whole slice is guarantee to be readable.
    unsafe fn elf(&self) -> &'static [u8];

    /// # Safety
    /// - `td` cannot be null.
    /// - `path` cannot be null and must point to a null-terminated string if `kernel` is `true`.
    unsafe fn kern_openat(
        &self,
        td: *mut Self::Thread,
        fd: c_int,
        path: *const c_char,
        kernel: bool,
        flags: c_int,
        mode: c_int,
    ) -> c_int;

    /// # Safety
    /// `td` cannot be null.
    unsafe fn kern_close(&self, td: *mut Self::Thread, fd: c_int) -> c_int;

    /// # Safety
    /// `base` must point to a valid address of the kernel. Behavior is undefined if format of the
    /// kernel is unknown.
    ///
    /// # Panics
    /// This function may panic if format of the kernel is unknown.
    unsafe fn get_mapped_elf(base: *const u8) -> &'static [u8] {
        // Get ELF loaded size.
        let e_phnum = base.add(0x38).cast::<u16>().read() as usize;
        let progs = core::slice::from_raw_parts(base.add(0x40), e_phnum * 0x38);
        let mut end = base as usize;

        for h in progs.chunks_exact(0x38) {
            // Skip non-loadable.
            let ty = ProgramType::new(u32::from_le_bytes(h[0x00..0x04].try_into().unwrap()));

            if !matches!(ty, ProgramType::PT_LOAD | ProgramType::PT_SCE_RELRO) {
                continue;
            }

            // Update end address.
            let addr = usize::from_le_bytes(h[0x10..0x18].try_into().unwrap());
            let len = usize::from_le_bytes(h[0x28..0x30].try_into().unwrap());
            let align = usize::from_le_bytes(h[0x30..0x38].try_into().unwrap());

            assert!(addr >= end); // Just in case if Sony re-order the programs.

            end = addr + len.next_multiple_of(align);
        }

        // Get loaded ELF.
        let len = end - (base as usize);

        core::slice::from_raw_parts(base, len)
    }
}
