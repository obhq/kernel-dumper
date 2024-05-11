#![no_std]

use self::elf::ProgramType;
use self::file::{File, OwnedFile};
use self::thread::Thread;
use self::uio::{Uio, UioSeg};
use core::ffi::{c_char, c_int};
use core::num::NonZeroI32;
use core::ptr::null_mut;

pub use korbis_macros::*;

pub mod elf;
pub mod file;
pub mod thread;
pub mod uio;

/// Provides methods to access the PS4 kernel for a specific version.
///
/// Most methods here are a direct call to the kernel so most of them are unsafe. A safe wrapper for
/// those methods are provides by [`KernelExt`], which is automatically implemented for any type
/// that implement [`Kernel`].
pub trait Kernel: Copy + Send + Sync + 'static {
    type File: File;
    type Thread: Thread;
    type Uio: Uio;

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
    unsafe fn elf(self) -> &'static [u8];

    /// # Safety
    /// `fp` cannot be null.
    unsafe fn fget_write(
        self,
        td: *mut Self::Thread,
        fd: c_int,
        unused: c_int,
        fp: *mut *mut Self::File,
    ) -> c_int;

    /// # Panics
    /// If [`File::refcnt()`] of `fp` is not zero.
    ///
    /// # Safety
    /// - `fp` cannot be null.
    unsafe fn fdrop(self, fp: *mut Self::File, td: *mut Self::Thread) -> c_int;

    /// # Safety
    /// - `td` cannot be null.
    /// - `path` cannot be null and must point to a null-terminated string if `seg` is [`UioSeg::Kernel`].
    unsafe fn kern_openat(
        self,
        td: *mut Self::Thread,
        fd: c_int,
        path: *const c_char,
        seg: UioSeg,
        flags: c_int,
        mode: c_int,
    ) -> c_int;

    /// # Safety
    /// `td` cannot be null.
    unsafe fn kern_close(self, td: *mut Self::Thread, fd: c_int) -> c_int;

    /// # Safety
    /// `td` cannot be null.
    unsafe fn kern_fsync(self, td: *mut Self::Thread, fd: c_int, fullsync: c_int) -> c_int;

    /// # Safety
    /// - `td` cannot be null.
    /// - `auio` cannot be null.
    unsafe fn kern_writev(self, td: *mut Self::Thread, fd: c_int, auio: *mut Self::Uio) -> c_int;

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

/// Provides wrapper methods for methods on [`Kernel`].
///
/// This trait is automatically implemented for any type that implement [`Kernel`].
pub trait KernelExt: Kernel {
    fn fget_write(self, td: *mut Self::Thread, fd: c_int) -> Result<OwnedFile<Self>, NonZeroI32>;
}

impl<T: Kernel> KernelExt for T {
    fn fget_write(self, td: *mut Self::Thread, fd: c_int) -> Result<OwnedFile<Self>, NonZeroI32> {
        let mut fp = null_mut();
        let errno = unsafe { self.fget_write(td, fd, 0, &mut fp) };

        match NonZeroI32::new(errno) {
            Some(v) => Err(v),
            None => Ok(unsafe { OwnedFile::new(self, fp) }),
        }
    }
}
