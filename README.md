# Kernel Dumper
[![CI](https://github.com/obhq/kernel-dumper/actions/workflows/ci.yml/badge.svg)](https://github.com/obhq/kernel-dumper/actions/workflows/ci.yml)

Kernel Dumper is a payload for PS4 kernel to dump the kernel. Only 11.00 is supported.

## Setup

Plug a USB drive to the PS4 and make sure the PS4 can write some files to it. You can test this by copy some game screenshots to it to see if it success without any errors.

## Running

You need to use TheFloW [PPPwn](https://github.com/TheOfficialFloW/PPPwn) with `--stage2` pointed to `kernel-dumper.bin` like the following:

```sh
sudo python3 pppwn.py --interface=enp0s3 --fw=1100 --stage2=kernel-dumper.bin
```

Wait for a notification `Dump completed!`. This may take a couple of minutes depend on how fast is your USB drive. Then shutdown the PS4 (not putting it into rest mode). Once the PS4 completely shutdown unplug the USB drive to grab `kernel.elf`.

To load this dump in Ghidra you need to load as a `Raw Binary` with a correct `Base Address` and `x86:default:64:little:gcc` as a `Language`. Use `readelf` to find a value for `Base Address`:

```sh
readelf -l kernel.elf
```

It will output something like:

```
Elf file type is EXEC (Executable file)
Entry point 0xffffffff9b68c8b0
There are 6 program headers, starting at offset 64

Program Headers:
  Type           Offset             VirtAddr           PhysAddr
                 FileSiz            MemSiz              Flags  Align
  PHDR           0x0000000000000040 0xffffffff9b480040 0xffffffff9b480040
                 0x0000000000000150 0x0000000000000150  R      0x8
  INTERP         0x0000000000000190 0xffffffff9b480190 0xffffffff9b480190
                 0x0000000000000007 0x0000000000000007  R      0x1
      [Requesting program interpreter: /DUMMY]
  LOAD           0x0000000000000000 0xffffffff9b480000 0xffffffff9b480000
                 0x0000000000cfe6d8 0x0000000000cfe6d8  R E    0x200000
  LOOS+0x1000010 0x0000000000cff000 0xffffffff9c57f000 0xffffffff9c57f000
                 0x0000000000020c70 0x0000000000200000  R      0x200000
  LOAD           0x0000000000d20000 0xffffffff9c9a0000 0xffffffff9c9a0000
                 0x0000000000605280 0x00000000012fda10  RW     0x200000
  DYNAMIC        0x0000000000cfe5c8 0xffffffff9c17e5c8 0xffffffff9c17e5c8
                 0x0000000000000110 0x0000000000000110  RW     0x8
```

The value for `Base Address` is `VirtAddr` of the first `LOAD` program (e.g. `0xffffffff9b480000` for the above dump). Each dump will have a different address due to ASLR so you can't use the above information with your dump.

## Building from source

### Prerequisites

- Rust on nightly channel
- [Project](https://github.com/ultimaweapon/project)
  - You can install with `cargo install project`

### Install additional Rust component

```sh
rustup component add rust-src llvm-tools
```

### Build

```sh
project build 11.00
```

## Development

You need to create `.cargo/config.toml` with the following content for rust-analyzer to work correctly:

```toml
[build]
target = "x86_64-unknown-none"
rustflags = ["--cfg", "fw=\"1100\"", "--cfg", "method=\"direct\"", "-Z", "unstable-options", "-C", "panic=immediate-abort"]

[unstable]
build-std = ["alloc", "core"]
```

## License

MIT
