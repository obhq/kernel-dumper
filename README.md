# Kernel Dumper

Kernel Dumper is a payload for PS4 kernel to dump the kernel. Only 11.00 is supported.

## Running

You need to use TheFloW [PPPwn](https://github.com/TheOfficialFloW/PPPwn) with `--stage2` pointed to `kernel-dumper.bin` like the following:

```sh
sudo python3 pppwn.py --interface=enp0s3 --fw=1100 --stage2=kernel-dumper.bin
```

## Building from source

### Prerequisites

- Rust on the latest stable channel

### Enable x86_64-unknown-none target

```sh
rustup target add x86_64-unknown-none
```

### Install additional Cargo commands

```sh
cargo install cargo-binutils
```

`cargo-binutils` required additional dependency which can be installed with the following command:

```sh
rustup component add llvm-tools
```

### Build

```sh
cargo objcopy --release -- -O binary kernel-dumper.bin
```

## License

MIT
