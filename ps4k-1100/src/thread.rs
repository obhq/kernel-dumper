/// Implementation of [`ps4k::thread::Thread`] for 11.00.
#[repr(C)]
pub struct Thread {
    pad: [u8; 0x398],
    ret: [usize; 2], // td_retval
}

impl ps4k::thread::Thread for Thread {
    fn ret(&self, i: usize) -> usize {
        self.ret[i]
    }
}
