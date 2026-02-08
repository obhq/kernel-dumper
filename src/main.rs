#![no_std]
#![no_main]

use crate::method::DumpMethod;
use core::arch::global_asm;
use core::cmp::min;
use core::ffi::c_int;
use core::mem::{size_of_val, zeroed};
use core::panic::PanicInfo;
use kernel::Kernel;
use okf::fd::OpenFlags;
use okf::MappedKernel;

#[cfg(method = "direct")]
mod direct;
mod method;
#[cfg(method = "syscall")]
mod syscall;

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
    // Setup dumping method.
    let kernel = Kernel::default();
    #[cfg(method = "syscall")]
    let method = unsafe { crate::syscall::SyscallMethod::new(&kernel) };
    #[cfg(method = "direct")]
    let method = crate::direct::DirectMethod::new(kernel);

    // Create dump file.
    let out = match method.open(
        c"/mnt/usb0/kernel.elf",
        OpenFlags::O_WRONLY | OpenFlags::O_CREAT | OpenFlags::O_TRUNC,
        0o777,
    ) {
        Ok(v) => v,
        Err(_) => {
            notify(&method, "Failed to open /mnt/usb0/kernel.elf");
            return;
        }
    };

    // Get ELF loaded size.
    let base = kernel.addr();
    let e_phnum: usize = unsafe { base.add(0x38).cast::<u16>().read().into() };
    let progs = unsafe { core::slice::from_raw_parts(base.add(0x40), e_phnum * 0x38) };
    let mut end = base as usize;

    for h in progs.chunks_exact(0x38) {
        // Skip non-loadable.
        let ty = u32::from_le_bytes(h[0x00..0x04].try_into().unwrap());

        if !matches!(ty, 0x1 | 0x61000010) {
            continue;
        }

        // Update end address.
        let addr = usize::from_le_bytes(h[0x10..0x18].try_into().unwrap());
        let len = usize::from_le_bytes(h[0x28..0x30].try_into().unwrap());
        let align = usize::from_le_bytes(h[0x30..0x38].try_into().unwrap());

        assert!(addr >= end); // Just in case if Sony re-order the programs.

        end = addr + len.next_multiple_of(align);
    }

    // Dump.
    let mut len = end - (base as usize);
    let mut next = base;

    while len != 0 {
        // Write file.
        let fd = out.as_raw_fd();
        let written = match method.write(fd, next, min(len, 0x4000)) {
            Ok(v) => v,
            Err(_) => {
                notify(&method, "Failed to write /mnt/usb0/kernel.elf");
                return;
            }
        };

        if written == 0 {
            notify(&method, "Not enough space to dump the kernel");
            return;
        }

        next = unsafe { next.add(written) };
        len -= written;
    }

    // Sync.
    if method.fsync(out.as_raw_fd()).is_err() {
        notify(
            &method,
            "Failed to synchronize changes to a /mnt/usb0/kernel.elf",
        );

        return;
    }

    notify(&method, "Dump completed!");
}

fn notify(method: &impl DumpMethod, msg: impl AsRef<[u8]>) {
    // Open notification device.
    let devs = [c"/dev/notification0", c"/dev/notification1"];
    let mut fd = None;

    for dev in devs {
        if let Ok(v) = method.open(dev, OpenFlags::O_WRONLY, 0) {
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
    method
        .write(
            fd.as_raw_fd(),
            &data as *const OrbisNotificationRequest as _,
            size_of_val(&data),
        )
        .ok();
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
