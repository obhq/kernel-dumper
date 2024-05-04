use core::arch::asm;

/// Implementation of `thread` structure.
#[repr(C)]
pub struct Thread {
    pad: [u8; 0x398],
    ret: [u64; 2], // td_retval
}

impl Thread {
    pub fn current() -> *mut Self {
        let mut p;

        unsafe {
            asm!("mov {}, gs:[0]", out(reg) p, options(readonly, pure, preserves_flags, nostack))
        };

        p
    }

    pub fn ret(&self) -> &[u64; 2] {
        &self.ret
    }
}
