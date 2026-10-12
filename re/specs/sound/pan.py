"""Pan motion of the QSound Z80 driver: a reference model in Python, checked against the real routines.

This is the executable form of SPEC.md section 9 (pan motion). `PanModel` is what the Rust port (M4-06) has to
reproduce. The check runs the ROM routines on z80mini.py (the real Kabuki decrypted bytes) and compares them with the
model for random parameters:
  * the setup routine 0x0251 (SFX start, modes 0..7, including the twin track of mode 7),
  * the per step handler 0x1007 -> table 0x101D (voice tracks) and the DSP register writes it makes,
  * the ADPCM handler table 0x2211 and the ADPCM setup table 0x05A8: they have the same logic on a slot of 0x20 bytes
    (every field 0x20 lower than in a track), which is checked by running both on the same memory image.

usage: python3 -I re/specs/sound/pan.py [--zip PATH]        (default: dino.zip in the repo root)
"""
import os
import random
import sys
import zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
sys.path.insert(0, HERE)
import kabuki  # noqa: E402
from z80mini import Z80  # noqa: E402

U16 = 0xFFFF


def neg16(v):
    return (-v) & U16


def s16(v):
    return v - 0x10000 if v & 0x8000 else v


class PanModel:
    """State of one track (or ADPCM slot) for the pan motion.

    mode   0 = no motion (field +0x36 of a track, +0x16 of a slot; it holds the request mode + 1, here the request mode + 1 too)
    p, f   pan (0..0x20) and its fraction (+0x37, +0x38)
    spread +0x39, lim +0x3A (target of the sweep), v = speed (+0x3B lo, +0x3C hi, signed 8.8 per step),
    v2 = second speed (+0x3D lo, +0x3E hi), delay (+0x3F, tracks only)
    """

    def __init__(self):
        self.mode = self.p = self.f = self.spread = self.lim = self.v = self.v2 = self.delay = 0

    def fields(self):
        return (self.mode, self.p, self.f, self.spread, self.lim, self.v, self.v2, self.delay)

    def copy(self):
        m = PanModel()
        m.__dict__.update(self.__dict__)
        return m

    # ---- setup (0x0251 and the routines of table 0x0269; ADPCM 0x05B8.. of table 0x05A8)
    @staticmethod
    def setup(mb):
        """mb: mailbox bytes C006 mode, C007 pan, C008 spread, C009:C00A speed, C00B:C00C second speed.
        Returns (primary, twin or None). The twin exists only in mode 7 (the caller still checks the priority)."""
        m = mb["mode"] & 7
        pan, spread = mb["pan"], mb["spread"]
        st = PanModel()
        st.mode = m + 1
        speed, speed2 = mb["speed"] & U16, mb["speed2"] & U16
        if m == 0:                                           # 0x0279: fixed pan
            st.mode = 0
            st.p = pan if pan < 0x21 else 0x20
        elif m == 1:                                         # 0x028A: bounce between 0 and 0x20
            v, a = speed, pan
            if a >= 0x11:
                a = 0x20 if a >= 0x21 else a
                v = neg16(speed)
            st.v, st.p = v, a
        elif m == 2:                                         # 0x02B4: swing between pan and pan +- spread
            v, b = speed, pan
            if b < 0x11:
                a = (b + spread) & 0xFF
                a = 0x20 if a >= 0x21 else a
            else:
                if b >= 0x21:
                    b = 0x20
                a = max(b - spread, 0)
                v = neg16(speed)
            st.lim, st.p, st.v, st.spread = a, b, v, spread
        elif m in (3, 5, 7):                                 # 0x02FD, 0x0353, 0x03C7: sweep up
            b = pan
            a = (b + spread) & 0xFF
            a = 0x20 if a >= 0x21 else a
            st.lim, st.p, st.v = a, b, speed
            if m == 5:
                st.v2 = speed2
        else:                                                # m == 4, 6: 0x0322, 0x0384: sweep down
            b = pan
            if b >= 0x21:
                b = 0x20
            st.lim, st.p, st.v = max(b - spread, 0), b, neg16(speed)
            if m == 6:
                st.v2 = neg16(speed2)
        st.f = 0
        twin = None
        if m == 7:                                           # twin on the next voice: same track, other direction
            twin = st.copy()
            b = pan if pan < 0x21 else 0x20
            twin.lim, twin.p, twin.f, twin.v, twin.delay = max(b - spread, 0), b, 0, neg16(speed), 4
        return st, twin

    # ---- one sequencer step (0x1007, table 0x101D / 0x2211). Returns the pan value written, or None.
    def step(self):
        if self.mode == 0:
            return None
        m = self.mode - 1
        if m == 0:                                           # 0x102D: jp 0x1196, nothing happens
            return None
        hl = ((self.p << 8) | self.f) + self.v & U16
        h, l = hl >> 8, hl & 0xFF
        neg = bool(self.v & 0x8000)
        if m == 1:                                           # 0x1030 bounce
            if h >= 0x21:
                h = 0 if neg else 0x20
                self.v = neg16(self.v)
        elif m == 2:                                         # 0x105C swing
            if not neg:
                if h >= self.lim:
                    h = self.lim
                    self.lim = max(h - self.spread, 0)
                    self.v = neg16(self.v)
            elif h <= self.lim or h >= 0x21:
                h = self.lim
                a = (h + self.spread) & 0xFF
                self.lim = 0x20 if a >= 0x21 else a
                self.v = neg16(self.v)
        elif m == 3:                                         # 0x10B4 sweep up, stop at lim
            if h >= self.lim:
                h, self.mode = self.lim, 0
        elif m == 4:                                         # 0x10CE sweep down, stop at lim
            if h <= self.lim or h >= 0x21:
                h, self.mode = self.lim, 0
        elif m == 5:                                         # 0x10F0 sweep up in two stages
            if h >= self.lim:
                h = self.lim
                if h == 0x20:
                    self.mode = 0
                else:
                    self.v, self.lim = self.v2, 0x20
        elif m == 6:                                         # 0x1120 sweep down in two stages
            if h <= self.lim or h >= 0x21:
                h = self.lim
                if h == 0:
                    self.mode = 0
                else:
                    self.v, self.lim = self.v2, 0
        else:                                                # m == 7, 0x1157 sweep to lim, either direction
            if not neg:
                if h >= self.lim:
                    h, self.mode = self.lim, 0
            elif h <= self.lim or h >= 0x21:
                h, self.mode = self.lim, 0
        self.p, self.f = h, l
        return h


# ---------------------------------------------------------------------------------------------------------
# Check against the ROM
# ---------------------------------------------------------------------------------------------------------
def load(zip_path):
    raw = zipfile.ZipFile(zip_path).read("cd_q.5k")
    return kabuki.decrypt(raw)


TRACK = {"mode": 0x36, "p": 0x37, "f": 0x38, "spread": 0x39, "lim": 0x3A, "v": 0x3B, "v2": 0x3D, "delay": 0x3F}
SLOT = {k: o - 0x20 for k, o in TRACK.items()}      # the slot has the same fields 0x20 lower (delay at +0x1F)


def put(cpu, base, offs, st):
    cpu.wr(base + offs["mode"], st.mode)
    cpu.wr(base + offs["p"], st.p)
    cpu.wr(base + offs["f"], st.f)
    cpu.wr(base + offs["spread"], st.spread)
    cpu.wr(base + offs["lim"], st.lim)
    cpu.wr16(base + offs["v"], st.v)
    cpu.wr16(base + offs["v2"], st.v2)
    if "delay" in offs:
        cpu.wr(base + offs["delay"], st.delay)


def get(cpu, base, offs):
    st = PanModel()
    st.mode, st.p, st.f = cpu.rd(base + offs["mode"]), cpu.rd(base + offs["p"]), cpu.rd(base + offs["f"])
    st.spread, st.lim = cpu.rd(base + offs["spread"]), cpu.rd(base + offs["lim"])
    st.v, st.v2 = cpu.rd16(base + offs["v"]), cpu.rd16(base + offs["v2"])
    st.delay = cpu.rd(base + offs["delay"]) if "delay" in offs else 0
    return st


def random_mailbox(rnd):
    return dict(mode=rnd.choice([0, 1, 2, 3, 4, 5, 6, 7, 5, 6, 7, 3, 4]),
                pan=rnd.choice([rnd.randrange(0, 0x21), rnd.randrange(0, 0x30), 0x10, 0, 0x20, 0x11, 0x0F]),
                spread=rnd.choice([0, rnd.randrange(0, 0x20), 0x12, 0x20]),
                speed=rnd.choice([rnd.randrange(0, 0x700), rnd.randrange(0, 0x40), 0x0380, 0x1500]),
                speed2=rnd.choice([0, rnd.randrange(0, 0x700), 0x1080]))


def main():
    zp = os.path.join(ROOT, "dino.zip")
    if "--zip" in sys.argv:
        zp = sys.argv[sys.argv.index("--zip") + 1]
    op, data = load(zp)
    rnd = random.Random(7)
    fails = []
    n_setup = n_step = n_slot = 0

    for case in range(4000):
        mb = random_mailbox(rnd)
        stereo = rnd.choice([0xFF, 0xFF, 0x00])
        voice = rnd.randrange(16)
        base = 0xF200 + 0x40 * voice
        twin_base = 0xF200 + 0x40 * ((voice + 1) & 15)
        prio, twin_prio = rnd.randrange(3, 11), rnd.randrange(0, 11)
        writes = []
        cpu = Z80(op, data)
        cpu.traps[0x0A13] = lambda c: writes.append((c.a, c.de))
        cpu.traps[0x1196] = lambda c: None                  # the LFO and envelope part starts here: stop
        cpu.wr(0xF040, stereo)
        cpu.wr(0xF00C, prio)
        cpu.wr(0xF00D, voice)
        cpu.wr(base + 0x32, prio)
        cpu.wr(twin_base + 0x32, twin_prio)
        cpu.wr(base + 0x20, 0xFF)
        iy = 0xF100
        for i, v in ((6, mb["mode"]), (7, mb["pan"]), (8, mb["spread"]), (9, mb["speed"] >> 8),
                     (10, mb["speed"] & 0xFF), (11, mb["speed2"] >> 8), (12, mb["speed2"] & 0xFF)):
            cpu.wr(iy + i, v)
        cpu.run(0x0251, ix=base, iy=iy)
        # model
        prim, twin = PanModel.setup(mb)
        n_setup += 1
        if get(cpu, base, TRACK).fields() != prim.fields():
            fails.append(("setup", mb, get(cpu, base, TRACK).fields(), prim.fields()))
            continue
        twin_expected = twin is not None and prio >= twin_prio
        twin_rom_active = cpu.rd(twin_base + 0x3F) == 4 and (mb["mode"] & 7) == 7 and twin_base != base
        if twin_expected != twin_rom_active:
            fails.append(("twin presence", mb, prio, twin_prio))
            continue
        if twin_expected and get(cpu, twin_base, TRACK).fields() != twin.fields():
            fails.append(("twin", mb, get(cpu, twin_base, TRACK).fields(), twin.fields()))
            continue
        # steps: run the ROM handler and the model for the primary and (if any) the twin
        for who, st, b, vv in [("primary", prim, base, voice)] + ([("twin", twin, twin_base, (voice + 1) & 15)]
                                                                  if twin_expected else []):
            cpu.wr(0xF00D, vv)
            for k in range(300):
                writes.clear()
                cpu.run(0x1007, ix=b)
                exp = st.step()
                n_step += 1
                got = [(a, de) for a, de in writes if 0x80 <= a < 0x90]
                if exp is None:
                    ok = not got
                else:
                    e = exp if stereo else 0x10
                    ok = got == [(0x80 + vv, 0x110 + e)]
                if not ok or get(cpu, b, TRACK).fields() != st.fields():
                    fails.append((f"step {k} {who}", mb, got, exp, get(cpu, b, TRACK).fields(), st.fields()))
                    break

    # ADPCM: the same logic with every field 0x20 lower. Run the slot handler on a copy of the same memory image.
    for case in range(1500):
        mb = random_mailbox(rnd)
        mb["mode"] = mb["mode"] & 7
        slot = rnd.randrange(3)
        sbase = 0xFA00 + 0x20 * slot
        writes = []
        cpu = Z80(op, data)
        cpu.traps[0x0A13] = lambda c: writes.append((c.a, c.de))
        cpu.wr(0xF040, 0xFF)
        cpu.wr(0xF00C, 5)
        cpu.wr(0xF00D, slot)
        iy = 0xF100
        for i, v in ((6, mb["mode"]), (7, mb["pan"]), (8, mb["spread"]), (9, mb["speed"] >> 8),
                     (10, mb["speed"] & 0xFF), (11, mb["speed2"] >> 8), (12, mb["speed2"] & 0xFF)):
            cpu.wr(iy + i, v)
        # setup: ix+0x16 = mode+1, then the routine of table 0x05A8
        cpu.wr(sbase + 0x16, (mb["mode"] & 7) + 1)
        target = cpu.rd16(0x05A8 + 2 * (mb["mode"] & 7))
        cpu.run(target, ix=sbase, iy=iy)
        prim, twin = PanModel.setup(mb)
        n_slot += 1
        got = get(cpu, sbase, SLOT)
        exp = prim.copy()
        exp.delay = 0
        if got.fields() != exp.fields():
            fails.append(("slot setup", mb, got.fields(), exp.fields()))
            continue
        if twin is not None:                                # mode 7: the next slot (2 wraps to 0) gets the twin
            tb = 0xFA00 + 0x20 * ((slot + 1) % 3)
            if get(cpu, tb, SLOT).fields() != twin.fields():
                fails.append(("slot twin", mb, get(cpu, tb, SLOT).fields(), twin.fields()))
                continue
        for k in range(300):
            writes.clear()
            cpu.run(0x21FD, ix=sbase)                       # `a = (ix+0x16); ret z; jp (table 0x2211)`
            e = prim.step()
            n_slot += 1
            got = [(a, de) for a, de in writes if 0x90 <= a < 0x94]
            ok = (not got) if e is None else got == [(0x90 + slot, 0x110 + e)]
            if cpu.rd(sbase + 0x16) == 0 and e is None:
                ok = True
            st = get(cpu, sbase, SLOT)
            pm = prim.copy()
            pm.delay = 0
            if not ok or st.fields() != pm.fields():
                fails.append((f"slot step {k}", mb, got, e, st.fields(), pm.fields()))
                break

    print(f"voice setups {n_setup}, voice steps {n_step}, ADPCM slot setups and steps {n_slot}")
    if fails:
        print(f"FAIL {len(fails)} cases, first: {fails[0]}")
        sys.exit(1)
    print("ok   the ROM routines and PanModel agree on every case (setup, twin of mode 7, steps, register writes, ADPCM slots)")


if __name__ == "__main__":
    main()
