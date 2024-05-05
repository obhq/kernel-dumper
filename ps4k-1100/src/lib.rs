#![no_std]

/// Implementation of [`ps4k::version::KernelVersion`] for 11.00.
pub struct KernelVersion {
    elf: &'static [u8],
}

impl ps4k::version::KernelVersion for KernelVersion {
    unsafe fn new(base: *const u8) -> Self {
        let elf = Self::get_mapped_elf(base);

        Self { elf }
    }

    unsafe fn elf(&self) -> &'static [u8] {
        self.elf
    }
}
