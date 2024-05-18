use crate::thread::Thread;
use crate::Kernel;
use core::ffi::c_int;
use korbis::uio::{IoVec, UioRw, UioSeg};

/// Implementation of [`korbis::uio::Uio`] for 11.00.
#[repr(C)]
pub struct Uio {
    iov: *mut IoVec,
    len: c_int,
    off: isize,
    res: isize,
    seg: UioSeg,
    op: UioRw,
    td: *mut Thread,
}

impl korbis::uio::Uio<Kernel> for Uio {
    unsafe fn new(
        td: *mut Thread,
        op: UioRw,
        seg: UioSeg,
        iov: *mut IoVec,
        len: usize,
    ) -> Option<Self> {
        // Check vec count.
        if len > Self::vec_max() {
            return None;
        }

        // Get total length.
        let mut res = 0usize;

        for i in 0..len {
            res = res.checked_add((*iov.add(i)).len)?;
        }

        if res > Self::io_max() {
            return None;
        }

        Some(Self {
            iov,
            len: len.try_into().unwrap(),
            off: -1,
            res: res.try_into().unwrap(),
            seg,
            op,
            td,
        })
    }
}
