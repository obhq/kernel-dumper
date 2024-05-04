#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;
use x86_64::registers::control::Cr0;

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
    "add rax, 0x100", // Offset of dynamic section configured in kernel-dumper.ld.
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
    // Remove address checking from copyin, copyout and copyinstr.
    let cr0 = Cr0::read_raw();

    unsafe { Cr0::write_raw(cr0 & !(1 << 16)) };
    unsafe { Cr0::write_raw(cr0) };
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
