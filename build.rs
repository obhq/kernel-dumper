use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let link = root.join("kernel-dumper.ld");
    let link = link.to_str().unwrap();

    println!("cargo::rustc-link-arg-bins=-T{link}");
    println!("cargo::rustc-check-cfg=cfg(method, values(\"syscall\"))");
    println!("cargo::rustc-check-cfg=cfg(fw, values(\"1100\"))");
}
