# USB Keep Alive (Rust / Embassy)

USB HID mouse emulator for Raspberry Pi Pico (RP2040) that sends imperceptible cursor movements every 10 seconds to prevent the host from sleeping.

Rust port of the C/TinyUSB version in `../usb-keep-alive/`.

## Building

```sh
cargo run --release
```

This builds the ELF and produces a UF2 at:
`target/thumbv6m-none-eabi/release/usb-keep-alive-rust.uf2`

Install the runner if needed: `cargo install elf2uf2-rs`

## Flashing

Hold **BOOTSEL** on the Pico and plug it in, then copy the UF2:

```sh
cp target/thumbv6m-none-eabi/release/usb-keep-alive-rust.uf2 /Volumes/RPI-RP2/
```

## LED Blink Patterns

| Pattern | State |
|---------|-------|
| Fast blink (250ms) | Not mounted — waiting for USB host |
| Slow blink (1000ms) | Mounted — keep-alive active |
| Very slow blink (2500ms) | Suspended by host |

## USB Device Info

- VID: `0x1506`, PID: `0x4004`
- Manufacturer: `will_tm`
- Product: `USB Keep Alive`
- Serial: unique flash ID (hex)
