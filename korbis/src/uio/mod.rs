use crate::Kernel;

/// Represents `uio` structure.
pub trait Uio: Sized {
    type Kernel: Kernel;

    /// Returns [`None`] if `len` is geater than [`Uio::vec_max()`] or total length of `iov` is
    /// greater than [`Uio::io_max()`].
    ///
    /// # Safety
    /// - `td` cannot be null.
    /// - `iov` cannot be null and must be valid up to `len`.
    unsafe fn new(
        td: *mut <Self::Kernel as Kernel>::Thread,
        op: UioRw,
        seg: UioSeg,
        iov: *mut IoVec,
        len: usize,
    ) -> Option<Self>;

    /// Returns value of `UIO_MAXIOV`.
    fn vec_max() -> usize {
        1024
    }

    /// Returns value of `IOSIZE_MAX`.
    fn io_max() -> usize {
        0x7fffffff
    }
}

/// Represents `uio_seg` enum.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UioSeg {
    /// UIO_USERSPACE
    User,
    /// UIO_SYSSPACE
    Kernel,
}

/// Represents `uio_rw` enum.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UioRw {
    /// UIO_READ
    Read,
    /// UIO_WRITE
    Write,
}

/// Represents `iovec` structure.
#[repr(C)]
pub struct IoVec {
    pub ptr: *mut u8,
    pub len: usize,
}
