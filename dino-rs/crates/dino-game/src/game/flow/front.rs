//! Front end: the system task 0x9D048 (coins, credits, start; slot 0), the coin interrupt code
//! 0x9D4BE, the state of the attract task (`attract.rs`, `boot.rs`) and the character select (game
//! mode 4, 0xAD46). Owner: flow / HUD port. Spec: `D4d-front-end-and-story.md` s3.10, s4.
//!
//! This is the minimal system task of M1-04: coin acceptance with its debounce, the conversion of coins
//! into credits (all coin modes of 0x9D038, the three chute modes), the press start screen when a
//! coin or credit is there, the start detection, the start of a game and the wait counter G.$4F6 for
//! the rank. Not ported yet (M2-02): the credit text and the prompts (0x9D66A, 0x9D752), SERVICE
//! (0x9D326), the coin lockout outputs (0x9D40E), the coin counter pulse (0x9D478), the TEST button
//! (0x9D5A4) and the play time statistics (0xA6B8A). The screen of a coin therefore shows the press
//! start screen without the prompts.

use crate::game::Game;
use crate::game::flow::attract::Stub;
use crate::game::flow::scene::Scene;

/// Character select struct G.$62C (cleared by 0xAD36).
#[derive(Clone, Copy, Debug, Default)]
pub struct Select {
    /// +3 taken cursor bits, +9 seconds left, +$A frame counter.
    pub taken: u8,
    pub seconds: u8,
    pub frames: u8,
}

/// One coin slot block of the original, 16 bytes at G.$47A + 0x10 * player.
#[derive(Clone, Copy, Debug, Default)]
pub struct CoinSlot {
    /// +0 debounce timer (0x32 frames from the first two high frames of a coin signal).
    pub timer: u8,
    /// +1 coins waiting to be converted.
    pub coins: u8,
    /// +2 the last four samples of the coin input, one bit each.
    pub shift: u8,
    /// +3 credits of this player (separate chutes), at most 9.
    pub credits: u8,
    /// +4 the coin count before the last coin / conversion.
    pub prev: u8,
}

/// The system task's variables G.$46A..G.$477 (cleared and set up by 0x9CFC6). Some are only kept
/// for the credit text and the coin counter of M2-02.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Sys {
    /// G.$46A credits of the common pool (the one chute modes), at most 9.
    pub credits: u8,
    /// G.$46B free play.
    pub free_play: bool,
    /// G.$46C "2 coins start, 1 coin continue" (coin mode 8).
    pub two_coin_start: bool,
    /// G.$46D the press start screen is up (0xFF).
    pub press_up: u8,
    /// G.$46E coins accepted, for the coin counter pulse (not ported).
    pub pulse: u8,
    /// G.$470 coins per credit batch, G.$471 credits per batch.
    pub coins_per_credit: u8,
    pub credits_per_coin: u8,
    /// G.$472 one chute for all players (common pool), G.$474 two players on one chute.
    pub common: bool,
    pub two_player: bool,
    /// G.$473 the credit text must be drawn again.
    pub redraw: bool,
    /// G.$475 / G.$476 coins of the common pool and the count before.
    pub common_coins: u8,
    pub common_prev: u8,
    /// G.$477 prompt hold timer (2 after a coin).
    pub hold: u8,
    /// G.$47A, G.$48A, G.$49A the coin slots of players 1 to 3.
    pub slots: [CoinSlot; 3],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Front {
    pub select: Select,
    /// G.-$702E set by the game over (0xAC20).
    pub game_over_flag: u8,
    /// G.$1A of the previous frame (G.$1F), for edge detection.
    pub prev_system: u8,
    /// G.$16..18 of the previous frame (G.$1B..1D), kept by `flow_frame`.
    pub prev_input: [u8; 3],
    /// Slot 0x10: the boot task 0xA69A, then the attract task 0x97C20 (`boot.rs`, `attract.rs`).
    pub task: Scene,
    /// Slot 0x20: the INSERT COIN blink task 0x98874 or the press start screen task 0x978EE.
    pub blink: Scene,
    pub press: Scene,
    /// G.-$7022 the title frame counter (0 to 0x258).
    pub title: u16,
    /// G.-$7030 the attract state byte (0, 2, 4, 6).
    pub attract_state: u8,
    /// G.$757A the demo counter (the demo row, M1-12).
    pub demo_counter: u16,
    /// Slot 0: the system task 0x9D048.
    pub sys_task: Scene,
    pub sys: Sys,
    /// Frames counted since power on (the original has no such counter; the stubs need it).
    pub ticks: u32,
    /// Passes of the system task to skip (the attract task is busy and the scan of the task slots
    /// does not come round, see `Game::flow_attract_busy`).
    pub sys_skip: u8,
    /// The stub screen that is up and the frame it started.
    pub stub: Stub,
    pub stub_start: u32,
    /// How many times the title, the profile cards and the ranking ran (the measured lengths of
    /// some frames differ by pass).
    pub title_runs: u16,
    pub profile_runs: u16,
    pub rank_runs: u16,
    /// What is different in this run of the title (`boot.rs`, `TITLE_RUNS`): the frames from the setup to
    /// its first pass, the cells of the blocks drawn in the setup frame, the blink task's first frame.
    pub title_run: (u8, u16, u8),
    /// The part of the clear 0xAE0 still to do in the next frames of the attract task (0 none, 1
    /// scroll2 and scroll3, 2 scroll3).
    pub clear_stage: u8,
    /// The game over ranking of 0x97C94 is up: the pass goes on after it.
    pub after_ranking_pass: bool,
}

/// Script positions of the system task.
mod sst {
    /// One pass of the loop 0x9D04C..0x9D072 per frame.
    pub const LOOP: u16 = 0;
    /// The fade out of 0x9D16E is over: clear all layers.
    pub const CLEAR: u16 = 1;
    /// After the clear: the game task is created.
    pub const CREATE: u16 = 2;
}

impl Game {
    /// 0x9CFC6, called by the boot task: clears G.$46A..G.$4A9 and sets the coin and chute modes
    /// from the settings (G.$777A and G.$777C) and the coins per credit from the table 0x9D038.
    pub fn flow_sys_init(&mut self) {
        let (a, c) = (self.flow.dip.dip_a, self.flow.dip.dip_c);
        let mut s = Sys { free_play: c & 4 != 0, common: a & 8 != 0, ..Default::default() };
        if a & 0x10 != 0 {
            // two players on one chute: the 2 player layout G.$4CF, a common pool
            self.flow.b4cf = 1;
            s.two_player = true;
            s.common = true;
        }
        if a & 0x40 != 0 {
            // coin mode 8: 2 coins start, 1 coin continue
            s.two_coin_start = true;
            s.coins_per_credit = 1;
            s.credits_per_coin = 1;
        } else {
            let t = &self.assets.frontend.text;
            let i = 2 * (a & 7) as u32;
            s.coins_per_credit = t.u8(0x9D038, i);
            s.credits_per_coin = t.u8(0x9D038, i + 1);
        }
        self.flow.front.sys = s;
        self.flow.front.sys_task = Scene::start(sst::LOOP);
    }

    /// 0x9D4BE: the coin input of the three slots, run by the vertical blank interrupt before the
    /// tasks. A coin is accepted when the input was high for two frames and low for the next two.
    pub fn flow_coin_isr(&mut self) {
        for i in 0..3 {
            let bit = self.w.coins & (1 << i) != 0;
            self.flow_coin_slot(i, bit);
        }
    }

    /// 0x9D50A: one slot. `bit` is the coin input of this frame.
    fn flow_coin_slot(&mut self, i: usize, bit: bool) {
        let mut slot = self.flow.front.sys.slots[i];
        slot.shift = ((slot.shift << 1) | bit as u8) & 0xF;
        if slot.timer == 0 {
            // idle: the first two high frames (shift 0b0011) start the timer
            if slot.shift == 3 {
                slot.timer = 0x32;
            }
            self.flow.front.sys.slots[i] = slot;
            return;
        }
        slot.timer -= 1;
        if slot.timer == 0 {
            // the coin took too long
        } else if slot.shift == 0 {
            slot.timer = 0;
        } else if slot.shift == 0xC {
            // accepted: two frames high, two frames low
            let s = &mut self.flow.front.sys;
            s.pulse = s.pulse.wrapping_add(1);
            slot.prev = slot.coins;
            if s.common {
                s.common_coins = s.common_coins.wrapping_add(1);
            } else {
                slot.coins = slot.coins.wrapping_add(1);
            }
            s.redraw = true;
            s.hold = 2;
            self.flow.front.sys.slots[i] = slot;
            if self.flow.in_game == 0 {
                self.sound(0xFF00); // 0xA5D2
            }
            self.sound(0x30); // the coin sound (d1 = 0x70010, d2 = 0x3800000)
            self.flow.front.sys.slots[i].timer = 0;
            return;
        }
        self.flow.front.sys.slots[i] = slot;
    }

    /// 0x9D386: the conversion of coins into credits, every frame. A batch of `coins_per_credit`
    /// coins gives `credits_per_coin` credits, at most 9.
    fn flow_sys_convert(&mut self) {
        let s = &mut self.flow.front.sys;
        let (cpc, add) = (s.coins_per_credit, s.credits_per_coin);
        if s.common {
            // 0x9D3A6
            if s.common_coins < cpc {
                return;
            }
            s.common_prev = s.common_coins;
            s.common_coins -= cpc;
            s.credits = s.credits.wrapping_add(add);
            if s.credits >= 10 {
                s.credits = 9;
            }
            return;
        }
        for slot in s.slots.iter_mut() {
            // 0x9D3DA
            if slot.coins < cpc {
                continue;
            }
            slot.prev = slot.coins;
            slot.coins -= cpc;
            slot.credits = slot.credits.wrapping_add(add);
            if slot.credits >= 10 {
                slot.credits = 9;
            }
        }
    }

    /// The system task 0x9D048: one pass per frame (conversion, the attract part 0x9D074, the wait
    /// counter 0x1827A, sleep 1), or the start of a game (the task sleeps through the fade out).
    pub fn flow_system_task(&mut self) {
        if !self.flow.front.sys_task.tick() {
            return;
        }
        match self.flow.front.sys_task.state {
            sst::LOOP => {
                // The attract task is busy (a clear of all layers that the port writes as a sleep):
                // the scan of the task slots does not come round to slot 0 in these frames.
                if self.flow.front.sys_skip > 0 || self.flow_attract_pass_lost() {
                    self.flow.front.sys_skip = self.flow.front.sys_skip.saturating_sub(1);
                    self.flow.front.sys_task.sleep(1);
                    return;
                }
                // 0x9D326: SERVICE (M2-02), then the conversion 0x9D386; 0x9D40E (lockout) and
                // 0x9D478 (coin counter pulse) are M2-02
                self.flow_sys_convert();
                if self.flow_sys_attract() {
                    return; // a game starts: the task sleeps through the fade out
                }
                self.flow_sys_pass_end();
            }
            sst::CLEAR => {
                // 0x9D174: sleep 1 after the fade, then 0xAE0 (1.3 frames)
                self.flow_clear_screen();
                self.flow.front.sys_task.wait(2, sst::CREATE);
            }
            _ => {
                // 0x9D180: the press start flag is cleared, the game task 0xA870 is created in slot
                // 0x20. The pass then ends like any other (0x9D05C, sleep 1).
                self.flow.front.sys.press_up = 0;
                self.flow.game_pending = false;
                self.flow_game_start();
                self.flow_sys_pass_end();
            }
        }
    }

    /// 0x9D05C and the sleep of the loop: the wait counter G.$4F6 (0x1827A), the TEST button
    /// (0x9D5A4, not mapped) and the statistics (0xA6B8A, not ported), sleep 1.
    fn flow_sys_pass_end(&mut self) {
        let s = self.flow.front.sys;
        let pool = ((s.credits as u16) << 8) | s.free_play as u16; // tst.w $46a(a5)
        self.rank_waited_tick(self.flow.in_game != 0, pool);
        self.flow.front.sys_task.wait(1, sst::LOOP);
    }

    /// 0x9D074: while nobody plays: the press start screen as soon as a coin or credit is there,
    /// and the detection of a start. True when a game starts.
    fn flow_sys_attract(&mut self) -> bool {
        if self.flow.in_game != 0 {
            return false;
        }
        let s = self.flow.front.sys;
        if !s.free_play {
            // 0x9D084: is a credit or a coin waiting?
            let waiting = if s.common {
                s.credits != 0 || s.common_coins != 0
            } else {
                s.slots.iter().any(|c| c.credits != 0) || s.slots.iter().any(|c| c.coins != 0)
            };
            if !waiting {
                return false;
            }
            if s.press_up == 0 {
                // 0x9D0C8: the attract tasks (slots 0x10 to 0x40) end, the press start screen
                self.flow_kill_front_tasks();
                self.flow.front.sys.press_up = 0xFF;
                self.flow_press_start_screen(); // 0x983EE
                self.flow.front.sys.redraw = true;
            }
            if self.flow.front.sys.redraw {
                // 0x9D102: the credit text and the prompts (0x9D66A, 0x9D752): M2-02
                self.flow.front.sys.redraw = false;
            }
        }
        // 0x9D124: who pressed start?
        self.flow.started = 0;
        self.flow_sys_start_detect();
        if self.flow.started == 0 {
            return false;
        }
        // 0x9D134: the tasks of slots 0x10 to 0x40 end, G.$4CC = 1, sound 0x31, fade out (0x87BA)
        self.flow_kill_front_tasks();
        self.flow.in_game = 1;
        self.flow.game_pending = true; // the game task does not exist for the next 18 frames
        self.sound(0x31); // d1 = 0x10
        self.fade.mask = 0x0F;
        self.start_fade_task(false); // 16 steps, the first one now
        self.flow.front.sys_task.wait(16, sst::CLEAR);
        true
    }

    /// `jsr 0x92E` for the slots 0x10 (attract task), 0x20 (blink or press start task), 0x30 (stage
    /// intro and clear tasks) and 0x40 (fade task).
    fn flow_kill_front_tasks(&mut self) {
        self.flow.front.task.kill();
        self.flow.front.blink.kill();
        self.flow.front.press.kill();
        self.flow.intro.live = false;
        self.flow.clear.live = false;
        self.fade.task = None;
    }

    /// 0x9D196: the start of each player: the start edge (the start button changed) with enough
    /// credits takes them and marks the player in G.$4CD (`flow.started`). In free play every
    /// player has 9 credits for the check and none after it.
    fn flow_sys_start_detect(&mut self) {
        let free = self.flow.front.sys.free_play;
        let two_coin = self.flow.front.sys.two_coin_start;
        let changed = self.w.system ^ self.flow.front.prev_system; // G.$1A eor G.$1F
        if self.flow.front.sys.two_player {
            // 0x9D266: player 1 and player 2 (bit 3 = both) share the pool
            if free {
                self.flow.front.sys.credits = 9;
            }
            if self.flow.front.sys.credits != 0 {
                self.flow_sys_start_pool(changed, 1, 1, if two_coin { 2 } else { 1 });
                self.flow_sys_start_pool(changed, 2, 3, if two_coin { 4 } else { 2 });
            }
            if free {
                self.flow.front.sys.credits = 0;
            }
        } else if self.flow.front.sys.common {
            // 0x9D21E
            if free {
                self.flow.front.sys.credits = 9;
            }
            if self.flow.front.sys.credits != 0 {
                for bit in [1u8, 2, 4] {
                    self.flow_sys_start_pool(changed, bit, bit, if two_coin { 2 } else { 1 });
                }
            }
            if free {
                self.flow.front.sys.credits = 0;
            }
        } else {
            // 0x9D1A6: separate credits
            if free {
                self.flow.front.sys.slots.iter_mut().for_each(|c| c.credits = 9);
            }
            for i in 0..3 {
                if self.flow.front.sys.slots[i].credits != 0 {
                    // 0x9D2A2
                    let need = if two_coin { 2 } else { 1 };
                    let bit = 1u8 << i;
                    let c = &mut self.flow.front.sys.slots[i].credits;
                    if need <= *c && changed & bit != 0 {
                        *c = c.saturating_sub(need);
                        self.flow.started |= bit;
                    }
                }
            }
            if free {
                self.flow.front.sys.slots.iter_mut().for_each(|c| c.credits = 0);
            }
        }
    }

    /// 0x9D2EC / 0x9D2DC: a start with the credits of the common pool (`need` credits, the start
    /// button `bit` changed, `mark` goes into G.$4CD).
    fn flow_sys_start_pool(&mut self, changed: u8, bit: u8, mark: u8, need: u8) {
        let c = &mut self.flow.front.sys.credits;
        if need <= *c && changed & bit != 0 {
            *c = c.saturating_sub(need);
            self.flow.started |= mark;
        }
    }

    /// 0x9D18A..0x9D18E: the game task 0xA870 is created: the game starts with the players that
    /// pressed start. The first pass of its mode 0 comes 2 frames later and sets mode 4 2 more frames after that (0xA978 needs 4
    /// frames).
    fn flow_game_start(&mut self) {
        self.flow_game_task_start(); // 0xA870: G.$4CC = 1, G.$4D5 = 1, ...
        // 0x4D46, 0x4D5E, 0x4D76: the three player objects (one rand() each)
        for i in 0..3 {
            self.flow_boot_player(i);
        }
        for i in 0..3 {
            self.players[i].b.live = (self.flow.started >> i & 1) as u8; // 0xA8CC: +$00 = 1
        }
        self.flow.wait = 3; // the first pass of mode 0 is 2 frames later (`flow_mode_start`)
    }

    /// Game mode 4 (0xAD46) character select. TODO(M2-03): screen, cursors, timer (0xAF7C, 0xB06A,
    /// 0xB122, 0xB25E). For now the players that pressed start get the heroes in order (Jack,
    /// Mustapha, Hannah) and the stage starts at once.
    pub fn flow_select_mode(&mut self) {
        for i in 0..3 {
            if self.flow.started >> i & 1 != 0 {
                self.player_join(i, i as u8);
            }
        }
        self.flow.chosen = self.flow.started;
        self.flow_stage_start();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        g.flow.front.sys = Sys { coins_per_credit: 1, credits_per_coin: 1, ..Default::default() };
        g
    }

    /// Feeds the coin input of slot 0 (`pattern`: '1' high, '0' low, one char per frame) and returns the
    /// frame (0-based) of each accepted coin.
    fn coins(g: &mut Game, pattern: &str) -> Vec<usize> {
        let mut at = Vec::new();
        for (k, c) in pattern.chars().enumerate() {
            let before = g.flow.front.sys.pulse;
            g.w.sounds.clear();
            g.w.coins = (c == '1') as u8;
            g.flow_coin_isr();
            if g.flow.front.sys.pulse != before {
                at.push(k);
                assert_eq!(g.w.sounds, [0xFF00, 0x30], "the coin sound after stopping the sound");
            }
        }
        at
    }

    /// 0x9D50A: two frames high and then two frames low make a coin, at the second low frame (the
    /// recording: held at 449 to 456, accepted at 458, the second frame after the release). A tap of one
    /// frame, or a coin held longer than the 50 frames of the timer, is not counted.
    #[test]
    fn a_coin_is_two_frames_high_then_two_low() {
        assert_eq!(coins(&mut game(), "0011000"), [5]);
        assert_eq!(coins(&mut game(), "00111111000"), [9]);
        assert!(coins(&mut game(), "001000000").is_empty(), "one frame is a tap");
        assert!(coins(&mut game(), "0010100000").is_empty(), "a gap of one frame is not a release");
        let long = format!("11{}000", "1".repeat(60));
        assert!(coins(&mut game(), &long).is_empty(), "the timer of 0x32 frames ran out");
        assert_eq!(coins(&mut game(), "001100001100"), [5, 11], "two coins");
        let mut g = game();
        coins(&mut g, "0011000");
        assert_eq!(g.flow.front.sys.slots[0].coins, 1);
        assert_eq!(g.flow.front.sys.slots[1].coins, 0, "only the slot of the input");
    }

    /// 0x9D3DA / 0x9D3A6: `coins_per_credit` coins make `credits_per_coin` credits, at most 9, in the
    /// player's slot or, with one chute, in the common pool.
    #[test]
    fn coins_become_credits() {
        let mut g = game();
        g.flow.front.sys = Sys { coins_per_credit: 2, credits_per_coin: 1, ..Default::default() };
        g.flow.front.sys.slots[0].coins = 1;
        g.flow_sys_convert();
        assert_eq!((g.flow.front.sys.slots[0].coins, g.flow.front.sys.slots[0].credits), (1, 0), "2 coins per credit");
        g.flow.front.sys.slots[0].coins = 2;
        g.flow_sys_convert();
        assert_eq!((g.flow.front.sys.slots[0].coins, g.flow.front.sys.slots[0].credits), (0, 1));
        g.flow.front.sys = Sys { coins_per_credit: 1, credits_per_coin: 6, ..Default::default() };
        g.flow.front.sys.slots[2].coins = 2;
        g.flow_sys_convert();
        g.flow_sys_convert();
        assert_eq!(g.flow.front.sys.slots[2].credits, 9, "6, then 12 capped at 9");
        g.flow.front.sys = Sys { coins_per_credit: 1, credits_per_coin: 1, common: true, ..Default::default() };
        coins(&mut g, "0011000");
        g.flow_sys_convert();
        assert_eq!((g.flow.front.sys.credits, g.flow.front.sys.slots[0].credits), (1, 0), "one chute: the pool");
    }

    /// 0x9D196 / 0x9D2A2: a start with a credit takes it and marks the player; without a credit
    /// nothing happens; "2 coins start" needs 2 credits; free play needs none and keeps none.
    #[test]
    fn a_start_takes_a_credit() {
        let mut g = game();
        g.w.system = 8 | 1; // start 1 pressed (bit 3 is always set)
        g.flow.front.prev_system = 8;
        g.flow.started = 0;
        g.flow_sys_start_detect();
        assert_eq!(g.flow.started, 0, "no credit");
        g.flow.front.sys.slots[0].credits = 1;
        g.flow.front.sys.slots[1].credits = 3;
        g.flow_sys_start_detect();
        assert_eq!((g.flow.started, g.flow.front.sys.slots[0].credits), (1, 0), "player 1 starts, player 2 did not press");
        g.flow.started = 0;
        g.flow.front.prev_system = 9; // the button is still down: no edge
        g.flow.front.sys.slots[0].credits = 1;
        g.flow_sys_start_detect();
        assert_eq!(g.flow.started, 0);
        g.flow.front.prev_system = 8;
        g.flow.front.sys.two_coin_start = true;
        g.flow_sys_start_detect();
        assert_eq!((g.flow.started, g.flow.front.sys.slots[0].credits), (0, 1), "2 credits are needed");
        g.flow.front.sys.slots[0].credits = 2;
        g.flow_sys_start_detect();
        assert_eq!((g.flow.started, g.flow.front.sys.slots[0].credits), (1, 0));
        // free play: everybody can start, no credit is left
        let mut g = game();
        g.flow.front.sys.free_play = true;
        g.flow.front.prev_system = 8;
        g.w.system = 8 | 2 | 4;
        g.flow_sys_start_detect();
        assert_eq!(g.flow.started, 6);
        assert!(g.flow.front.sys.slots.iter().all(|c| c.credits == 0));
        // the common pool, two players on one chute: player 2 takes 2 credits and marks bits 1 and 2 (G.$4CD = 3)
        let mut g = game();
        g.flow.front.sys = Sys { common: true, two_player: true, credits: 2, ..Default::default() };
        g.flow.front.prev_system = 8;
        g.w.system = 8 | 2;
        g.flow_sys_start_detect();
        assert_eq!((g.flow.started, g.flow.front.sys.credits), (3, 0));
    }
}
