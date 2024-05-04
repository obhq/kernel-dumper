use crate::pcpu::CpuContext;

/// Implementation of `thread` structure.
#[repr(C)]
pub struct Thread {
    pad: [u8; 0x398],
    ret: [u64; 2], // td_retval
}

impl Thread {
    pub fn current() -> *mut Self {
        unsafe { (*CpuContext::current()).thread() }
    }

    pub fn ret(&self) -> &[u64; 2] {
        &self.ret
    }
}
