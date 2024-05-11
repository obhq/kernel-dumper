use crate::method::{OpenFlags, OwnedFd};
use crate::DumpMethod;
use core::ffi::{c_int, CStr};
use core::num::NonZeroI32;
use korbis::thread::Thread;
use korbis::uio::UioSeg;
use korbis::Kernel;

/// Implementation of [`DumpMethod`] using internal kernel functions.
///
/// This method require a first dump from syscall method for required function addresses.
pub struct DirectMethod<K> {
    kernel: K,
}

impl<K> DirectMethod<K> {
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
    ) -> Result<OwnedFd<Self>, NonZeroI32> {
        let td = Thread::current();
        let errno = unsafe {
            self.kernel.kern_openat(
                td,
                -100,
                path.as_ptr(),
                UioSeg::Kernel,
                flags.bits() as _,
                mode,
            )
        };

        match NonZeroI32::new(errno) {
            Some(v) => Err(v),
            None => Ok(unsafe { OwnedFd::new(self, (*td).ret(0).try_into().unwrap()) }),
        }
    }

    fn write(&self, fd: c_int, buf: *const u8, len: usize) -> Result<usize, NonZeroI32> {
        Ok(len)
    }

    fn fsync(&self, fd: c_int) -> Result<(), NonZeroI32> {
        Ok(())
    }

    fn close(&self, fd: c_int) -> Result<(), NonZeroI32> {
        let td = Thread::current();
        let errno = unsafe { self.kernel.kern_close(td, fd) };

        match NonZeroI32::new(errno) {
            Some(v) => Err(v),
            None => Ok(()),
        }
    }
}
