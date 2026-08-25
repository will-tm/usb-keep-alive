use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let rp2040 = env::var_os("CARGO_FEATURE_RP2040").is_some();
    let rp2350 = env::var_os("CARGO_FEATURE_RP2350").is_some();

    let memory_x = match (rp2040, rp2350) {
        (true, false) => "memory-rp2040.x",
        (false, true) => "memory-rp2350.x",
        _ => panic!("enable exactly one of the `rp2040` or `rp2350` features"),
    };

    // Put the chip's memory layout where the linker can find it as `memory.x`
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("memory.x"), fs::read(memory_x).unwrap()).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={memory_x}");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    // link-rp.x only places the RP2040 stage-2 bootloader; the RP2350 boot ROM
    // reads an IMAGE_DEF block instead, which memory-rp2350.x positions.
    if rp2040 {
        println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    }
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
