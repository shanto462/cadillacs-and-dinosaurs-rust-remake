//! Sound requests of the 68000 side: the queue write 0xA63C and the small routines that choose
//! the sound code for a player voice (0x95CE..0x9C0C), a hit (0x962A) and the extra life jingle
//! (0xA4CA). Only the sound code is recorded (`Game::sound`); the pan / volume longs d1 and d2 of
//! the queue entry belong to the sound port (plan phase 3, task 4).
//!
//! Many routines alternate between two codes with `bchg #n,$43A(a5)`: the toggle bits of the
//! byte G.$43A are shared by all callers, so the codes depend on the history of all requests.

use super::Game;

impl Game {
    /// `bchg.b #bit,$43A(a5)`: flips the bit and returns what it was (`bne` taken when true).
    fn snd_toggle(&mut self, bit: u8) -> bool {
        let old = self.w.snd_toggle >> bit & 1 != 0;
        self.w.snd_toggle ^= 1 << bit;
        old
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

    /// The player voice and effect routines 0x95CE..0x9C0C, called with a6 = player `i`.
    pub fn snd_player(&mut self, i: usize, routine: u32) {
        let (id, k20, voice, wading, y, tag) = {
            let p = &self.players[i];
            (p.b.id, p.b.k20 as u32, p.voice as u32, p.b.wading, p.b.yi(), 0u8)
        };
        let t = &self.assets.text;
        let code: u8 = match routine {
            // 0x95CE: 0x38 / 0xA8 (toggle bit 0; the first request after a reset is 0xA8).
            0x95CE => {
                if self.snd_toggle(0) { 0x38 } else { 0xA8 }
            }
            // 0x9796: 0x72 / 0xF8 (toggle bit 3).
            0x9796 => return self.snd_9796(),
            // 0x986C / 0x98CC: id 2 uses the second row of three.
            0x986C => t.u8(0x98C6, if id == 2 { 3 } else { 0 } + voice),
            0x98CC => t.u8(0x9926, if id == 2 { 3 } else { 0 } + voice),
            // 0x992C / 0x99E8 / 0x9A36: +$20 (character * 4) selects the row of four.
            0x992C => t.u8(0x9974, k20 + voice),
            0x99E8 => t.u8(0x9A26, k20 + voice),
            0x9A36 => t.u8(0x9A74, k20 + voice),
            // 0x9984: rows of three by id (0, 1, 2).
            0x9984 => t.u8(0x99DE, if id == 2 { 6 } else if id == 1 { 3 } else { 0 } + voice),
            // 0x9AF4: players (+$82 == 0) use the voice table; other objects toggle bit 4.
            0x9AF4 => {
                if tag == 0 {
                    t.u8(0x9B5C, voice)
                } else if self.snd_toggle(4) {
                    0xEF
                } else {
                    0xF0
                }
            }
            // 0x9B60: 0x6C / 0xF2 (toggle bit 5, shared with 0x9BCE). 0x9B9E: 0x75.
            0x9B60 => {
                if self.snd_toggle(5) { 0x6C } else { 0xF2 }
            }
            0x9B9E => 0x75,
            // 0x9BCE: 0x71 / 0xF7 (toggle bit 5).
            0x9BCE => {
                if self.snd_toggle(5) { 0x71 } else { 0xF7 }
            }
            // 0x9C0C: no sound while wading (+$53 negative) below height 0x20; players use the
            // voice table, other objects toggle bit 6 (0xE0 / 0xE1).
            0x9C0C => {
                if (wading as i8) < 0 && y < 0x20 {
                    return;
                }
                if tag == 0 {
                    t.u8(0x9C72, voice)
                } else if self.snd_toggle(6) {
                    0xE0
                } else {
                    0xE1
                }
            }
            r => panic!("unknown player sound routine {r:#x}"),
        };
        self.sound(code as u16);
    }

    /// 0x9796 (knockdown / landing thud of players and enemies): 0x72 / 0xF8, toggle bit 3. The code
    /// does not depend on the object.
    pub fn snd_9796(&mut self) {
        let code = if self.snd_toggle(3) { 0x72 } else { 0xF8 };
        self.sound(code);
    }

    /// 0xA354 (code 0x79) / 0x97D2 (code 0x58): an enemy's death voice. The hit library picks
    /// 0x97D2 when `+$63` is positive (`tst.b $63(a6) ; bgt`), 0xA354 otherwise.
    pub fn snd_enemy_death(&mut self, b63: u8) {
        self.sound(if (b63 as i8) > 0 { 0x58 } else { 0x79 });
    }

    /// 0x962A: the hit sound of an attack record (`sound` = record byte 4, a byte offset into
    /// the table 0x96AA of 3-byte entries: toggle bit number, code, code). The original toggles
    /// the bit in a register copy of G.$43C only, so G.$43C decides and never changes.
    pub fn snd_hit(&mut self, sound: u8) {
        let off = sound as i8 as i32 as u32; // ext.w d0 (then used as a word offset)
        let t = &self.assets.text;
        let bit = t.u8(0x96AA, off) as u32;
        let set = (self.w.snd_hit_toggle as u32 >> bit) & 1 != 0;
        let code = t.u8(0x96AA, off + 1 + set as u32);
        self.sound(code as u16);
    }

    /// 0xA4CA: extra life jingle.
    pub fn snd_extra_life(&mut self) {
        self.sound(0x4A);
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

    /// 0x95CE and 0x9BCE alternate with `bchg`: the first request after a clear takes the second
    /// code, and the toggle bits are shared per bit, not per routine (0x9B60 and 0x9BCE share bit 5).
    #[test]
    fn alternating_codes_follow_the_toggle_byte() {
        let mut g = game_with_tables(Vec::new());
        g.snd_player(0, 0x95CE);
        g.snd_player(0, 0x95CE);
        g.snd_player(0, 0x9BCE);
        g.snd_player(0, 0x9B60);
        g.snd_player(0, 0x9BCE);
        assert_eq!(g.w.sounds, vec![0xA8, 0x38, 0xF7, 0x6C, 0xF7]);
        assert_eq!(g.w.snd_toggle, 0b0010_0000);
    }

    /// The hit sound table has 3-byte entries (bit number, code if the bit is clear, code if set).
    #[test]
    fn hit_sound_picks_the_code_by_the_toggle_word() {
        let t = vec![0, 0, 0, 1, 0x11, 0x22, 2, 0x33, 0x44];
        let mut g = game_with_tables(vec![(0x96AA, t)]);
        g.snd_hit(3);
        g.w.snd_hit_toggle = 0b10;
        g.snd_hit(3);
        g.snd_hit(6);
        assert_eq!(g.w.sounds, vec![0x11, 0x22, 0x33]);
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
}
