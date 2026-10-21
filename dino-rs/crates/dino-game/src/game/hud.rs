//! HUD on the scroll1 text layer and text printing. Owner: flow / HUD port.
//! Spec: `re/specs/flow/SPEC.md` sections 2-4.
//!
//! The HUD code of the original writes words into scroll1 VRAM at absolute addresses
//! (0x908000..0x90BFFF; a cell is a code word then an attribute word; +4 = next row,
//! +0x80 = next column, +0x2000 = row 32). The port keeps those addresses (`s1_*` helpers), so
//! each routine reads like the disassembly. In game the scroll1 camera is (0, 0), so rows
//! 34..61 are on screen: row 34 is the top line (screen y = (row - 34) * 8).
//!
//! Entry points for other ports:
//! * `hud_hit` (0x16B18), `hud_hit_credit` (0x16B36): show an enemy's name and vitality bar in the
//!   HUD of the player who hit it (call where the original calls them, with the same `kind`).
//! * `hud_food` (0x1693C kind 3), `hud_treasure` (0x16A5A kind 4): item messages.
//! * `hud_add_score` (0x10036): add a BCD score value to the player credited by `who` (+$02).
//! * `hud_life_bar(i)` (0x16446): redraw player i's vitality bar after its HP changed.
//! * `g.hud.boss` (G.-$70BC): a boss sets it to its own handle; the idle HUD then shows its bar.
//! * Score, lives, high score place: `g.players[i]` (+$100, +$A3, +$10A); `g.hud.stats[i]`
//!   keeps +$110 and +$114.

use super::vram::PaletteRam;
use super::{Game, Handle, Slot, slot_of};

/// Scroll1 VRAM base address.
const S1: u32 = 0x908000;

/// Player fields only the HUD and the flow use, kept here (the original stores them in the
/// player object at the given offsets). Score (+$100), lives (+$A3), high score place (+$10A),
/// extra life given (+$10E) and +$11C are fields of `Player` (one copy, as in the original).
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// +$110 score when the extra-life check last ran (0x6B98).
    pub score_prev: u32,
    /// +$114 low vitality blink active.
    pub low_hp: u8,
}

/// One entry of the high score table at G.$703E (16 bytes; only the fields the code reads).
#[derive(Clone, Copy, Debug, Default)]
pub struct HiScore {
    /// +0 score (BCD).
    pub score: u32,
    /// +4 initials.
    pub name: [u8; 4],
    /// +8 character, +9 stage, +$A unknown (100, 98, ... in the default table).
    pub chr: u8,
    pub stage: u8,
    pub b0a: u8,
}

/// Per-player HUD slot (G.$6BA8 / $6BB8 / $6BC8).
#[derive(Clone, Copy, Debug, Default)]
pub struct HudSlot {
    /// +0 player index.
    pub idx: u8,
    /// +1 frames to wait before the next message, +2 message step (0 first draw, 1 final draw).
    pub delay: u8,
    pub step: u8,
    /// +3 display timer (60 frames), +4 kind of the message on display (1 = follow the object).
    pub timer: u8,
    pub kind: u8,
    /// +5 hud_portrait blink counter (60 when the enemy's vitality reached 0).
    pub blink: u8,
    /// +6 object of the message on display (low word of the long pointer).
    pub tracked: Handle,
    /// +$A scroll1 address of the message area (hud_portrait at +0, name at +0x180, bar at +0x184).
    pub base: u32,
    /// +$E / +$F kind and name id saved for the hud_portrait blink.
    pub saved_kind: u8,
    pub saved_id: u8,
    /// G.-$7096 / -$7094 / -$7092 weapon display active flag.
    pub weapon_active: u16,
    /// G.-$709E / -$709C / -$709A weapon ammo on display.
    pub weapon_ammo: u16,
    /// G.-$70A6 / -$70A4 / -$70A2 weapon gun index on display.
    pub weapon_gun: u16,
}

/// Message being drawn (G.$6B98, `a3` in the HUD code).
#[derive(Clone, Copy, Debug, Default)]
pub struct HudCur {
    /// +4 kind, +5 name id.
    pub kind: u8,
    pub id: u8,
    /// +6 bar segments left, +8 bar attribute, +$A extra bars * 2, +$E bar segments.
    pub segs: i16,
    pub attr: u16,
    pub extra: u16,
    pub segs_total: i16,
}

/// A 16-byte message record of the per-player rings (G.$6BDE / $6CDE / $6DDE), as words.
/// Word 0 = kind << 8 | name id; 0xFFFF = free. The other words depend on the producer.
pub type Msg = [u16; 8];
const FREE: Msg = [0xFFFF; 8];

#[derive(Clone)]
pub struct Hud {
    pub slots: [HudSlot; 3],
    pub cur: HudCur,
    pub rings: [[Msg; 16]; 3],
    /// G.$6BD8 / $6BDA / $6BDC write and G.$6BD9 / $6BDB / $6BDD read offsets (0x00..0xF0).
    pub ring_w: [u8; 3],
    pub ring_r: [u8; 3],
    /// G.-$70BC boss whose bar the idle HUD shows (0 = none); G.-$70BA its message.
    pub boss: Handle,
    pub boss_msg: Msg,
    /// G.-$70AE scroll1 address where the last enemy bar started.
    pub bar_start: u32,
    /// G.-$7098 weapon display needs a redraw.
    pub weapon_dirty: u16,
    /// G.-$7134 any score changed, G.-$7133..-$7131 per player.
    pub score_dirty: bool,
    pub score_dirty_p: [bool; 3],
    /// G.$50A low vitality blink phase (0 / 1), G.$50B its timer.
    pub blink_phase: u8,
    pub blink_timer: u8,
    pub stats: [Stats; 3],
    /// G.$703E high score table (50 entries).
    pub hiscores: Vec<HiScore>,
    /// Set by `hud_objects_tail` so `hud_update` does not run it a second time this frame.
    tail_done: bool,
}

impl Default for Hud {
    fn default() -> Self {
        Hud {
            slots: Default::default(),
            cur: Default::default(),
            rings: [[FREE; 16]; 3],
            ring_w: [0; 3],
            ring_r: [0; 3],
            boss: 0,
            // no routine sets G.-$70BA up: it is 0 after the reset and after the RAM clear of a demo (0x99F10) until 0x17204 writes it
            boss_msg: [0; 8],
            bar_start: 0,
            weapon_dirty: 0,
            score_dirty: false,
            score_dirty_p: [false; 3],
            blink_phase: 0,
            blink_timer: 0,
            stats: Default::default(),
            hiscores: vec![HiScore::default(); 50],
            tail_done: false,
        }
    }
}

/// 0x11C4: binary to BCD (bits 0..12). For each bit set the table 0x11F4 (the four digits of the power of two, low byte first) is added with two `abcd`: the
/// first (low byte) starts with X = 0, the second (high byte) takes the carry of the first, and the carry out of the high byte is lost.
pub fn to_bcd(v: u16) -> u16 {
    let (mut lo, mut hi) = (0u8, 0u8);
    for bit in (0..=12).rev() {
        if v & (1 << bit) != 0 {
            let w: u32 = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096][bit];
            let b = to_bcd_table(w);
            let (l, x) = abcd(lo, (b & 0xFF) as u8, 0);
            let (h, _) = abcd(hi, (b >> 8) as u8, x);
            lo = l;
            hi = h;
        }
    }
    ((hi as u16) << 8) | lo as u16
}

fn to_bcd_table(w: u32) -> u16 {
    let s = format!("{w:04}");
    u16::from_str_radix(&s, 16).unwrap()
}

/// `abcd.b` of two bytes with the extend flag `x`: the sum and the carry out (the new X).
fn abcd(a: u8, b: u8, x: u8) -> (u8, u8) {
    let mut lo = (a & 15) + (b & 15) + x;
    let mut c = 0;
    if lo > 9 {
        lo -= 10;
        c = 1;
    }
    let mut hi = (a >> 4) + (b >> 4) + c;
    let mut out = 0;
    if hi > 9 {
        hi -= 10;
        out = 1;
    }
    ((hi << 4) | lo, out)
}

/// 4-byte BCD add with the 68000 `abcd` chain; returns (sum, carry out).
pub fn bcd_add32(a: u32, b: u32) -> (u32, bool) {
    let mut x = 0u8;
    let mut out = 0u32;
    for i in 0..4 {
        let (da, db) = ((a >> (8 * i)) as u8, (b >> (8 * i)) as u8);
        let mut lo = (da & 15) + (db & 15) + x;
        let mut c = 0;
        if lo > 9 {
            lo -= 10;
            c = 1;
        }
        let mut hi = (da >> 4) + (db >> 4) + c;
        x = 0;
        if hi > 9 {
            hi -= 10;
            x = 1;
        }
        out |= (((hi << 4) | lo) as u32) << (8 * i);
    }
    (out, x != 0)
}

/// 4-byte BCD subtract with `sbcd` and an incoming X flag; returns (difference, borrow out).
pub(crate) fn bcd_sub32(a: u32, b: u32, mut x: u8) -> (u32, u8) {
    let mut out = 0u32;
    for i in 0..4 {
        let (da, db) = ((a >> (8 * i)) as i16, (b >> (8 * i)) as i16);
        let mut lo = (da & 15) - (db & 15) - x as i16;
        let mut hi = ((da >> 4) & 15) - ((db >> 4) & 15);
        if lo < 0 {
            lo += 10;
            hi -= 1;
        }
        x = 0;
        if hi < 0 {
            hi += 10;
            x = 1;
        }
        out |= ((((hi << 4) | lo) as u8) as u32) << (8 * i);
    }
    (out, x)
}

impl Game {
    // ---------------------------------------------------------------- scroll1 writes

    pub(crate) fn hud_cell(a: u32) -> usize {
        ((a.wrapping_sub(S1) / 4) & 0xFFF) as usize
    }

    /// One cell: code word at `a`, attribute word at `a + 2`.
    pub fn hud_put(&mut self, a: u32, code: u16, attr: u16) {
        let c = &mut self.vram.scroll1[Self::hud_cell(a)];
        c.code = code;
        c.attr = attr;
    }

    /// One word store at `a` (code word if `a` is 4-aligned, attribute word otherwise).
    pub fn hud_word(&mut self, a: u32, v: u16) {
        let c = &mut self.vram.scroll1[Self::hud_cell(a)];
        if a & 2 == 0 {
            c.code = v;
        } else {
            c.attr = v;
        }
    }

    pub(crate) fn hud_read(&self, a: u32) -> u16 {
        let c = &self.vram.scroll1[Self::hud_cell(a)];
        if a & 2 == 0 { c.code } else { c.attr }
    }

    /// 0xC0A hud_print: `s` = attribute byte, text, '@'. '/' starts the next row at the
    /// first column. Each character code is the tile code.
    pub fn hud_print(&mut self, s: &[u8], a: u32) {
        let attr = s[0] as u16 | 0x180;
        let (mut a1, mut a2) = (a, a);
        for &c in &s[1..] {
            match c {
                b'@' => return,
                b'/' => {
                    a2 += 4;
                    a1 = a2;
                }
                _ => {
                    self.hud_put(a1, c as u16, attr);
                    a1 += 0x80;
                }
            }
        }
    }

    /// 0xF94: fill a rectangle with spaces (attr 0x180). `r` = (col, row - 32, width, height).
    pub fn hud_clear_rect(&mut self, r: [u8; 4]) {
        let row = if r[1] >= 0x20 { r[1] - 0x20 } else { r[1] };
        let base = 0x90A000 + ((r[0] as u32) << 7) + ((row as u32) << 2);
        for c in 0..r[2] as u32 {
            for k in 0..r[3] as u32 {
                self.hud_put(base + c * 0x80 + k * 4, 0x20, 0x180);
            }
        }
    }

    fn hud_two_players(&self) -> bool {
        self.flow.two_player_mode()
    }

    // ---------------------------------------------------------------- object fields

    fn hud_body(&self, h: Handle) -> Option<&super::body::Body> {
        slot_of(h).map(|_| self.body(h))
    }

    fn hud_is_player(h: Handle) -> bool {
        matches!(slot_of(h), Some(Slot::Player(_)))
    }

    /// +$04 as a word (state, sub-state).
    fn hud_state_word(&self, h: Handle) -> u16 {
        self.hud_body(h).map(|b| ((b.mode as u16) << 8) | b.sub as u16).unwrap_or(0)
    }

    fn hud_frame_class(&self, h: Handle) -> u8 {
        self.hud_body(h).map(|b| b.kind).unwrap_or(0)
    }

    // ---------------------------------------------------------------- init

    /// 0x1504E (boot): default high score table. Scores 100000 down by 10000 with the original's
    /// BCD borrow, so entries 11..49 wrap to 99990000.. (negative as signed longs; the rank
    /// compare is signed, so they rank below 0).
    pub fn hud_hiscore_init(&mut self) {
        let t = &self.assets.text;
        let (mut d0, d1) = (0x0010_0000u32, 0x0001_0000u32);
        let (mut chr, mut stage, mut b0a) = (0u8, 8u8, 100u8);
        for i in 0..50 {
            let e = &mut self.hud.hiscores[i];
            e.score = d0;
            for k in 0..4 {
                e.name[k] = t.u8(0x150A4, (i * 4 + k) as u32);
            }
            e.chr = chr;
            e.stage = stage;
            e.b0a = b0a;
            // X is clear: the previous subq.b #2 (or the task start) left no borrow.
            d0 = bcd_sub32(d0, d1, 0).0;
            chr = (chr + 1) & 3;
            stage -= 1;
            if stage == 0 {
                stage = 1;
            }
            b0a = b0a.wrapping_sub(2);
        }
    }

    /// 0x16894: HUD slots at stage start (also clears the boss pointer).
    pub fn hud_init(&mut self) {
        self.hud.boss = 0;
        let bases: &[u32] = if self.hud_two_players() { &[0x90A014, 0x90AD14] } else { &[0x90A014, 0x90A814, 0x90B014] };
        for (i, &b) in bases.iter().enumerate() {
            let s = &mut self.hud.slots[i];
            s.idx = i as u8;
            s.blink = 0;
            s.base = b;
        }
    }

    /// 0x4CEE: empty message rings.
    pub fn hud_rings_init(&mut self) {
        self.hud.ring_w = [0; 3];
        self.hud.ring_r = [0; 3];
        self.hud.rings = [[FREE; 16]; 3];
    }

    // ---------------------------------------------------------------- message producers

    /// The original stores only the record bytes of its producer; words from `keep` on keep the old
    /// contents of the slot.
    fn hud_ring_push(&mut self, ring: usize, m: Msg, keep: usize) {
        let w = self.hud.ring_w[ring];
        let e = &mut self.hud.rings[ring][(w >> 4) as usize];
        if (e[0] as i16) >= 0 {
            return; // ring full: the message is dropped
        }
        let old = *e;
        *e = m;
        e[keep..8].copy_from_slice(&old[keep..8]);
        self.hud.ring_w[ring] = w.wrapping_add(0x10);
    }

    fn hud_msg_of(&self, kind: u8, e: Handle, w6: u16) -> Msg {
        let b = self.body(e);
        // `move.l a6,(a4)`: the object's word address sign extended, so the high word is 0xFFFF
        [((kind as u16) << 8) | b.id as u16, b.hp_bar, b.hp_prev, w6, 0xFFFF, e, 0, 0]
    }

    /// 0x16B18: `e` (the port calls it for a hero, the 18 calls of the hero's hit reactions) was hit by its last attacker (+$68): when the
    /// attacker's +$82 is 0 (a hero, or a pool C / D object; not pools A, B, E) and it is not `e`, queue `kind` (2: a hero was hit) in the
    /// HUD of the player in the attacker's +$02 (none when negative). Natural play: only a hero's hit on another hero (friendly fire).
    pub fn hud_hit(&mut self, e: Handle, kind: u8) {
        // movea.w $68(a6),a0 ; tst.b $82(a0) ; bne (the pool tag: 4, 8, 0xC for pools A, B, E; a hero and pools C / D hold 0 there,
        // where +$82 of a pool C / D slot is +$02 of the next slot) ; cmpa.w a0,a6 ; beq ; move.b $2(a0),d1 ; bpl: the ring of that player
        let a = self.body(e).attacker;
        let base = if a >= 0x8000 { super::trace::RAM | a as u32 } else { a as u32 };
        if self.orig_u8(base + 0x82) != 0 || a == e {
            return;
        }
        let who = self.orig_u8(base + 2);
        self.hud_push_hit(e, who, kind);
    }

    /// 0x16B36: as `hud_hit`, for the player credited in `e`'s +$02 (negative = nobody).
    pub fn hud_hit_credit(&mut self, e: Handle, kind: u8) {
        let who = self.body(e).who;
        self.hud_push_hit(e, who, kind);
    }

    fn hud_push_hit(&mut self, e: Handle, who: u8, kind: u8) {
        if (who as i8) < 0 || who > 2 {
            return;
        }
        let m = self.hud_msg_of(kind, e, self.body(e).hp);
        self.hud_ring_push(who as usize, m, 6);
    }

    /// 0x1693C with kind 3: food `item` was eaten by its +$70 player. Record: +6 = item +$6C
    /// (100 healed, 99 = points instead), +8 = food id (+$26), +$A = item pointer, a long: `a6` is sign extended on both paths (0x28AE2
    /// `movea.w`, and the pool E loop), so its high word is 0xFFFF; word 7 is not written. Quirk kept: for name id 2 the item's +$6A is set to 100.
    pub fn hud_food(&mut self, item: Handle) {
        let p = self.body(item).partner;
        if !Self::hud_is_player(p) {
            return;
        }
        let b = self.body(item);
        let who = b.who;
        if who > 2 {
            return;
        }
        let m = [(3u16 << 8) | b.id as u16, b.hp_bar, b.hp_prev, b.hp, b.var26, 0xFFFF, item, 0];
        if b.id == 2 {
            self.body_mut(item).hp_bar = 0x64;
        }
        self.hud_ring_push(who as usize, m, 7);
    }

    /// 0x16A5A with kind 4: treasure `item` picked up by its +$70 player. +6 = treasure id, +8 the item pointer (a long, `a6` of the pool E
    /// loop, sign extended: high word 0xFFFF; MAME ep3_hannah frame 1780); words 6 and 7 are not written.
    pub fn hud_treasure(&mut self, item: Handle) {
        let p = self.body(item).partner;
        if !Self::hud_is_player(p) {
            return;
        }
        let b = self.body(item);
        let who = b.who;
        if who > 2 {
            return;
        }
        let m = [(4u16 << 8) | b.id as u16, b.hp_bar, b.hp_prev, b.var26, 0xFFFF, item, 0, 0];
        self.hud_ring_push(who as usize, m, 6);
    }

    // ---------------------------------------------------------------- score

    /// 0x10036: add the BCD value at byte offset `off` of table 0x10292 to the score of the
    /// player in `h`'s +$02 (negative = nobody). Caps at 99999999.
    pub fn hud_add_score(&mut self, h: Handle, off: u32) {
        let who = self.body(h).who;
        self.hud_add_score_to(who, self.assets.text.u32(0x10292, off));
    }

    /// The body of 0x10036 for a player index (`who` as in +$02).
    pub fn hud_add_score_to(&mut self, who: u8, value: u32) {
        if (who as i8) < 0 {
            return;
        }
        let i = (who as usize).min(2); // 0x10046: 0 -> P1, 1 -> P2, anything else -> P3
        let p = &mut self.players[i];
        let (v, carry) = bcd_add32(p.score, value);
        p.score = if carry { 0x9999_9999 } else { v };
        self.hud.score_dirty = true;
        self.hud.score_dirty_p[i] = true;
    }

    /// 0x101B6: up to 8 score digits, right to left from `a`; leading zeros are not drawn.
    fn hud_draw_score(&mut self, mut a: u32, mut v: u32) {
        for d1 in (0..8).rev() {
            if v == 0 && d1 <= 6 {
                return;
            }
            self.hud_put(a, 0x124 + (v & 0xF) as u16, 0x181);
            a = a.wrapping_sub(0x80);
            v >>= 4;
        }
    }

    fn hud_score_pos(&self, i: usize) -> u32 {
        if self.hud_two_players() { [0x90AA88, 0x90B788, 0x90B788][i] } else { 0x90A788 + 0x800 * i as u32 }
    }

    /// The object-phase tail of 0xE90E that belongs to the HUD: score display 0x100DA and the
    /// low vitality blink 0x1667E. The original calls it from the object update, after the held
    /// players; `hud_update` runs it if the object update did not.
    pub fn hud_objects_tail(&mut self) {
        self.hud.tail_done = true;
        self.hud_score_update();
        self.hud_low_hp_blink();
    }

    /// 0x100DA: redraw changed scores, then the extra-life check.
    fn hud_score_update(&mut self) {
        if self.spawn.g.b4d5 != 0 || !self.hud.score_dirty {
            self.hud.score_dirty = false;
            return;
        }
        self.hud_rank_all();
        let n = if self.hud_two_players() { 2 } else { 3 };
        for i in 0..n {
            if self.hud.score_dirty_p[i] {
                self.hud_draw_score(self.hud_score_pos(i), self.players[i].score);
                self.hud.score_dirty_p[i] = false;
                self.hud_extend_check(i);
            }
        }
        self.hud.score_dirty = false;
    }

    /// 0x101E6: all scores of the present players (and the ranks).
    pub fn hud_draw_all_scores(&mut self) {
        self.hud_rank_all();
        let n = if self.hud_two_players() { 2 } else { 3 };
        for i in 0..n {
            if self.players[i].b.live != 0 {
                self.hud_draw_score(self.hud_score_pos(i), self.players[i].score);
                self.hud_extend_check(i);
            }
        }
    }

    /// 0x6B98: one extra life when the score passes the next threshold of the DIP's list
    /// (once per player: +$10E).
    fn hud_extend_check(&mut self, i: usize) {
        let t = &self.assets.text;
        let prev = self.hud.stats[i].score_prev;
        let score = self.players[i].score;
        self.hud.stats[i].score_prev = score;
        if self.players[i].b10e != 0 {
            return;
        }
        let list = 0x6BFE + t.u16(0x6BFE, self.flow.extend_sel() as u32) as u32;
        let mut k = 0;
        while prev >= t.u32(list, 4 * k) {
            k += 1;
        }
        if score < t.u32(list, 4 * k) {
            return;
        }
        let p = &mut self.players[i];
        p.b10e = 1;
        p.b11c = 0xF0;
        p.lives = p.lives.wrapping_add(1);
        let lives = self.hud_lives_sum();
        let (b4cf, b4e6) = (self.flow.b4cf(), self.flow.b4e6);
        self.rank_event_1843c(lives, b4cf, b4e6); // 0x6BDC
        // 0xA4CA: "extra life" sound, panned by the player's screen X.
        self.snd_extra_life();
        self.hud_player_block(i);
    }

    /// Sum of +$A3 of the present players (0x1843C).
    pub fn hud_lives_sum(&self) -> u8 {
        (0..3).filter(|&i| self.players[i].b.live != 0).fold(0u8, |a, i| a.wrapping_add(self.players[i].lives))
    }

    // ---------------------------------------------------------------- rank ("1ST" ... "10TH")

    /// 0x15948: rank text of every present player.
    pub fn hud_rank_all(&mut self) {
        for i in 0..3 {
            if self.players[i].b.live != 0 {
                self.hud_rank_display(i);
            }
        }
    }

    /// 0x15974.
    fn hud_rank_display(&mut self, i: usize) {
        let t = &self.assets.text;
        let mut a = t.u32(0x15ACE, if self.hud_two_players() { 0xC } else { 0 } + 4 * i as u32);
        self.hud_rank_compute(i);
        let r = self.players[i].b10a;
        if (r as i8) > 10 {
            for _ in 0..4 {
                self.hud_put(a, 0x20, 0x180);
                a += 0x80;
            }
            return;
        }
        let d0 = to_bcd(r as u16);
        let tens = (d0 >> 4) & 0xF;
        self.hud_put(a, if tens == 0 { 0x20 } else { tens }, 0x180);
        a += 0x80;
        self.hud_put(a, d0 & 0xF, 0x180);
        a += 0x80;
        let mut d2 = r.wrapping_sub(1) as i8;
        if d2 > 3 {
            d2 = 3;
        }
        let t = &self.assets.text;
        let (c0, c1) = (t.u8(0x159F8, 2 * d2 as u32), t.u8(0x159F8, 2 * d2 as u32 + 1));
        self.hud_put(a, c0 as u16, 0x180);
        self.hud_put(a + 0x80, c1 as u16, 0x180);
    }

    /// 0x15A2E: move up past every high score entry this score reaches (signed compares), then
    /// one place down for each present player with a higher score.
    fn hud_rank_compute(&mut self, i: usize) {
        loop {
            let r = self.players[i].b10a as i8;
            let k = r.wrapping_sub(2);
            if k < 0 {
                break;
            }
            let e = self.hud.hiscores.get(k as usize).map(|e| e.score).unwrap_or(0);
            if (self.players[i].score as i32) < (e as i32) {
                break;
            }
            self.players[i].b10a = (r - 1) as u8;
        }
        let me = self.players[i].score as i32;
        // Table 0x15A98: the two other players, in index order.
        for o in (0..3).filter(|&o| o != i) {
            if self.players[o].b.live != 0 && me < self.players[o].score as i32 && (self.players[i].b10a as i8) <= 0x32 {
                self.players[i].b10a += 1;
            }
        }
    }

    // ---------------------------------------------------------------- player block

    /// 0x1631A / 0x1637E: scroll1 address of player i's HUD block (face at col 0, row 34).
    fn hud_block_base(&self, i: usize) -> u32 {
        let t = &self.assets.text;
        if self.hud_two_players() { t.u32(0x1638E, 4 * i.min(1) as u32) } else { t.u32(0x1632A, 4 * i as u32) }
    }

    /// 0x162F8 (3 players) / 0x16358 (2 players): face, lives, name, vitality bar, ranks.
    pub fn hud_player_block(&mut self, i: usize) {
        let mut a = self.hud_block_base(i);
        let chr = self.players[i].b.id as u32 & 3;
        // 0x16396 face: 3x3 tiles, columns c, c+1, c+2; rows +0, +0x10, +0x3A0.
        let t = &self.assets.text;
        let (mut code, attr) = (t.u16(0x163D4, 2 * chr), t.u16(0x163DC, 2 * chr));
        for _ in 0..3 {
            self.hud_put(a, code, attr);
            self.hud_put(a + 4, code + 0x10, attr);
            self.hud_put(a + 8, code + 0x3A0, attr);
            code += 1;
            a += 0x80;
        }
        self.hud_put(a, 0x3D, 0x180);
        a += 0x80;
        // 0x163E4 lives: the count is the tile code.
        self.hud_put(a, self.players[i].lives as u16, 0x180);
        a -= 0x7C;
        // 0x163F4 name (no attribute byte in these strings).
        let name = self.assets.text.table_string(0x1641E, chr);
        for &c in name.iter().take_while(|&&c| c != b'@') {
            self.hud_put(a, c as u16, 0x180);
            a += 0x80;
        }
        if self.hud_two_players() {
            self.hud_life_bar_2p(i);
        } else {
            self.hud_life_bar_3p(i);
        }
        self.hud_rank_all();
    }

    /// 0x162AE (camera hook before an area fades in, and stage start): redraw the blocks and
    /// scores of the present players.
    pub fn hud_redraw(&mut self) {
        let n = if self.hud_two_players() { 2 } else { 3 };
        for i in 0..n {
            if self.players[i].b.live != 0 {
                self.hud_player_block(i);
            }
        }
        self.hud_draw_all_scores();
    }

    /// 0x16446: player i's vitality bar (the original calls it right after any HP change).
    pub fn hud_life_bar(&mut self, i: usize) {
        if self.hud_two_players() {
            self.hud_life_bar_2p(i);
        } else {
            self.hud_life_bar_3p(i);
        }
    }

    /// HP as the bar code sees it: 0 and 100 kept, anything else minus 1.
    fn hud_bar_hp(&self, i: usize) -> u16 {
        let hp = self.players[i].b.hp;
        if hp == 0 || hp == 100 { hp } else { hp - 1 }
    }

    /// 0x16446 (3 players): right end at col 15, 11 segments of 8 HP, left end at col 3.
    fn hud_life_bar_3p(&mut self, i: usize) {
        let mut a = self.hud_block_base(i) + 0x788;
        let hp = self.hud_bar_hp(i);
        let t = &self.assets.text;
        // 0x164E4 right end
        let right = if hp == 100 {
            0x983
        } else if (hp as i16) <= 0x5E {
            0x989
        } else {
            t.u16(0x1653C, 2 * ((hp + 2) & 7) as u32)
        };
        self.hud_put(a, right, 0x18D);
        a -= 0x80;
        let level = (hp + 2) >> 3;
        for d5 in (1..=11u16).rev() {
            let code = if level == d5 {
                self.assets.text.u16(0x16520, 2 * ((hp + 2) & 7) as u32)
            } else if level < d5 {
                0x90B
            } else {
                0x903
            };
            self.hud_put(a, code, 0x18D);
            a -= 0x80;
        }
        // 0x164B6 left end
        let left = if (hp as i16) >= 6 { 0x90C } else { self.assets.text.u16(0x16530, 2 * (hp & 7) as u32) };
        self.hud_put(a, left, 0x18D);
    }

    /// 0x16548 (2 players): HP scaled by 1.5, right end at col 22 (code only), 18 segments.
    fn hud_life_bar_2p(&mut self, i: usize) {
        let mut a = self.hud_block_base(i) + 0xB08;
        self.hud_word(a, 0x98A);
        a -= 0x80;
        let hp = self.hud_bar_hp(i);
        let hp15 = hp + (hp >> 1);
        let level = (hp15 + 2) >> 3;
        for d5 in (1..=18u16).rev() {
            let code = if level == d5 {
                self.assets.text.u16(0x16520, 2 * ((hp15 + 2) & 7) as u32)
            } else if level < d5 {
                0x90B
            } else {
                0x903
            };
            self.hud_put(a, code, 0x18D);
            a -= 0x80;
        }
        let left = if (hp15 as i16) >= 6 { 0x90C } else { self.assets.text.u16(0x16530, 2 * (hp15 & 7) as u32) };
        self.hud_put(a, left, 0x18D);
    }

    /// 0x165FC (boss defeated): clear the HUD block of players that are not playing and not
    /// in the continue countdown (+$A2 == 6 keeps it).
    pub fn hud_clear_idle_blocks(&mut self) {
        let n = if self.hud_two_players() { 2 } else { 3 };
        for i in 0..n {
            if self.players[i].b.live == 0 && self.flow.join[i].state != 6 {
                let off = if self.hud_two_players() { 0x1666A + 12 } else { 0x1666A } + 4 * i as u32;
                let t = &self.assets.text;
                let r = [t.u8(off, 0), t.u8(off, 1), t.u8(off, 2), t.u8(off, 3)];
                self.hud_clear_rect(r);
            }
        }
    }

    // ---------------------------------------------------------------- low vitality blink

    /// 0x1667E: players with HP <= 15 blink rows 34-35 of their block (palette 2 / normal).
    fn hud_low_hp_blink(&mut self) {
        // 0x166DE timer: every 6 frames toggle the phase
        let (t, borrow) = self.hud.blink_timer.overflowing_sub(1);
        self.hud.blink_timer = t;
        if borrow {
            self.hud.blink_timer = 5;
            self.hud.blink_phase = (self.hud.blink_phase + 1) & 1;
        }
        for i in 0..3 {
            let p = &self.players[i].b;
            if p.live != 0 && (p.hp as i16) <= 0xF {
                self.hud.stats[i].low_hp = 1;
                self.hud_low_hp_attrs(i);
            } else {
                if self.hud.stats[i].low_hp != 0 {
                    self.hud.blink_phase = 0;
                    self.hud_low_hp_attrs(i);
                }
                self.hud.stats[i].low_hp = 0;
            }
        }
    }

    /// 0x16704: attribute palettes of rows 34-35, cols 3..15 (2 players: 3..22).
    fn hud_low_hp_attrs(&mut self, i: usize) {
        if i > 1 && self.hud_two_players() {
            return; // the 2-player base table has two entries
        }
        let mut a = self.hud_block_base(i) + 0x182;
        let (n, sel) = if self.hud_two_players() { (20, self.hud.blink_phase + 2) } else { (13, self.hud.blink_phase) };
        let tbl = self.assets.text.u32(0x1677C, 4 * sel as u32);
        for k in 0..n {
            let (p0, p1) = (self.assets.text.u16(tbl, 4 * k), self.assets.text.u16(tbl, 4 * k + 2));
            let v = (self.hud_read(a) & 0xFFE0) | p0;
            self.hud_word(a, v);
            let v = (self.hud_read(a + 4) & 0xFFE0) | p1;
            self.hud_word(a + 4, v);
            a += 0x80;
        }
    }

    // ---------------------------------------------------------------- per-frame HUD 0x16BEA

    /// 0x16BEA: message areas of the present players.
    pub fn hud_update(&mut self) {
        if !self.hud.tail_done {
            self.hud_objects_tail();
        }
        self.hud.tail_done = false;
        let n = if self.hud_two_players() { 2 } else { 3 };
        for i in 0..n {
            if self.players[i].b.live != 0 {
                self.hud_msg_update(i);
                self.hud_portrait_blink(i);
            }
        }
    }

    /// 0x16CE6 (3 players) / 0x16F2A (2 players).
    fn hud_msg_update(&mut self, i: usize) {
        let two = self.hud_two_players();
        self.hud_weapon(i); // 0x2389C
        if self.hud.slots[i].timer == 0 {
            self.hud_boss_bar(i); // 0x17188 / 0x17290
        }
        let r = (self.hud.ring_r[i] >> 4) as usize;
        let m = self.hud.rings[i][r];
        let s = self.hud.slots[i];
        if (m[0] as i16) < 0 {
            if s.kind == 1 {
                let k = self.hud_frame_class(s.tracked);
                if k == 0xA {
                    self.hud.slots[i].kind = 0;
                    self.hud.slots[i].timer = 0x3C;
                    return;
                }
                let st = self.hud_state_word(s.tracked);
                if st == 0x206 || st == 0x208 {
                    return;
                }
                self.hud_clear_msg_area(s.base);
                let sl = &mut self.hud.slots[i];
                sl.kind = 0;
                sl.timer = 0;
                self.hud.weapon_dirty = 1;
                self.hud_weapon(i);
                self.hud_boss_bar(i);
                return;
            }
            if s.timer == 0 {
                self.hud_weapon_reset(i); // 0x2388E
                self.hud_boss_bar(i);
                return;
            }
            self.hud.slots[i].timer -= 1;
            if self.hud.slots[i].timer != 1 {
                return;
            }
            if s.kind == 1 && self.hud_frame_class(s.tracked) == 0xE {
                return;
            }
            self.hud_clear_msg_area(s.base);
            self.hud.slots[i].kind = 0;
            self.hud.weapon_dirty = 1;
            self.hud_weapon(i);
            self.hud_boss_bar(i);
            return;
        }
        if s.delay != 0 {
            self.hud.slots[i].delay -= 1;
            return;
        }
        let kind = (m[0] >> 8) as u8;
        if kind == 3 {
            self.hud.rings[i][r][0] = 0xFFFF;
            self.hud_food_msg(i, m[4], m[3]);
            self.hud_msg_done(i);
            return;
        }
        if kind == 4 {
            self.hud.rings[i][r][0] = 0xFFFF;
            self.hud_treasure_msg(i, m[3]);
            self.hud_msg_done(i);
            return;
        }
        if s.step == 0 {
            self.hud.slots[i].blink = 0;
            self.hud.cur.kind = kind;
            self.hud.cur.id = m[0] as u8;
            self.hud.slots[i].kind = kind;
            let (mut d1, mut d0) = (m[1], m[2]);
            if m[3] == 0 || kind == 1 {
                self.hud.slots[i].tracked = m[5];
            } else {
                if two {
                    d0 = d0.wrapping_add((d0 as i16 >> 1) as u16);
                    d1 = d1.wrapping_add((d1 as i16 >> 1) as u16);
                }
                self.hud_enemy_bar(i, s.base + 0x184, d0, d1);
                self.hud_portrait(s.base);
                self.hud_enemy_name(i);
                self.hud.slots[i].delay = 3;
                self.hud.slots[i].step = 1;
                return;
            }
        }
        // 0x16EA8: final draw with the HP after the hit.
        self.hud.cur.kind = kind;
        self.hud.cur.id = m[0] as u8;
        let (mut d1, mut d0) = (m[1], m[3]);
        if two {
            d0 = d0.wrapping_add((d0 as i16 >> 1) as u16);
            d1 = d1.wrapping_add((d1 as i16 >> 1) as u16);
        }
        self.hud_final_draw(i, d0, d1);
        self.hud.rings[i][r][0] = 0xFFFF;
        self.hud_msg_done(i);
    }

    /// Common tail: blink setup, bar, hud_portrait and name with the final HP (0x16EBC..0x16F12).
    fn hud_final_draw(&mut self, i: usize, d0: u16, d1: u16) {
        let sl = &mut self.hud.slots[i];
        sl.blink = 0;
        if d0 == 0 {
            sl.blink = 0x3C;
            sl.saved_kind = self.hud.cur.kind;
            sl.saved_id = self.hud.cur.id;
        }
        let base = sl.base;
        self.hud_enemy_bar(i, base + 0x184, d0, d1);
        self.hud_portrait(base);
        self.hud_enemy_name(i);
    }

    /// 0x16F00..0x16F24: delays, then advance the read offset.
    fn hud_msg_done(&mut self, i: usize) {
        let sl = &mut self.hud.slots[i];
        sl.delay = 10;
        sl.step = 0;
        sl.timer = 0x3C;
        self.hud.ring_r[i] = self.hud.ring_r[i].wrapping_add(0x10);
    }

    /// 0x17190 (3 players) / 0x17298 (2 players): with nothing on display, show the boss bar.
    fn hud_boss_bar(&mut self, i: usize) {
        let boss = self.hud.boss;
        if boss == 0 {
            return;
        }
        let p = &self.players[i.min(2)].b;
        if p.live == 0 {
            return;
        }
        let two = self.hud_two_players();
        if two {
            if (i < 2 && p.hp == 0) || (p.mode as i8) < 2 {
                return;
            }
        } else if p.hp == 0 {
            return;
        }
        self.hud.slots[i].tracked = boss;
        let b = self.body(boss);
        // 0x17204: `move.l a3,(a4)` with a3 = `movea.w G.-$70BC`, a sign extended word, so the high word is 0xFFFF (MAME, M6-09).
        let m = [b.id as u16, b.hp_bar, b.hp_prev, b.hp, 0xFFFF, boss, 0, 0];
        self.hud.boss_msg = m;
        if m[3] == 0 {
            self.hud.boss = 0;
        }
        let m = self.hud.boss_msg;
        self.hud.cur.kind = 0;
        self.hud.cur.id = m[0] as u8;
        let (mut d1, mut d0) = (m[1], m[3]);
        if two {
            d0 = d0.wrapping_add((d0 as i16 >> 1) as u16);
            d1 = d1.wrapping_add((d1 as i16 >> 1) as u16);
        }
        self.hud.boss_msg[0] = 0xFFFF;
        self.hud_final_draw(i, d0, d1);
        let sl = &mut self.hud.slots[i];
        sl.delay = 10;
        sl.step = 0;
        sl.timer = 0x3C;
    }

    /// 0x177F6: clear the 2-row message area (16 columns, 24 with 2 players).
    fn hud_clear_msg_area(&mut self, mut a: u32) {
        let n = if self.hud_two_players() { 24 } else { 16 };
        for _ in 0..n {
            self.hud_put(a, 0x20, 0);
            self.hud_put(a + 4, 0x20, 0);
            a += 0x80;
        }
    }

    /// 0x173BC: hud_portrait blink after the enemy's vitality reached 0 (skull / hud_portrait).
    fn hud_portrait_blink(&mut self, i: usize) {
        let sl = &mut self.hud.slots[i];
        if sl.blink == 0 {
            return;
        }
        sl.blink -= 1;
        let b = sl.blink;
        if b & 2 == 0 {
            return;
        }
        let mut a = sl.base;
        if b & 1 == 0 {
            // 0x17446: redraw the saved hud_portrait and name
            self.hud.cur.kind = self.hud.slots[i].saved_kind;
            self.hud.cur.id = self.hud.slots[i].saved_id;
            self.hud_portrait(a);
            self.hud_enemy_name(i);
            return;
        }
        for k in 0..3 {
            let t = &self.assets.text;
            let (c0, c1) = (t.u16(0x17436, 4 * k), t.u16(0x17436, 4 * k + 2));
            self.hud_put(a, c0, 0x18B);
            self.hud_put(a + 4, c1, 0x18B);
            a += 0x80;
        }
    }

    /// 0x179B6: 3x2 hud_portrait of the current message (player portraits for kind 2).
    fn hud_portrait(&mut self, mut a: u32) {
        let (kind, id) = (self.hud.cur.kind, self.hud.cur.id as u32);
        let t = &self.assets.text;
        let (tiles, pals) = if kind == 2 { (0x17A82, 0x17A6E) } else { (0x17ABE, 0x17A1E) };
        let attr = t.u16(pals, 2 * id) + 0x180;
        for k in 0..3 {
            let t = &self.assets.text;
            let (c0, c1) = (t.u16(tiles, 16 * id + 4 * k), t.u16(tiles, 16 * id + 4 * k + 2));
            self.hud_put(a, c0, attr);
            self.hud_put(a + 4, c1, attr);
            a += 0x80;
        }
    }

    /// 0x17D2E: name of the current message at +0x180 (player names for kind 2).
    fn hud_enemy_name(&mut self, i: usize) {
        let table = if self.hud.cur.kind == 2 { 0x17D70 } else { 0x17D78 };
        let s = self.assets.text.table_string(table, self.hud.cur.id as u32);
        let a = self.hud.slots[i].base + 0x180;
        self.hud_print(&s, a);
    }

    /// 0x174F2 (3 players) / 0x17676 (2 players): enemy vitality bar. `max` (+$6A, at most 100)
    /// sets the bar length (1 tile = 8 HP); HP above 100 recolours the bar (one colour set per
    /// extra 100 HP).
    fn hud_enemy_bar(&mut self, i: usize, mut a: u32, hp: u16, max: u16) {
        let two = self.hud_two_players();
        if !two {
            self.hud.bar_start = a;
        }
        self.hud.cur.segs = 0;
        let d0 = self.hud_bar_colours(i, hp) as i16;
        let mut d3 = d0;
        let mut d2 = max as i16;
        d2 = d2.wrapping_sub(6);
        if d2 >= 0 {
            loop {
                d2 = d2.wrapping_sub(8);
                if d2 <= 0 {
                    break;
                }
                self.hud.cur.segs += 1;
            }
        }
        self.hud.cur.segs_total = self.hud.cur.segs;
        let attr = self.hud.cur.attr;
        let (mid, right, left, cap_max) = if two { (0x177C4, 0x177E4, 0x177D6, 0x90) } else { (0x175EE, 0x17600, 0x1760E, 0x5E) };
        let mut right_end = false;
        if d0 >= 6 {
            self.hud.cur.segs -= 1;
            d3 -= 6;
            self.hud_put(a, 0x91C, attr);
            a += 0x80;
            while d3 > 8 {
                self.hud_put(a, 0x913, attr);
                a += 0x80;
                d3 -= 8;
                self.hud.cur.segs -= 1;
            }
            right_end = self.hud.cur.segs < 0;
        }
        let t = &self.assets.text;
        if right_end || d0 > cap_max {
            // right end piece, no advance
            self.hud_put(a, t.u16(right, 2 * d3 as u32), attr);
            if two {
                self.hud_put(a + 0x80, 0x98A, attr);
            }
        } else {
            let tbl = if d0 < 6 { left } else { mid };
            self.hud_put(a, t.u16(tbl, 2 * d3 as u32), attr);
            a += 0x80;
            loop {
                self.hud.cur.segs -= 1;
                if self.hud.cur.segs < 0 {
                    if two {
                        // 0x17792: one more empty segment and the end cap
                        self.hud_put(a, 0x91B, attr);
                        a += 0x80;
                        self.hud_put(a, 0x99A, attr);
                    } else {
                        self.hud_put(a, 0x999, attr);
                    }
                    break;
                }
                self.hud_put(a, 0x91B, attr);
                a += 0x80;
            }
        }
        self.hud_bar_clear_rest(a);
    }

    /// 0x1761C: blank the rest of the message line after the bar.
    fn hud_bar_clear_rest(&mut self, mut a: u32) {
        let two = self.hud_two_players();
        let mut d3: i16 = if two { 0x13 } else { 0xD };
        d3 -= self.hud.cur.segs_total;
        a += 0x80;
        d3 -= 1;
        if d3 <= 0 || d3 == 1 {
            return;
        }
        loop {
            if !two && (a as i32) > (self.hud.bar_start + 0x580) as i32 {
                d3 = 0;
            }
            self.hud_put(a, 0x20, self.hud.cur.attr);
            a += 0x80;
            d3 -= 1;
            if d3 == -1 {
                return;
            }
        }
    }

    /// 0x17834 (3 players, 100 HP per bar) / 0x178DE (2 players, 150): bar attribute and the
    /// colours 3-5 of the player's palette (13 / 14 / 15) for the number of extra bars.
    /// Returns the HP left in the last bar.
    fn hud_bar_colours(&mut self, i: usize, hp: u16) -> u16 {
        let two = self.hud_two_players();
        let (attrs, per, cols) = if two { (0x178D2, 0x96, 0x1793C) } else { (0x17828, 0x64, 0x17892) };
        let t = &self.assets.text;
        let k = 2 * (i as u32).min(2);
        self.hud.cur.attr = t.u16(attrs, k) + 0x180;
        let pal_off = t.u16(attrs + 6, k) as usize;
        let mut d3 = hp as i16;
        let mut d4 = 0u16;
        while d3 > per {
            d3 -= per;
            d4 += 2;
        }
        self.hud.cur.extra = d4;
        let base = PaletteRam::SCROLL1 + (pal_off + 6) / 2;
        for k in 0..3usize {
            let c = self.assets.text.u16(cols, (d4 as u32) * 4 + 2 * k as u32);
            let w = &mut self.pal.words[base + k];
            *w = (*w & 0xF000) | c;
        }
        self.pal.dirty = true;
        d3 as u16
    }

    // ---------------------------------------------------------------- item messages

    /// 0x242DC / 0x24606: clear the message area from `base` (16 or 24 columns, 2 rows).
    fn hud_item_clear(&mut self, i: usize) {
        let base = self.hud.slots[i].base;
        self.hud_clear_msg_area(base);
    }

    /// 0x23F24: food message. `id` = food id (1-based), `heal` = 100 healed, else points.
    fn hud_food_msg(&mut self, i: usize, id: u16, heal: u16) {
        let idx = id.wrapping_sub(1);
        self.hud_item_clear(i);
        let lo = idx & 0xFF;
        if lo > 0x37 || lo > 0x23 || idx >= 20 {
            return; // 0x23F90 / 0x23FB2 limits; entries 20.. point into code (not reached [I])
        }
        let mut a = self.hud.slots[i].base;
        for k in 0..3 {
            let t = &self.assets.text;
            let (c0, c1) = (t.u16(0x24016, 4 * k), t.u16(0x24016, 4 * k + 2));
            self.hud_put(a, c0, 0xC);
            self.hud_put(a + 4, c1, 0xC);
            a += 0x80;
        }
        let base = self.hud.slots[i].base;
        let s = self.assets.text.table_string(0x240A2, idx as u32);
        self.hud_print(&s, base + 0x180);
        let s = if heal == 100 { self.assets.text.table_string(0x240A0, 0) } else { self.assets.text.table_string(0x240CA, idx as u32) };
        self.hud_print(&s, base + 0x184);
        self.hud.slots[i].timer = 0x3C;
        self.hud.slots[i].blink = 0;
    }

    /// 0x24312: treasure message (icon, name and points on two rows).
    fn hud_treasure_msg(&mut self, i: usize, id: u16) {
        let idx = id.wrapping_sub(1);
        self.hud_item_clear(i);
        let lo = idx & 0xFF;
        if lo > 0x23 || idx >= 17 {
            return; // 0x243AC limit; entry 16 keeps the treasure 17 quirk
        }
        let mut a = self.hud.slots[i].base;
        for k in 0..3 {
            let t = &self.assets.text;
            let (c0, c1) = (t.u16(0x243FE, 4 * k), t.u16(0x243FE, 4 * k + 2));
            self.hud_put(a, c0, 0xC);
            self.hud_put(a + 4, c1, 0xC);
            a += 0x80;
        }
        let base = self.hud.slots[i].base;
        let s = self.assets.text.table_string(0x24436, idx as u32);
        self.hud_print(&s, base + 0x180);
        self.hud.slots[i].timer = 0x3C;
    }

    // ---------------------------------------------------------------- weapon display

    /// 0x2389C: weapon icon, name and ammo of the held weapon (bottom rows of the message
    /// area, +0x5C).
    fn hud_weapon(&mut self, i: usize) {
        if self.w.vertical_scene != 0 || i >= 3 {
            return;
        }
        if self.players[i].item != 0 {
            // 0x238C4: the two helpers work on copies (G.-$70A0 ammo, G.-$70A8 gun index).
            self.hud.slots[i].weapon_active = 1;
            let mut ammo = self.hud.slots[i].weapon_ammo;
            let mut gun = self.hud.slots[i].weapon_gun;
            self.hud_weapon_check(i, &mut ammo, &mut gun);
            self.hud_weapon_draw(i, ammo, gun);
            self.hud.slots[i].weapon_ammo = ammo;
            self.hud.slots[i].weapon_gun = gun;
        } else if self.hud.slots[i].weapon_active != 0 {
            self.hud.slots[i].weapon_active = 0;
            self.hud.slots[i].weapon_ammo = 0;
            self.hud_weapon_clear(i);
        }
    }

    /// 0x239B0: a new gun index or a new ammo count sets the redraw flag G.-$7098.
    fn hud_weapon_check(&mut self, i: usize, ammo: &mut u16, gun: &mut u16) {
        let item_type = self.players[i].item_type;
        if item_type >> 8 == 0 {
            return self.hud_weapon_clear(i);
        }
        let g = (item_type & 0xFF).wrapping_sub(4);
        if g != *gun {
            *gun = g;
            self.hud.weapon_dirty = 1;
        } else {
            let a = self.body(self.players[i].item).hp; // +$6C of the held item
            if a == *ammo {
                return; // 0x239EE: no change, no type test
            }
            *ammo = a;
            if a == 0 {
                return self.hud_weapon_clear(i); // 0x23A20
            }
            self.hud.weapon_dirty = 1;
        }
        // 0x23A04: types above 9 have no display
        if item_type & 0xFF > 9 {
            self.hud.weapon_dirty = 0;
            self.hud_weapon_clear(i);
        }
    }

    /// 0x23A24: the guns (types 4 to 9) redraw only with the flag set; the other weapons every frame.
    fn hud_weapon_draw(&mut self, i: usize, ammo: u16, gun: u16) {
        let item_type = self.players[i].item_type;
        let t = item_type & 0xFF;
        if item_type >> 8 == 0 || t > 9 {
            return;
        }
        if t >= 4 {
            if self.hud.weapon_dirty != 0 {
                self.hud_draw_gun_icon(i, gun);
                self.hud_draw_gun_name(i, gun);
                self.hud_draw_cartridge_bar(i, gun, ammo);
                self.hud_draw_gun_ammo(i, gun, ammo);
            }
        } else {
            self.hud_draw_arm_icon(i);
            self.hud_clear_cartridge_bar(i);
            self.hud_draw_weapon_name(i, self.players[i].item);
        }
    }

    /// 0x2388E: clear the weapon redraw flag, then 0x2389C.
    fn hud_weapon_reset(&mut self, i: usize) {
        if self.w.vertical_scene != 0 {
            return;
        }
        self.hud.weapon_dirty = 0;
        self.hud_weapon(i);
    }

    /// 0x23EC0: erase the weapon display area (16 cols x 2 rows, or 24 cols with 2 players).
    fn hud_weapon_clear(&mut self, i: usize) {
        let p = &self.players[i];
        if p.item != 0 && (p.item_type >> 8) != 0 && p.b.live == 1 && (p.item_type & 0xFF) < 4 {
            return;
        }
        let base = self.hud.slots[i].base;
        let mut a = base + 0x5C;
        let cols = if self.hud_two_players() { 24 } else { 16 };
        for _ in 0..cols {
            self.hud_put(a, 0x20, 0);
            self.hud_put(a + 4, 0x20, 0);
            a += 0x80;
        }
    }

    /// 0x23A90: draw the 2x3 gun icon at base + 0x5C.
    fn hud_draw_gun_icon(&mut self, i: usize, gun_idx: u16) {
        let base = self.hud.slots[i].base;
        let mut a = base + 0x5C;
        let t = &self.assets.text;
        let off = (gun_idx as u32) * 16;
        for col in 0..3 {
            let c0 = t.u16(0x23AC6, off + col * 4);
            let c1 = t.u16(0x23AC6, off + col * 4 + 2);
            self.hud_put(a, c0, 0xC);
            self.hud_put(a + 4, c1, 0xC);
            a += 0x80;
        }
    }

    /// 0x23B26: draw the ARM icon (3 cols x 2 rows) at base + 0x5C.
    fn hud_draw_arm_icon(&mut self, i: usize) {
        let base = self.hud.slots[i].base;
        let mut a = base + 0x5C;
        let t = &self.assets.text;
        for col in 0..3 {
            let c0 = t.u16(0x23B52, col * 4);
            let c1 = t.u16(0x23B52, col * 4 + 2);
            self.hud_put(a, c0, 0xC);
            self.hud_put(a + 4, c1, 0xC);
            a += 0x80;
        }
    }

    /// 0x23C2C: print gun name at base + 0x35C.
    fn hud_draw_gun_name(&mut self, i: usize, gun_idx: u16) {
        let base = self.hud.slots[i].base;
        let s = self.assets.text.table_string(0x23C58, gun_idx as u32);
        self.hud_print(&s, base + 0x35C);
    }

    /// 0x23CAC: print other weapon name at base + 0x25C.
    fn hud_draw_weapon_name(&mut self, i: usize, held_item: Handle) {
        let base = self.hud.slots[i].base;
        let var26 = (self.body(held_item).var26 & 0xFFFE) as u32;
        let s = self.assets.text.table_string(0x23CDE, var26 / 2);
        self.hud_print(&s, base + 0x25C);
    }

    /// 0x23DDC: draw 12-cell cartridge bar at base + 0x1E0 (0x184 + 0x5C).
    fn hud_draw_cartridge_bar(&mut self, i: usize, gun_idx: u16, ammo: u16) {
        if ammo == 0 {
            self.hud_weapon_clear(i);
            return;
        }
        // 0x23E6E: step * ammo; guns 2 and 3 start at 1 and halve the sum
        let step = self.assets.text.u16(0x23EB4, gun_idx as u32 * 2);
        let mut d0: u16 = if gun_idx == 2 || gun_idx == 3 {
            1u16.wrapping_add(step.wrapping_mul(ammo)) >> 1
        } else {
            step.wrapping_mul(ammo)
        };
        let base = self.hud.slots[i].base;
        let mut a = base + 0x1E0;
        for _ in 0..12 {
            let code = if d0 >= 2 {
                d0 -= 2;
                0xCF
            } else if d0 == 1 {
                d0 -= 1;
                0xDF
            } else {
                0x20
            };
            self.hud_put(a, code, 0xB);
            a += 0x80;
        }
    }

    /// 0x23E42: clear the cartridge bar (12 cells with spaces at base + 0x1E0).
    fn hud_clear_cartridge_bar(&mut self, i: usize) {
        let base = self.hud.slots[i].base;
        let mut a = base + 0x1E0;
        for _ in 0..12 {
            self.hud_put(a, 0x20, 0xB);
            a += 0x80;
        }
    }

    /// 0x23B62: draw the 2-digit ammo at base + 0x25C. Red when ammo <= threshold.
    fn hud_draw_gun_ammo(&mut self, i: usize, gun_idx: u16, ammo: u16) {
        let thresh = self.assets.text.u16(0x23BD0, gun_idx as u32 * 2);
        let attr = if ammo > thresh { 0 } else { 2 };
        let bcd = if (ammo as usize) < 80 {
            self.assets.text.u8(0x23BDC, ammo as u32)
        } else {
            to_bcd(ammo) as u8
        };
        let tens = (bcd >> 4) & 0xF;
        let ones = bcd & 0xF;
        let base = self.hud.slots[i].base;
        let a = base + 0x25C;
        let c_tens = if tens == 0 { 0x20 } else { tens as u16 };
        self.hud_put(a, c_tens, attr);
        self.hud_put(a + 0x80, ones as u16, attr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    fn rom_assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            for p in &["dino.zip", "../dino.zip", "../../../dino.zip"] {
                if Path::new(p).exists() {
                    if let Ok(pack) = dino_extract::extract(Path::new(p)) {
                        return Some(Box::leak(Box::new(pack)));
                    }
                }
            }
            None
        })
    }

    fn cell(g: &Game, a: u32) -> (u16, u16) {
        let idx = (a.wrapping_sub(0x908000) / 4) as usize & 0xFFF;
        (g.vram.scroll1[idx].code, g.vram.scroll1[idx].attr)
    }

    /// 0x16B18 tests the attacker's +$82 (any pool) and takes the ring from its +$02: a hero gets the record, an enemy (pool tag 4)
    /// does not, a pool C object does when +$82 (that is +$02 of the next slot) is 0 and its own +$02 is a player.
    #[test]
    fn hero_hit_record_follows_the_attackers_82_and_02() {
        let mut a = AssetPack::default();
        a.enemy.stages = (0..8).map(|_| Default::default()).collect(); // the field visit of `orig_u8` reads the wave tables
        let mut g = Game::new(Box::leak(Box::new(a)));
        g.hud_rings_init();
        g.enemies = crate::game::pool_a_init();
        let hero = Game::player_handle(0);
        let pushed = |g: &Game| g.hud.ring_w;
        // a hero hit by hero 2 (friendly fire): the record goes to player 2's ring
        g.players[1].b.who = 1;
        g.players[0].b.attacker = Game::player_handle(1);
        g.hud_hit(hero, 2);
        assert_eq!(pushed(&g), [0, 0x10, 0]);
        // a hero hit by an enemy credited to player 1 (+$02 = 0): the pool tag 4 at +$82 stops it
        let e = crate::game::POOL_A.0;
        g.enemies.slots[0].b.live = 1;
        g.enemies.slots[0].b.who = 0;
        g.players[0].b.attacker = e;
        g.hud_hit(hero, 2);
        assert_eq!(pushed(&g), [0, 0x10, 0]);
        // a pool C object of player 3: its +$82 is +$02 of the next slot
        let c0 = crate::game::POOL_C.0;
        for k in 0..2 {
            g.effects.slots[k].b.live = 1;
        }
        g.effects.slots[0].b.who = 2;
        g.effects.slots[1].b.who = 1; // +$82 of slot 0 is not 0: no record
        g.players[0].b.attacker = c0;
        g.hud_hit(hero, 2);
        assert_eq!(pushed(&g), [0, 0x10, 0]);
        g.effects.slots[1].b.who = 0; // +$82 = 0: the record goes to player 3
        g.hud_hit(hero, 2);
        assert_eq!(pushed(&g), [0, 0x10, 0x10]);
        // the hero is its own attacker (0x16B26: cmpa.w a0,a6)
        g.players[0].b.attacker = hero;
        g.players[0].b.who = 0;
        g.hud_hit(hero, 2);
        assert_eq!(pushed(&g), [0, 0x10, 0x10]);
    }

    #[test]
    fn bcd() {
        assert_eq!(to_bcd(10), 0x10);
        assert_eq!(to_bcd(51), 0x51);
        // the carry of the low byte goes to the high byte (the vitality 100 of the clear tally is 0x0100, 150 kills 0x0150 ...)
        assert_eq!(to_bcd(100), 0x0100);
        assert_eq!(to_bcd(199), 0x0199);
        assert_eq!(to_bcd(200), 0x0200);
        assert_eq!(to_bcd(8191), 0x8191);
        // bits above 12 are not looked at
        assert_eq!(to_bcd(0x2000 | 5), 5);
        assert_eq!(bcd_add32(0x0000_9500, 0x0000_0500), (0x0001_0000, false));
        assert_eq!(bcd_add32(0x9999_9999, 1), (0, true));
        assert_eq!(bcd_sub32(0x0010_0000, 0x0001_0000, 0), (0x0009_0000, 0));
        assert_eq!(bcd_sub32(0, 0x0001_0000, 0).0, 0x9999_0000);
    }

    #[test]
    fn gun_hud_display_and_ammo_colors() {
        let Some(assets) = rom_assets() else { return };
        let mut g = Game::new(assets);
        g.hud_init();

        let gun_configs: [(u16, u16, u16); 6] = [
            (0, 6, 2),   // GUN: max 6, thresh 2
            (1, 6, 2),   // RIFLE: max 6, thresh 2
            (2, 48, 12), // UZI: max 48, thresh 12
            (3, 48, 12), // M-16A1: max 48, thresh 12
            (4, 6, 2),   // SHOT GUN: max 6, thresh 2
            (5, 4, 1),   // BAZOOKA: max 4, thresh 1
        ];

        let item_h = g.items.handle(0);
        g.players[0].item = item_h;
        g.items.slots[0].b.live = 1;

        let base = g.hud.slots[0].base; // 0x90A014

        for (gun_idx, max_ammo, thresh) in gun_configs {
            let item_type = (1 << 8) | (4 + gun_idx);
            g.players[0].item_type = item_type;

            // 1. Full ammo: white ammo number, full cartridge bar (12 segments)
            g.items.slots[0].b.hp = max_ammo;
            g.hud_weapon_reset(0);
            g.hud_weapon(0);

            assert_eq!(g.hud.slots[0].weapon_active, 1);
            assert_eq!(g.hud.slots[0].weapon_gun, gun_idx);
            assert_eq!(g.hud.slots[0].weapon_ammo, max_ammo);

            // Icon tiles at base + 0x5C have attr 0xC
            assert_eq!(cell(&g, base + 0x5C).1, 0xC);
            assert_eq!(cell(&g, base + 0x5C + 4).1, 0xC);

            // Ammo number at base + 0x25C: attr should be 0 (white) because max_ammo > thresh
            assert_eq!(cell(&g, base + 0x25C).1, 0);

            // Cartridge bar at base + 0x1E0: all 12 cells should be 0xCF with attr 0xB
            for c in 0..12 {
                assert_eq!(cell(&g, base + 0x1E0 + c * 0x80), (0xCF, 0xB));
            }

            // 2. Ammo at red threshold: attr should be 2 (red)
            g.items.slots[0].b.hp = thresh;
            g.hud_weapon(0);
            assert_eq!(g.hud.slots[0].weapon_ammo, thresh);
            assert_eq!(cell(&g, base + 0x25C).1, 2, "ammo at threshold should be red (attr 2)");

            // 3. Ammo at 1: attr should be 2 (red)
            g.items.slots[0].b.hp = 1;
            g.hud_weapon(0);
            assert_eq!(g.hud.slots[0].weapon_ammo, 1);
            assert_eq!(cell(&g, base + 0x25C).1, 2, "ammo at 1 should be red (attr 2)");
        }
    }

    #[test]
    fn other_weapon_types_and_empty_guns() {
        let Some(assets) = rom_assets() else { return };
        let mut g = Game::new(assets);
        g.hud_init();

        let item_h = g.items.handle(0);
        g.players[0].item = item_h;
        g.items.slots[0].b.live = 1;
        let base = g.hud.slots[0].base;

        // Other weapon types (item_type_lo = 1, 2, 3):
        // Var26: 4 = KNIFE, 8 = STICK, 14 = CLUB, 16 = TORCH, etc.
        g.players[0].item_type = (1 << 8) | 2; // melee swing type
        g.items.slots[0].b.var26 = 4; // KNIFE
        g.hud_weapon_reset(0);

        assert_eq!(g.hud.slots[0].weapon_active, 1);
        // ARM icon at base + 0x5C with attr 0xC
        assert_eq!(cell(&g, base + 0x5C).1, 0xC);
        // Cartridge bar at base + 0x1E0 is cleared to spaces (0x20)
        assert_eq!(cell(&g, base + 0x1E0).0, 0x20);

        // Empty gun held as throwable (e.g. item_type_lo = 1, var26 = 2 -> "GUN")
        g.players[0].item_type = (1 << 8) | 1;
        g.items.slots[0].b.var26 = 2; // GUN
        g.hud_weapon_reset(0);

        assert_eq!(g.hud.slots[0].weapon_active, 1);
        assert_eq!(cell(&g, base + 0x5C).1, 0xC);
        assert_eq!(cell(&g, base + 0x1E0).0, 0x20);
    }

    #[test]
    fn three_player_hud_blocks() {
        let Some(assets) = rom_assets() else { return };
        let mut g = Game::new(assets);
        g.hud_init();

        // 3 player mode: bases 0x90A014, 0x90A814, 0x90B014
        assert_eq!(g.hud.slots[0].base, 0x90A014);
        assert_eq!(g.hud.slots[1].base, 0x90A814);
        assert_eq!(g.hud.slots[2].base, 0x90B014);

        // Give weapon to player 0, 1 (middle), and 2 (right)
        for i in 0..3 {
            let item_h = g.items.handle(i);
            g.players[i].item = item_h;
            g.players[i].item_type = (1 << 8) | 4; // GUN
            g.items.slots[i].b.live = 1;
            g.items.slots[i].b.hp = 6;
            g.hud_weapon_reset(i);

            let base = g.hud.slots[i].base;
            assert_eq!(cell(&g, base + 0x5C).1, 0xC);
            assert_eq!(cell(&g, base + 0x25C).1, 0); // white ammo
            assert_eq!(cell(&g, base + 0x1E0), (0xCF, 0xB)); // full bar
        }
    }

    #[test]
    fn weapon_clear_on_drop() {
        let Some(assets) = rom_assets() else { return };
        let mut g = Game::new(assets);
        g.hud_init();

        let item_h = g.items.handle(0);
        g.players[0].item = item_h;
        g.players[0].item_type = (1 << 8) | 4;
        g.items.slots[0].b.live = 1;
        g.items.slots[0].b.hp = 6;
        g.hud_weapon_reset(0);
        assert_eq!(g.hud.slots[0].weapon_active, 1);

        // Drop weapon
        g.players[0].item = 0;
        g.hud_weapon(0);

        assert_eq!(g.hud.slots[0].weapon_active, 0);
        assert_eq!(g.hud.slots[0].weapon_ammo, 0);

        let base = g.hud.slots[0].base;
        // Icon area cleared
        assert_eq!(cell(&g, base + 0x5C).0, 0x20);
        assert_eq!(cell(&g, base + 0x5C + 4).0, 0x20);
    }

    #[test]
    fn item_messages_food_and_treasure_including_quirk() {
        let Some(assets) = rom_assets() else { return };
        let mut g = Game::new(assets);
        g.hud_init();

        let base = g.hud.slots[0].base;

        // Food message
        g.hud_food_msg(0, 1, 100); // BARBECUE, heal 100
        assert_eq!(g.hud.slots[0].timer, 0x3C);
        // FooD icon at base
        assert_eq!(cell(&g, base).1, 0xC);
        assert_eq!(cell(&g, base + 4).1, 0xC);

        // Treasure message
        g.hud_treasure_msg(0, 1); // GOLD
        assert_eq!(g.hud.slots[0].timer, 0x3C);
        // Pts icon at base
        assert_eq!(cell(&g, base).1, 0xC);
        assert_eq!(cell(&g, base + 4).1, 0xC);

        // Treasure 17 quirk: id 17 (idx 16) must not be dropped
        g.hud_treasure_msg(0, 17);
        assert_eq!(g.hud.slots[0].timer, 0x3C);
        assert_eq!(cell(&g, base).1, 0xC);
    }
}
