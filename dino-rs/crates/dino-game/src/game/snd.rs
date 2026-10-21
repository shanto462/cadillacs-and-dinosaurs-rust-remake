//! Sound requests of the 68000 side: the queue write 0xA63C and the small routines that choose
//! the sound code for a player voice (0x95CE..0x9C0C), a hit (0x962A) and the extra life jingle
//! (0xA4CA). Only the sound code is recorded (`Game::sound`); the pan / volume longs d1 and d2 of
//! the queue entry belong to the sound port (plan phase 3, task 4).
//!
//! Many routines alternate between two codes with `bchg #n,$43A(a5)`: the toggle bits of the
//! byte G.$43A are shared by all callers, so the codes depend on the history of all requests.

use super::body::Body;
use super::{Game, Handle, slot_of};
use dino_assets::HitBox;

/// 0xA63C: one entry of the sound queue, the code word and the two longs the sound port reads: d1 (the pan mode in the
/// high word, the pan byte in the low byte, 0x10 = no pan) and d2 (volume and the like). Every request is recorded
/// with its d1 and d2 (`Game::sound_ex`, `W.snd_reqs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SndRequest {
    pub code: u16,
    pub d1: u32,
    pub d2: u32,
}

/// 0x610..0x69E: does the sound flush send this queued code to the sound chip? A code whose high
/// byte is 0xFF (0xFF00 stop, 0xFF05, 0xFF07 fade, 0xFF0B reset) always goes. Any other code goes
/// only in a game (G.$4CC is not 0, which includes the press-start-to-select moment), or when it
/// is exactly 0x0030 (the coin sound), or when the demo sound setting is on (G.$777C bit 5, factory
/// off). So the title, profiles, demo and movie are silent and only the coin sound is heard.
#[allow(dead_code)] // used by the audio side when it exists
pub fn snd_passes(code: u16, in_game: bool, demo_sound: bool) -> bool {
    code & 0xFF00 == 0xFF00 || in_game || code == 0x0030 || demo_sound
}

impl Game {
    /// The requests of this frame that reach the sound chip (the attract filter of 0x610). The
    /// game code keeps calling `sound()` as the original queues every code; the audio side reads
    /// this list instead of `w.sounds`. (The flush also sends one ring entry per frame when the
    /// Z80 is ready: that timing belongs to the sound port.)
    #[allow(dead_code)] // read by the audio side when it exists
    pub fn snd_heard(&self) -> Vec<u16> {
        let (in_game, demo) = (self.flow.in_game != 0, self.flow.dip.demo_sound());
        self.w.sounds.iter().copied().filter(|&c| snd_passes(c, in_game, demo)).collect()
    }

    /// `bchg.b #bit,$43A(a5)`: flips the bit and returns what it was (`bne` taken when true).
    fn snd_toggle(&mut self, bit: u8) -> bool {
        let old = self.w.snd_toggle >> bit & 1 != 0;
        self.w.snd_toggle ^= 1 << bit;
        old
    }

    /// `bchg.b #bit,$43B(a5)`: flips the bit of the toggle byte of the item and prop sounds and returns what it was
    /// (`bne` taken when true). The item sounds (`item/shared.rs`) flip the same byte.
    #[allow(dead_code)] // used by the scenery props of M3
    pub fn snd_toggle_b(&mut self, bit: u8) -> bool {
        let old = self.spawn.g.snd_toggle_b >> bit & 1 != 0;
        self.spawn.g.snd_toggle_b ^= 1 << bit;
        old
    }

    /// `bchg.b #bit,$43E(a5)`: the biker's engine and fall (0xA128, 0xA164). Returns the old bit (`bne` when it was set).
    pub fn snd_toggle_e(&mut self, bit: u8) -> bool {
        let old = self.w.snd_toggle_e >> bit & 1 != 0;
        self.w.snd_toggle_e ^= 1 << bit;
        old
    }

    /// 0xA63C with its three registers (d0 = code, d1, d2): the same as `sound_ex`.
    pub fn snd_request(&mut self, code: u16, d1: u32, d2: u32) {
        self.sound_ex(code, d1, d2);
    }

    /// The requests of this frame, oldest first, and empties the list (the tests take them; the frame clears them too).
    #[allow(dead_code)] // used by the tests of the scenery props
    pub fn snd_take_requests(&mut self) -> Vec<SndRequest> {
        std::mem::take(&mut self.w.snd_reqs)
    }

    /// The requests of this frame for the audio side (M4-12, `crate::audio`): every request (`W.snd_reqs`) that the
    /// attract filter of 0x610 lets through, in queue order, each with its d1 and d2 (M4-13: no pairing needed).
    pub fn snd_feed_requests(&self) -> Vec<SndRequest> {
        let (in_game, demo) = (self.flow.in_game != 0, self.flow.dip.demo_sound());
        self.w.snd_reqs.iter().filter(|r| snd_passes(r.code, in_game, demo)).copied().collect()
    }

    /// 0x9E64..0x9E84 (the same lines are in 0x9EA2, 0x9EDC and 0x9F12): the pan byte of an object at world X `x`.
    /// The screen position `x - camera X` (G.$744) is 0 when it is negative and 0x17F at 0x180 or more (`bcs`,
    /// unsigned), then `(pos & 0x1F8) >> 3` indexes the table 0xA66A of 48 bytes (0 at the left edge to 0x20 at the
    /// right edge, 0x10 in the middle).
    #[allow(dead_code)] // used by the scenery props of M3
    pub fn snd_pan_x(&self, x: i16) -> u8 {
        self.assets.enemy.t.u8(0xA66A, self.snd_pan_index(x))
    }

    /// The index into 0xA66A of the pan of world X `x` (`(pos & 0x1F8) >> 3`, 0 .. 0x2F), as the routines leave it in d2.
    pub fn snd_pan_index(&self, x: i16) -> u32 {
        let mut d = x.wrapping_sub(self.cam.s2.cam_x as i16); // sub.w $744(a5),d0
        if d < 0 {
            d = 0; // bpl
        }
        if d as u16 >= 0x180 {
            d = 0x17F;
        }
        ((d as u16 & 0x1F8) >> 3) as u32
    }

    /// 0x9E60: the break sound of walls, doors and weapons with the pan of the object at world X `x` (+$08): code 0x6F, or
    /// 0xF6 when bit 0 of G.$43B was clear (`bchg`, so the first request after a clear is 0xF6 and they alternate; the
    /// item code flips the same bit), d1 = the pan byte, d2 = 0.
    #[allow(dead_code)] // used by the scenery props of M3
    pub fn snd_break(&mut self, x: i16) {
        let pan = self.snd_pan_x(x);
        let old = self.snd_toggle_b(0);
        self.snd_request(if old { 0x6F } else { 0xF6 }, pan as u32, 0);
    }

    /// 0xA502 (called at the start of 0x18834): each player's voice bank +$118 from which
    /// players are present (table 0xA53A, 4 bytes per mix).
    pub fn snd_update_voice_banks(&mut self) {
        let mut mix = 0u32;
        for (i, bit) in [4u32, 8, 0x10].into_iter().enumerate() {
            if self.players[i].b.live != 0 {
                mix |= bit;
            }
        }
        let t = &self.assets.text;
        for i in 0..3 {
            self.players[i].voice = t.u8(0xA53A, mix + i as u32);
        }
    }

    /// The player voice and effect routines 0x95CE..0x9C0C, called with a6 = player `i`. Each request carries the pan
    /// byte of the player's X as d1 (`snd_pan_x`) and the constant d2 of its routine (`move.l #..,d2` before the call).
    pub fn snd_player(&mut self, i: usize, routine: u32) {
        self.snd_player_d2(i, routine, 0)
    }

    /// `snd_player` for the routines that do not set d2 themselves and send what the caller left in it: 0x9C0C (the
    /// landing) and 0x9C20 (its entry after the wading test, the pick-up voice). `d2_left` is the caller's d2 at the `jsr` (0 when the caller's value is not modelled); the other
    /// routines ignore it.
    pub fn snd_player_d2(&mut self, i: usize, routine: u32, d2_left: u32) {
        let (id, k20, voice, wading, y, x, tag) = {
            let p = &self.players[i];
            (p.b.id, p.b.k20 as u32, p.voice as u32, p.b.wading, p.b.yi(), p.b.xi(), 0u8)
        };
        let t = &self.assets.text;
        // (code, d2)
        let (code, d2): (u8, u32) = match routine {
            // 0x95CE: 0x38 / 0xA8 (toggle bit 0; the first request after a reset is 0xA8).
            0x95CE => (if self.snd_toggle(0) { 0x38 } else { 0xA8 }, 0),
            // 0x9796: 0x72 / 0xF8 (toggle bit 3).
            0x9796 => return self.snd_9796(x),
            // 0x986C / 0x98CC: id 2 uses the second row of three. d2 = 0x2000000 / 0x5608D0.
            0x986C => (t.u8(0x98C6, if id == 2 { 3 } else { 0 } + voice), 0x0200_0000),
            0x98CC => (t.u8(0x9926, if id == 2 { 3 } else { 0 } + voice), 0x0056_08D0),
            // 0x992C / 0x99E8 / 0x9A36: +$20 (character * 4) selects the row of four. d2 = 0xD000000 (special), 0 (0x99E8 and
            // 0x9A36 end with `moveq #0,d2` before the `bra 0xA63C`: the pan index in d2 is cleared).
            0x992C => (t.u8(0x9974, k20 + voice), 0x0D00_0000),
            0x99E8 => (t.u8(0x9A26, k20 + voice), 0),
            0x9A36 => (t.u8(0x9A74, k20 + voice), 0),
            // 0x9984: rows of three by id (0, 1, 2). d2 = 0xD000000.
            0x9984 => (t.u8(0x99DE, if id == 2 { 6 } else if id == 1 { 3 } else { 0 } + voice), 0x0D00_0000),
            // 0x9AF4: players (+$82 == 0) use the voice table; other objects toggle bit 4. d2 = 0xC41240.
            0x9AF4 => {
                if tag == 0 {
                    (t.u8(0x9B5C, voice), 0x00C4_1240)
                } else if self.snd_toggle(4) {
                    (0xEF, 0x00C4_1240)
                } else {
                    (0xF0, 0x00C4_1240)
                }
            }
            // 0x9B60: 0x6C / 0xF2 (toggle bit 5, shared with 0x9BCE). 0x9B9E: 0x75. d2 = 0.
            0x9B60 => (if self.snd_toggle(5) { 0x6C } else { 0xF2 }, 0),
            0x9B9E => (0x75, 0),
            // 0x9BCE: 0x71 / 0xF7 (toggle bit 5).
            0x9BCE => (if self.snd_toggle(5) { 0x71 } else { 0xF7 }, 0),
            // 0x9C0C: no sound while wading (+$53 negative) below height 0x20; players use the
            // voice table, other objects toggle bit 6 (0xE0 / 0xE1). The routine never writes d2: the request
            // carries the d2 the caller had (`d2_left`). 0x9C20 is the same routine after the wading test (the
            // pick-up voice of 0x19832 calls it).
            0x9C0C | 0x9C20 => {
                if routine == 0x9C0C && (wading as i8) < 0 && y < 0x20 {
                    return;
                }
                if tag == 0 {
                    (t.u8(0x9C72, voice), d2_left)
                } else if self.snd_toggle(6) {
                    (0xE0, d2_left)
                } else {
                    (0xE1, d2_left)
                }
            }
            r => panic!("unknown player sound routine {r:#x}"),
        };
        // d1: the mode (the object's +$24 facing picks the second value) in the high word, then the word of the
        // routine (the low byte is replaced by the pan byte): 0x986C 0x20xx, 0x98CC 0x06xx, 0x992C and 0x9984 mode 7
        // (`move.l #$70000,d1`, no word), 0x9AF4 (mode 5 or 6, 0x12xx).
        let facing_right = self.players[i].b.facing != 0;
        let hi = match routine {
            0x986C => (if facing_right { 4 } else { 3 }) << 16 | 0x2000,
            0x98CC => (if facing_right { 6 } else { 5 }) << 16 | 0x600,
            0x992C | 0x9984 => 7 << 16,
            0x9AF4 => (if facing_right { 6 } else { 5 }) << 16 | 0x1200, // moveq #5 / #6,d1 ; swap ; move.w #$1200,d1
            _ => 0,
        };
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(code as u16, hi | pan, d2);
    }

    /// 0x9A84: the kill voice of the player who owns an object (an enemy burned to death calls it from 0x2AC56 / 0x49EFA).
    /// `who` is a6's +$02, the owner slot (negative = nobody: `bmi rts`). The code is the table 0x9AE4 by the owner's
    /// character (+$20) and voice bank (+$118) (0x2C..0x2F, 0xA0..0xA7), d1 = the pan byte, d2 = 0. A bug of the original: the
    /// pan position is `move.w $8(a0),d2` with a0 the table row (not the object), so the word is the ROM word 8 bytes into the
    /// table row, minus the camera X. Slots 3 and up index past the table of three players: no sound here.
    pub fn snd_9a84(&mut self, who: u8) {
        if (who as i8) < 0 || who >= 3 {
            return;
        }
        let (k20, voice) = (self.players[who as usize].b.k20 as u32, self.players[who as usize].voice as u32);
        let t = &self.assets.text;
        let code = t.u8(0x9AE4, k20 + voice);
        let pos = t.u16(0x9AE4, k20 + 8) as i16;
        let pan = self.snd_pan_x(pos) as u32;
        self.sound_ex(code as u16, pan, 0);
    }

    /// 0x9AF4 for an object that is not a hero (+$82 set: an enemy): the splash of 0x606E. Code 0xEF, or 0xF0 when bit 4 of G.$43A was clear (`bchg`, the
    /// player voices flip the same bit), d1 = (5, or 6 when +$24 is nonzero) << 16 | 0x1200 | the pan byte of its X, d2 = 0xC41240.
    pub fn snd_splash_object(&mut self, x: i16, facing: u8) {
        let code = if self.snd_toggle(4) { 0xEF } else { 0xF0 };
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(code, ((if facing != 0 { 6 } else { 5 }) << 16) | 0x1200 | pan, 0x00C4_1240);
    }

    /// 0x9796 (knockdown / landing thud of players and enemies): 0x72 / 0xF8, toggle bit 3. The code
    /// does not depend on the object.
    /// d1 = the pan byte of the object's X (`x`, from the caller's a6), d2 = 0.
    pub fn snd_9796(&mut self, x: i16) {
        let code = if self.snd_toggle(3) { 0x72 } else { 0xF8 };
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(code, pan, 0);
    }

    /// 0xA354 (code 0x79, d1 = (3, or 4 when +$24 is nonzero) << 16 | 0x2000 | pan, d2 = 0x2400000) and 0x97D2 (code
    /// 0x58, d1 = (5, or 6) << 16 | 0xA00 | pan, d2 = 0x5608D0): an enemy's death voice. The hit library picks
    /// 0x97D2 when `+$63` is positive (`tst.b $63(a6) ; bgt`), 0xA354 otherwise. `x` and `facing` are a6's +$08 and +$24.
    pub fn snd_enemy_death(&mut self, b63: u8, x: i16, facing: u8) {
        let pan = self.snd_pan_x(x) as u32;
        if (b63 as i8) > 0 {
            self.snd_97d2(x, facing);
        } else {
            let mode = if facing != 0 { 4 } else { 3 };
            self.sound_ex(0x79, (mode << 16) | 0x2000 | pan, 0x0240_0000);
        }
    }

    /// 0x97D2: the death voice 0x58 of an enemy (also called directly by the burn knockdown at 0x2ACE0): d1 = (5, or 6 when +$24 is
    /// nonzero) << 16 | 0xA00 | the pan byte of X, d2 = 0x5608D0.
    pub fn snd_97d2(&mut self, x: i16, facing: u8) {
        let pan = self.snd_pan_x(x) as u32;
        let mode = if facing != 0 { 6 } else { 5 };
        self.sound_ex(0x58, (mode << 16) | 0xA00 | pan, 0x0056_08D0);
    }

    /// 0x962A: the hit sound of an attack record (`sound` = record byte 4, a byte offset into
    /// the table 0x96AA of 3-byte entries: toggle bit number, code, code). The original toggles
    /// the bit in a register copy of G.$43C only, so G.$43C decides and never changes.
    /// d1 = (3, or 4 when the byte +$24 of a6 is not 0) << 16 | 0x2000 | the pan byte of a2's X (+$08), d2 = 0x15000000.
    /// `ax` is a2's X, `a6_24` the byte +$24 of the a6 the caller left (`snd_a6_byte`; an object's facing when a6 is an object).
    pub fn snd_hit(&mut self, sound: u8, ax: i16, a6_24: u8) {
        let off = sound as i8 as i32 as u32; // ext.w d0 (then used as a word offset)
        let t = &self.assets.text;
        let bit = t.u8(0x96AA, off) as u32;
        let set = (self.w.snd_hit_toggle as u32 >> bit) & 1 != 0;
        let code = t.u8(0x96AA, off + 1 + set as u32);
        let mode = if a6_24 != 0 { 4 } else { 3 };
        let pan = self.snd_pan_x(ax) as u32;
        self.sound_ex(code as u16, (mode << 16) | 0x2000 | pan, 0x1500_0000);
    }

    /// The byte that 0x962A tests at +$24 of a6 (`tst.b $24(a6)`: the mode 4 when it is not 0, else 3). None of the 21 callers of
    /// 0x962A loads a6 for the sound: a6 is what the code before the call left, a 24-bit address here:
    /// - the box record that the box test 0x128F8 stepped to (`snd_a6_128f8`): a ROM byte of the box tables;
    /// - a table that a damage routine pointed a6 at: 0x2BB4E (the end of 0x2B904, 0x2BA10), 0x2BC4E (the end of 0x2BA2A, 0x2BB08),
    ///   0x11CC4 (the reaction table of the monster passes, 0x1144C / 0x11524 / 0x11BEC / 0x12406);
    /// - the box record after 0x6422, which saves a6 with `move.w a6,-(a7)` and restores it with `movea.w (a7)+,a6` (0x6434 / 0x64EE): the
    ///   low word sign extended, so a byte of the low program (below 0x8000) or of RAM (`snd_a6_word`);
    /// - an object (0xFF0000 + its handle: the partner +$70, or the object of the routine): its +$24, the facing.
    ///
    /// RAM that is not an object start is not kept by the port (0).
    pub(crate) fn snd_a6_byte(&self, a6: u32) -> u8 {
        let a = a6.wrapping_add(0x24) & 0xFF_FFFF;
        if a >= 0xFF_0000 {
            let h = a6 as u16;
            return if slot_of(h).is_some() { self.body(h).facing } else { 0 };
        }
        self.snd_rom_u8(a)
    }

    /// An object as a6 (`lea (a2),a6`, `movea.w $70(a2),a6`): the RAM address of its slot.
    pub(crate) fn snd_a6_obj(h: Handle) -> u32 {
        0xFF_0000 | h as u32
    }

    /// 0x6422 (and 0x125C0 / 0x12686 around 0x2B904) keep a6 as a word only: `movea.w (a7)+,a6` sign extends the low word.
    pub(crate) fn snd_a6_word(a6: u32) -> u32 {
        a6 as u16 as i16 as i32 as u32 & 0xFF_FFFF
    }

    /// 0x128F8: the address a6 holds when the box test returns with a hit (carry set); `ab` is a2 (the attacker), `db` a3 (the
    /// defender), `ib` the defender's box index (d1) and `hb` the defender box that matched.
    /// - 0x12916 (neither has multi boxes, +$4A): a6 = table + 12 * ib, then six `(a6)+` reads (0x1293C, 0x12944, 0x12966, 0x12968, 0x1298E,
    ///   0x12996): + 12;
    /// - 0x129A8 (attacker single, defender multi): a6 = the sub table + 2 (after the count word, 0x129C4) + 12 per box tried before the hit;
    /// - 0x12A5C (attacker multi, defender single): a6 = table + 12 * ib (no post-increments);
    /// - 0x12B0E (both multi): a6 = the sub table + 2, restored from the stack at the hit (0x12BBC).
    pub(crate) fn snd_a6_128f8(&self, ab: &Body, db: &Body, ib: u8, hb: &HitBox) -> u32 {
        let table = db.box_table & 0xFF_FFFF;
        // `add.w d1,d1 ; move.w (a6,d1.w),d1 ; lea (a6,d1.w),a6`: the sub table of a multi box entry
        let sub = || table.wrapping_add(self.snd_rom_i16(table.wrapping_add(2 * ib as u32)) as i32 as u32);
        match (ab.multi_box, db.multi_box) {
            (false, false) => table.wrapping_add(12 * ib as u32).wrapping_add(12),
            (false, true) => {
                let k = self.assets.sprites.hitboxes.multi(table, ib).iter().position(|b| b == hb).unwrap_or(0) as u32;
                sub().wrapping_add(2).wrapping_add(12 * k)
            }
            (true, false) => table.wrapping_add(12 * ib as u32),
            (true, true) => sub().wrapping_add(2),
        }
    }

    /// A ROM byte that a quirk reads as data (the box tables, the low program): the hit box bank, the extracted tables, then the program image.
    pub(crate) fn snd_rom_u8(&self, addr: u32) -> u8 {
        let addr = addr & 0xFF_FFFF;
        let bank = &self.assets.sprites.hitboxes;
        if addr >= bank.base {
            let offset = (addr - bank.base) as usize;
            if let Some(&w) = bank.words.get(offset / 2) {
                return if offset.is_multiple_of(2) { (w as u16 >> 8) as u8 } else { w as u8 };
            }
        }
        for (start, data) in self.assets.enemy.t.blobs.iter().chain(self.assets.text.blobs.iter()) {
            if addr >= *start && ((addr - start) as usize) < data.len() {
                return data[(addr - start) as usize];
            }
        }
        self.assets.enemy.t.program.get(addr as usize).copied().unwrap_or(0)
    }

    /// The big-endian word of `snd_rom_u8`.
    pub(crate) fn snd_rom_i16(&self, addr: u32) -> i16 {
        i16::from_be_bytes([self.snd_rom_u8(addr), self.snd_rom_u8(addr.wrapping_add(1))])
    }

    /// 0xA4CA: extra life jingle 0x4A. The routine computes the pan byte into d1 and then overwrites d1 with the constant
    /// (`move.l #$72000,d1`, a bug of the original), so the pan never counts; d2 = 0x10000000.
    pub fn snd_extra_life(&mut self) {
        self.sound_ex(0x4A, 0x0007_2000, 0x1000_0000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::text::TextData;
    use dino_assets::AssetPack;

    fn game_with_tables(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.text = TextData { blobs };
        Game::new(Box::leak(Box::new(a)))
    }

    /// A game whose pan table 0xA66A (48 bytes, `enemy.t`) counts 0, 1, 2 ... (synthetic: no ROM bytes).
    fn game_with_pan_table() -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = dino_assets::enemy::ByteTables { blobs: vec![(0xA66A, (0..0x30).collect())], ..Default::default() };
        Game::new(Box::leak(Box::new(a)))
    }

    /// 0x95CE and 0x9BCE alternate with `bchg`: the first request after a clear takes the second
    /// code, and the toggle bits are shared per bit, not per routine (0x9B60 and 0x9BCE share bit 5).
    #[test]
    fn alternating_codes_follow_the_toggle_byte() {
        let mut g = game_with_pan_table(); // the players' d1 is a pan byte (0xA66A)
        g.snd_player(0, 0x95CE);
        g.snd_player(0, 0x95CE);
        g.snd_player(0, 0x9BCE);
        g.snd_player(0, 0x9B60);
        g.snd_player(0, 0x9BCE);
        assert_eq!(g.w.sounds, vec![0xA8, 0x38, 0xF7, 0x6C, 0xF7]);
        assert_eq!(g.w.snd_toggle, 0b0010_0000);
        // d1 is the pan byte of player 0's X (0 here, the table counts 0, 1, 2 ...), d2 is the routine's constant
        assert_eq!(g.w.snd_reqs.iter().map(|r| (r.d1, r.d2)).collect::<Vec<_>>(), vec![(0, 0); 5]);
    }

    /// M4-19: the splash of an object that is not a player (0x9AF4, `+$82 != 0`) toggles bit 4 of G.$43A (the first request after a clear
    /// is 0xF0, then 0xEF), d1 = (5, or 6 when +$24 is set) << 16 | 0x1200 | pan, d2 = 0xC41240; the direct call of the enemy death voice
    /// 0x97D2 (burn knockdown, 0x2ACE0) has the mode 5 / 6, 0xA00, the pan and d2 = 0x5608D0.
    #[test]
    fn enemy_splash_and_death_voice_arguments() {
        let mut g = game_with_pan_table();
        g.snd_splash_object(0, 0);
        g.snd_splash_object(0, 1);
        g.snd_splash_object(0, 0);
        g.snd_97d2(0, 1);
        g.snd_97d2(0, 0);
        let r: Vec<_> = g.w.snd_reqs.iter().map(|r| (r.code, r.d1, r.d2)).collect();
        assert_eq!(
            r,
            vec![
                (0xF0, 0x5_1200, 0xC4_1240),
                (0xEF, 0x6_1200, 0xC4_1240),
                (0xF0, 0x5_1200, 0xC4_1240),
                (0x58, 0x6_0A00, 0x56_08D0),
                (0x58, 0x5_0A00, 0x56_08D0),
            ]
        );
        assert_eq!(g.w.snd_toggle & 0x10, 0x10, "bit 4 of G.$43A after three flips");
    }

    /// The attract filter (0x610): control codes and the coin sound always pass, everything else
    /// only in a game or with the demo sound setting on.
    #[test]
    fn attract_filter_passes_control_codes_and_the_coin_sound() {
        for code in [0xFF00u16, 0xFF05, 0xFF07, 0xFF0B, 0xFFFF] {
            assert!(snd_passes(code, false, false), "{code:#x}");
        }
        assert!(snd_passes(0x0030, false, false)); // the coin
        for code in [0x0088u16, 0x0080, 0x0031, 0x0013, 0x0130, 0x0024] {
            assert!(!snd_passes(code, false, false), "{code:#x}"); // title tune, ranking, start, ...
            assert!(snd_passes(code, true, false), "{code:#x}"); // in a game
            assert!(snd_passes(code, false, true), "{code:#x}"); // demo sound on
        }
    }

    /// `snd_heard` applies it to the frame's requests, with the game state and the setting.
    #[test]
    fn heard_sounds_follow_the_game_state_and_the_setting() {
        let mut g = game_with_tables(Vec::new());
        for c in [0xFF00, 0x88, 0x30, 0x31] {
            g.sound(c);
        }
        assert_eq!(g.snd_heard(), vec![0xFF00, 0x30]); // attract, factory settings
        g.flow.in_game = 1;
        assert_eq!(g.snd_heard(), vec![0xFF00, 0x88, 0x30, 0x31]);
        g.flow.in_game = 0;
        g.flow_apply_config(crate::game::flow::Config { demo_sound: true, ..Default::default() });
        assert_eq!(g.snd_heard(), vec![0xFF00, 0x88, 0x30, 0x31]);
    }

    /// The hit sound table has 3-byte entries (bit number, code if the bit is clear, code if set).
    #[test]
    fn hit_sound_picks_the_code_by_the_toggle_word() {
        let t = vec![0, 0, 0, 1, 0x11, 0x22, 2, 0x33, 0x44];
        let mut a = AssetPack::default();
        a.text = TextData { blobs: vec![(0x96AA, t)] };
        a.enemy.t = dino_assets::enemy::ByteTables { blobs: vec![(0xA66A, (0..0x30).collect())], ..Default::default() };
        let mut g = Game::new(Box::leak(Box::new(a)));
        g.snd_hit(3, 0, 0);
        g.w.snd_hit_toggle = 0b10;
        g.snd_hit(3, 0, 0);
        g.snd_hit(6, 0, 0);
        assert_eq!(g.w.sounds, vec![0x11, 0x22, 0x33]);
        // d1 = (3 or 4) << 16 | 0x2000 | pan, d2 = 0x15000000 (the facing of the victim picks 3 or 4)
        assert_eq!(g.w.snd_reqs.iter().map(|r| (r.d1, r.d2)).collect::<Vec<_>>(), vec![(0x3_2000, 0x1500_0000); 3]);
    }

    /// 0xA502: the voice bank of each player from the players present (4, 8, 0x10 per player).
    #[test]
    fn voice_banks_follow_the_player_mix() {
        let mut table = vec![0u8; 0x20];
        table[4 + 1] = 7; // only player 1 present: player 2's bank (a made-up value)
        table[0x1C] = 3; // all three present: player 1's bank
        let mut g = game_with_tables(vec![(0xA53A, table)]);
        g.players[0].b.live = 1;
        g.snd_update_voice_banks();
        assert_eq!([g.players[0].voice, g.players[1].voice, g.players[2].voice], [0, 7, 0]);
        for p in g.players.iter_mut() {
            p.b.live = 1;
        }
        g.snd_update_voice_banks();
        assert_eq!(g.players[0].voice, 3);
    }

    /// 0x9E60 as D4e acceptance 9 describes the toggle: two breaks give 0xF6 then 0x6F (the first request after a
    /// clear takes the second code), or the other order when bit 0 of G.$43B was set; the byte is shared with the
    /// item sounds, and the other bits stay. The ring keeps d1 (the pan byte) and d2.
    #[test]
    fn the_break_sound_alternates_with_the_toggle_of_g43b() {
        let mut g = game_with_pan_table();
        g.cam.s2.cam_x = 0x100;
        g.snd_break(0x100 + 8 * 5);
        g.snd_break(0x100 + 8 * 5);
        g.snd_break(0x100 + 8 * 5);
        assert_eq!(g.w.sounds, vec![0xF6, 0x6F, 0xF6]);
        assert_eq!(g.spawn.g.snd_toggle_b, 1);
        g.spawn.g.snd_toggle_b = 0b0100_0001; // bit 0 already set: the other order, other bits untouched
        g.w.sounds.clear();
        g.snd_break(0x100);
        g.snd_break(0x100);
        assert_eq!(g.w.sounds, vec![0x6F, 0xF6]);
        assert_eq!(g.spawn.g.snd_toggle_b, 0b0100_0001, "two flips");
        let ring = g.snd_take_requests();
        assert_eq!(ring.len(), 5);
        assert_eq!((ring[0], ring[1].code, ring[3].d1), (SndRequest { code: 0xF6, d1: 5, d2: 0 }, 0x6F, 0));
        assert!(g.snd_take_requests().is_empty(), "taken");
    }

    /// The pan: the screen position is clamped to 0 .. 0x17F, then one table byte per 8 pixels.
    #[test]
    fn the_pan_follows_the_screen_position() {
        let mut g = game_with_pan_table();
        g.cam.s2.cam_x = 0x300;
        assert_eq!(g.snd_pan_x(0x2F0), 0, "left of the screen");
        assert_eq!(g.snd_pan_x(0x300), 0);
        assert_eq!(g.snd_pan_x(0x307), 0);
        assert_eq!(g.snd_pan_x(0x308), 1);
        assert_eq!(g.snd_pan_x(0x300 + 0x17F), 0x2F);
        assert_eq!(g.snd_pan_x(0x300 + 0x180), 0x2F, "right of the screen");
        assert_eq!(g.snd_pan_x(0x300 + 0x7000), 0x2F);
    }

    /// A game with the text tables `blobs` and a pan table 0xA66A that counts 0, 1, 2 ... (synthetic: no ROM bytes).
    fn game_with_voice_tables(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.text = TextData { blobs };
        a.enemy.t = dino_assets::enemy::ByteTables { blobs: vec![(0xA66A, (0..0x30).collect())], ..Default::default() };
        Game::new(Box::leak(Box::new(a)))
    }

    /// Punch 3 (0x99E8) and punch 4 (0x9A36) end with `moveq #0,d2`: d1 = the pan byte, d2 = 0. The dash voice (0x9984) and
    /// the special (0x992C) have the pan mode 7 in d1 and d2 = 0xD000000 (M4-17: MAME records of the hero chain).
    #[test]
    fn hero_move_voices_send_the_registers_of_their_routines() {
        let mut g = game_with_voice_tables(vec![
            (0x9A26, (0x40..0x50).collect()),
            (0x9A74, (0x50..0x60).collect()),
            (0x99DE, (0x60..0x69).collect()),
            (0x9974, (0x70..0x80).collect()),
        ]);
        g.cam.s2.cam_x = 0;
        g.players[0].b.x = 0x30 << 16; // pan index 6 (0x30 / 8)
        g.snd_player(0, 0x99E8);
        g.snd_player(0, 0x9A36);
        g.snd_player(0, 0x9984);
        g.snd_player(0, 0x992C);
        let r: Vec<_> = g.w.snd_reqs.iter().map(|r| (r.code, r.d1, r.d2)).collect();
        assert_eq!(r, vec![(0x40, 6, 0), (0x50, 6, 0), (0x60, 0x7_0006, 0x0D00_0000), (0x70, 0x7_0006, 0x0D00_0000)]);
    }

    /// The voice bank (0xA502) picks the column of the voice tables: with two players the higher slot has bank 1 and plays
    /// the twin code (the real tables: Jack's punch 3 is 0x4F, 0xCA, 0xCB; D4e acceptance 8). Synthetic tables here.
    #[test]
    fn a_second_player_plays_the_twin_codes_of_its_bank() {
        let mut mix = vec![0u8; 0x20];
        mix[(4 | 8) + 1] = 1; // players 1 and 2 present: slot 1 has bank 1
        mix[(4 | 8 | 0x10) + 2] = 2; // all three: slot 2 has bank 2
        let mut g = game_with_voice_tables(vec![(0xA53A, mix), (0x9A26, vec![0x10, 0x11, 0x12, 0, 0x20, 0x21, 0x22, 0, 0, 0, 0, 0, 0, 0, 0, 0])]);
        for i in 0..2 {
            g.players[i].b.live = 1;
        }
        g.snd_update_voice_banks();
        g.snd_player(0, 0x99E8);
        g.snd_player(1, 0x99E8);
        g.players[2].b.live = 1;
        g.snd_update_voice_banks();
        g.snd_player(2, 0x99E8);
        assert_eq!(g.w.sounds, vec![0x10, 0x11, 0x12], "bank 0, 1 and 2 of the same row");
    }

    /// The landing voice 0x9C0C never writes d2: the request carries the caller's d2 (`snd_player_d2`). The pick-up voice
    /// 0x9C20 is the same routine behind the wading test: it still sounds when the player wades below height 0x20.
    #[test]
    fn the_landing_and_pickup_voices_send_the_callers_d2() {
        let mut g = game_with_voice_tables(vec![(0x9C72, vec![0x62, 0xDE, 0xDF])]);
        g.snd_player_d2(0, 0x9C0C, 0x1F00);
        g.snd_player_d2(0, 0x9C20, 0x35);
        g.players[0].b.wading = 0x80; // +$53 negative and Y below 0x20: the landing is silent
        g.snd_player_d2(0, 0x9C0C, 0x77);
        g.snd_player_d2(0, 0x9C20, 0x36);
        let r: Vec<_> = g.w.snd_reqs.iter().map(|r| (r.code, r.d1, r.d2)).collect();
        assert_eq!(r, vec![(0x62, 0, 0x1F00), (0x62, 0, 0x35), (0x62, 0, 0x36)]);
    }

    /// 0xA4CA: the extra life jingle. The routine computes the pan byte and then overwrites d1 with a constant.
    #[test]
    fn the_extra_life_jingle_has_a_constant_d1() {
        let mut g = game_with_voice_tables(Vec::new());
        g.players[0].b.x = 0x80 << 16;
        g.snd_extra_life();
        assert_eq!(g.w.snd_reqs[0], SndRequest { code: 0x4A, d1: 0x72000, d2: 0x1000_0000 });
    }

    /// 0x9A84: the killer's voice. The code comes from the row of the owner's character and bank, the pan from the ROM
    /// word 8 bytes into that row (a bug of the original, the position is read from the table pointer).
    #[test]
    fn the_kill_voice_reads_its_pan_word_from_the_table_row() {
        let mut row = vec![0x2Cu8, 0xA0, 0xA1, 0, 0x2E, 0xA4, 0xA5, 0, 0x00, 0x30, 0, 0, 0, 0, 0, 0];
        row.extend([0u8; 8]);
        let mut g = game_with_voice_tables(vec![(0x9AE4, row)]);
        g.cam.s2.cam_x = 0x10;
        g.players[1].b.k20 = 4;
        g.players[1].voice = 1;
        g.snd_9a84(1);
        // the word at row + 4 + 8 is 0 here: position 0 - camera X 0x10 is negative: pan index 0
        // (code: table row 4, bank 1)
        assert_eq!(g.w.snd_reqs[0], SndRequest { code: 0xA4, d1: 0, d2: 0 });
        g.snd_9a84(0xFF); // bmi: nobody owns the object
        g.snd_9a84(3); // past the three players
        assert_eq!(g.w.snd_reqs.len(), 1);
    }

    /// G.$43C (the word the hit sound 0x962A reads) has no writer in the program, and it is 0 in every MAME recording
    /// (all eight episodes, three players): the hit sounds always take the first code of their pair. `snd_hit` does not
    /// write the word either, so a hit and the next hit send the same code.
    #[test]
    fn the_hit_toggle_word_g43c_is_never_written() {
        let t = vec![0, 0, 0, 1, 0x11, 0x22];
        let mut a = AssetPack::default();
        a.text = TextData { blobs: vec![(0x96AA, t)] };
        a.enemy.t = dino_assets::enemy::ByteTables { blobs: vec![(0xA66A, (0..0x30).collect())], ..Default::default() };
        let mut g = Game::new(Box::leak(Box::new(a)));
        assert_eq!(g.w.snd_hit_toggle, 0);
        g.snd_hit(3, 0, 0);
        g.snd_hit(3, 0, 0);
        assert_eq!(g.w.sounds, vec![0x11, 0x11]);
        assert_eq!(g.w.snd_hit_toggle, 0);
    }

    /// The byte 0x962A tests at +$24 of a6 (M10-22): an object's facing when a6 is an object (0xFF0000 + its handle), a ROM byte when a6
    /// is a ROM address (the box bank, the tables, the program image), 0 for other RAM; the word restore of 0x6422 sign extends the low
    /// word (a box record in 0x10xxxx becomes a low program address, one at 0x....8000 and up becomes RAM).
    #[test]
    fn the_hit_sound_mode_byte_follows_the_a6_the_caller_left() {
        let mut a = AssetPack::default();
        a.sprites.hitboxes = dino_assets::HitBoxBank { base: 0x100000, words: vec![0; 0x20].into_iter().chain([0x0100]).collect() };
        a.enemy.t = dino_assets::enemy::ByteTables { blobs: vec![(0x2BB4E, vec![7; 0x30]), (0xA66A, vec![0; 0x30])], program: (0..=0xFFu8).cycle().take(0x40).collect(), ..Default::default() };
        a.text = TextData { blobs: vec![(0x96AA, vec![0, 0x39, 0x39])] };
        let mut g = Game::new(Box::leak(Box::new(a)));
        let h = Game::player_handle(1);
        g.players[1].b.facing = 1;
        assert_eq!(g.snd_a6_byte(Game::snd_a6_obj(h)), 1, "an object: its facing");
        g.players[1].b.facing = 0;
        assert_eq!(g.snd_a6_byte(Game::snd_a6_obj(h)), 0);
        assert_eq!(g.snd_a6_byte(0x100000 + 0x40 - 0x24), 0x01, "the box bank: the high byte of the word at 0x100040");
        assert_eq!(g.snd_a6_byte(0x2BB4E), 7, "a table blob");
        assert_eq!(g.snd_a6_byte(0x10), 0x34, "the program image");
        assert_eq!(g.snd_a6_byte(0xFF_8100), 0, "RAM that is no object start");
        assert_eq!(Game::snd_a6_word(0x0010_0C18), 0x0C18, "movea.w: the low word, positive");
        assert_eq!(Game::snd_a6_word(0x0010_B274), 0xFF_B274, "movea.w: negative, the RAM of a5's half");
        assert_eq!(Game::snd_a6_word(Game::snd_a6_obj(h)), Game::snd_a6_obj(h), "an object handle survives the word restore");
        let (x, f) = (g.players[1].b.xi(), 0);
        g.snd_hit(0, x, f); // a made-up table 0x96AA (bit 0, the codes 0x39 / 0x39): mode 3
        assert_eq!(g.w.snd_reqs[0].d1 >> 16, 3);
        g.snd_hit(0, x, g.snd_a6_byte(0x2BB4E));
        assert_eq!(g.w.snd_reqs[1].d1 >> 16, 4, "a nonzero byte picks the mode 4");
    }

    /// Every request keeps its d1 and d2 (the original's ring of 63 entries is drained by the Z80 flush, not here).
    #[test]
    fn every_request_keeps_its_d1_and_d2() {
        let mut g = game_with_tables(Vec::new());
        for k in 0..70 {
            g.snd_request(k, k as u32, 1);
        }
        assert_eq!((g.snd_take_requests().len(), g.w.sounds.len()), (70, 70));
    }
}
