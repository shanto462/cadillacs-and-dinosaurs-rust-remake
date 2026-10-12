"""Decodes the sound sequences of the Z80 driver into readable listings (no emulator involved).

Format (re/specs/sound/SPEC.md section 7, checked against the Z80 listing at 0x0A28 and 0x0C76):
  header at the code table pointer: byte h (0 = music, 1..0x7F = SFX with priority h), then 16 big endian words,
  one per QSound voice: offset of that voice's track from the header (0 = unused).
  track byte >= 0x20: note. length index = (b >> 5) - 1 into the length tables, pitch = (b & 0x1F) + base (0 = rest).
  track byte < 0x20: command, operands follow (see CMD_ARGS).
Time: the sequencer steps 62.5 times per second (every 4th tick of the 250 Hz Z80 interrupt). A note adds
length*256 to a counter; every step subtracts the tempo word, so a note lasts length*256/tempo steps.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

LEN_NORMAL = (3, 6, 12, 24, 48, 96, 192)       # table 0x0C57
LEN_TRIPLET = (2, 4, 8, 16, 32, 64, 128)       # table 0x0C50
LEN_DOT = (0, 9, 18, 36, 72, 144, 0)           # table 0x0C5E
NOTE_BASE = (0, 12, 24, 36, 48, 60, 72, 84, 24, 36, 48, 60, 72, 84, 96, 108)   # table 0x0C65 by flags & 0xF
STEPS_PER_SECOND = 62.5
NAMES = ["c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a", "a#", "b"]
# command: (name, operand bytes)
CMD = {
    0x00: ("triplet toggle", 0), 0x01: ("tie toggle", 0), 0x02: ("dot next note", 0), 0x03: ("octave bank toggle", 0),
    0x04: ("flags", 1), 0x05: ("tempo", 2), 0x06: ("gate", 1), 0x07: ("volume", 1), 0x08: ("instrument", 1),
    0x09: ("octave", 1), 0x0A: ("music transpose", 1), 0x0B: ("transpose", 1), 0x0C: ("detune", 1), 0x0D: ("portamento", 1),
    0x0E: ("loop0", 3), 0x0F: ("loop1", 3), 0x10: ("loop2", 3), 0x11: ("loop3", 3),
    0x12: ("break0", 3), 0x13: ("break1", 3), 0x14: ("break2", 3), 0x15: ("break3", 3),
    0x16: ("jump", 2), 0x17: ("end", 0), 0x18: ("pan", 1), 0x19: ("echo send", 1), 0x1A: ("music volume", 1),
    0x1B: ("vibrato depth", 1), 0x1C: ("tremolo depth", 1), 0x1D: ("LFO speed", 1), 0x1E: ("LFO flags", 1), 0x1F: ("unused", 0),
}


def s16(hi, lo):
    v = (hi << 8) | lo
    return v - 0x10000 if v & 0x8000 else v


def note_name(p):
    if p <= 0:
        return "?"
    return f"{NAMES[(p - 1) % 12]}{(p - 1) // 12}"


class Track:
    def __init__(self, z, voice, ptr, is_music, sfx_tempo=0):
        self.z, self.voice, self.start, self.ptr = z, voice, ptr, ptr
        self.is_music = is_music
        self.flags = 0
        self.tempo = sfx_tempo
        self.t = 0.0                    # steps
        self.counters = [0, 0, 0, 0]
        self.done = False
        self.loop_marks = {}            # jump target -> time of first arrival
        self.events = []                # (time, text)
        self.instruments = []
        self.notes = 0
        self.intro = None
        self.loop_len = None
        self.end_time = None
        self.warn = []
        self.count = 0
        self.unit_left = 0.0

    def byte(self):
        b = self.z[self.ptr]
        self.ptr += 1
        return b


def decode_code(z, ptr, global_tempo_holder, max_events=60000, max_steps=3000.0):
    """Simulates every voice of a header. Returns dict(kind, priority, tracks=[Track...])."""
    h = z[ptr]
    is_music = h == 0
    offs = [(z[ptr + 1 + 2 * v] << 8) | z[ptr + 2 + 2 * v] for v in range(16)]
    tracks = [Track(z, v, ptr + o, is_music) for v, o in enumerate(offs) if o]
    tempo = {"music": 0}
    # discrete event simulation: always advance the track with the smallest clock so tempo changes act in time order
    active = list(tracks)
    while active:
        tr = min(active, key=lambda t: t.t)
        if tr.done or tr.count > max_events or tr.t > max_steps:
            if not tr.done and tr.count > max_events:
                tr.warn.append("event limit")
            if not tr.done and tr.t > max_steps:
                tr.warn.append(f"stopped after {max_steps / STEPS_PER_SECOND:.0f} s")
            tr.done = True
            active.remove(tr)
            continue
        step(tr, tempo, is_music)
        if tr.done:
            active.remove(tr)
    return dict(kind="music" if is_music else "sfx", priority=h if not is_music else None, tracks=tracks, header=ptr)


def step(tr, tempo, is_music):
    z = tr.z
    tr.count += 1
    at = tr.ptr
    b = tr.byte()
    cur_tempo = tempo["music"] if is_music else tr.tempo
    if b >= 0x20:
        idx = (b >> 5) - 1
        f = tr.flags
        if f & 0x10:
            units = LEN_DOT[idx]
            tr.flags &= ~0x10
            kind = "."
        elif f & 0x20:
            units = LEN_TRIPLET[idx]
            kind = "t"
        else:
            units = LEN_NORMAL[idx]
            kind = ""
        n5 = b & 0x1F
        if n5 == 0:
            txt = f"r{units}{kind}"
        else:
            p = n5 + NOTE_BASE[f & 0xF]
            txt = f"{note_name(p)}:{units}{kind}" + ("~" if f & 0x40 else "")
            tr.notes += 1
        tr.events.append((tr.t, txt))
        if cur_tempo <= 0:
            if "tempo 0" not in tr.warn:
                tr.warn.append("tempo 0 at first note")
            tr.t += units * 256.0 / 256.0      # unknown tempo: assume 0x0100
        else:
            tr.t += units * 256.0 / cur_tempo
        return
    name, n = CMD[b]
    args = [tr.byte() for _ in range(n)]
    if name == "tempo":
        v = (args[0] << 8) | args[1]
        if is_music:
            tempo["music"] = v
        else:
            tr.tempo = v
        tr.events.append((tr.t, f"tempo={v:#06x}"))
    elif name == "flags":
        tr.flags = (tr.flags & 0x97) | args[0]
        tr.events.append((tr.t, f"flags={args[0]:#04x}"))
    elif name == "octave":
        tr.flags = (tr.flags & 0xF8) | args[0]
        tr.events.append((tr.t, f"oct={args[0]}"))
    elif name == "triplet toggle":
        tr.flags ^= 0x20
        tr.events.append((tr.t, "triplet"))
    elif name == "tie toggle":
        tr.flags ^= 0x40
        tr.events.append((tr.t, "tie"))
    elif name == "dot next note":
        tr.flags |= 0x10
    elif name == "octave bank toggle":
        tr.flags ^= 0x08
        tr.events.append((tr.t, "bank"))
    elif name == "instrument":
        if args[0] not in tr.instruments:
            tr.instruments.append(args[0])
        tr.events.append((tr.t, f"inst={args[0]}"))
    elif name.startswith("loop"):
        k = int(name[4])
        off = s16(args[1], args[2])
        if tr.counters[k] == 0:
            tr.counters[k] = args[0]
            tr.ptr += off
        else:
            tr.counters[k] -= 1
            if tr.counters[k] != 0:
                tr.ptr += off
            else:
                pass
        tr.events.append((tr.t, f"loop{k}x{args[0]}"))
    elif name.startswith("break"):
        k = int(name[5])
        if tr.counters[k] == 1:
            tr.counters[k] = 0
            tr.flags = (tr.flags & 0x97) | args[0]
            tr.ptr += s16(args[1], args[2])
            tr.events.append((tr.t, f"break{k}"))
    elif name == "jump":
        target = tr.ptr + s16(args[0], args[1])
        tr.events.append((tr.t, f"jump->{target:#x}"))
        if target in tr.loop_marks:
            tr.intro = tr.loop_marks[target] if tr.intro is None else tr.intro
            tr.loop_len = tr.t - tr.loop_marks[target]
            tr.end_time = tr.t
            tr.done = True
            return
        tr.loop_marks[target] = tr.t
        tr.ptr = target
    elif name == "end":
        tr.end_time = tr.t
        tr.done = True
        tr.events.append((tr.t, "END"))
    else:
        v = args[0] if args else None
        tr.events.append((tr.t, f"{name}={v}" if v is not None else name))


def fmt_tracks(res, instr_info):
    """Text block for one decoded code."""
    lines = []
    for tr in res["tracks"]:
        steps = tr.end_time if tr.end_time is not None else tr.t
        if tr.loop_len is not None:
            ln = f"intro {tr.intro / STEPS_PER_SECOND:.2f} s then loops every {tr.loop_len / STEPS_PER_SECOND:.2f} s ({tr.intro:.0f} + {tr.loop_len:.0f} steps)"
        else:
            ln = f"{steps / STEPS_PER_SECOND:.2f} s ({steps:.0f} steps, about {steps / STEPS_PER_SECOND * 60:.0f} frames at 60 Hz)"
        insts = ", ".join(f"{i}(sample {instr_info(i)})" for i in tr.instruments)
        lines.append(f"  voice {tr.voice:2d}  track @{tr.start:06X}  {tr.notes} notes  length {ln}  instruments: {insts or '-'}" + (f"  WARN {'; '.join(tr.warn)}" if tr.warn else ""))
        # compact event list
        row = [txt for t, txt in tr.events[:400]]
        if len(tr.events) > 400:
            row.append(f"... ({len(tr.events) - 400} more events)")
        text = " ".join(row)
        # wrap
        w = 150
        cur = "      "
        for tok in text.split(" "):
            if len(cur) + len(tok) + 1 > w:
                lines.append(cur)
                cur = "      "
            cur += tok + " "
        lines.append(cur)
    return lines
