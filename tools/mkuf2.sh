#!/usr/bin/env bash
# Build a UF2 for the given chip.
#
#   tools/mkuf2.sh rp2040 <out.uf2>
#   tools/mkuf2.sh rp2350 <out.uf2>
#
# elf2uf2-rs stamps every UF2 with the RP2040 family ID, so RP2350 images get
# their family ID rewritten afterwards -- the payload itself is already correct.
set -euo pipefail

CHIP="${1:?usage: mkuf2.sh <rp2040|rp2350> <out.uf2>}"
OUT="${2:?usage: mkuf2.sh <rp2040|rp2350> <out.uf2>}"

case "$CHIP" in
    rp2040) TARGET=thumbv6m-none-eabi;         FEATURES=(--features rp2040) ;;
    rp2350) TARGET=thumbv8m.main-none-eabihf;  FEATURES=(--no-default-features --features rp2350) ;;
    *) echo "unknown chip: $CHIP" >&2; exit 1 ;;
esac

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo build --release --target "$TARGET" "${FEATURES[@]}"

ELF="target/$TARGET/release/usb-keep-alive-rust"
mkdir -p "$(dirname "$OUT")"
elf2uf2-rs "$ELF" "$OUT"

if [ "$CHIP" = rp2350 ]; then
    python3 tools/uf2_family.py "$OUT" rp2350
fi

python3 tools/uf2_family.py --check "$OUT" "$CHIP"
echo "built $OUT ($CHIP)"
