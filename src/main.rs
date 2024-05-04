#![no_std]
#![no_main]

use crate::fs::{open, OpenFlags};
use crate::thread::Thread;
use core::arch::global_asm;
use core::ffi::{c_int, c_void};
use core::panic::PanicInfo;
use core::ptr::null;
use x86_64::registers::control::Cr0;
use x86_64::registers::model_specific::LStar;
use x86_64::VirtAddr;

mod errno;
mod fs;
mod pcpu;
mod thread;

// The job of this custom entry point is:
//
// - Get address where our payload is loaded.
// - Do ELF relocation on our payload.
global_asm!(
    ".globl _start",
    ".section .text.entry",
    "_start:",
    "lea rdi, [rip]",
    "sub rdi, 7", // 7 is size of "lea rdi, [rip]".
    "mov rax, rdi",
    "add rax, 0x80", // Offset of dynamic section configured in kernel-dumper.ld.
    "xor r8, r8",
    "0:",
    "mov rsi, [rax]",
    "mov rcx, [rax+8]",
    "add rax, 16",
    "test rsi, rsi", // Check if DT_NULL.
    "jz 1f",
    "cmp rsi, 7", // Check if DT_RELA.
    "jz 2f",
    "cmp rsi, 8", // Check if DT_RELASZ.
    "jz 3f",
    "jmp 0b",
    "2:", // Keep DT_RELA.
    "mov rdx, rdi",
    "add rdx, rcx",
    "jmp 0b",
    "3:", // Keep DT_RELASZ.
    "mov r8, rcx",
    "jmp 0b",
    "1:",
    "test r8, r8", // Check if no more DT_RELA entries.
    "jz main",
    "mov rsi, [rdx]",
    "mov rax, [rdx+8]",
    "mov rcx, [rdx+16]",
    "add rdx, 24",
    "sub r8, 24",
    "test eax, eax", // Check if R_X86_64_NONE.
    "jz main",
    "cmp eax, 8", // Check if R_X86_64_RELATIVE.
    "jnz 1b",
    "add rsi, rdi",
    "add rcx, rdi",
    "mov [rsi], rcx",
    "jmp 1b",
);

#[no_mangle]
pub extern "C" fn main(_: *const u8) {
    // Get base address of the kernel.
    let aslr = LStar::read() - 0xffffffff822001c0;
    let base = aslr + 0xffffffff82200000;

    // Remove address checking from copyin, copyout and copyinstr.
    let cr0 = Cr0::read_raw();

    unsafe { Cr0::write_raw(cr0 & !(1 << 16)) };
    unsafe { patch_kernel(base) };
    unsafe { Cr0::write_raw(cr0) };

    // Get kernel addresses.
    unsafe { SYSENTS = (base + 0x1101760).as_ptr() };

    // Create dump file.
    let out = match open(
        c"/mnt/usb0/kernel.elf",
        OpenFlags::O_WRONLY | OpenFlags::O_CREAT | OpenFlags::O_TRUNC,
        0o777,
    ) {
        Ok(v) => v,
        Err(_) => return,
    };

    drop(out);
}

/// # Safety
/// - `base` must be a valid base address of the kernel.
/// - `WP` flag must not be set on `CR0`.
unsafe fn patch_kernel(base: VirtAddr) {
    let base = base.as_mut_ptr::<u8>();
    let patches = [
        (0x2DDF42usize, [0x90u8; 2].as_slice()), // copyout_patch1
        (0x2DDF4E, &[0x90; 3]),                  // copyout_patch2
        (0x2DE037, &[0x90; 2]),                  // copyin_patch1
        (0x2DE043, &[0x90; 3]),                  // copyin_patch2
        (0x2DE4E3, &[0x90; 2]),                  // copyinstr_patch1
        (0x2DE4EF, &[0x90; 3]),                  // copyinstr_patch2
        (0x2DE520, &[0x90; 2]),                  // copyinstr_patch3
    ];

    for (off, patch) in patches {
        base.add(off)
            .copy_from_nonoverlapping(patch.as_ptr(), patch.len());
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

/// Implementation of `sysent` structure.
#[repr(C)]
struct Sysent {
    narg: c_int,
    handler: unsafe extern "C" fn(td: *mut Thread, uap: *const c_void) -> c_int,
    pad: [u8; 0x20],
}

/// Syscall table.
static mut SYSENTS: *const [Sysent; 678] = null();
