//! Difficulty: rank, level and aggression (0x17F8A..0x18834), and the stage time limit
//! (0x1037E / 0x103FE / 0x104A8). Owner: enemy port. See `re/specs/enemy/SPEC.md` section 6.
//!
//! Globals: G.$5EC aggression (0..7), G.$5EE level (`w.level`), G.$5F0 rank (`w.rank`), and the
//! rest of the struct at G.$5EC kept here with their offsets.

use super::Game;

/// Rank struct at G.$5EC (offsets from $5EC) and the time-limit bytes.
#[derive(Default)]
pub struct Rank {
    /// +$00 G.$5EC aggression 0..7.
    pub aggression: u16,
    /// +$06 G.$5F2 forced-aggression camera X limit (spawn command 8), +$08 G.$5F4 forced value.
    pub force_x: u16,
    pub force_val: u16,
    /// +$0A G.$5F6 rank speed 0..15, +$0C G.$5F8 rank-up countdown.
    pub speed: u16,
    pub timer: u16,
    /// +$0E / +$12 pending rank-ups and their 1800-frame timer.
    pub ups: u16,
    pub ups_t: u16,
    /// +$10 / +$14 pending rank-downs and their 1200-frame timer.
    pub downs: u16,
    pub downs_t: u16,
    /// +$18 G.$604, +$1A G.$606 countdowns; +$1C G.$608 calm period (aggression held at 0).
    pub w604: u16,
    pub w606: u16,
    pub calm: u16,
    /// +$16 = G.$602 rank system freeze (tested by 0x186CC; cleared at game and area start).
    pub freeze: u16,
    /// G.$4F6 frames waited at the title with credits (written by the system task 0x1827A).
    pub w4f6: u16,
    /// G.$777B: bits 0-2 difficulty 0..7, bits 3-4 timer level (rank speed = level + 1). The settings file
    /// `dino.cfg` (`flow/config.rs`, cooked by 0xAC794) is the source: the game start copies `Flow::dip.dip_b` here.
    pub dip: u8,
    /// G.$4E8 / G.$4E9 stage time (BCD minutes, seconds), G.$4EA second divider, G.$4EB lock.
    pub time_min: u8,
    pub time_sec: u8,
    pub time_div: u8,
    pub time_lock: u8,
    /// Set once the game-start / stage-start rank setup ran (see `rank_tick`).
    pub started: bool,
}

/// 0x187B8: frames between automatic rank-ups, by rank speed.
pub const RANK_PERIOD: u32 = 0x187B8;

impl Game {
    /// 0x17F8A: rank at game start.
    pub fn rank_game_start(&mut self) {
        let t = &self.assets.enemy.t;
        self.rank.ups = 0;
        self.rank.ups_t = 0;
        self.rank.downs = 0;
        self.rank.downs_t = 0;
        self.rank.freeze = 0;
        let w = self.rank.w4f6;
        // 0x17FA8: bands at 720 / 3600 / 10800 frames (unsigned compares).
        let band = if w < t.u16(0x180B4, 0) {
            0
        } else if w < t.u16(0x180B4, 2) {
            0x10
        } else if w < t.u16(0x180B4, 4) {
            0x20
        } else {
            0x30
        };
        let off = t.u16(0x180BA, ((self.rank.dip as u32 & 7) * 2) + band);
        let r = self.rand() & 0x1F;
        let v = self.assets.enemy.t.u8(0x180FA + off as u32, r as u32).wrapping_sub(1);
        self.w.rank = v as u16; // move.w d0 (byte loaded with moveq #0 / move.b)
        self.rank.w4f6 = 0;
        self.rank.speed = ((self.rank.dip as u16 & 0x18) >> 3) + 1;
        self.rank_area_start();
    }

    /// 0x1827A, run by the system task every frame: G.$4F6 counts the frames the machine waits
    /// with a credit (or in free play) outside a game; a game clears it. `pool` is the word at
    /// G.$46A (the credits of the common pool in the high byte, the free play flag in the low byte):
    /// with separate chutes (the factory setting) the pool stays 0 and the counter does not run.
    /// The counter wraps from 0xFFFF to 0x8000. (The game start 0x17F8A reads it only after this
    /// routine cleared it in the pass that creates the game task, so the rank band is always 0.)
    pub fn rank_waited_tick(&mut self, in_game: bool, pool: u16) {
        if in_game {
            self.rank.w4f6 = 0;
        } else if pool != 0 {
            self.rank.w4f6 = self.rank.w4f6.wrapping_add(1);
            if self.rank.w4f6 == 0 {
                self.rank.w4f6 = 0x8000;
            }
        }
    }

    /// 0x1800E: per-stage rank adjustment, then area start.
    pub fn rank_stage_start(&mut self) {
        let t = &self.assets.enemy.t;
        let a = self.w.stage as u32 * 8;
        let mut d = (self.w.rank as i16).wrapping_add(t.i16(0x1823A, a));
        if d < 0 {
            d = 0;
        }
        if d as u16 > t.u16(0x1823A, a + 2) {
            d = t.i16(0x1823A, a + 2);
        }
        if (d as u16) < t.u16(0x1823A, a + 4) {
            d = t.i16(0x1823A, a + 4);
        }
        self.w.rank = d as u16;
        self.rank_area_start();
    }

    /// 0x18048: area start.
    pub fn rank_area_start(&mut self) {
        let r = &mut self.rank;
        r.force_x = 0;
        r.force_val = 0;
        r.freeze = 0;
        r.w604 = 0;
        r.w606 = 0;
        r.calm = 0;
        r.timer = self.assets.enemy.t.u16(RANK_PERIOD, 2 * r.speed as u32);
        let k = self.w.stage as u32 * 4 + self.cam.s2.area as u32;
        self.rank.aggression = self.assets.enemy.t.u8(0x18094, k).wrapping_sub(1) as u16;
        self.rank_compute_level();
    }

    /// 0x18592: clamp the rank by player count and look up the level (0x1860C).
    pub fn rank_compute_level(&mut self) {
        let mut r = self.w.rank;
        match self.w.players_minus_one {
            0 => {
                if r > 0x18 {
                    r = 0x18;
                }
            }
            1 => {
                if r < 4 {
                    r = 4;
                } else if r > 0x1C {
                    r = 0x1C;
                }
            }
            _ => {
                if r < 8 {
                    r = 4;
                }
            }
        }
        self.w.rank = r;
        let col = match self.w.players_minus_one {
            0 => 0,
            2 => 4,
            _ => 2,
        };
        let idx = r.wrapping_mul(6).wrapping_add(col);
        self.w.level = self.assets.enemy.t.u16(0x1860C, idx as u32);
    }

    /// 0x186CC rank tick (first call of the game frame).
    pub fn rank_tick(&mut self) {
        if !self.rank.started {
            // TEMP until the flow port calls these at game start / stage start (0x17F8A, 0x1800E).
            self.rank.started = true;
            self.rank.dip = self.flow.dip.dip_b;
            self.rank_game_start();
            self.rank_stage_start();
            self.rank_time_area_start();
        }
        if self.rank.freeze != 0 || self.w.scripted != 0 || self.spawn.g.b4d5 != 0 {
            return;
        }
        if self.cam.s2.cam_x < self.rank.force_x {
            self.rank.aggression = self.rank.force_val;
        } else {
            self.rank.force_x = 0;
        }
        let (t, r) = (self.rank.timer, &mut self.rank);
        r.timer = t.wrapping_sub(1);
        if t == 0 {
            let v = self.w.rank.wrapping_add(1);
            self.w.rank = v.min(0x1F);
            self.rank.timer = self.assets.enemy.t.u16(RANK_PERIOD, 2 * self.rank.speed as u32);
        }
        let r = &mut self.rank;
        if r.ups != 0 {
            r.ups_t = r.ups_t.wrapping_sub(1);
            if r.ups_t == 0 {
                r.ups -= 1;
                r.ups_t = 0x708;
                self.w.rank = self.w.rank.wrapping_add(1).min(0x1F);
            }
        }
        let r = &mut self.rank;
        if r.downs != 0 {
            r.downs_t = r.downs_t.wrapping_sub(1);
            if r.downs_t == 0 {
                r.downs -= 1;
                r.downs_t = 0x4B0;
                self.w.rank = self.w.rank.checked_sub(1).unwrap_or(0);
            }
        }
        let r = &mut self.rank;
        if r.w604 != 0 {
            r.w604 -= 1;
        }
        if r.w606 != 0 {
            r.w606 -= 1;
        }
        self.rank_compute_level();
        if self.rank.calm != 0 {
            self.rank.aggression = 0;
            self.rank.calm -= 1;
            if self.rank.calm == 0 {
                self.rank_reroll_aggression();
            }
        }
    }

    /// 0x182A2: next aggression from the Markov table (or the script-forced value).
    pub fn rank_reroll_aggression(&mut self) {
        if self.rank.freeze != 0 {
            return;
        }
        let v = if self.rank.force_x != 0 {
            self.rank.force_val
        } else {
            // `cmpi.b #1, $7B4(a5)`: the camera lock state (G.$7B4 = `cam.s2.lock78`) 1 selects the second table.
            let table = if self.cam.s2.lock78 == 1 { 0x18364 } else { 0x182E4 };
            let r = self.rand() & 0xF;
            let off = (self.rank.aggression << 4).wrapping_add(r);
            self.assets.enemy.t.u8(table, off as u32) as u16
        };
        self.rank.aggression = v;
    }

    /// 0x18566: rank speed += d (clamped 0..15), restart the rank-up countdown.
    pub fn rank_speed_add(&mut self, d: i16) {
        if self.rank.freeze != 0 {
            return;
        }
        let mut s = (self.rank.speed as i16).wrapping_add(d);
        if s < 0 {
            self.rank.speed = 0;
            return;
        }
        if s >= 0x10 {
            s = 0xF;
        }
        self.rank.speed = s as u16;
        self.rank.timer = 0;
    }

    /// 0x18410: player event (G.$4D0 == 0xC): rank speed + 3.
    pub fn rank_event_18410(&mut self, b4d0: u8) {
        if self.rank.freeze == 0 && b4d0 == 0xC {
            let d = self.assets.enemy.t.i16(0x187DC, 0);
            self.rank_speed_add(d);
        }
    }

    /// 0x183E4: a player recovered from a hit (0x1C0F4 / 0x1F276): the rank drops by table[0x187D8]
    /// (not below 0), then a cooldown of table[0x187DA] frames in G.$604 keeps it from dropping
    /// again; the level follows (0x18592).
    pub fn rank_event_183e4(&mut self) {
        if self.rank.freeze != 0 || self.rank.w604 != 0 {
            return;
        }
        let t = &self.assets.enemy.t;
        let r = (self.w.rank as i16).wrapping_sub(t.u16(0x187D8, 0) as i16);
        self.w.rank = if r < 0 { 0 } else { r as u16 };
        self.rank.w604 = t.u16(0x187DA, 0);
        self.rank_compute_level();
    }

    /// 0x1842C: rank speed - 1.
    pub fn rank_event_1842c(&mut self) {
        let d = self.assets.enemy.t.i16(0x187DE, 0);
        self.rank_speed_add(d);
    }

    /// 0x1843C: rank change after an extra life / continue. `lives` = sum of the present
    /// players' +$A3; `b4cf` = G.$4CF; `b4e6` = G.$4E6. [I] meaning of the inputs.
    pub fn rank_event_1843c(&mut self, lives: u8, b4cf: u8, b4e6: u8) {
        let k = (lives as u32 * 2) + (b4cf == 0) as u32;
        let v = self.assets.enemy.t.u8(0x184C4, k);
        if v == 0 {
            self.w.rank = self.w.rank.checked_sub(3).unwrap_or(0);
            return;
        }
        let d = v.wrapping_sub(b4e6);
        if d == 2 {
            self.w.rank = (self.w.rank + 2).min(0x1F);
        } else if (d as i8) >= 3 {
            self.w.rank = (self.w.rank + 4).min(0x1F);
        }
    }

    /// 0x184E2: a player continues / joins. `player_a3` = that player's +$A3 (nonzero adds the
    /// rank-speed change), plus the inputs of `rank_event_1843c`.
    pub fn rank_event_continue(&mut self, player_a3: u8, lives: u8, b4cf: u8, b4e6: u8) {
        if self.rank.freeze != 0 {
            return;
        }
        let t = &self.assets.enemy.t;
        let mut d = t.u16(0x187E0, (self.rank.dip as u32 & 7) * 2);
        if self.rank.w606 != 0 {
            d = t.u16(0x187F0, 0);
        }
        self.w.rank = self.w.rank.checked_sub(d).unwrap_or(0);
        self.rank.w606 = self.assets.enemy.t.u16(0x187F2, 0);
        self.rank_event_1843c(lives, b4cf, b4e6);
        self.rank_compute_level();
        // 0x18522
        self.rank.calm = 0x4B0;
        if player_a3 != 0 {
            let mut i = self.w.players_minus_one as u16;
            if i == 2 {
                i += self.rand() & 1;
            }
            let row = 0x187F4 + (self.rank.dip as u32 & 7) * 8;
            let d = self.assets.enemy.t.i16(row, 2 * i as u32);
            self.rank_speed_add(d);
        }
    }

    /// 0x1031A: stage time at area start (table 0x1033E by stage and area).
    pub fn rank_time_area_start(&mut self) {
        let k = self.w.stage as u32 * 8 + self.cam.s2.area as u32 * 2;
        let v = self.assets.enemy.t.u16(0x1033E, k);
        self.rank_set_time(v);
        self.rank.time_div = 0x32;
        self.rank.time_lock = 0;
    }

    /// 0x1037E stage time (one BCD second per 51 frames), then 0x103FE time over.
    pub fn time_limit(&mut self) {
        self.rank_time_over();
        self.rank_time_tick();
    }

    /// 0x1037E.
    fn rank_time_tick(&mut self) {
        // The test-mode freeze checks ($4AA / $23 / $24) are not ported.
        if self.spawn.g.b4d5 != 0 || self.w.scripted != 0 || self.spawn.g.car_mode != 0 {
            return;
        }
        if self.rank.time_div != 0 {
            self.rank.time_div -= 1;
            return;
        }
        if self.rank.time_min == 0 && self.rank.time_sec == 0 {
            return;
        }
        let (m, s) = (self.rank.time_min, self.rank.time_sec);
        let (s2, borrow) = bcd_sub(s, 1);
        let (m2, s2) = if borrow {
            let (m2, b2) = bcd_sub(m, 1);
            if b2 { (0, 0x59) } else { (m2, 0x59) }
        } else {
            (m, s2)
        };
        self.rank.time_min = m2;
        self.rank.time_sec = s2;
        self.rank.time_div = 0x32;
    }

    /// 0x103FE: when the time is 0, the words TIME and OVER fly in (0x96450) and every player with +$04 < 4 loses all HP
    /// (0x1046A, in the same frame: the heroes of the scenes `m515_s12`, `m515_s21` have HP 0 in the frame the lock G.$4EB
    /// is set). The lock stays set until a top up (0x104A8) clears it.
    fn rank_time_over(&mut self) {
        let r = &self.rank;
        if self.spawn.g.b4d5 != 0 || self.w.scripted != 0 || r.time_lock != 0 || r.time_min != 0 || r.time_sec != 0 || r.time_div != 0 {
            return;
        }
        // 0x1042A: `tst.b $518(a5)` and `tst.b $7B4(a5)` (the camera lock state, G.$7B4 = `cam.s2.lock78`).
        if self.spawn.g.car_mode != 0 && self.cam.s2.lock78 != 0 {
            self.spawn.g.car_mode = 2;
            self.rank_add_time(0x300);
            return;
        }
        self.rank.time_lock = 1;
        self.fx_spawn_time_over(); // 0x96450 (a full pool of effects makes fewer words, the heroes die anyway)
        for i in 0..3 {
            let b = &mut self.players[i].b;
            if b.live != 0 && b.mode < 4 {
                b.hp = 0;
            }
        }
    }

    /// 0x10482 (the end of the lives check 0x1BFD0, called by `player_dead_continue` for a hero who respawns, once per hero):
    /// 600 frames more for the wave time budget G.$50C and 3:30 on the stage clock (at most 4:00; the lock G.$4EB is
    /// cleared, which also ends the words TIME OVER).
    pub fn rank_respawn_time(&mut self) {
        self.spawn.g.wave_timer = self.spawn.g.wave_timer.wrapping_add(0x258);
        self.rank_add_time(0x330);
    }

    /// 0x1049E: a join adds 600 wave frames and 3:30 stage time, capped at 4:00.
    pub fn rank_join_time(&mut self) {
        self.spawn.g.wave_timer = self.spawn.g.wave_timer.wrapping_add(600);
        self.rank_add_time(0x330);
    }

    /// 0x104A8: add BCD time `v` (high byte minutes, low byte seconds), capped at 04:00.
    pub fn rank_add_time(&mut self, v: u16) {
        let (mut m, mut s) = (self.rank.time_min, self.rank.time_sec);
        let (s2, carry) = bcd_add(s, v as u8, false);
        s = s2;
        let mut x = false;
        if s >= 0x60 {
            s = bcd_sub(s, 0x60).0;
            x = true;
        }
        if carry {
            x = true; // carry out of the seconds digit pair also propagates
        }
        m = bcd_add(m, (v >> 8) as u8, x).0;
        self.rank.time_min = m;
        self.rank.time_sec = s;
        if ((m as u16) << 8 | s as u16) >= 0x400 {
            self.rank.time_min = 4;
            self.rank.time_sec = 0;
        }
        self.rank.time_lock = 0;
    }

    /// Spawn command 0xC (0xFD9C): set the stage time (BCD word).
    pub fn rank_set_time(&mut self, v: u16) {
        self.rank.time_min = (v >> 8) as u8;
        self.rank.time_sec = v as u8;
    }
}

/// 68000 `abcd` on two BCD bytes with extend in; returns (result, carry).
fn bcd_add(a: u8, b: u8, x: bool) -> (u8, bool) {
    let mut lo = (a & 0xF) + (b & 0xF) + x as u8;
    let mut carry_lo = false;
    if lo > 9 {
        lo += 6;
        carry_lo = true;
    }
    let mut hi = (a >> 4) + (b >> 4) + if carry_lo || lo > 0xF { 1 } else { 0 };
    let mut carry = false;
    if hi > 9 {
        hi += 6;
        carry = true;
    }
    (((hi & 0xF) << 4) | (lo & 0xF), carry)
}

/// 68000 `sbcd` (a - b, no extend in); returns (result, borrow).
fn bcd_sub(a: u8, b: u8) -> (u8, bool) {
    let av = (a >> 4) as i16 * 10 + (a & 0xF) as i16;
    let bv = (b >> 4) as i16 * 10 + (b & 0xF) as i16;
    let mut r = av - bv;
    let borrow = r < 0;
    if borrow {
        r += 100;
    }
    ((((r / 10) as u8) << 4) | (r % 10) as u8, borrow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    /// 0x103FE: at 0:00 with the divider at 0 and the lock clear, the lock is set, the words TIME and OVER are made (0x96450,
    /// pool C class 0x70, variants 0 and 1) and every hero whose state byte is below 4 loses all HP in that frame (0x1046A); a
    /// dying hero keeps his HP. The lock stops a second time over.
    #[test]
    fn time_over_makes_the_words_and_kills_the_heroes() {
        let mut g = game();
        for p in g.players.iter_mut().take(3) {
            p.b.live = 1;
            p.b.hp = 100;
        }
        g.players[1].b.mode = 2;
        g.players[2].b.mode = 4;
        g.rank.time_div = 1; // not yet
        g.time_limit();
        assert_eq!((g.rank.time_lock, g.effects.free_count()), (0, 48));
        g.rank.time_div = 0;
        g.time_limit();
        assert_eq!(g.rank.time_lock, 1);
        assert_eq!(g.players.iter().map(|p| p.b.hp).collect::<Vec<_>>(), [0, 0, 100]);
        let words: Vec<(u16, u16)> = g.effects.slots.iter().filter(|e| e.b.live != 0).map(|e| (e.b.k20, e.b.var26)).collect();
        assert_eq!(words, [(0x70, 1), (0x70, 0)], "slot 46 holds OVER, slot 47 TIME");
        g.time_limit();
        assert_eq!(g.effects.free_count(), 46, "the lock: no second pair");
    }

    /// 0x10482: a respawning hero adds 3:30 to the clock (at most 4:00) and 600 frames to G.$50C, and clears the lock; two heroes
    /// add it twice.
    #[test]
    fn a_respawn_tops_up_the_clock_once_for_each_hero() {
        let mut g = game();
        g.rank.time_lock = 1;
        g.spawn.g.wave_timer = 0x10;
        g.rank_respawn_time();
        assert_eq!((g.rank.time_min, g.rank.time_sec, g.rank.time_lock, g.spawn.g.wave_timer), (3, 0x30, 0, 0x268));
        g.rank_respawn_time();
        assert_eq!((g.rank.time_min, g.rank.time_sec, g.spawn.g.wave_timer), (4, 0, 0x4C0), "capped at 4:00");
        g.rank.time_min = 0;
        g.rank.time_sec = 0x45;
        g.rank_respawn_time();
        assert_eq!((g.rank.time_min, g.rank.time_sec), (4, 0), "0:45 + 3:30 = 4:15, capped");
        g.rank.time_min = 0;
        g.rank.time_sec = 0x20;
        g.rank_respawn_time();
        assert_eq!((g.rank.time_min, g.rank.time_sec), (3, 0x50));
    }

    #[test]
    fn bcd() {
        assert_eq!(bcd_sub(0x00, 1), (0x99, true));
        assert_eq!(bcd_sub(0x10, 1), (0x09, false));
        assert_eq!(bcd_add(0x45, 0x30, false), (0x75, false));
        assert_eq!(bcd_add(0x75, 0x30, false), (0x05, true));
    }
}
