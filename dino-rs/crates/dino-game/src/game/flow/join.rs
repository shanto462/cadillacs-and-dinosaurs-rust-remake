//! Join task 0x6C4A: absent-player prompts, quick hero strip and pick.
//! Continue is in `cont.rs`; game over and name entry are M2-21/22.

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
                4 => self.flow_join_strip(i),
                6 => {
                    // 0x72EE: remove the dead player's participation before continuing.
                    if self.players[i].b.live == 0 {
                        self.flow.chosen &= !(1 << i);
                        self.flow.started &= !(1 << i);
                        if self.flow.b4d7 == 0 {
                            self.flow.join[i] = Join { state: 8, seconds: 0x14, frames: 0x64 };
                            self.flow_join_clear(i, 0x7FB6, 0x7FC2);
                            self.flow_continue_draw_hud(i); // 0x7328
                        }
                        if self.flow.b4d7 != 0 {
                            // 0x7392: no continue. The HUD prompt is cleared, then the qualification.
                            self.flow_join_clear(i, 0x7FB6, 0x7FC2);
                            self.name_qualify(i);
                        }
                    }
                }
                8 => self.flow_continue_player(i),
                10 => self.name_state(i), // 0x74A4: name entry
                0xC => self.flow_join_game_over(i),
                _ => {}
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
            // 0x6D46: the game task is still completing its old mode in this
            // video frame. The continue setup first completes next frame.
            self.flow.wait = 2;
            self.flow_continue_begin();
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
            // TODO(port): 0xA6B74, the join
            // counters G.$75BA and G.$75BC + 2 * who (statistics, not read by the game loop).
            self.rank_event_18410(self.flow.mode); // 0x6E98, before the player init's RNG call.
            self.rank_join_time(); // 0x6E9E
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
        if self.flow.mode != 4 {
            self.flow_join_scores(); // 0x70A8
        }
        let b = &mut self.players[i].b;
        b.set_state(0x0002_0000); // 0x4DE8
        b.hp = 100;
        b.hp_prev = 100;
        b.hp_bar = 100;
        b.facing = 1;
        self.flow.join[i] = Join { state: 4, seconds: if self.flow.b4cf != 0 { 6 } else { 0x11 }, frames: 1 };
        self.players[i].w80 = 0;
        self.flow_join_cursor(i, -1); // 0x70D4
        self.flow_join_cursor(i, 1); // 0x70DA
        self.flow.started |= 1 << i;
        self.flow_join_clear(i, 0x7FB6, 0x7FC2); // 0x7F86
    }

    /// 0x70F0..0x71FC: quick select, then let 0x18B38 run the ordinary drop-in.
    fn flow_join_strip(&mut self, i: usize) {
        if self.players[i].b.live == 0 {
            if self.flow.mode != 0xC || (self.players.iter().any(|p| p.b.live != 0) && !self.flow_join_prompt_visible()) {
                return;
            }
            self.flow_join_timer(i); // 0x7138
            self.flow_join_flash(i); // 0x713C
            // 0x71FE reads the object's own input history, not the system task's.
            let p = &mut self.players[i];
            p.prev = p.held;
            p.held = self.w.input[i];
            p.pressed = p.held & (p.held ^ p.prev);
            let edge = p.pressed;
            if edge != 0 {
                p.w80 = 0;
                let direction = if edge & 2 != 0 { -1 } else if edge & 1 != 0 { 1 } else { 0 };
                if direction != 0 {
                    self.flow_join_cursor(i, direction);
                    self.flow_join_flash(i);
                }
            }
            let j = self.flow.join[i];
            let locked = j.seconds == if self.flow.b4cf == 0 { 0x10 } else { 5 } && j.frames >= 0x14;
            let start = self.w.system & (self.w.system ^ self.flow.front.prev_system) & (1 << i) != 0;
            if (locked || (edge & 0x10 == 0 && !start)) && (j.seconds as i8) >= 0 {
                return;
            }
        }
        self.players[i].b.live = 1; // 0x71DA
        self.flow.chosen |= 1 << i;
        self.flow.join[i].state = 6;
        if self.flow.mode != 4 {
            self.flow_join_clear(i, 0x7FB6, 0x7FC2);
        }
    }

    /// 0x723E: move in select order, skipping only heroes held by strip or play states.
    pub(super) fn flow_join_cursor(&mut self, i: usize, direction: i8) {
        let t = &self.assets.frontend.text;
        loop {
            let pos = t.u8(0x7232, self.players[i].b.id as u32).wrapping_add(direction as u8) & 3;
            let hero = t.u8(0x7236, 2 * pos as u32);
            self.players[i].b.id = hero;
            self.players[i].b.k20 = t.u8(0x7236, 2 * pos as u32 + 1) as u16;
            if !(0..3).any(|k| k != i && self.flow.started & (1 << k) != 0
                && matches!(self.flow.join[k].state, 4 | 6) && self.players[k].b.id == hero) {
                break;
            }
        }
    }

    /// 0x7476 / 0x748A: increment only the last BCD score byte, saturating it at 99.
    pub(super) fn flow_join_scores(&mut self) {
        for p in &mut self.players {
            let lo = p.score as u8;
            let next = if lo >= 0x99 { 0x99 } else if lo & 15 == 9 { lo.wrapping_add(7) } else { lo.wrapping_add(1) };
            p.score = (p.score & !0xFF) | next as u32;
        }
    }

    /// 0x7D5C: BCD countdown, 40 frames per step (70 in the two-player layout).
    fn flow_join_timer(&mut self, i: usize) {
        let j = &mut self.flow.join[i];
        j.frames = j.frames.wrapping_sub(1);
        if j.frames != 0 { return; }
        j.frames = if self.flow.b4cf == 0 { 40 } else { 70 };
        if (j.seconds as i8) < 0 { return; }
        j.seconds = if j.seconds == 0 { 0x99 } else if j.seconds & 15 == 0 { j.seconds.wrapping_sub(7) } else { j.seconds.wrapping_sub(1) };
        if (j.seconds as i8) >= 0 { self.flow_join_draw_timer(i); }
    }

    /// 0x7D8E: the red countdown, small digits in 3P and big digits in 2P.
    fn flow_join_draw_timer(&mut self, i: usize) {
        if self.spawn.g.b4d5 != 0 { return; }
        let t = &self.assets.frontend.text;
        let n = self.flow.join[i].seconds;
        if self.flow.b4cf == 0 {
            let at = t.u32(0x7E20, 4 * i as u32);
            self.hud_put(at, if n >> 4 == 0 { 0x20 } else { (n >> 4) as u16 }, 0x182);
            self.hud_put(at + 0x80, (n & 15) as u16, 0x182);
        } else {
            let at = t.u32(0x7E2C, 4 * i as u32);
            // 0x7DF0 uses LSL, so the first big digit is always zero.
            self.text_big(t.slice_from(0x7E34), Some(at), true);
            self.text_big(t.slice_from(0x7E34 + 6 * (n & 15) as u32), Some(at + 0x100), true);
        }
    }

    /// 0x74E2: the GAME OVER state of the HUD block of a player (state 0xC). The counter `+$80` (a byte) runs down one a frame; a start press
    /// of the player, or the end of the count (a borrow), makes the block free again (state 0) and blanks its rectangle (0x7F86 / 0x7FCA). A player
    /// who comes from the ending with the count at 0 leaves at once.
    fn flow_join_game_over(&mut self, i: usize) {
        let leave = if self.flow.mode == 0x10 {
            true // 0x74E2: the continue screen sets the block free
        } else if self.spawn.g.b4d5 != 0 {
            return; // 0x7560
        } else if self.players[i].pressed & 0x10 != 0 {
            true // 0x74F4: btst #4,$A8(a6)
        } else {
            let starts = self.w.system & !self.flow.front.prev_system; // 0x74FE: the new presses of the start buttons
            if starts & (1 << i) != 0 {
                true
            } else {
                let b = (self.players[i].w80 >> 8) as u8;
                self.players[i].w80 = (self.players[i].w80 & 0xFF) | (b.wrapping_sub(1) as u16) << 8; // subq.b #1,$80(a6)
                b == 0 // no borrow: bcc 0x754C, the prompt of the count
            }
        };
        if !leave {
            // 0x754C: the label of the block by the frame counter (the join task runs before the game task counts it): the word G.$4DA & $3F = 0 draws the
            // GAME OVER string 0x875C at the origin of the player (0x8196, only while G.$4D5 is 0), 0x30 draws the blank block (0x80A2).
            match self.flow.frame & 0x3F {
                0 if self.flow_join_prompt_visible() => {
                    let at = self.flow_join_prompt_at(i);
                    let t = &self.assets.frontend.text;
                    self.text_words(t.slice_from(0x875C), at);
                }
                0x30 => self.flow_join_prompt(i, 0x843A),
                _ => {}
            }
            return;
        }
        self.flow.join[i].state = 0; // 0x7538
        if self.flow.mode == 0x10 {
            self.flow_join_clear(i, 0x7FFA, 0x8006); // 0x7FCA
        } else {
            self.flow_join_clear(i, 0x7FB6, 0x7FC2); // bra 0x7F86
        }
    }

    /// 0x7F86 / 0x7FCA / 0x800E -> 0xF94: blank the ROM rectangle in the HUD block.
    pub(super) fn flow_join_clear(&mut self, i: usize, three: u32, two: u32) {
        if self.spawn.g.b4d5 != 0 { return; }
        let t = &self.assets.frontend.text;
        let a = if self.flow.b4cf == 0 { three } else { two } + 4 * i as u32;
        let col = t.u8(a, 0) as u32;
        let row = t.u8(a, 1) as u32;
        for x in 0..t.u8(a, 2) as u32 {
            for y in 0..t.u8(a, 3) as u32 {
                self.hud_put(0x90A000 + (col + x) * 0x80 + (row % 32 + y) * 4, 0x20, 0x180);
            }
        }
    }

    /// 0x75B2: HUD strip redraw and arrow/countdown flashing on a 256-frame cycle.
    fn flow_join_flash(&mut self, i: usize) {
        let old = self.players[i].w80;
        self.players[i].w80 = old.wrapping_add(1);
        match old & 0xFF {
            0 => self.flow_join_draw_strip(i),
            0x70 | 0xF0 => {
                self.flow_join_clear(i, 0x803C, 0x8048);
                self.flow_join_draw_timer(i);
            }
            0x80 => {
                let t = &self.assets.frontend.text;
                let at = t.u32(if self.flow.b4cf == 0 { 0x82D4 } else { 0x82E0 }, 4 * i as u32);
                self.text_words(t.slice_from(0x86F2), at);
                self.flow_join_draw_timer(i);
            }
            _ => {}
        }
    }

    /// 0x7AE4 / 0x7BC8 / 0x75F0 / 0x780C: face, name, countdown and rating.
    fn flow_join_draw_strip(&mut self, i: usize) {
        let t = &self.assets.frontend.text;
        let hero = self.players[i].b.id as u32;
        let at = t.u32(if self.flow.b4cf == 0 { 0x7B34 } else { 0x7B40 }, 4 * i as u32);
        let attr = t.u16(0x7B2C, 2 * hero);
        for x in 0..4u32 {
            for y in 0..4u32 {
                self.hud_put(at + x * 0x80 + y * 4, t.u16(0x7B48, hero * 32 + x * 8 + y * 2), attr);
            }
        }
        let base = if self.flow.b4cf == 0 { 0x7C00 } else { 0x7CFC };
        let at = t.u32(if self.flow.b4cf == 0 { 0x7BF4 } else { 0x7CF4 }, 4 * i as u32);
        let s = t.slice_from(base + t.u16(base, 2 * hero) as u32);
        if self.flow.b4cf == 0 { self.text_words(s, at); } else { self.text_big(s, Some(at), true); }
        self.flow_join_draw_timer(i);
        let (origins, base) = if self.flow.b4cf == 0 {
            (0x7640, if self.flow.region.0 == 0 { 0x764C } else { 0x7654 })
        } else {
            (0x785C, if self.flow.region.0 == 0 { 0x7864 } else { 0x786C })
        };
        // 0x7608 clears d0 after computing the Japanese offset. Keep that quirk.
        let offset = if self.flow.b4cf == 0 && self.flow.region.0 == 0 { 0 } else { t.u16(base, 2 * hero) as u32 };
        self.text_words(t.slice_from(base + offset), t.u32(origins, 4 * i as u32));
    }

    /// 0x81FE / 0x825A: the hold G.$4D5 suppresses the prompts outside the character select (mode 4: another position). The scripted control
    /// (G.$4DD) does not (the recording ep2_jack draws the blinking INSERT COIN while the area walk is scripted).
    fn flow_join_prompt_visible(&self) -> bool {
        self.spawn.g.b4d5 == 0
    }

    /// 0x8242 / 0x8292: prompt origin, read from the ROM layout table.
    pub(super) fn flow_join_prompt_at(&self, i: usize) -> u32 {
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
        for i in 0..3 {
            if self.players[i].b.live == 0 && self.flow.join[i].state == 4 {
                self.flow_join_draw_strip(i); // 0x7592..0x759E
            }
        }
        for i in 0..3 {
            if self.flow.join[i].state == 8 {
                self.flow_continue_draw_hud(i); // 0x75AC
            }
        }
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

    /// 0x754C: the GAME OVER label of a block in state 0xC is drawn when G.$4DA & $3F is 0 (0x8196: the string 0x875C at the origin of the player) and erased
    /// at $30 (0x80A2, the blank block 0x843A); the other frames draw nothing (m523_ep1_2p_full 28371: the erase of the block of P1, 1 frame late before).
    #[test]
    fn the_game_over_label_follows_the_frame_counter() {
        let Some(a) = assets() else { return };
        let mut g = Game::new(a);
        g.flow.mode = 0xC;
        g.flow.join[0].state = 0xC;
        g.players[0].w80 = 0x0A00; // the counter byte of the block, far from a borrow
        g.spawn.g.b4d5 = 0;
        let at = g.flow_join_prompt_at(0);
        let text = &a.frontend.text;
        let first = text.u16(0x875C, 2); // the first code word after the attribute word
        g.flow.frame = 5;
        g.flow_join_game_over(0);
        assert_ne!(g.hud_read(at), first, "no draw at 5");
        g.flow.frame = 0x40; // & 0x3F = 0
        g.flow_join_game_over(0);
        assert_eq!(g.hud_read(at), first, "the label at 0");
        g.flow.frame = 0x30;
        g.flow_join_game_over(0);
        assert_eq!(g.hud_read(at), 0x20, "the blank block at 0x30");
        assert_eq!(g.flow.join[0].state, 0xC, "the block stays in the game over state");
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

    #[test]
    fn join_strip_and_clock_rules() {
        let Some(a) = assets() else { return };
        let mut g = Game::new(a);
        g.flow.mode = 0xC;
        g.flow.started = 1;
        g.flow.join[0].state = 6;
        g.players[0].b.live = 1;
        g.flow_join_init(1);
        assert_eq!(g.players[1].b.id, 2);
        assert_eq!(g.players.iter().map(|p| p.score).collect::<Vec<_>>(), vec![1, 1, 1]);
        g.flow_join_strip(1);
        assert_eq!((g.flow.join[1].seconds, g.flow.join[1].frames), (0x10, 40));
        for _ in 0..39 { g.flow_join_strip(1); }
        assert_eq!(g.flow.join[1].seconds, 0x10);
        g.flow_join_strip(1);
        assert_eq!((g.flow.join[1].seconds, g.flow.join[1].frames), (9, 40));
        // A dead hero in the continue countdown no longer holds his choice.
        g.flow.join[0].state = 8;
        g.flow_join_cursor(1, -1);
        assert_eq!(g.players[1].b.id, 0);
        for time in [0x100, 0x350] {
            g.rank_set_time(time);
            g.spawn.g.wave_timer = 0xFF00;
            g.rank_join_time();
            assert_eq!((g.rank.time_min, g.rank.time_sec), (4, 0));
            assert_eq!(g.spawn.g.wave_timer, 0x158);
        }
        g.players[0].score = 0x123499;
        g.flow_join_scores();
        assert_eq!(g.players[0].score, 0x123499);
    }

    /// D4d A14: replay the join task from each MAME snapshot, including HUD cells.
    #[test]
    fn join_a14_strip_matches_mame() {
        let Some(a) = assets() else { return };
        for name in ["flow_j-p2", "flow_j-p2-wait", "flow_j-p2-moves", "flow_j-p23"] {
            let path = format!("../../../re/out/traces/{name}.mame.trc");
            if !Path::new(&path).exists() { continue; }
            let mut rd = TraceReader::open(&path).unwrap();
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut checked = 0;
            while let Some(f) = rd.next() {
                let ram = &rd.region("ram").unwrap().data;
                let gfx = &rd.region("gfx").unwrap().data;
                if let Some((pr, pg)) = &previous {
                    if f >= 1813 && pr[0x84D0] == 0xC {
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
                            if pr[at + 0xA2] != 4 { continue; }
                            assert_eq!((g.flow.join[i].state, g.flow.join[i].seconds, g.flow.join[i].frames),
                                (ram[at + 0xA2], ram[at + 0xB0], ram[at + 0xB1]), "{name} frame {f} P{} timer/pick", i+1);
                            assert_eq!(g.players[i].b.id, ram[at+3], "{name} frame {f} cursor");
                            if g.flow.join[i].state == 6 { checked += 1; continue; }
                            for col in 16*i..16*i+16 {
                                for row in 34..38 {
                                    let idx = crate::game::vram::Vram::index1(col, row);
                                    let off = 0x8000 + idx*4;
                                    let expected = (u16::from_be_bytes([gfx[off], gfx[off+1]]), u16::from_be_bytes([gfx[off+2], gfx[off+3]]));
                                    let cell = g.vram.scroll1[idx];
                                    assert_eq!((cell.code,cell.attr), expected, "{name} frame {f} P{} cell {col},{row}", i+1);
                                }
                            }
                            checked += 1;
                        }
                    }
                }
                previous = Some((ram.clone(),gfx.clone()));
            }
            assert!(checked > 0);
            eprintln!("{name}: strip timers, cursor, pick and HUD equal for {checked} player frames");
        }
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
