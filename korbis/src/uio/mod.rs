/// Represents `uio_seg` enum.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UioSeg {
    /// UIO_USERSPACE
    User,
    /// UIO_SYSSPACE
    Kernel,
}
