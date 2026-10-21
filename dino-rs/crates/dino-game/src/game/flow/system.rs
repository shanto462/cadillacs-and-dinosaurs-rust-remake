//! The system task (task slot 0, routine 0x9D048 to 0x9D8EE) and the coin interrupt code 0x9D4BE: coins, credits, the
//! coin and chute modes, free play, SERVICE, the credit text and the prompts of the press start screen, the coin
//! lockout and coin counter outputs, and the start of a game. Owner: flow / HUD port. Spec: `D4d-front-end-and-story.md`
//! s3.10 and s4 (M2-02; the minimal task of M1-04 lived in `front.rs`).
//!
//! One pass of the task per frame (0x9D04C): SERVICE and the conversion of coins into credits (0x9D326, 0x9D386), the
//! coin lockout outputs (0x9D40E), the coin counter pulse (0x9D478), the attract part (0x9D074: the press start
//! screen as soon as a coin or credit is there, the credit text and the prompts, the start detection 0x9D196), the wait
//! counter for the rank (0x1827A, `rank.rs`), sleep 1. The coin input itself is read by the interrupt (0x9D4BE: the
//! debounce) before the tasks run.
//!
//! Modes (the cooked settings G.$777A / G.$777C, `config.rs`, set up by 0x9CFC6):
//! * coin mode 0 to 7: coins per credit G.$470 and credits per coin G.$471 from the table 0x9D038 (1/1, 1/2, 1/3, 1/4,
//!   1/6, 2/1, 3/1, 4/1: data, read from the ROM); coin mode 8 is "2 coins start, 1 coin continue" (G.$46C, one credit per coin,
//!   a start takes 2 credits);
//! * chute mode 0: three players, three chutes, credits per player (G.$47D, G.$48D, G.$49D); 1: one chute, a common pool of
//!   credits (G.$46A, G.$472); 2: two players, one chute (the pool and G.$474, and the 2 player layout flag G.$4CF, which this
//!   task sets as 0x9CFF6 does; the HUD, select and demo layouts that depend on it were never measured and are not part of
//!   this task: the 3 player layout stays the one that is tested);
//! * free play (G.$46B): no press start screen, every player may start at once, the credit text is never drawn;
//! * the credit cap is 9.
//!
//! Not ported on purpose: the TEST button (0x9D5A4: it leaves for the test menu, no key is mapped to it) and the play time
//! statistics 0xA6B8A (G.$75C2 .. G.$75C6, they are only read by the test menu). The outputs G.$21 (coin lockout and counter
//! bits) are kept as state, nothing in the port reads them.

use crate::game::Game;
use crate::game::flow::scene::Scene;

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

/// The system task's variables G.$46A..G.$477 (cleared and set up by 0x9CFC6) and the coin outputs G.$21.
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
    /// G.$46E coins accepted that the coin counter has not pulsed yet.
    pub pulse: u8,
    /// G.$46F the coin counter pulse timer (0x3F, counts down; bit 0 of G.$21 is set at 0x3F and cleared at 0x1F).
    pub pulse_timer: u8,
    /// G.$470 coins per credit batch, G.$471 credits per batch.
    pub coins_per_credit: u8,
    pub credits_per_coin: u8,
    /// G.$472 one chute for all players (common pool), G.$474 two players on one chute.
    pub common: bool,
    pub two_player: bool,
    /// G.$473 the credit text must be drawn again (set by a coin and by SERVICE, cleared by the draw).
    pub redraw: bool,
    /// G.$475 / G.$476 coins of the common pool and the count before.
    pub common_coins: u8,
    pub common_prev: u8,
    /// G.$477 prompt hold timer (2 after a coin; the join task of a game counts it down, 0x6D52).
    pub hold: u8,
    /// G.$47A, G.$48A, G.$49A the coin slots of players 1 to 3.
    pub slots: [CoinSlot; 3],
    /// G.$21 the output bits of the coin hardware: 4, 8 and 0x20 let a player's chute take coins (0x2C for the common pool),
    /// bit 0 is the coin counter (0x9D40E, 0x9D478). The vertical blank sends them to the ports (0x7EC); the port has none.
    pub outputs: u8,
}

/// Script positions of the system task.
mod sst {
    /// One pass of the loop 0x9D04C..0x9D072 per frame.
    pub const LOOP: u16 = 0;
    /// The fade out of 0x9D16E is over: clear all layers.
    pub const CLEAR: u16 = 1;
    /// After the clear: the game task is created.
    pub const CREATE: u16 = 2;
    /// The rest of the pass that created the press start screen, which took the system task 3 frames (0x983EE).
    pub const TAIL: u16 = 3;
    /// The first pass after the game task was created (0x9D18E). The first run of the game task (0xA870: the clears of its RAM,
    /// the three player inits) takes the CPU for about 3 frames, in which the scan of the task slots does not come round to slot 0:
    /// the pass of the next frame runs, the two after it are lost (the coin counter timer G.$46F of `flow_c-three-start`, creation
    /// in 617: it runs in 618, stands still in 619 and 620 and runs again in 621).
    pub const AFTER: u16 = 4;
}

/// The cells of scroll3 the clear 0xAE0 of the game start has done in the frame it clears scroll2 in (the recording
/// `flow_m113-freestart`, a start during the demo: cells 3077 to 3586 of its scroll3 go in that frame, 3587 to 3790 in the next).
const CLEAR3_FIRST_START: usize = 3587;

/// ROM addresses (the immediates of the code; all inside the blobs `FRONT_TABLES` lists) of the credit text and prompts.
mod rom {
    /// 0x9D038 (coins, credits) pairs by coin mode, read by 0x9CFC6.
    pub const COIN_RATIOS: u32 = 0x9D038;
    /// 0x9D666: the rectangle 0xF94 clears before the text is drawn (column, row, width, height).
    pub const CLEAR_RECT: u32 = 0x9D666;
    /// 0x9D984, 0x9D988, 0x9D98C: the scroll1 address (long) of the credit line of player 1, 2 and 3 (the middle one for the pool).
    pub const LINE_ADDR: u32 = 0x9D984;
    /// 0x9D990: "   CREDIT  0  " (0xC0A: attribute byte, text); the 0 is replaced by the credit count.
    pub const CREDIT: u32 = 0x9D990;
    /// 0x9D9A0: "CREDIT 0 (0/0)" (0xC9A: attribute word, words): credits, coins waiting and coins per credit replace the 0s.
    pub const CREDIT_COINS: u32 = 0x9D9A0;
    /// 0x9D9C0: "    COIN   0  " the credit count replaces the 0 (coin mode 8).
    pub const COIN: u32 = 0x9D9C0;
    /// 0x9D9F8: PUSH START BUTTON in the big font (0xCE4: column, row, attribute word, glyph words).
    pub const BIG_PUSH_START: u32 = 0x9D9F8;
    /// 0x9DA20: INSERT ADDITIONAL COIN in the big font (a coin is in, a credit is not yet there).
    pub const BIG_INSERT_MORE_COIN: u32 = 0x9DA20;
    /// The line under PUSH START BUTTON that says how many players can start: (text, number mark) for 1, 2 and 3 players.
    pub const ONE_PLAYER: (u32, u32) = (0x9DA52, 0x9DA76);
    pub const TWO_PLAYERS: (u32, u32) = (0x9DA8A, 0x9DAB2);
    pub const THREE_PLAYERS: (u32, u32) = (0x9DAC6, 0x9DAF2);
    /// 0x9DB0A: "2 COIN START" and "1 COIN CONTINUE" in the big font (coin mode 8 without credit).
    pub const BIG_TWO_COIN_START: u32 = 0x9DB0A;
    /// 0x9DB48: "AND" / "INSERT ADDITIONAL COIN(S)" / "FOR ADDITIONAL PLAYER(S)" (0xBB6: byte string with positions).
    pub const AND_INSERT_MORE: u32 = 0x9DB48;
    /// 0x9DBA0: "<n> COIN <n> CREDIT" in the big font, the first digit is replaced (0xCD2 at 0x90A5D4) by the glyphs of
    /// the table 0x9DBC0 (6 bytes per coin count: attribute word, glyph word, end).
    pub const BIG_COIN_CREDIT: u32 = 0x9DBA0;
    pub const BIG_DIGITS: u32 = 0x9DBC0;
    /// The tall font texts of the separate chute prompts (0xE68: attribute word, '//' = next position, glyph words).
    pub const TALL_INSERT_COIN: u32 = 0x9DC20;
    pub const TALL_INSERT_MORE_COIN: u32 = 0x9DC44;
    pub const TALL_PUSH_START: u32 = 0x9DC86;
    /// 0x9DCC8: "<n>COIN 1CREDIT" (0xC0A) under the tall prompt of a player with coins waiting; 0x9DCD7: " 2COIN START  /1COIN CONTINUE".
    pub const SMALL_COIN_CREDIT: u32 = 0x9DCC8;
    pub const SMALL_COIN_START: u32 = 0x9DCD7;
}

/// 0x9D3F8 / 0x9D3C4: add `n` credits, at most 9 (`add.b`, `cmpi.b #$a`, `bcs`).
fn credit_add(credits: u8, n: u8) -> u8 {
    let v = credits.wrapping_add(n);
    if v >= 10 { 9 } else { v }
}

impl Game {
    /// 0x9CFC6, called by the boot task: clears G.$46A..G.$4A9 and sets the coin and chute modes
    /// from the settings (G.$777A and G.$777C) and the coins per credit from the table 0x9D038.
    pub fn flow_sys_init(&mut self) {
        let (a, c) = (self.flow.dip.dip_a, self.flow.dip.dip_c);
        // 0x9CFC6 clears G.$46A..G.$4A9; the outputs G.$21 are not in that range
        let mut s = Sys { free_play: c & 4 != 0, common: a & 8 != 0, outputs: self.flow.front.sys.outputs, ..Default::default() };
        if a & 0x10 != 0 {
            // 0x9CFF6: two players on one chute: the 2 player layout flag G.$4CF, G.$474 and a common pool
            self.flow.b4cf = 1;
            s.two_player = true;
            s.common = true;
        }
        if a & 0x40 != 0 {
            // 0x9D014: coin mode 8, 2 coins start and 1 coin continue; the word 0x101 into G.$470
            s.two_coin_start = true;
            s.coins_per_credit = 1;
            s.credits_per_coin = 1;
        } else {
            // 0x9D022: the pair of the coin mode, one word of the table 0x9D038 into G.$470
            let t = &self.assets.frontend.text;
            let i = 2 * (a & 7) as u32;
            s.coins_per_credit = t.u8(rom::COIN_RATIOS, i);
            s.credits_per_coin = t.u8(rom::COIN_RATIOS, i + 1);
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
            self.sound_ex(0x30, 0x70010, 0x3800000); // the coin sound at 0x9D348 / 0x9D598 (checked)
            self.flow.front.sys.slots[i].timer = 0;
            return;
        }
        self.flow.front.sys.slots[i] = slot;
    }

    /// 0x9D326: SERVICE (G.$1A bit 7, a press since the last frame) queues the coin sound (without stopping the sound before
    /// it) and adds one credit to every player, or one to the common pool, at any time, in a game too; the credit text is
    /// redrawn. Then falls into the conversion 0x9D386, which every pass runs.
    fn flow_sys_service(&mut self) {
        let pressed = self.w.system & (self.w.system ^ self.flow.front.prev_system) & 0x80; // G.$1A, G.$1F
        if pressed != 0 {
            self.sound_ex(0x30, 0x70010, 0x3800000); // 0x9D348 / 0x9D598 (checked)
            let s = &mut self.flow.front.sys;
            if s.common {
                s.credits = credit_add(s.credits, 1); // 0x9D3C4
            } else {
                for slot in s.slots.iter_mut() {
                    slot.credits = credit_add(slot.credits, 1); // 0x9D3F8
                }
            }
            s.redraw = true;
        }
        self.flow_sys_convert();
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
            s.credits = credit_add(s.credits, add);
            return;
        }
        for slot in s.slots.iter_mut() {
            // 0x9D3DA
            if slot.coins < cpc {
                continue;
            }
            slot.prev = slot.coins;
            slot.coins -= cpc;
            slot.credits = credit_add(slot.credits, add);
        }
    }

    /// 0x9D40E: the coin lockout. Each chute is enabled (its bit of G.$21 set: 4, 8, 0x20; 0x2C for the one chute modes)
    /// while the credits plus one batch of `credits_per_coin` stay below 10, and locked out (bit cleared) when the next coin
    /// would pass the cap of 9, or always in free play.
    fn flow_sys_lockout(&mut self) {
        let s = &mut self.flow.front.sys;
        let room = |credits: u8| credits.wrapping_add(s.credits_per_coin) < 10; // 0x9D45A
        let (free, common, pool, own) = (s.free_play, s.common, room(s.credits), [0, 1, 2].map(|i| room(s.slots[i].credits)));
        let mut set = |bit: u8, open: bool| {
            if open {
                s.outputs |= bit; // 0x9D46A
            } else {
                s.outputs &= !bit; // 0x9D470
            }
        };
        if free {
            set(0x2C, false); // 0x9D412
        } else if common {
            set(0x2C, pool); // 0x9D422
        } else {
            set(4, own[0]); // 0x9D43A, 0x9D446, 0x9D452
            set(8, own[1]);
            set(0x20, own[2]);
        }
    }

    /// 0x9D478: the coin counter pulse. Every accepted coin (G.$46E) starts a pulse of G.$46F = 0x3F frames: bit 0 of G.$21
    /// is set one pass after it starts and cleared when the timer reaches 0x1F, so the counter is on for 0x20 frames.
    fn flow_sys_pulse(&mut self) {
        let s = &mut self.flow.front.sys;
        let d0 = s.pulse_timer;
        if d0 != 0 {
            s.pulse_timer -= 1;
            if d0 == 0x3F {
                s.outputs |= 1; // 0x9D4AC
            } else if d0 == 0x1F {
                s.outputs &= !1; // 0x9D4B4
            }
            return;
        }
        if s.pulse != 0 {
            s.pulse -= 1;
            s.pulse_timer = 0x3F;
        }
    }

    /// The system task 0x9D048: one pass per frame (SERVICE and conversion, the lockout, the counter pulse, the attract part
    /// 0x9D074, the wait counter 0x1827A, sleep 1), or the start of a game (the task sleeps through the fade out).
    pub fn flow_system_task(&mut self) {
        // a clear of all layers that the original needs more than a frame for (0xAE0, here by the game start) goes on at the start
        // of the frame; the system task is the first task of the frame
        if self.flow.in_game != 0 || self.flow.front.press_call != 0 {
            self.flow_attract_clear_step(); // (with nobody playing, the attract task does this)
        }
        if self.flow.front.press_call != 0 {
            // 0x983EE goes on in the second and third frame of its call (see `flow_press_start_screen`)
            self.flow.front.press_call -= 1;
            if self.flow.front.press_call == 0 {
                self.flow_press_start_rest();
            }
        }
        if !self.flow.front.sys_task.tick() {
            return;
        }
        match self.flow.front.sys_task.state {
            sst::LOOP => self.flow_sys_pass(),
            sst::AFTER => {
                // the first pass after the game task was created: it runs, the next two are lost (see `sst::AFTER`)
                self.flow_sys_pass();
                self.flow.front.sys_skip = 2;
            }
            sst::CLEAR => {
                // 0x9D174: sleep 1 after the fade, then 0xAE0 (1.3 frames: the sprite buffers and scroll1 now, scroll2 in the next
                // frame, scroll3 in the one after, see `flow_attract_clear`; the step runs at the start of each frame below)
                self.flow_attract_clear();
                self.flow.front.clear3_cut = CLEAR3_FIRST_START; // measured: the clear starts later in the frame here
                self.flow.front.sys_task.wait(2, sst::CREATE);
            }
            sst::TAIL => {
                // the rest of the pass: the start detection and the end of the pass
                if self.flow_sys_attract_tail() {
                    return;
                }
                self.flow_sys_pass_end();
            }
            _ => {
                // 0x9D180: the press start flag is cleared, the game task 0xA870 is created in slot
                // 0x20. The pass then ends like any other (0x9D05C, sleep 1).
                self.flow.front.sys.press_up = 0;
                self.flow.game_pending = false;
                self.flow_game_start();
                self.flow_sys_pass_end_to(sst::AFTER);
            }
        }
    }

    /// One pass of the loop 0x9D04C: SERVICE with the conversion, the lockout, the coin counter pulse, the attract part.
    fn flow_sys_pass(&mut self) {
        // The attract task is busy (a clear of all layers that the port writes as a sleep):
        // the scan of the task slots does not come round to slot 0 in these frames.
        if self.flow.front.sys_skip > 0 {
            self.flow.front.sys_skip = self.flow.front.sys_skip.saturating_sub(1);
            self.flow.front.sys_task.sleep(1);
            return;
        }
        self.flow_sys_service(); // 0x9D04C: 0x9D326 with 0x9D386
        self.flow_sys_lockout(); // 0x9D050
        self.flow_sys_pulse(); // 0x9D054
        if self.flow_sys_attract() {
            return; // 0x9D058: a game starts, the task sleeps through the fade out
        }
        self.flow_sys_pass_end();
    }

    /// 0x9D05C and the sleep of the loop: the wait counter G.$4F6 (0x1827A), the TEST button
    /// (0x9D5A4, not mapped) and the statistics (0xA6B8A, not ported), sleep 1.
    fn flow_sys_pass_end(&mut self) {
        self.flow_sys_pass_end_to(sst::LOOP);
    }

    fn flow_sys_pass_end_to(&mut self, next: u16) {
        let s = self.flow.front.sys;
        let pool = ((s.credits as u16) << 8) | s.free_play as u16; // tst.w $46a(a5)
        self.rank_waited_tick(self.flow.in_game != 0, pool);
        self.flow.front.sys_task.wait(1, next);
    }

    /// 0x9D074: while nobody plays: the press start screen as soon as a coin or credit is there,
    /// and the detection of a start. True when a game starts.
    fn flow_sys_attract(&mut self) -> bool {
        if self.flow.in_game != 0 {
            return false;
        }
        let s = self.flow.front.sys;
        if !s.free_play {
            // 0x9D084: is a credit or a coin waiting? (the one chute modes look at the pool, the others at the sum of the
            // three slots' credits and then of their coins, as bytes)
            let waiting = if s.common {
                s.credits != 0 || s.common_coins != 0
            } else {
                let sum = |f: fn(&CoinSlot) -> u8| s.slots.iter().fold(0u8, |a, c| a.wrapping_add(f(c)));
                sum(|c| c.credits) != 0 || sum(|c| c.coins) != 0
            };
            if !waiting {
                return false;
            }
            if s.press_up == 0 {
                // 0x9D0C8: the attract tasks (slots 0x10 to 0x40) end, the press start screen
                self.flow_kill_front_tasks();
                self.flow.front.sys.press_up = 0xFF;
                self.flow_press_start_screen(); // 0x983EE
                self.flow.front.sys.redraw = true; // 0x9D0FC
                // the call takes the system task 3 frames of CPU time (the recording of a coin during the demo: G.$4CD, which
                // the demo set, is cleared by the next lines 3 frames after the call): the rest of the pass runs then
                self.flow.front.sys_task.wait(3, sst::TAIL);
                return true;
            }
        }
        self.flow_sys_attract_tail()
    }

    /// 0x9D102 to the end of 0x9D074: the credit text and the prompts, then the start detection. True when a game starts.
    fn flow_sys_attract_tail(&mut self) -> bool {
        if self.flow.in_game != 0 {
            return false;
        }
        if !self.flow.front.sys.free_play && self.flow.front.sys.redraw {
            // 0x9D102: clear the text rectangle (0xF94), then the credit lines (0x9D66A) and the prompts (0x9D752)
            self.flow.front.sys.redraw = false;
            let t = &self.assets.frontend.text;
            let rect = [0, 1, 2, 3].map(|i| t.u8(rom::CLEAR_RECT, i));
            self.hud_clear_rect(rect);
            self.flow_sys_credit_text();
            self.flow_sys_prompts();
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
        self.flow.start_tick = 1;
        self.flow.game_pending = true; // the game task does not exist for the next 18 frames
        self.sound_ex(0x31, 0x10, 0); // 0x9D168 (checked)
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
        self.flow.tally.live = false;
        self.fade.task = None;
    }

    // ------------------------------------------------------------------------------------------------------ start

    /// 0x9D196: the start of each player: the start edge (the start button changed) with enough
    /// credits takes them and marks the player in G.$4CD (`flow.started`). In free play every
    /// player has 9 credits for the check and none after it. Quirk kept: the edge is any change of the button (G.$1A
    /// eor G.$1F), so the release of a start button counts like the press.
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

    // ------------------------------------------------------------------------------------------------------ text

    /// The string at the ROM address `a` of the front end tables.
    fn flow_sys_rom(&self, a: u32) -> &'static [u8] {
        self.assets.frontend.text.slice_from(a)
    }

    /// 0x9D66A: the credit lines of the press start screen: one per player (columns at the bottom of the screen), or one in the
    /// middle for the common pool. Nothing in a game.
    fn flow_sys_credit_text(&mut self) {
        if self.flow.in_game != 0 {
            return;
        }
        let s = self.flow.front.sys;
        let t = &self.assets.frontend.text;
        let addr = [0, 1, 2].map(|i| (0..4).fold(0u32, |a, k| (a << 8) | t.u8(rom::LINE_ADDR, 4 * i + k) as u32));
        if !s.common {
            for (i, slot) in s.slots.iter().enumerate() {
                self.flow_sys_credit_line(addr[i], slot.credits, slot.coins); // 0x9D67C, 0x9D690, 0x9D6A4
            }
        } else {
            self.flow_sys_credit_line(addr[1], s.credits, s.common_coins); // 0x9D6B8
        }
    }

    /// 0x9D6C8: one credit line at the scroll1 address `a1` for `credits` credits and `coins` coins waiting (shown as 0 at the
    /// cap of 9 credits). Free play shows none; "2 coins start" shows `COIN n`; one coin per credit shows `CREDIT n`; more coins per
    /// credit with one credit per coin show `CREDIT n (coins/coins per credit)`; other ratios show `CREDIT n`.
    fn flow_sys_credit_line(&mut self, a1: u32, credits: u8, coins: u8) {
        let coins = if credits == 9 { 0 } else { coins };
        let s = self.flow.front.sys;
        if s.free_play {
            return; // 0x9D746
        }
        if s.two_coin_start {
            // 0x9D732
            self.hud_print(self.flow_sys_rom(rom::COIN), a1);
            self.hud_put(a1 + 0x580, credits as u16, 0);
        } else if s.coins_per_credit != 1 && s.credits_per_coin == 1 {
            // 0x9D70A (the test order of 0x9D6E2: one coin per credit first, then one credit per coin)
            self.text_words(self.flow_sys_rom(rom::CREDIT_COINS), a1);
            self.hud_put(a1 + 0x380, credits as u16, 0);
            self.hud_put(a1 + 0x500, coins as u16, 0);
            self.hud_put(a1 + 0x600, s.coins_per_credit as u16, 0);
        } else {
            // 0x9D6F6
            self.hud_print(self.flow_sys_rom(rom::CREDIT), a1);
            self.hud_put(a1 + 0x580, credits as u16, 0);
        }
    }

    /// 0x9D752: the prompts: the big font PUSH START BUTTON / INSERT COIN lines in the middle for the common pool, the tall
    /// font prompt of each player for separate chutes (the credit lines are drawn again after them, 0x9D8EA). Nothing in a game
    /// or in free play (the blink task prints FREE PLAY).
    fn flow_sys_prompts(&mut self) {
        if self.flow.in_game != 0 {
            return;
        }
        if !self.flow.front.sys.common {
            // 0x9D8AE: separate chutes, the tall prompt at 0x90A050 / 0x90A850 / 0x90B0D0 and the small line at +0x14
            for (i, (a1, a2)) in [(0x90A050u32, 0x90A064u32), (0x90A850, 0x90A864), (0x90B0D0, 0x90B0E4)].into_iter().enumerate() {
                self.flow_sys_prompt_player(i, a1, a2);
            }
            self.flow_sys_credit_text(); // 0x9D8EA
            return;
        }
        if self.flow.front.sys.free_play {
            return; // 0x9D76A
        }
        if self.flow.front.sys.two_coin_start {
            self.flow_sys_prompts_pool_two_coin(); // 0x9D81A
        } else {
            self.flow_sys_prompts_pool(); // 0x9D776
        }
    }

    /// 0x9D8EE: the prompt of one player with separate chutes: PUSH START BUTTON with a credit, INSERT COIN without; with coins
    /// waiting but no credit INSERT ADDITIONAL COIN and the ratio line (`n COIN 1 CREDIT`, or the "2 COIN START" lines).
    fn flow_sys_prompt_player(&mut self, i: usize, a1: u32, a2: u32) {
        let s = self.flow.front.sys;
        let slot = s.slots[i];
        if s.free_play {
            return; // 0x9D96E
        }
        if s.two_coin_start {
            // 0x9D93C: a start needs 2 credits
            if slot.credits == 0 {
                self.text_tall_a1(self.flow_sys_rom(rom::TALL_INSERT_COIN), a1); // 0x9D97A
            } else if slot.credits >= 2 {
                self.text_tall_a1(self.flow_sys_rom(rom::TALL_PUSH_START), a1); // 0x9D94E
            } else {
                self.text_tall_a1(self.flow_sys_rom(rom::TALL_INSERT_MORE_COIN), a1); // 0x9D958
                self.hud_print(self.flow_sys_rom(rom::SMALL_COIN_START), a2);
            }
        } else if slot.credits != 0 {
            self.text_tall_a1(self.flow_sys_rom(rom::TALL_PUSH_START), a1); // 0x9D906
        } else if slot.coins != 0 {
            // 0x9D918: coins are waiting for the next credit
            self.text_tall_a1(self.flow_sys_rom(rom::TALL_INSERT_MORE_COIN), a1);
            self.hud_print(self.flow_sys_rom(rom::SMALL_COIN_CREDIT), a2);
            self.hud_put(a2, s.coins_per_credit as u16, 0); // 0x9D930: the 0 of the string
        } else {
            self.text_tall_a1(self.flow_sys_rom(rom::TALL_INSERT_COIN), a1); // 0x9D97A
        }
    }

    /// The big font text for the number of credits of the pool (0x9D7DA to 0x9D80A): the text and the number mark, 1 credit
    /// = one player, 2 = up to two, more = up to three (up to two on the two player cabinet). `credits` is the pool.
    fn flow_sys_players_line(&mut self, credits: u8) {
        let (text, mark) = if credits == 1 {
            rom::ONE_PLAYER
        } else if credits == 2 || self.flow.front.sys.two_player {
            rom::TWO_PLAYERS
        } else {
            rom::THREE_PLAYERS
        };
        self.text_big(self.flow_sys_rom(text), None, true); // 0xCE4
        self.text_words_at(self.flow_sys_rom(mark), true); // 0xC4C
    }

    /// 0x9D776: the pool prompts without "2 coins start": no credit yet (a coin is waiting) shows INSERT ADDITIONAL COIN (and, when a
    /// credit takes more than one coin and one coin gives one credit, the ratio line "n COIN 1 CREDIT" with the digit glyph of the
    /// coin count), a credit shows PUSH START BUTTON and how many players can start.
    fn flow_sys_prompts_pool(&mut self) {
        let s = self.flow.front.sys;
        if s.credits == 0 {
            self.text_big(self.flow_sys_rom(rom::BIG_INSERT_MORE_COIN), None, true); // 0x9D786
            if s.credits_per_coin != 1 || s.coins_per_credit == 1 {
                return; // 0x9D790, 0x9D79A
            }
            self.text_big(self.flow_sys_rom(rom::BIG_COIN_CREDIT), None, true); // 0x9D7A4
            // 0x9D7AE: 6 bytes per coin count: the glyph of the digit replaces the placeholder at 0x90A5D4
            let digit = rom::BIG_DIGITS + 6 * s.coins_per_credit as u32;
            self.text_big(self.flow_sys_rom(digit), Some(0x90A5D4), true); // 0xCD2
        } else {
            self.text_big(self.flow_sys_rom(rom::BIG_PUSH_START), None, true); // 0x9D7D0
            self.flow_sys_players_line(s.credits);
        }
    }

    /// 0x9D81A: the pool prompts of "2 coins start, 1 coin continue": a start takes 2 credits, so the number of players is
    /// the credits halved; the extra lines tell how to add players.
    fn flow_sys_prompts_pool_two_coin(&mut self) {
        let credits = self.flow.front.sys.credits;
        let players = credits >> 1;
        if players == 0 {
            self.text_big(self.flow_sys_rom(rom::BIG_INSERT_MORE_COIN), None, true); // 0x9D824
            self.text_big(self.flow_sys_rom(rom::BIG_TWO_COIN_START), None, true); // 0x9D82E
            return;
        }
        self.text_big(self.flow_sys_rom(rom::BIG_PUSH_START), None, true); // 0x9D838
        self.flow_sys_players_line(players);
        // 0x9D884: the lines "AND INSERT ADDITIONAL COIN(S) FOR ADDITIONAL PLAYER(S)" while more players could be added (up to 3 players,
        // 2 on the two player cabinet) and a credit is left over (odd)
        let limit = if self.flow.front.sys.two_player { 4 } else { 6 };
        if limit < credits || credits & 1 == 0 {
            return;
        }
        self.text_bytes(self.flow_sys_rom(rom::AND_INSERT_MORE), true); // 0x9D8A2: 0xBB6
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::flow::Config;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    /// The asset pack of the user's ROM (`../../../dino.zip`), `None` when it is not there (the tests that need it skip).
    fn rom() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

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

    /// The start edge is any change of the start button (0x9D2A2 uses the exclusive or of G.$1A and G.$1F): releasing
    /// the button takes a credit like pressing it did.
    #[test]
    fn the_release_of_a_start_button_counts_too() {
        let mut g = game();
        g.flow.front.sys.slots[0].credits = 2;
        g.flow.front.prev_system = 8 | 1; // the button was down
        g.w.system = 8; // and is up now
        g.flow_sys_start_detect();
        assert_eq!((g.flow.started, g.flow.front.sys.slots[0].credits), (1, 1));
    }

    /// 0x9D326: SERVICE adds a credit to every player (the pool with one chute), at most 9, queues the coin sound
    /// without stopping the sound, redraws the credit text, and only the press counts, not holding or releasing.
    #[test]
    fn service_adds_a_credit_to_every_player() {
        let mut g = game();
        g.flow.front.prev_system = 8;
        g.w.system = 8 | 0x80;
        g.flow_sys_service();
        let c = |g: &Game| g.flow.front.sys.slots.map(|s| s.credits);
        assert_eq!(c(&g), [1, 1, 1]);
        assert_eq!(g.w.sounds, [0x30]);
        assert!(g.flow.front.sys.redraw);
        g.flow.front.sys.redraw = false;
        g.flow.front.prev_system = 8 | 0x80; // held
        g.flow_sys_service();
        assert_eq!(c(&g), [1, 1, 1], "holding gives nothing");
        g.w.system = 8; // released
        g.flow_sys_service();
        assert_eq!((c(&g), g.flow.front.sys.redraw), ([1, 1, 1], false), "releasing gives nothing");
        // the cap of 9
        g.flow.front.sys.slots[1].credits = 9;
        g.flow.front.prev_system = 8;
        g.w.system = 8 | 0x80;
        g.flow_sys_service();
        assert_eq!(c(&g), [2, 9, 2]);
        // one chute: the pool gets the credit, the players' own credits do not change
        let mut g = game();
        g.flow.front.sys.common = true;
        g.flow.front.prev_system = 8;
        g.w.system = 8 | 0x80;
        g.flow_sys_service();
        assert_eq!((g.flow.front.sys.credits, g.flow.front.sys.slots.map(|s| s.credits)), (1, [0, 0, 0]));
        // a SERVICE press in a game works too (the credit text is not drawn, the join prompts use the credits)
        let mut g = game();
        g.flow.in_game = 1;
        g.flow.front.prev_system = 8;
        g.w.system = 8 | 0x80;
        g.flow_sys_service();
        assert_eq!(g.flow.front.sys.slots.map(|s| s.credits), [1, 1, 1]);
    }

    /// 0x9D40E: a chute is enabled (its bit of G.$21 set) while one more batch of credits stays below 10, locked
    /// out at the cap, and in free play. The one chute modes use the bits 0x2C together.
    #[test]
    fn the_lockout_follows_the_credits() {
        let mut g = game();
        g.flow_sys_lockout();
        assert_eq!(g.flow.front.sys.outputs, 4 | 8 | 0x20);
        g.flow.front.sys.slots[1].credits = 9;
        g.flow_sys_lockout();
        assert_eq!(g.flow.front.sys.outputs, 4 | 0x20, "player 2 is at the cap");
        g.flow.front.sys.slots[1].credits = 8;
        g.flow.front.sys.credits_per_coin = 2; // 8 + 2 = 10
        g.flow_sys_lockout();
        assert_eq!(g.flow.front.sys.outputs, 4 | 0x20, "a batch of 2 credits would pass the cap");
        g.flow.front.sys.slots[1].credits = 7;
        g.flow_sys_lockout();
        assert_eq!(g.flow.front.sys.outputs, 4 | 8 | 0x20);
        g.flow.front.sys.free_play = true;
        g.flow_sys_lockout();
        assert_eq!(g.flow.front.sys.outputs, 0, "free play takes no coins");
        // the pool
        let mut g = game();
        g.flow.front.sys.common = true;
        g.flow_sys_lockout();
        assert_eq!(g.flow.front.sys.outputs, 0x2C);
        g.flow.front.sys.credits = 9;
        g.flow_sys_lockout();
        assert_eq!(g.flow.front.sys.outputs, 0);
        // the coin counter bit is not touched by the lockout
        g.flow.front.sys.outputs = 1;
        g.flow.front.sys.credits = 0;
        g.flow_sys_lockout();
        assert_eq!(g.flow.front.sys.outputs, 1 | 0x2C);
    }

    /// 0x9D478: an accepted coin pulses the coin counter, bit 0 of G.$21: on from the pass after the timer starts (0x3F)
    /// for 0x20 frames, off at 0x1F; a second coin pulses after the first has ended.
    #[test]
    fn the_coin_counter_pulses() {
        let mut g = game();
        g.flow.front.sys.pulse = 2;
        let mut on = Vec::new();
        for k in 0..0x90 {
            g.flow_sys_pulse();
            on.push(g.flow.front.sys.outputs & 1 != 0);
            let _ = k;
        }
        let runs: Vec<(usize, usize)> = {
            let mut v = Vec::new();
            let mut k = 0;
            while k < on.len() {
                if on[k] {
                    let s = k;
                    while k < on.len() && on[k] {
                        k += 1;
                    }
                    v.push((s, k - s));
                } else {
                    k += 1;
                }
            }
            v
        };
        // pass 0 starts the first timer, pass 1 sets the bit; 0x20 frames on, off at the pass of 0x1F; the next
        // pulse starts the pass after the timer ends
        assert_eq!(runs, [(1, 0x20), (0x41, 0x20)]);
        assert_eq!(g.flow.front.sys.pulse, 0);
    }

    /// 0x9CFC6: the eight coin modes read their (coins, credits) pair from the table 0x9D038 (D4d s4.2: 1 coin 1, 2, 3, 4 or 6
    /// credits, 2, 3 or 4 coins 1 credit), mode 8 is "2 coins start, 1 coin continue"; the chute modes set the pool, the two
    /// player flag and the layout flag G.$4CF; free play comes from the settings.
    #[test]
    fn every_coin_and_chute_mode_is_set_up_from_the_settings() {
        let Some(a) = rom() else { return };
        let init = |c: Config| {
            let mut g = Game::new(a);
            g.flow_apply_config(c);
            g.flow_sys_init();
            g
        };
        let ratios = [(1, 1), (1, 2), (1, 3), (1, 4), (1, 6), (2, 1), (3, 1), (4, 1)];
        for (mode, (coins, credits)) in ratios.into_iter().enumerate() {
            let g = init(Config { coin_mode: mode as u8, ..Config::default() });
            let s = g.flow.front.sys;
            assert_eq!((s.coins_per_credit, s.credits_per_coin, s.two_coin_start), (coins, credits, false), "coin mode {mode}");
        }
        let s = init(Config { coin_mode: 8, ..Config::default() }).flow.front.sys;
        assert_eq!((s.coins_per_credit, s.credits_per_coin, s.two_coin_start), (1, 1, true), "coin mode 8");
        let chute = |m: u8| {
            let g = init(Config { chute_mode: m, ..Config::default() });
            let s = g.flow.front.sys;
            (s.common, s.two_player, g.flow.b4cf)
        };
        assert_eq!(chute(0), (false, false, 0), "three players, three chutes");
        assert_eq!(chute(1), (true, false, 0), "three players, one chute");
        assert_eq!(chute(2), (true, true, 1), "two players, one chute: G.$4CF = 1, the layouts that follow it were not measured");
        assert!(init(Config { free_play: true, ..Config::default() }).flow.front.sys.free_play);
        assert!(!init(Config::default()).flow.front.sys.free_play);
    }

    /// The press start screen and its text for a game that has been running `ticks` frames from power on, with the coins and the
    /// SERVICE presses `input(tick)` gives: (the scroll1 cells, the credits of the three players, the sounds of the last tick).
    fn run_press_screen(a: &'static AssetPack, ticks: i32, input: impl Fn(i32) -> (u8, u8)) -> (Vec<(u16, u16)>, [u8; 3], Vec<u16>) {
        let mut g = Game::new(a);
        g.flow_boot();
        let mut sounds = Vec::new();
        for k in 0..ticks {
            let (coins, system) = input(k);
            g.w.coins = coins;
            g.w.system = 8 | system;
            g.w.sounds.clear();
            g.tick();
            sounds = g.w.sounds.clone();
        }
        let cells = g.vram.scroll1.iter().map(|c| (c.code, c.attr)).collect();
        (cells, g.flow.front.sys.slots.map(|s| s.credits), sounds)
    }

    /// SERVICE from power on: it queues the coin sound (in the frame of the press only), gives every player a credit and brings up the press start screen with the
    /// text of three coins (`flow_c-three-start`, whose picture equals MAME in `mame_check.rs`): the same scroll1, the same
    /// credits. A held SERVICE button counts once.
    #[test]
    fn service_brings_up_the_press_start_screen_like_three_coins() {
        let Some(a) = rom() else { return };
        let coin = |k: i32| ((449..=456).contains(&k) as u8 | ((499..=506).contains(&k) as u8) << 1 | ((549..=556).contains(&k) as u8) << 2, 0);
        let (by_coins, credits_a, _) = run_press_screen(a, 700, coin);
        assert_eq!(credits_a, [1, 1, 1]);
        let service = |k: i32| (0, if (449..=470).contains(&k) { crate::input::SERVICE } else { 0 }); // held for 22 frames
        let (by_service, credits_b, _) = run_press_screen(a, 700, service);
        assert_eq!(credits_b, [1, 1, 1], "held, but counted once");
        let (_, _, sounds) = run_press_screen(a, 450, service); // tick 449, the frame of the press, is the last one
        assert_eq!(sounds, [0x30], "the coin sound in the frame of the press");
        assert!(by_coins == by_service, "scroll1 differs");
    }
}
