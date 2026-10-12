#!/usr/bin/env python3
"""Copy a reference prefix without another MAME run. Includes the sync record."""
import argparse
import json
import struct
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    limit = parser.add_mutually_exclusive_group(required=True)
    limit.add_argument("--frames", type=int)
    limit.add_argument("--control-frames", type=int,
                       help="stop after N game frames from first Jack control in Episode 1")
    args = parser.parse_args()
    target = args.frames if args.frames is not None else args.control_frames
    if target <= 0:
        parser.error("frame count must be positive")
    data = args.source.read_bytes()
    if data[:8] != b"DINOTRC1":
        raise ValueError("not a DINOTRC1 trace")
    size, = struct.unpack_from("<I", data, 8)
    header = json.loads(data[12:12 + size])
    start = pos = 12 + size
    block = header["block"]
    ram = bytearray(0x10000)
    control = None
    control_vblank = None
    def word(offset):
        return int.from_bytes(ram[offset:offset + 2], "big")
    while pos < len(data):
        frame, flags, vblank, lag = struct.unpack_from("<iIII", data, pos)
        pos += 16
        for region in header["regions"]:
            count, = struct.unpack_from("<I", data, pos)
            pos += 4
            for _ in range(count):
                index, = struct.unpack_from("<I", data, pos)
                pos += 4
                if region["name"] == "ram":
                    ram[index * block:(index + 1) * block] = data[pos:pos + block]
                pos += block
        extra, = struct.unpack_from("<I", data, pos)
        pos += 4 + extra
        if pos > len(data):
            raise ValueError("truncated record")
        if args.control_frames is not None and control is None:
            if (ram[0x84CC] and ram[0x84D0] == 0x0C and ram[0x84DD] == 0
                    and ram[0x84D9] == 0 and ram[0xB274] and ram[0xB277] == 0
                    and ram[0xB278:0xB27C] == b"\x02\0\0\0" and ram[0xB38E] == 0):
                control = word(0x8000)
                control_vblank = vblank
        done = (frame + 1 == args.frames if args.frames is not None else
                control is not None and ((word(0x8000) - control) & 0xFFFF) >= target)
        if done:
            header["frames"] = frame + 1
            if control is not None:
                header["control_vblank"] = control_vblank
                header["control_frames"] = target
            encoded = json.dumps(header, separators=(",", ":")).encode()
            args.output.write_bytes(b"DINOTRC1" + struct.pack("<I", len(encoded))
                                    + encoded + data[start:pos])
            print(f"{args.output}: {frame + 1} frames, end vblank {vblank}, "
                  f"control vblank {control_vblank}, {args.output.stat().st_size} bytes")
            return
    raise ValueError("trace ended before the requested prefix")


if __name__ == "__main__":
    main()
