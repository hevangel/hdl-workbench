#!/usr/bin/env python3
"""Repair an FST whose time table was written compressed but flagged raw.

`fst-writer` < the vendored fix chose raw vs zlib storage with `>` instead of
`>=`, so when zlib output came out EXACTLY the size of its input it wrote
compressed bytes while recording compressed_len == uncompressed_len -- which
every reader takes as "this section is stored raw". Readers then parse zlib's
own header as varints and the dump decodes to nonsense timestamps (the first
two are always 120 and 214, from the `78 5e` magic).

Only a length field is wrong; the bytes are intact. Writing the TRUE
uncompressed length back makes the two lengths differ again, so readers inflate
and the real times come back. Nothing moves, so the file size and every block
offset stay valid.

    fst-repair-timetable.py <file.fst> [-o out.fst]     # default: repair a copy
    fst-repair-timetable.py <file.fst> --check          # report only
"""
import argparse, os, shutil, struct, sys, zlib

VC_TYPES = (1, 5, 8)
ZLIB_MAGIC = (b"\x78\x9c", b"\x78\x5e", b"\x78\x01", b"\x78\xda")


def find_vc_block(f, size):
    off = 0
    while off < size:
        f.seek(off)
        head = f.read(9)
        if len(head) < 9:
            return None
        btype = head[0]
        blen = struct.unpack(">Q", head[1:9])[0]
        if btype in VC_TYPES:
            return off, blen
        if blen == 0:
            return None
        off += 1 + blen
    return None


def inspect(path):
    size = os.path.getsize(path)
    with open(path, "rb") as f:
        blk = find_vc_block(f, size)
        if blk is None:
            return None
        off, blen = blk
        end = off + 1 + blen
        f.seek(end - 24)
        unc, comp, items = struct.unpack(">QQQ", f.read(24))
        f.seek(end - 24 - comp)
        payload = f.read(comp)
    return dict(end=end, unc=unc, comp=comp, items=items, payload=payload)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("fst")
    ap.add_argument("-o", "--out")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()

    info = inspect(a.fst)
    if info is None:
        sys.exit("no value-change block found; not an FST?")

    zlibbed = info["payload"][:2] in ZLIB_MAGIC
    if not (info["unc"] == info["comp"] and zlibbed):
        print(f"{a.fst}: time table is fine "
              f"(uncompressed={info['unc']} compressed={info['comp']}, "
              f"{'zlib' if zlibbed else 'raw'} payload)")
        return

    try:
        real = zlib.decompress(info["payload"])
    except zlib.error as e:
        sys.exit(f"payload claims zlib but will not inflate: {e}")

    print(f"{a.fst}: AFFECTED -- {info['items']} time items, "
          f"{info['comp']} compressed bytes flagged as raw; "
          f"true uncompressed length is {len(real)}")
    if a.check:
        return

    out = a.out or (a.fst + ".repaired")
    shutil.copyfile(a.fst, out)
    with open(out, "r+b") as f:
        if len(real) == info["comp"]:
            # The break-even case that caused this: zlib output is exactly the
            # size of its input, so the two lengths are LEGITIMATELY equal and
            # correcting the length field would change nothing. Store what the
            # writer should have stored -- the raw bytes -- over the compressed
            # ones. Identical length, so no offset in the file moves.
            f.seek(info["end"] - 24 - info["comp"])
            f.write(real)
            how = "replaced the payload with its raw form"
        else:
            # Lengths only looked equal; recording the true uncompressed length
            # makes them differ again and the reader inflates.
            f.seek(info["end"] - 24)
            f.write(struct.pack(">Q", len(real)))
            how = f"corrected the uncompressed length to {len(real)}"
    print(f"wrote {out} ({how})")


if __name__ == "__main__":
    main()
