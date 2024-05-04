use crate::thread::Thread;
use crate::SYSENTS;
use bitflags::bitflags;
use core::ffi::{c_int, CStr};
use core::num::NonZeroI32;

pub fn open(path: &CStr, flags: OpenFlags, mode: c_int) -> Result<OwnedFd, NonZeroI32> {
    // Setup arguments.
    let td = Thread::current();
    let args = [path.as_ptr() as usize, flags.bits() as usize, mode as usize];

    // Invoke handler.
    let handler = unsafe { (*SYSENTS)[5].handler };
    let errno = unsafe { handler(td, args.as_ptr().cast()) };

    match NonZeroI32::new(errno) {
        Some(v) => Err(v),
        None => Ok(OwnedFd(unsafe { (*td).ret()[0].try_into().unwrap() })),
    }
}

pub fn write(fd: c_int, buf: *const u8, len: usize) -> Result<usize, NonZeroI32> {
    // Setup arguments.
    let td = Thread::current();
    let args = [fd as usize, buf as usize, len];

    // Invoke handler.
    let handler = unsafe { (*SYSENTS)[4].handler };
    let errno = unsafe { handler(td, args.as_ptr().cast()) };

    match NonZeroI32::new(errno) {
        Some(v) => Err(v),
        None => Ok(unsafe { (*td).ret()[0] }),
    }
}

/// Encapsulate an opened file descriptor.
pub struct OwnedFd(c_int);

impl OwnedFd {
    pub fn as_raw_fd(&self) -> c_int {
        self.0
    }
}

impl Drop for OwnedFd {
    fn drop(&mut self) {
        let td = Thread::current();
        let args = [self.0 as usize];
        let handler = unsafe { (*SYSENTS)[6].handler };
        let errno = unsafe { handler(td, args.as_ptr().cast()) };

        assert_eq!(errno, 0);
    }
}

bitflags! {
    /// Flags for [`open()`].
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
