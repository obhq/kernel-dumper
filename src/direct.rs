use crate::method::OwnedFd;
use crate::DumpMethod;
use core::ffi::{c_int, CStr};
use core::num::NonZeroI32;
use okf::fd::OpenFlags;
use okf::pcpu::Pcpu;
use okf::thread::Thread;
use okf::uio::{IoVec, Uio, UioSeg};
use okf::Kernel;

/// Implementation of [`DumpMethod`] using internal kernel functions.
///
/// This method require a first dump from syscall method for required function addresses.
pub struct DirectMethod<K> {
    kernel: K,
}

impl<K: Kernel> DirectMethod<K> {
    pub fn new(kernel: K) -> Self {
        Self { kernel }
    }
}

impl<K: Kernel> DumpMethod for DirectMethod<K> {
    fn open(
        &self,
        path: &CStr,
        flags: OpenFlags,
        mode: c_int,
    ) -> Result<OwnedFd<'_, Self>, NonZeroI32> {
        let td = K::Pcpu::curthread();
        let errno = unsafe {
            self.kernel
                .kern_openat(td, -100, path.as_ptr(), UioSeg::Kernel, flags, mode)
        };

        match NonZeroI32::new(errno) {
            Some(v) => Err(v),
            None => Ok(unsafe { OwnedFd::new(self, (*td).ret(0).try_into().unwrap()) }),
        }
    }

    fn write(&self, fd: c_int, buf: *const u8, len: usize) -> Result<usize, NonZeroI32> {
        // Setup iovec.
        let mut iov = IoVec {
            ptr: buf.cast_mut(),
            len,
        };

        // Write.
        let td = K::Pcpu::curthread();
        let mut io = unsafe { Uio::write(&mut iov, td).unwrap() };
        let errno = unsafe { self.kernel.kern_writev(td, fd, &mut io) };

        match NonZeroI32::new(errno) {
            Some(v) => Err(v),
            None => Ok(unsafe { (*td).ret(0) }),
        }
    }

    fn fsync(&self, fd: c_int) -> Result<(), NonZeroI32> {
        let td = K::Pcpu::curthread();
        let errno = unsafe { self.kernel.kern_fsync(td, fd, 1) };

        match NonZeroI32::new(errno) {
            Some(v) => Err(v),
            None => Ok(()),
        }
    }

    fn close(&self, fd: c_int) -> Result<(), NonZeroI32> {
        let td = K::Pcpu::curthread();
        let errno = unsafe { self.kernel.kern_close(td, fd) };

        match NonZeroI32::new(errno) {
            Some(v) => Err(v),
            None => Ok(()),
        }
    }
}
