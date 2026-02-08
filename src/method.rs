use core::ffi::{c_int, CStr};
use core::num::NonZeroI32;
use okf::fd::OpenFlags;

/// Provides method to dump the kernel.
pub trait DumpMethod: Sized {
    fn open(
        &self,
        path: &CStr,
        flags: OpenFlags,
        mode: c_int,
    ) -> Result<OwnedFd<'_, Self>, NonZeroI32>;
    fn write(&self, fd: c_int, buf: *const u8, len: usize) -> Result<usize, NonZeroI32>;
    fn fsync(&self, fd: c_int) -> Result<(), NonZeroI32>;
    fn close(&self, fd: c_int) -> Result<(), NonZeroI32>;
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
