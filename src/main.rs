#![no_std]
#![no_main]

use crate::elf::ProgramType;
use crate::fs::{open, write, OpenFlags};
use crate::thread::Thread;
use core::arch::global_asm;
use core::cmp::min;
use core::ffi::{c_int, c_void};
use core::mem::{size_of_val, zeroed};
use core::panic::PanicInfo;
use core::ptr::null;
use x86_64::registers::control::Cr0;
use x86_64::registers::model_specific::LStar;
use x86_64::VirtAddr;

mod elf;
mod errno;
mod fs;
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
        Err(_) => {
            notify("Failed to open /mnt/usb0/kernel.elf");
            return;
        }
    };

    // Get kernel size.
    let e_phnum = unsafe { (base + 0x38).as_ptr::<u16>().read() as usize };
    let data = unsafe { core::slice::from_raw_parts((base + 0x40).as_ptr::<u8>(), e_phnum * 0x38) };
    let mut end = base.as_u64() as usize;

    for h in data.chunks_exact(0x38) {
        // Skip non-loadable.
        let ty = ProgramType::new(u32::from_le_bytes(h[0x00..0x04].try_into().unwrap()));

        if !matches!(ty, ProgramType::PT_LOAD | ProgramType::PT_SCE_RELRO) {
            continue;
        }

        // Check if program follow the previous one.
        let addr = usize::from_le_bytes(h[0x10..0x18].try_into().unwrap());

        if addr < end {
            notify("Some ELF programs overlapped!");
            return;
        }

        // Update end address.
        let len = usize::from_le_bytes(h[0x28..0x30].try_into().unwrap());
        let align = usize::from_le_bytes(h[0x30..0x38].try_into().unwrap());

        end = addr + len.next_multiple_of(align);
    }

    // Dump.
    let mut data = unsafe { core::slice::from_raw_parts(base.as_ptr::<u8>(), end) };

    while !data.is_empty() {
        // Write file.
        let fd = out.as_raw_fd();
        let len = min(data.len(), 0x4000);
        let buf = &data[..len];
        let bytes = match write(fd, buf.as_ptr(), buf.len()) {
            Ok(v) => v,
            Err(_) => {
                notify("Failed to write /mnt/usb0/kernel.elf");
                return;
            }
        };

        if bytes == 0 {
            notify("Not enough space to dump the kernel");
            return;
        }

        data = &data[bytes..];
    }

    notify("Dump completed!");
}

fn notify(msg: impl AsRef<[u8]>) {
    // Open notification device.
    let devs = [c"/dev/notification0", c"/dev/notification1"];
    let mut fd = None;

    for dev in devs {
        if let Ok(v) = open(dev, OpenFlags::O_WRONLY, 0) {
            fd = Some(v);
            break;
        }
    }

    // Check if we have a device to write to.
    let fd = match fd {
        Some(v) => v,
        None => return,
    };

    // Setup notification.
    let mut data: OrbisNotificationRequest = unsafe { zeroed() };
    let msg = msg.as_ref();
    let len = min(data.message.len() - 1, msg.len());

    data.target_id = -1;
    data.use_icon_image_uri = 1;
    data.message[..len].copy_from_slice(&msg[..len]);

    // Write notification.
    write(
        fd.as_raw_fd(),
        &data as *const OrbisNotificationRequest as _,
        size_of_val(&data),
    )
    .ok();
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

/// By OSM-Made.
#[repr(C)]
struct OrbisNotificationRequest {
    ty: c_int,
    req_id: c_int,
    priority: c_int,
    msg_id: c_int,
    target_id: c_int,
    user_id: c_int,
    unk1: c_int,
    unk2: c_int,
    app_id: c_int,
    error_num: c_int,
    unk3: c_int,
    use_icon_image_uri: u8,
    message: [u8; 1024],
    icon_uri: [u8; 1024],
    unk: [u8; 1024],
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
