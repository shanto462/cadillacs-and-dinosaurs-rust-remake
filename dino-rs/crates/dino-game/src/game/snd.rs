//! Sound requests of the 68000 side: the queue write 0xA63C and the small routines that choose
//! the sound code for a player voice (0x95CE..0x9C0C), a hit (0x962A) and the extra life jingle
//! (0xA4CA). Only the sound code is recorded (`Game::sound`); the pan / volume longs d1 and d2 of
//! the queue entry belong to the sound port (plan phase 3, task 4).
//!
//! Many routines alternate between two codes with `bchg #n,$43A(a5)`: the toggle bits of the
//! byte G.$43A are shared by all callers, so the codes depend on the history of all requests.

use super::Game;

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
            // 0x992C / 0x99E8 / 0x9A36: +$20 (character * 4) selects the row of four. d2 = 0xD000000, the pan index
            // (the word d2 keeps after the table lookup), 0.
            0x992C => (t.u8(0x9974, k20 + voice), 0x0D00_0000),
            0x99E8 => (t.u8(0x9A26, k20 + voice), self.snd_pan_index(x)),
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
            // voice table, other objects toggle bit 6 (0xE0 / 0xE1). d2 not checked yet (M4-13 open item).
            0x9C0C => {
                if (wading as i8) < 0 && y < 0x20 {
                    return;
                }
                if tag == 0 {
                    (t.u8(0x9C72, voice), 0)
                } else if self.snd_toggle(6) {
                    (0xE0, 0)
                } else {
                    (0xE1, 0)
                }
            }
            r => panic!("unknown player sound routine {r:#x}"),
        };
        // d1: the mode (the object's +$24 facing picks the second value) in the high word, then the word of the
        // routine (the low byte is replaced by the pan byte): 0x986C 0x20xx, 0x98CC 0x06xx, 0x992C 0x7 (no word).
        let facing_right = self.players[i].b.facing != 0;
        let hi = match routine {
            0x986C => (if facing_right { 4 } else { 3 }) << 16 | 0x2000,
            0x98CC => (if facing_right { 6 } else { 5 }) << 16 | 0x600,
            0x992C => 7 << 16,
            _ => 0,
        };
        let pan = self.snd_pan_x(x) as u32;
        self.sound_ex(code as u16, hi | pan, d2);
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
            let mode = if facing != 0 { 6 } else { 5 };
            self.sound_ex(0x58, (mode << 16) | 0xA00 | pan, 0x0056_08D0);
        } else {
            let mode = if facing != 0 { 4 } else { 3 };
            self.sound_ex(0x79, (mode << 16) | 0x2000 | pan, 0x0240_0000);
        }
    }

    /// 0x962A: the hit sound of an attack record (`sound` = record byte 4, a byte offset into
    /// the table 0x96AA of 3-byte entries: toggle bit number, code, code). The original toggles
    /// the bit in a register copy of G.$43C only, so G.$43C decides and never changes.
    /// d1 = (3, or 4 when the object at a6 faces right: +$24 nonzero) << 16 | 0x2000 | the pan byte of the
    /// attacker's X (a2, +$08), d2 = 0x15000000. `ax` is a2's X, `victim_facing` is a6's +$24.
    pub fn snd_hit(&mut self, sound: u8, ax: i16, victim_facing: u8) {
        let off = sound as i8 as i32 as u32; // ext.w d0 (then used as a word offset)
        let t = &self.assets.text;
        let bit = t.u8(0x96AA, off) as u32;
        let set = (self.w.snd_hit_toggle as u32 >> bit) & 1 != 0;
        let code = t.u8(0x96AA, off + 1 + set as u32);
        let mode = if victim_facing != 0 { 4 } else { 3 };
        let pan = self.snd_pan_x(ax) as u32;
        self.sound_ex(code as u16, (mode << 16) | 0x2000 | pan, 0x1500_0000);
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
