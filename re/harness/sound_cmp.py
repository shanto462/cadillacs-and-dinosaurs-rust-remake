"""Compare QSound register traces (M4-09). Original: none (tool).

MAME (re/harness/probe/sound.sh) and the port write the same text. The port
example dump_regs CODE [d1] [d2] belongs to crates/dino-sound, which is not
built yet (M4-02 onward). Until then, point both sides at .qs.trc files.

    # sound code=0030 d1=00000000 d2=00000000 frames=240
    f <frame> r <hh> v <hhhh>     commit on Z80 port 0xD002 (helper 0x0A13)
    f <frame> b <hh>              write to Z80 port 0xD003 (ROM bank)

Frame 0 is the first video frame after the ring write. Blank lines and
comments other than the header are ignored.

    re/.venv/bin/python -I re/harness/sound_cmp.py MAME.qs.trc PORT.qs.trc
    re/.venv/bin/python -I re/harness/sound_cmp.py --align-first A.qs.trc B.qs.trc
    re/.venv/bin/python -I re/harness/sound_cmp.py --mame DIR --port DIR 0x00 0x30
    re/.venv/bin/python -I re/harness/sound_cmp.py --mame DIR --port DIR --list codes.txt

A list line is `sound CODE [d1] [d2]`, `CODE`, or `CODE:d1:d2`. Numbers are
decimal or 0x hex, the same rule as sound.sh. The file name is CODE.qs.trc
(four hex digits), or CODE_D1_D2.qs.trc when d1 or d2 is not 0.

The report names the first write that differs: its index, frame, register
(or bank), both values, and a few writes around it. It also gives the
difference count and the time of that write. Time is the video frame. The
millisecond figure uses 59.6374 Hz (MAME, sound spec section 2).

--align-first subtracts a constant from every port frame so the first
register commit (an r line, not a bank write) lands on the MAME frame.
M4-01: one code recorded alone and the same code recorded after another code
can sit about one video frame apart. The option removes that constant offset.
A different value still counts. If either side has no register write, frames
are left as they are.

Exit 0 when every pair matches. Exit 1 when a write differs. Exit 2 when an
argument, a path, or a trace line is wrong.
"""

import argparse
import os
import re
import sys

# MAME video rate for dino (sound spec section 2). Display only.
VIDEO_HZ = 59.6374

HEADER_RE = re.compile(
    r"^#\s*sound\s+code=([0-9A-Fa-f]+)\s+d1=([0-9A-Fa-f]+)\s+d2=([0-9A-Fa-f]+)"
    r"(?:\s+frames=(\d+))?\s*$",
    re.I,
)
EVENT_RE = re.compile(
    r"^f\s+(-?\d+)\s+(?:r\s+([0-9A-Fa-f]+)\s+v\s+([0-9A-Fa-f]+)|b\s+([0-9A-Fa-f]+))\s*$",
    re.I,
)

# Voice field of register v*8+k (SPEC section 8). k == 0 is the sample bank,
# and voice 0's bank register is 0x78, not 0.
VOICE_FIELD = {1: "start", 2: "pitch", 3: "phase", 4: "loop", 5: "end", 6: "volume"}
ADPCM_FIELD = ("start", "end", "bank", "volume")


class ParseError(Exception):
    pass


class Ev(object):
    __slots__ = ("frame", "kind", "reg", "value", "line")

    def __init__(self, frame, kind, reg, value, line):
        self.frame = frame
        self.kind = kind  # "r" register commit, "b" ROM bank
        self.reg = reg
        self.value = value  # None for a bank write
        self.line = line

    def copy(self):
        return Ev(self.frame, self.kind, self.reg, self.value, self.line)

    def same(self, other):
        return (
            self.frame == other.frame
            and self.kind == other.kind
            and self.reg == other.reg
            and self.value == other.value
        )


class Trace(object):
    def __init__(self, path):
        self.path = path
        self.code = None
        self.d1 = None
        self.d2 = None
        self.window = None
        self.events = []

    def header_text(self):
        if self.code is None:
            return None
        text = "code %04x d1 %08x d2 %08x" % (self.code, self.d1, self.d2)
        if self.window is not None:
            text += " frames %d" % self.window
        return text


def hex_min(n, width):
    if n < 0:
        return "-%x" % (-n)
    return "%0*x" % (width, n)


def fmt_ev(ev):
    if ev is None:
        return "<end>"
    if ev.kind == "b":
        return "f %d b %s" % (ev.frame, hex_min(ev.reg, 2))
    return "f %d r %s v %s" % (ev.frame, hex_min(ev.reg, 2), hex_min(ev.value, 4))


def role(ev):
    """Short name for a write. Register numbers follow SPEC section 8."""
    if ev is None:
        return ""
    if ev.kind == "b":
        return "rom bank %d" % (ev.reg & 0x7F)
    reg = ev.reg
    if 0 <= reg <= 0x7F:
        if reg % 8 == 0:
            voice = (reg // 8 + 1) & 15
            return "voice %d bank" % voice
        name = VOICE_FIELD.get(reg % 8)
        if name:
            return "voice %d %s" % (reg // 8, name)
        return ""
    if 0x80 <= reg <= 0x8F:
        return "voice %d pan" % (reg - 0x80)
    if 0x90 <= reg <= 0x92:
        return "adpcm %d pan" % (reg - 0x90)
    if reg == 0x93:
        return "echo feedback"
    if 0xBA <= reg <= 0xC9:
        return "voice %d echo" % (reg - 0xBA)
    if 0xCA <= reg <= 0xD5:
        n = (reg - 0xCA) // 4
        return "adpcm %d %s" % (n, ADPCM_FIELD[(reg - 0xCA) % 4])
    if 0xD6 <= reg <= 0xD8:
        return "adpcm %d gate" % (reg - 0xD6)
    if reg == 0xD9:
        return "echo length"
    if 0xDA <= reg <= 0xE7:
        return "init"
    return ""


def parse_num(text):
    try:
        return int(text, 0)
    except ValueError:
        raise ValueError("bad number %r" % text)


def parse_request(text):
    """CODE, CODE:d1[:d2], or 'sound CODE [d1] [d2]'."""
    parts = text.split()
    if parts and parts[0].lower() == "sound":
        parts = parts[1:]
    if not parts:
        raise ValueError("missing code")
    if len(parts) == 1:
        fields = parts[0].split(":")
        if len(fields) > 3:
            raise ValueError("bad code %r" % text)
        while len(fields) < 3:
            fields.append("0")
        code, d1, d2 = (parse_num(x) for x in fields)
    else:
        nums = [parse_num(p) for p in parts[:3]]
        while len(nums) < 3:
            nums.append(0)
        code, d1, d2 = nums[:3]
    return code & 0xFFFF, d1 & 0xFFFFFFFF, d2 & 0xFFFFFFFF


def trace_filename(code, d1, d2):
    name = "%04x" % (code & 0xFFFF)
    if d1 or d2:
        name = "%s_%08x_%08x" % (name, d1 & 0xFFFFFFFF, d2 & 0xFFFFFFFF)
    return name + ".qs.trc"


def load_trace(path):
    tr = Trace(path)
    try:
        fh = open(path, "r", encoding="utf-8")
    except OSError as err:
        raise ParseError("cannot read %s: %s" % (path, err))
    with fh:
        for n, raw in enumerate(fh, 1):
            line = raw.strip()
            if not line:
                continue
            if line.startswith("#"):
                m = HEADER_RE.match(line)
                if m and tr.code is None:
                    tr.code = int(m.group(1), 16)
                    tr.d1 = int(m.group(2), 16)
                    tr.d2 = int(m.group(3), 16)
                    if m.group(4) is not None:
                        tr.window = int(m.group(4))
                continue
            m = EVENT_RE.match(line)
            if not m:
                raise ParseError("%s:%d: bad trace line: %s" % (path, n, line))
            frame = int(m.group(1))
            if m.group(4) is not None:
                tr.events.append(Ev(frame, "b", int(m.group(4), 16), None, n))
            else:
                tr.events.append(
                    Ev(frame, "r", int(m.group(2), 16), int(m.group(3), 16), n)
                )
    return tr


def first_register(events):
    for ev in events:
        if ev.kind == "r":
            return ev
    return None


def align_port(mame, port):
    """Shift every port frame so the first register writes share a frame.

    Returns (events, shift, note). shift is the amount added to each port
    frame (negative when the port trace starts later).
    """
    a = first_register(mame)
    b = first_register(port)
    if a is None or b is None:
        missing = []
        if a is None:
            missing.append("mame")
        if b is None:
            missing.append("port")
        if len(missing) == 2:
            note = "align: no register write on either side, frames not shifted"
        else:
            note = "align: no register write on %s, frames not shifted" % missing[0]
        return port, 0, note
    delta = b.frame - a.frame
    if delta == 0:
        note = "align: first register writes already share frame %d" % a.frame
        return port, 0, note
    shifted = []
    for ev in port:
        copy = ev.copy()
        copy.frame -= delta
        shifted.append(copy)
    note = (
        "align: port frame numbers %+d "
        "(first register write was mame frame %d, port frame %d)"
        % (-delta, a.frame, b.frame)
    )
    return shifted, -delta, note


def count_kinds(events):
    regs = 0
    for ev in events:
        if ev.kind == "r":
            regs += 1
    return regs, len(events) - regs


def first_difference(mame, port):
    """Return (count, index). index is 0-based, or None when the traces match.

    Extra writes on the longer side each count as one difference. index points
    at the first one. If that side has ended, index == len(that side).
    """
    n = min(len(mame), len(port))
    count = 0
    index = None
    for i in range(n):
        if not mame[i].same(port[i]):
            count += 1
            if index is None:
                index = i
    extra = abs(len(mame) - len(port))
    if extra:
        count += extra
        if index is None:
            index = n
    return count, index


def frame_clock(frame):
    return "frame %d (%.1f ms)" % (frame, frame * 1000.0 / VIDEO_HZ)


def event_at(events, index):
    if index is None or index >= len(events):
        return None
    return events[index]


def register_text(ev):
    if ev is None:
        return "<end>"
    if ev.kind == "b":
        return "bank %s (%s)" % (hex_min(ev.reg, 2), role(ev))
    label = role(ev)
    if label:
        return "%s (%s)" % (hex_min(ev.reg, 2), label)
    return hex_min(ev.reg, 2)


def value_text(ev):
    if ev is None:
        return "<end>"
    if ev.kind == "b":
        return hex_min(ev.reg, 2)
    return hex_min(ev.value, 4)


def describe_pair(mame_ev, port_ev):
    lines = []
    if mame_ev is not None and port_ev is not None and mame_ev.frame == port_ev.frame:
        lines.append("time: %s" % frame_clock(mame_ev.frame))
    else:
        left = "<end>" if mame_ev is None else frame_clock(mame_ev.frame)
        right = "<end>" if port_ev is None else frame_clock(port_ev.frame)
        lines.append("time: mame %s, port %s" % (left, right))
    same_reg = (
        mame_ev is not None
        and port_ev is not None
        and mame_ev.kind == port_ev.kind == "r"
        and mame_ev.reg == port_ev.reg
    )
    same_bank = (
        mame_ev is not None
        and port_ev is not None
        and mame_ev.kind == port_ev.kind == "b"
    )
    if same_reg:
        lines.append("register: %s" % register_text(mame_ev))
    elif same_bank:
        lines.append("register: bank")
    else:
        lines.append(
            "register: mame %s, port %s" % (register_text(mame_ev), register_text(port_ev))
        )
    lines.append("values: mame %s, port %s" % (value_text(mame_ev), value_text(port_ev)))
    lines.append("mame: %s" % fmt_ev(mame_ev))
    lines.append("port: %s" % fmt_ev(port_ev))
    return lines


def context_lines(label, events, index, window):
    lines = ["  %s" % label]
    if not events:
        lines.append("    > <end>")
        return lines
    if index >= len(events):
        start = max(0, len(events) - window)
        for i in range(start, len(events)):
            lines.append(format_context(i, events[i], False))
        lines.append("    > <end>")
        return lines
    start = max(0, index - window)
    end = min(len(events), index + window + 1)
    for i in range(start, end):
        lines.append(format_context(i, events[i], i == index))
    return lines


def format_context(index, ev, mark):
    flag = ">" if mark else " "
    extra = role(ev)
    if extra:
        extra = "  " + extra
    return "    %s %d  %s%s" % (flag, index + 1, fmt_ev(ev), extra)


def span_text(events):
    if not events:
        return "no writes"
    lo = min(ev.frame for ev in events)
    hi = max(ev.frame for ev in events)
    return "frames %d to %d" % (lo, hi)


def count_word(n, one, many):
    return "%d %s" % (n, one if n == 1 else many)


def writes_text(events):
    regs, banks = count_kinds(events)
    return "%d writes (%d register, %d bank)" % (len(events), regs, banks)


def quote_token(text):
    if text == "-" or re.match(r"^[A-Za-z0-9_.:@+-]+$", text):
        return text
    return '"%s"' % text


def result_line(diff_n, mame, port, index, align):
    mame_ev = event_at(mame, index)
    port_ev = event_at(port, index)

    def frame_of(ev):
        return "-" if ev is None else str(ev.frame)

    def shown(ev):
        return "-" if ev is None else fmt_ev(ev)

    first = "-" if index is None else str(index + 1)
    return (
        "result differences=%d writes_mame=%d writes_port=%d first_write=%s "
        "frame_mame=%s frame_port=%s mame=%s port=%s align=%d"
        % (
            diff_n,
            len(mame),
            len(port),
            first,
            frame_of(mame_ev),
            frame_of(port_ev),
            quote_token(shown(mame_ev)),
            quote_token(shown(port_ev)),
            align,
        )
    )


def compare_traces(mame_tr, port_tr, align, context):
    """Return (lines, difference count). Port frames are shifted when align is set."""
    lines = []
    lines.append("mame: %s" % mame_tr.path)
    lines.append("port: %s" % port_tr.path)
    mh = mame_tr.header_text()
    ph = port_tr.header_text()
    if mh:
        lines.append("header mame: %s" % mh)
    if ph:
        lines.append("header port: %s" % ph)
    port_events = port_tr.events
    shift = 0
    if align:
        port_events, shift, note = align_port(mame_tr.events, port_tr.events)
        lines.append(note)
    diff_n, index = first_difference(mame_tr.events, port_events)
    if diff_n == 0:
        lines.append(
            "equal: %s, 0 differences, writes on %s"
            % (writes_text(mame_tr.events), span_text(mame_tr.events))
        )
    else:
        lines.append(
            "differ: %s, mame %s, port %s"
            % (
                count_word(diff_n, "difference", "differences"),
                writes_text(mame_tr.events),
                writes_text(port_tr.events),
            )
        )
        lines.append("first: write %d" % (index + 1))
        mame_ev = event_at(mame_tr.events, index)
        port_ev = event_at(port_events, index)
        lines.extend(describe_pair(mame_ev, port_ev))
        lines.append("context:")
        lines.extend(context_lines("mame", mame_tr.events, index, context))
        lines.extend(context_lines("port", port_events, index, context))
    lines.append(result_line(diff_n, mame_tr.events, port_events, index, shift))
    return lines, diff_n


def read_list(path):
    items = []
    try:
        fh = open(path, "r", encoding="utf-8")
    except OSError as err:
        raise ParseError("cannot read %s: %s" % (path, err))
    with fh:
        for n, raw in enumerate(fh, 1):
            line = raw.split("#", 1)[0].strip()
            if not line:
                continue
            try:
                items.append(parse_request(line))
            except ValueError as err:
                raise ParseError("%s:%d: %s" % (path, n, err))
    return items


def build_parser():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "paths",
        nargs="*",
        help="two .qs.trc files, or codes when --mame and --port are set",
    )
    ap.add_argument("--mame", metavar="DIR", help="directory of MAME .qs.trc files")
    ap.add_argument("--port", metavar="DIR", help="directory of port .qs.trc files")
    ap.add_argument("--list", metavar="FILE", help="codes to compare, one request per line")
    ap.add_argument(
        "--align-first",
        action="store_true",
        help="shift port frames so the first register write matches",
    )
    ap.add_argument(
        "--context",
        type=int,
        default=2,
        help="writes to show before and after the first difference (default 2)",
    )
    return ap


def main(argv=None):
    ap = build_parser()
    args = ap.parse_args(argv)
    if args.context < 0:
        print("sound_cmp: --context must be >= 0", file=sys.stderr)
        return 2

    batch = args.mame is not None or args.port is not None or args.list is not None
    if batch:
        if not args.mame or not args.port:
            print("sound_cmp: batch mode needs --mame DIR and --port DIR", file=sys.stderr)
            return 2
        if not os.path.isdir(args.mame):
            print("sound_cmp: not a directory: %s" % args.mame, file=sys.stderr)
            return 2
        if not os.path.isdir(args.port):
            print("sound_cmp: not a directory: %s" % args.port, file=sys.stderr)
            return 2
        codes = []
        if args.list:
            try:
                codes.extend(read_list(args.list))
            except ParseError as err:
                print("sound_cmp: %s" % err, file=sys.stderr)
                return 2
        for token in args.paths:
            try:
                codes.append(parse_request(token))
            except ValueError as err:
                print("sound_cmp: %s" % err, file=sys.stderr)
                return 2
        if not codes:
            print("sound_cmp: batch mode needs at least one code", file=sys.stderr)
            return 2
        return run_batch(args, codes)

    if len(args.paths) != 2:
        ap.print_usage(sys.stderr)
        print("sound_cmp: give two trace files, or --mame and --port with codes", file=sys.stderr)
        return 2
    try:
        mame_tr = load_trace(args.paths[0])
        port_tr = load_trace(args.paths[1])
    except ParseError as err:
        print("sound_cmp: %s" % err, file=sys.stderr)
        return 2
    lines, diff_n = compare_traces(mame_tr, port_tr, args.align_first, args.context)
    print("\n".join(lines))
    return 1 if diff_n else 0


def run_batch(args, codes):
    differ = 0
    errors = 0
    blocks = []
    for code, d1, d2 in codes:
        name = trace_filename(code, d1, d2)
        mame_path = os.path.join(args.mame, name)
        port_path = os.path.join(args.port, name)
        title = "code %s" % name[:-len(".qs.trc")]
        if not os.path.isfile(mame_path) or not os.path.isfile(port_path):
            errors += 1
            missing = []
            if not os.path.isfile(mame_path):
                missing.append("mame %s" % mame_path)
            if not os.path.isfile(port_path):
                missing.append("port %s" % port_path)
            blocks.append(title + "\nmissing: " + ", ".join(missing))
            continue
        try:
            mame_tr = load_trace(mame_path)
            port_tr = load_trace(port_path)
        except ParseError as err:
            errors += 1
            blocks.append(title + "\nerror: %s" % err)
            continue
        lines, diff_n = compare_traces(mame_tr, port_tr, args.align_first, args.context)
        if diff_n:
            differ += 1
        blocks.append(title + "\n" + "\n".join(lines))
    blocks.append(
        "batch: %s, %d differ, %d missing" % (count_word(len(codes), "code", "codes"), differ, errors)
    )
    print("\n\n".join(blocks))
    if errors:
        return 2
    return 1 if differ else 0


if __name__ == "__main__":
    sys.exit(main())
