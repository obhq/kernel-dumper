#![no_std]

use self::elf::ProgramType;

pub mod elf;

/// Struct to access internal kernel functions and variables.
pub struct Kernel {
    elf: &'static [u8],
}

impl Kernel {
    /// # Safety
    /// `base` must point to a valid address of the kernel. Behavior is undefined if format of the
    /// kernel is unknown. This should never happens unless the PS4 boot loader has been changed in
    /// the future.
    ///
    /// # Panics
    /// This function may panic if format of the kernel is unknown.
    pub unsafe fn new(base: *const u8) -> Self {
        // Get ELF loaded size.
        let e_phnum = base.add(0x38).cast::<u16>().read() as usize;
        let progs = core::slice::from_raw_parts(base.add(0x40), e_phnum * 0x38);
        let mut end = base as usize;

        for h in progs.chunks_exact(0x38) {
            // Skip non-loadable.
            let ty = ProgramType::new(u32::from_le_bytes(h[0x00..0x04].try_into().unwrap()));

            if !matches!(ty, ProgramType::PT_LOAD | ProgramType::PT_SCE_RELRO) {
                continue;
            }

            // Update end address.
            let addr = usize::from_le_bytes(h[0x10..0x18].try_into().unwrap());
            let len = usize::from_le_bytes(h[0x28..0x30].try_into().unwrap());
            let align = usize::from_le_bytes(h[0x30..0x38].try_into().unwrap());

            assert!(addr >= end); // Just in case if Sony re-order the programs.

            end = addr + len.next_multiple_of(align);
        }

        // Get loaded ELF.
        let len = end - (base as usize);
        let elf = unsafe { core::slice::from_raw_parts(base, len) };

        Self { elf }
    }

    /// Returns loaded ELF of the kernel.
    ///
    /// # Safety
    /// The returned slice can contains `PF_W` programs. That mean the memory covered by this slice
    /// can mutate at any time. The whole slice is guarantee to be readable.
    pub unsafe fn elf(&self) -> &'static [u8] {
        self.elf
    }
}
