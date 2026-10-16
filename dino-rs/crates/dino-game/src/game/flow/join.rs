//! Join task 0x6C4A. M2-18 ports the absent-player states and in-level prompts.
//! The hero strip, pick and drop-in are M2-19; continue and game over are M2-20/21.

use crate::game::Game;

/// Per-player join / continue state (player +$A2, +$B0 seconds, +$B1 frames).
#[derive(Clone, Copy, Debug, Default)]
pub struct Join {
    pub state: u8,
    pub seconds: u8,
    pub frames: u8,
}

impl Game {
    /// 0xA978: mode 0 creates task 0x6C4A in slot 0x10.
    pub fn flow_join_start(&mut self) {
        self.flow.join = Default::default();
    }

    /// 0x6C4A / 0x6D60: one pass of the in-level join task, in player order.
    /// Modes 0 and 4 still use the existing select task, which owns the same sleeper.
    pub fn flow_join_task(&mut self) {
        if matches!(self.flow.mode, 0 | 4) || !self.flow.join_task.tick() {
            return;
        }
        for i in 0..3 {
            self.players[i].b.who = i as u8;
        }
        let n = if self.flow.b4cf != 0 { 2 } else { 3 };
        for i in 0..n {
            match self.flow.join[i].state {
                0 => self.flow_join_absent(i),
                2 => self.flow_join_credit(i),
                4 => {
                    // 0x70F0 / 0x71DA: an already active object finishes choosing.
                    // TODO(M2-19): 0x70F8..0x71D8, strip timer, cursor and pick.
                    if self.players[i].b.live != 0 {
                        self.players[i].b.live = 1;
                        self.flow.chosen |= 1 << i;
                        self.flow.join[i].state = 6;
                    }
                }
                6 => {
                    // 0x72EE: remove the dead player's participation before continuing.
                    if self.players[i].b.live == 0 {
                        self.flow.chosen &= !(1 << i);
                        self.flow.started &= !(1 << i);
                        if self.flow.b4d7 == 0 {
                            self.flow.join[i] = Join { state: 8, seconds: 0x14, frames: 0x64 };
                        }
                        // TODO(M2-20/21): no-continue qualification and countdown drawing.
                    }
                }
                _ => {} // M2-20/21: continue, name entry and GAME OVER.
            }
        }
        // 0x6C8A..0x6D46: nobody playing, or only continue countdowns left.
        if self.flow.mode < 0x10
            && (self.flow.started == 0
                || (self.flow.b4d7 == 0 && !self.flow.join[..n].iter().any(|j| matches!(j.state, 4 | 6)) && self.flow.join[..n].iter().any(|j| j.state == 8)))
        {
            self.flow.saved_mode = [self.flow.mode, self.flow.sub, (self.flow.sub2 >> 8) as u8, self.flow.sub2 as u8];
            self.flow.mode = 0x10;
            self.flow.sub = 0;
            self.flow.sub2 = 0;
            // The second path (0x6D40) also holds the HUD and clears G.$5E4 (name entry, M2-22: not modeled yet).
            if self.flow.started != 0 {
                self.spawn.g.b4d5 = 1;
            }
        }
        // 0x6D4A: this hold belongs to the task, not the game frame counter.
        self.flow.front.sys.hold = self.flow.front.sys.hold.saturating_sub(1);
        self.flow.join_task.wait(1, 0);
    }

    /// 0x6D7C..0x6E28: absent without credit. The transition consumes one RNG call.
    fn flow_join_absent(&mut self, i: usize) {
        if self.flow.b4db != 0 {
            return;
        }
        let s = self.flow.front.sys;
        let (credits, coins) = self.flow_join_balance(i);
        // 0x6D84..0x6DC8: in play or free play first; the shared chute draws the credit line before its test (0x6DB8).
        let ready = self.players[i].b.live != 0 || s.free_play || {
            if s.common && s.hold != 0 {
                self.flow_join_credit_line(i);
            }
            credits != 0 || coins != 0
        };
        if ready {
            self.flow.join[i].state = 2;
            self.flow.front.sys.hold = 2;
            let _ = self.rand() & 0x1F; // 0x6DD8: anti-patch scan itself is omitted.
            return;
        }
        match self.flow.frame & 0x7F {
            0 => self.flow_join_prompt(i, 0x84D6),
            0x30 | 0x70 => self.flow_join_prompt(i, 0x843A),
            0x3F => self.flow_join_prompt(i, 0x852E),
            _ => {}
        }
    }

    /// 0x6E2A..0x6F6E: absent with coins, credit or free play. A request enters state 4.
    fn flow_join_credit(&mut self, i: usize) {
        if self.flow.b4db != 0 {
            return;
        }
        if self.players[i].b.live != 0 {
            self.flow.front.sys.hold = 3;
            self.flow_join_init(i);
            return;
        }
        let s = self.flow.front.sys;
        let (credits, coins) = self.flow_join_balance(i);
        if !s.free_play && credits == 0 && coins == 0 {
            self.flow.join[i].state = 0;
            return;
        }
        if !s.free_play && s.hold != 0 {
            self.flow_join_credit_line(i);
        }
        let cycle = if s.free_play || (credits != 0 && (!s.two_coin_start || credits >= 2)) { 0x3F } else { 0x7F };
        match self.flow.frame & cycle {
            0 => self.flow_join_prompt(
                i,
                if s.free_play {
                    0x8640
                } else if cycle == 0x3F {
                    0x867E
                } else {
                    0x8586
                },
            ),
            0x30 | 0x70 => self.flow_join_prompt(i, 0x843A),
            0x3F if cycle == 0x7F => {
                self.flow_join_prompt(i, if s.two_coin_start { 0x85C4 } else { 0x8602 });
                if !s.two_coin_start && self.flow_join_prompt_visible() {
                    let a = self.flow_join_prompt_at(i) + 4;
                    self.hud_put(a, s.coins_per_credit as u16, 0);
                }
            }
            _ => {}
        }
        if self.flow_join_start_check(i) {
            // TODO(M2-19): 0x1049E clock and wave budget; 0x7476 score; strip drawing. TODO(port): 0xA6B74, the join
            // counters G.$75BA and G.$75BC + 2 * who (statistics, not read by the game loop).
            self.rank_event_18410(self.flow.mode); // 0x6E98, before the player init's RNG call.
            self.flow.front.sys.hold = 3;
            self.flow_join_init(i);
        }
    }

    /// 0x6E52 / 0x6E70: credit and unconverted coin counts for this chute mode.
    fn flow_join_balance(&self, i: usize) -> (u8, u8) {
        let s = self.flow.front.sys;
        if s.common { (s.credits, s.common_coins) } else { (s.slots[i].credits, s.slots[i].coins) }
    }

    /// 0x6F70..0x7070: own start edge, or attack edge outside Japan with separate chutes.
    fn flow_join_start_check(&mut self, i: usize) -> bool {
        let s = self.flow.front.sys;
        let cost = if s.two_coin_start { 2 } else { 1 };
        if !s.free_play && self.flow_join_balance(i).0 < cost {
            return false;
        }
        let start = self.w.system & (self.w.system ^ self.flow.front.prev_system) & (1 << i) != 0;
        let attack = self.flow.region.0 != 0 && !s.common && self.w.input[i] & (self.w.input[i] ^ self.flow.front.prev_input[i]) & 0x10 != 0;
        if !start && !attack {
            return false;
        }
        if !s.free_play {
            let c = if s.common { &mut self.flow.front.sys.credits } else { &mut self.flow.front.sys.slots[i].credits };
            let result = c.wrapping_sub(cost);
            *c = if (result as i8) < 0 { 0 } else { result }; // 0x700E / 0x701E: SUB.B then BPL.
        }
        true
    }

    /// 0x7072..0x70EC: initialise the joining object and its strip counters.
    fn flow_join_init(&mut self, i: usize) {
        self.flow_boot_player(i); // 0x4D46 / 0x4D5E / 0x4D76, one power-factor RNG call.
        self.flow_join_prompt(i, 0x843A); // 0x80A2
        let b = &mut self.players[i].b;
        b.set_state(0x0002_0000); // 0x4DE8
        b.hp = 100;
        b.hp_prev = 100;
        b.hp_bar = 100;
        b.facing = 1;
        self.flow.join[i] = Join { state: 4, seconds: if self.flow.b4cf != 0 { 6 } else { 0x11 }, frames: 1 };
        self.flow.started |= 1 << i;
        // TODO(M2-19): 0x723E cursor reservation and 0x7F86 strip redraw.
    }

    /// 0x81FE / 0x825A: HUD hold suppresses prompts outside character select.
    fn flow_join_prompt_visible(&self) -> bool {
        self.spawn.g.b4d5 == 0 && self.w.scripted == 0
    }

    /// 0x8242 / 0x8292: prompt origin, read from the ROM layout table.
    fn flow_join_prompt_at(&self, i: usize) -> u32 {
        self.assets.frontend.text.u32(if self.flow.b4cf == 0 { 0x8242 } else { 0x8292 }, 4 * i as u32)
    }

    /// 0x8050 / 0x80A2 / 0x8104 / 0x81C6..0x81FE: draw a ROM prompt string.
    fn flow_join_prompt(&mut self, i: usize, string: u32) {
        if !self.flow_join_prompt_visible() {
            return;
        }
        let t = &self.assets.frontend.text;
        if self.flow.b4cf != 0 && matches!(string, 0x843A | 0x84D6 | 0x852E) {
            let at = t.u32(0x80F4, 4 * i as u32);
            let string = match string {
                0x843A => 0x8478,
                0x84D6 => 0x8514,
                _ => 0x856C,
            };
            let s = t.slice_from(string);
            if string == 0x8478 {
                self.text_words(s, at);
            } else {
                self.text_big(s, Some(at), true);
            }
        } else {
            let at = self.flow_join_prompt_at(i);
            self.text_words(t.slice_from(string), at);
        }
    }

    /// 0x82E8..0x83D4: refresh the small credit line while G.$477 is nonzero.
    fn flow_join_credit_line(&mut self, i: usize) {
        let s = self.flow.front.sys;
        if self.spawn.g.b4d5 != 0 || (s.common && self.flow.b4cf == 0) {
            return;
        }
        let t = &self.assets.frontend.text;
        let at = t.u32(if s.common { 0x83E2 } else { 0x83D6 }, 4 * i as u32);
        let (credits, coins) = self.flow_join_balance(i);
        let coins = if credits == 9 { 0 } else { coins };
        if s.free_play {
            self.hud_print(t.slice_from(0x842A), at);
        } else if s.two_coin_start {
            self.hud_print(t.slice_from(0x841A), at);
            self.hud_put(at + 0x580, credits as u16, 0x180);
        } else if s.coins_per_credit != 1 && s.credits_per_coin == 1 {
            self.text_words(t.slice_from(0x83FA), at);
            self.hud_put(at + 0x380, credits as u16, 0x180);
            self.hud_put(at + 0x500, coins as u16, 0x180);
            self.hud_put(at + 0x600, s.coins_per_credit as u16, 0x180);
        } else {
            self.hud_print(t.slice_from(0x83EA), at);
            self.hud_put(at + 0x580, credits as u16, 0x180);
        }
    }

    /// 0x7562 / 0x7580: redraw absent-player strips after the HUD slides in.
    pub fn flow_join_prompts(&mut self) {
        // These redraw states 4 and 8 only. Both drawing paths belong to M2-19/20.
    }

    /// Game mode 0x10 (0xE9E6). TODO(M2-20): continue screen.
    pub fn flow_continue_mode(&mut self) {
        self.flow.mode = 0x14;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::TraceReader;
    use dino_assets::AssetPack;
    use std::{path::Path, sync::OnceLock};

    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let p = Path::new("../../../dino.zip");
            p.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(p).unwrap())))
        })
    }

    #[test]
    fn join_direct_start_keeps_present_players_and_accepts_a_coin() {
        let Some(a) = assets() else { return };
        let mut g = Game::new(a);
        g.start_stage(0, &[0]);
        for frame in 0..25 {
            g.w.coins = if (5..13).contains(&frame) { 2 } else { 0 };
            g.tick();
        }
        assert_ne!(g.players[0].b.live, 0);
        assert_eq!(g.flow.join[0].state, 6);
        assert_eq!(g.flow.join[1].state, 2);
        assert_eq!(g.flow.front.sys.slots[1].credits, 1);
    }

    #[test]
    fn join_edges_credit_modes_and_inhibits() {
        let Some(a) = assets() else { return };
        // Jump, held attack, Japan's attack and common-pool attack cannot join.
        for (cur, prev, start, common, japan, credits, two, free, expected) in [
            (0x20, 0, 0, false, false, 1, false, false, false),
            (0x10, 0x10, 0, false, false, 1, false, false, false),
            (0x10, 0, 0, false, true, 1, false, false, false),
            (0x10, 0, 0, true, false, 1, false, false, false),
            (0x10, 0, 0, false, false, 0, false, false, false),
            (0x10, 0, 0, false, false, 1, false, false, true),
            (0, 0, 2, true, false, 1, false, false, true),
            (0, 0, 2, false, false, 1, true, false, false),
            (0, 0, 2, false, false, 2, true, false, true),
            (0, 0, 2, true, false, 0, false, true, true),
            (0, 0, 1, false, false, 1, false, false, false),
        ] {
            let mut g = Game::new(a);
            g.flow.region.0 = if japan { 0 } else { 2 };
            let s = &mut g.flow.front.sys;
            s.common = common;
            s.credits = credits;
            s.slots[1].credits = credits;
            s.two_coin_start = two;
            s.free_play = free;
            g.w.system = start;
            g.w.input[1] = cur;
            g.flow.front.prev_input[1] = prev;
            assert_eq!(g.flow_join_start_check(1), expected);
            assert_eq!(g.flow_join_balance(1).0, credits - if expected && !free { if two { 2 } else { 1 } } else { 0 });
        }
        let mut g = Game::new(a);
        g.flow.b4db = 1;
        g.flow.front.sys.free_play = true;
        g.flow.join[1].state = 2;
        g.w.system = 2;
        g.flow_join_credit(1);
        assert_eq!(g.flow.join[1].state, 2, "G.$4DB inhibits the request");
    }

    /// D4d A14: step from MAME's previous frame, including the system task before slot 0x10.
    #[test]
    fn join_a14_states_and_prompts_match_mame() {
        let Some(a) = assets() else { return };
        for (name, last) in [("flow_j-p2", 1873), ("flow_j-coin-only", 2712), ("flow_j-nocredit", 2112)] {
            let path = format!("../../../re/out/traces/{name}.mame.trc");
            if !Path::new(&path).exists() {
                continue;
            }
            let mut rd = TraceReader::open(&path).unwrap();
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut checked = 0;
            while let Some(f) = rd.next() {
                let ram = &rd.region("ram").unwrap().data;
                let gfx = &rd.region("gfx").unwrap().data;
                if let Some((pr, pg)) = &previous {
                    if f >= 1813 && f <= last {
                        let mut g = Game::new(a);
                        g.trace_load(pr, pg);
                        g.w.system = ram[0x801A];
                        g.w.coins = ram[0x8020];
                        g.w.input.copy_from_slice(&ram[0x8016..0x8019]);
                        g.flow_coin_isr();
                        g.flow_system_task();
                        g.flow_join_task();
                        for i in 1..3 {
                            let at = 0xB274 + 0x180 * i;
                            assert_eq!(g.flow.join[i].state, ram[at + 0xA2], "{name} frame {f} P{} state", i + 1);
                            // The strip starts in the request frame and belongs to M2-19.
                            if g.flow.join[i].state >= 4 {
                                continue;
                            }
                            for col in 16 * i..16 * i + 16 {
                                for row in 34..38 {
                                    let idx = crate::game::vram::Vram::index1(col, row);
                                    let off = 0x8000 + idx * 4;
                                    let expected = (u16::from_be_bytes([gfx[off], gfx[off + 1]]), u16::from_be_bytes([gfx[off + 2], gfx[off + 3]]));
                                    let cell = g.vram.scroll1[idx];
                                    assert_eq!((cell.code, cell.attr), expected, "{name} frame {f} P{} cell {col},{row}", i + 1);
                                }
                            }
                        }
                        checked += 1;
                    }
                }
                if f >= last {
                    break;
                }
                previous = Some((ram.clone(), gfx.clone()));
            }
            assert!(checked > 0, "{name}: recording did not reach the join timeline");
            eprintln!("{name}: join states and absent-player prompt cells equal for {checked} frames");
        }
    }
}
