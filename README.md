# USB Keep Alive

USB HID mouse firmware for the Raspberry Pi Pico that nudges the cursor one
pixel every 10 seconds, keeping the host from going to sleep. Written in Rust
with [Embassy](https://embassy.dev).

Supports **RP2040** (Pico) and **RP2350** (Pico 2).

## Build

Prebuilt UF2s are attached to each [release](https://github.com/will-tm/usb-keep-alive/releases).

To build them yourself you need [`elf2uf2-rs`](https://crates.io/crates/elf2uf2-rs):

```sh
cargo install elf2uf2-rs
tools/mkuf2.sh rp2040 dist/usb-keep-alive-rp2040.uf2
tools/mkuf2.sh rp2350 dist/usb-keep-alive-rp2350.uf2
```

`cargo build --release` on its own targets the RP2040. For the RP2350:

```sh
cargo build --release --target thumbv8m.main-none-eabihf \
    --no-default-features --features rp2350
```

Prefer `tools/mkuf2.sh` for anything you intend to flash: `elf2uf2-rs` stamps
every image with the RP2040 family ID, and the RP2350 boot ROM rejects that,
so the script corrects it.

## Flash

Hold **BOOTSEL** while plugging the board in, then copy the UF2 to the drive
it mounts as:

```sh
cp dist/usb-keep-alive-rp2040.uf2 /Volumes/RPI-RP2/   # Pico
cp dist/usb-keep-alive-rp2350.uf2 /Volumes/RP2350/    # Pico 2
```

The board reboots and starts running immediately.

## USB device

- VID `0x1506`, PID `0x4004`
- Manufacturer `will_tm`, product `USB Keep Alive`
- Serial: unique 8-byte chip ID as hex — SPI flash UID on RP2040, OTP chip ID
  on RP2350

## Layout

| Path | |
|---|---|
| `src/main.rs` | firmware |
| `memory-rp2040.x`, `memory-rp2350.x` | per-chip linker layouts, selected by `build.rs` |
| `tools/mkuf2.sh` | build a flashable UF2 |
| `tools/uf2_family.py` | rewrite/verify a UF2 family ID |
