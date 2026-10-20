//! Pool C class 33 (+$20 = 0x84), handler 0x96B78: the bullet spark of a gun shot. 0x96B40 (`Game::weapon_spawn_flash`, the shot's
//! call after the gun sound) makes it at the weapon with +$24 the weapon's facing and +$60 the kind word of the gun. Its first update
//! (mode 0) makes the gunshot flashes (pool C class 31, `fx31.rs`): the records of the kind (table 0x96D2A, kinds 4 to 9), one flash per
//! count, each with the direction of its place in a scatter row (0x96D8A for a facing owner, 0x96D9A for facing 0; one random row per spark).
//! Then the spark flies up and back from the weapon (vx 2.0 px, vy 4.0 px, gravity -0.703 px per update, life 0x1E + the sequence byte),
//! its picture is the table 0x96D06; after its first bounce (+$60 high byte flag) it is drawn on odd life counts only.
//! The sequence byte G.$515 (`World::ricochet_seq`) counts the sparks mod 4 (0x96BA0).

use super::*;

/// 0x96D06: the spark animation (effect table, one animation).
const SPARK_ANIMS: u32 = 0x96D06;
/// 0x96D2A: six word offsets by kind - 4 (kinds 4 to 9), then six 14 byte records: +0 count - 1 (negative: no flash), +2 the word (anim << 8 |
/// velocity selector), +4 the X offset of the flashes (negated for facing 0), +6 their Y offset (zero extended), +8 the byte that frees the spark
/// when 0, +10 and +12 the X (negated for facing 0) and Y offset of the spark. The scatter rows follow at 0x96D8A and 0x96D9A.
const RECORDS: u32 = 0x96D2A;
/// 0x96D8A: rows of four directions for a facing owner (+$24 not 0).
const SCATTER_FACING: u32 = 0x96D8A;
/// 0x96D9A: rows of four directions for facing 0.
const SCATTER_BACK: u32 = 0x96D9A;
/// +$6C, the gravity of the spark (a long, -0x4800 = -0.703 px per update).
const GRAVITY: i32 = -0x4800;

impl Game {
    /// 0x96B78: mode 0 (init, then the flight in the same update), 2 the flight; 4 and 6 free the slot (0x96D00).
    pub(super) fn fx_ricochet(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                if !self.fx_ricochet_init(i) {
                    return;
                }
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        self.fx_ricochet_flight(i);
    }

    /// The record of a gun kind (the word offset by kind - 4 at 0x96D2A). `None` for kinds outside 4..9 (no gun has them).
    fn ricochet_record(&self, kind: u8) -> Option<u32> {
        let k = kind.wrapping_sub(4);
        (k < 6).then(|| RECORDS + u32::from(self.assets.enemy.t.u16(RECORDS, u32::from(k) * 2)))
    }

    /// 0x96B78 mode 0 up to the mode change: the sequence byte, the flashes, the position and the velocities. Returns false when the spark is freed.
    fn fx_ricochet_init(&mut self, i: usize) -> bool {
        let a = self.assets;
        let fx = self.fx_data();
        self.w.ricochet_seq = self.w.ricochet_seq.wrapping_add(1) & 3; // addi.b #1 ; andi.b #3 on G.$515
        let kind = (self.effects.slots[i].owner60 & 0xFF) as u8;
        let Some(rec) = self.ricochet_record(kind) else {
            self.fx_free(i);
            return false;
        };
        let t = &a.enemy.t;
        // 0x96BB2: a negative count makes no flash (bmi)
        if t.i8(rec, 0) >= 0 {
            self.fx_ricochet_flashes(i, rec);
        }
        // 0x96C1C: the flag byte +8 of the record frees the spark when it is 0
        if t.u8(rec, 8) == 0 {
            self.fx_free(i);
            return false;
        }
        // 0x96C26: kinds 6 and 7 give no spark when the sequence byte is odd
        if (kind == 6 || kind == 7) && self.w.ricochet_seq & 1 != 0 {
            self.fx_free(i);
            return false;
        }
        let (dx, dy) = (t.i16(rec, 10), t.i16(rec, 12));
        let e = &mut self.effects.slots[i];
        let facing = e.b.facing != 0;
        // 0x96C44: the offset of the spark (the X offset is negated for facing 0)
        let dx = if facing { dx } else { dx.wrapping_neg() };
        let (nx, ny) = (e.b.xi().wrapping_add(dx), e.b.yi().wrapping_add(dy));
        set_word(&mut e.b.x, nx);
        set_word(&mut e.b.y, ny);
        // 0x96C5E: vx = -2.0 px for facing 1, +2.0 for facing 0; vy = +4.0 px
        e.vx64 = if facing { -0x20000 } else { 0x20000 };
        e.vy68 = 0x40000;
        e.b.anim_table = SPARK_ANIMS;
        e.b.draw_flags = 0x10;
        fx_start(&mut e.b, fx, 0); // jsr 0x120E, animation 0
        // 0x96C98: the life is the sequence byte + 0x1E. The original keeps it in the high byte of the word +$62 (off_x) and the
        // speed long vx64 at +$64 in the words +$64 (off_y) and +$66 (w66): the stale bytes of a freed spark stay in those words.
        e.off_y = (e.vx64 >> 16) as i16;
        e.w66 = e.vx64 as u16;
        let life = self.w.ricochet_seq.wrapping_add(0x1E);
        e.off_x = ((life as u16) << 8 | (e.off_x as u16 & 0xFF)) as i16;
        e.b.mode = e.b.mode.wrapping_add(2);
        e.owner60 &= 0x00FF; // clr.b $60(a6): the bounce flag
        true
    }

    /// 0x96BB6 .. 0x96C18: the flashes of the record `rec` (the loop of 0x967C4), one random scatter row per spark.
    fn fx_ricochet_flashes(&mut self, i: usize, rec: u32) {
        let a = self.assets;
        let facing = self.effects.slots[i].b.facing != 0;
        let row = (self.rand() & 3) as u32 * 4; // jsr 0x119C ; andi.l #3 ; lsl.w #2
        let t = &a.enemy.t;
        let scatter = (if facing { SCATTER_FACING } else { SCATTER_BACK }) + row;
        let sel = t.u16(rec, 2);
        // the X offset is the word +4 (negated for facing 0), the Y offset the word +6 zero extended
        let w4 = t.u16(rec, 4);
        let hi = if facing { w4 } else { w4.wrapping_neg() };
        let off = ((hi as u32) << 16).wrapping_add(t.u16(rec, 6) as u32);
        // 0x96BDA .. 0x96BFE: clr.l $68(a6), then move.w (the selector) to $64 and the offset long to $68. This is before the early
        // frees of 0x96C1C / 0x96C40, so a freed spark leaves them in its slot (the next object of the slot reads them).
        let e = &mut self.effects.slots[i];
        e.off_y = sel as i16;
        e.vy68 = off as i32;
        let count = u32::from(t.u8(rec, 0)) + 1; // dbra: the count - 1 plus one turns
        for k in 0..count {
            let dir = t.u8(scatter, k); // move.b (a2)+, $22(a6)
            self.effects.slots[i].b.dir = dir;
            let b = &self.effects.slots[i].b;
            let pos = (b.xi(), b.yi(), b.zi());
            self.fx_spawn_muzzle(pos, sel, off, dir); // jsr 0x967C4
        }
    }

    /// 0x96CAC: the life (+$62 counts down; below zero frees), the picture, the motion, the draw (after the bounce only on odd life counts).
    fn fx_ricochet_flight(&mut self, i: usize) {
        let fx = self.fx_data();
        let e = &mut self.effects.slots[i];
        let life = ((e.off_x as u16) >> 8) as u8; // subq.b #1, $62 (the high byte of off_x)
        let life = life.wrapping_sub(1);
        e.off_x = ((life as u16) << 8 | (e.off_x as u16 & 0xFF)) as i16;
        if life & 0x80 != 0 {
            return self.fx_free(i);
        }
        fx_tick(&mut e.b, fx); // jsr 0x15F4 (the result is not read)
        e.b.x = e.b.x.wrapping_add(e.vx64);
        e.vy68 = e.vy68.wrapping_add(GRAVITY);
        e.b.y = e.b.y.wrapping_add(e.vy68);
        let draw = if e.owner60 >> 8 != 0 {
            life & 1 != 0 // 0x96CEE: after the bounce, drawn when the life is odd
        } else {
            if e.vy68 < 0 {
                e.owner60 = (e.owner60 & 0x00FF) | 0x0100; // move.b #1, $60: the bounce flag
            }
            true
        };
        if draw {
            self.draw_list_add_check(self.effects.handle(i)); // jmp 0x49CA
        }
    }
}
