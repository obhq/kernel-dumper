use crate::thread::Thread;
use x86_64::registers::segmentation::{Segment64, GS};

/// Implementation of `pcpu` structure.
#[repr(C)]
pub struct CpuContext {
    thread: *mut Thread, // pc_curthread
}

impl CpuContext {
    pub fn current() -> *mut Self {
        GS::read_base().as_mut_ptr()
    }

    pub fn thread(&self) -> *mut Thread {
        self.thread
    }
}
