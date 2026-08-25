#!/usr/bin/env python3
"""Rewrite or verify the family ID of a UF2 file.

elf2uf2-rs predates the RP2350 and stamps every image with the RP2040 family
ID. The RP2350 boot ROM rejects that, so RP2350 images need the field fixed.
Only the 32-bit family ID at offset 28 of each 512-byte block changes; the
payload is untouched.

    uf2_family.py <file.uf2> rp2350            rewrite in place
    uf2_family.py --check <file.uf2> rp2040    verify only
"""
import struct
import sys

FAMILIES = {"rp2040": 0xE48BFF56, "rp2350": 0xE48BFF59}
NAMES = {v: k for k, v in FAMILIES.items()}
MAGIC0, MAGIC1, MAGIC_END = 0x0A324655, 0x9E5D5157, 0x0AB16F30
FAMILY_PRESENT = 0x00002000
BLOCK = 512


def blocks(data):
    if not data or len(data) % BLOCK:
        raise SystemExit(f"not a UF2: {len(data)} bytes is not a multiple of {BLOCK}")
    total = len(data) // BLOCK
    for i in range(total):
        off = i * BLOCK
        m0, m1, flags, addr, psize, no, count, family = struct.unpack_from("<8I", data, off)
        if (m0, m1) != (MAGIC0, MAGIC1):
            raise SystemExit(f"block {i}: bad start magic")
        if struct.unpack_from("<I", data, off + 508)[0] != MAGIC_END:
            raise SystemExit(f"block {i}: bad end magic")
        if not flags & FAMILY_PRESENT:
            raise SystemExit(f"block {i}: familyID flag not set")
        if no != i or count != total:
            raise SystemExit(f"block {i}: bad numbering ({no}/{count})")
        if psize > 476:
            raise SystemExit(f"block {i}: payload too large ({psize})")
        yield i, off, family, addr


def main(argv):
    check = "--check" in argv
    argv = [a for a in argv if a != "--check"]
    if len(argv) != 3:
        raise SystemExit(__doc__)

    path, chip = argv[1], argv[2]
    if chip not in FAMILIES:
        raise SystemExit(f"unknown chip {chip!r}, expected one of {sorted(FAMILIES)}")
    want = FAMILIES[chip]

    data = bytearray(open(path, "rb").read())
    found, addrs, count = set(), [], 0
    for _, off, family, addr in blocks(data):
        found.add(family)
        addrs.append(addr)
        count += 1
        if not check:
            struct.pack_into("<I", data, off + 28, want)

    if check:
        if found != {want}:
            got = ", ".join(f"{f:#x} ({NAMES.get(f, '?')})" for f in sorted(found))
            raise SystemExit(f"{path}: expected {chip} ({want:#x}), found {got}")
        print(f"{path}: OK -- {count} blocks, {chip} ({want:#x}), "
              f"{min(addrs):#x}-{max(addrs):#x}")
    else:
        open(path, "wb").write(bytes(data))
        print(f"{path}: {count} blocks -> {chip} ({want:#x})")


if __name__ == "__main__":
    main(sys.argv)
