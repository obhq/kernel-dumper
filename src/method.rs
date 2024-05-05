use bitflags::bitflags;
use core::ffi::{c_int, CStr};
use core::num::NonZeroI32;

/// Provides method to dump the kernel.
pub trait DumpMethod: Sized {
    fn open(&self, path: &CStr, flags: OpenFlags, mode: c_int)
        -> Result<OwnedFd<Self>, NonZeroI32>;
    fn write(&self, fd: c_int, buf: *const u8, len: usize) -> Result<usize, NonZeroI32>;
    fn fsync(&self, fd: c_int) -> Result<(), NonZeroI32>;
    fn close(&self, fd: c_int) -> Result<(), NonZeroI32>;
}

bitflags! {
    /// Flags for [`DumpMethod::open()`].
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct OpenFlags: u32 {
        const O_RDONLY = 0x00000000;
        const O_WRONLY = 0x00000001;
        const O_RDWR = 0x00000002;
        const O_ACCMODE = Self::O_WRONLY.bits() | Self::O_RDWR.bits();
        const O_SHLOCK = 0x00000010;
        const O_EXLOCK = 0x00000020;
        const O_CREAT = 0x00000200;
        const O_TRUNC = 0x00000400;
        const O_EXCL = 0x00000800;
        const O_EXEC = 0x00040000;
        const O_CLOEXEC = 0x00100000;
        const UNK1 = 0x00400000;
    }
}

/// Encapsulate an opened file descriptor.
pub struct OwnedFd<'a, T: DumpMethod> {
    method: &'a T,
    fd: c_int,
}

impl<'a, T: DumpMethod> OwnedFd<'a, T> {
    pub fn new(method: &'a T, fd: c_int) -> Self {
        Self { method, fd }
    }

    pub fn as_raw_fd(&self) -> c_int {
        self.fd
    }
}

impl<'a, T: DumpMethod> Drop for OwnedFd<'a, T> {
    fn drop(&mut self) {
        self.method.close(self.fd).unwrap();
    }
}
