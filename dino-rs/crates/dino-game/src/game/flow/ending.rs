//! The ending: game mode 0x1C (the routine 0xAC48 jumps to 0xA69B8). After the clear of Episode 8 the game task runs this routine
//! once a frame. Each frame it updates the objects (the shake 0x222D6, pool B 0x23604, pool A 0x23420, the sprite list 0x13FA6, in
//! that order and without the pool C, the heroes or the HUD) as soon as the sub state G.$4D1 is not 0, then runs the routine of
//! the sub state (jump table 0xA69E6):
//!
//! | sub | routine | what |
//! | --- | --- | --- |
//! | 0 | 0xA69F2 | clear all, brightness, the heroes are out (`+$00` = 0), G.$4D7 = G.$4DB = G.$507 = G.$5E4 = 1, the name entry blocks of the players of G.$4CD (0x15AE2), the pools, palette set 0, the cameras 0, G.-$6FFA = 0x14F0 (5360 frames) |
//! | 2 | 0xA6ABA | the master of the scenes: pool B 82 |
//! | 4 | 0xA6ADA | counts G.-$6FFA down to 0 |
//! | 6 | 0xA6AE8 | waits while a player is in the name entry (join state 0xA, 0xEFC2) |
//! | 8 | 0xA6AF6 | sleeps 180 frames, then mode 0x14 (the game over of the attract loop), sub 0, G.$4D2 = 0 and the clear 0xAE0 |
//! | 0xA | 0xA6B10 | a wait loop that jump or attack shorten: no code enters it (the debug build) |
//!
//! The scenes themselves are objects: the master 82 (`prop/b82_85.rs`) for the corridor, the sunset and the still picture, the desert
//! master 81 (`prop/b81.rs`, the walkers, the staff roll, the Cadillac and the end text) after it.

use crate::game::Game;

/// The cells of scroll2 and scroll3 the clear 0xAE0 of sub 0 has done when the video frames end (measured in the six recordings of
/// `flow_ending`: the cells of scroll2 are cleared up to a cell between 1367 and 1384 at the end of the first frame, which is where the 68000
/// is when the frame ends and depends on the work the frame did before the pass (the objects of the level, the number of heroes); the cells of
/// scroll3 up to 3223 are cleared at the end of the second frame and from 3584 on they are not).
const CLEAR2_FIRST: usize = 1376;
const CLEAR3_FIRST: usize = 3500;

/// The state of the ending that the original keeps in RAM outside the object pools.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ending {
    /// G.-$6FFA: frames left of the show (0x14F0 at the start, sub 4 counts it down).
    pub timer: u16,
    /// Sub 8 has started its sleep of 180 frames (`jsr 0xA8C`): the next run goes on after it.
    pub sleeping: bool,
    /// 1 while the second part of sub 0 is still to run (the frame after the lag of the first).
    pub phase: u8,
    /// The master 82 has made its pictures (state 0: palettes and clear in the frame it ran in) and its four pages are still to be
    /// written: they come in the next video frame that runs a task (`b82_pages`).
    pub pages_for: Option<crate::game::Handle>,
}

impl Game {
    /// 0xA69B8: one pass of the ending routine (called by the game task in the mode 0x1C).
    pub fn flow_ending_mode(&mut self) {
        if self.flow.ending.sleeping {
            // 0xA6AFC: the code after the sleep inside sub 8
            self.flow.ending.sleeping = false;
            self.flow.mode = 0x14;
            self.flow.sub = 0;
            self.flow.sub2 = 0;
            self.flow_clear_screen(); // jmp 0xAE0
            return;
        }
        if let Some(h) = self.flow.ending.pages_for.take() {
            self.b82_pages(h); // the rest of the state 0 of the master 82 (the frame after its lag)
        }
        if self.flow.ending.phase == 1 {
            // the second part of sub 0 runs in the video frame after the lag, and the pass of sub 2 follows it in the same frame
            self.flow.ending.phase = 0;
            self.flow_ending_start_b(); // (the pass counter G.$0 is counted once by the game task: the pass of sub 2)
        }
        if self.flow.sub != 0 {
            self.screen_shake(); // 0x222D6
            self.pool_b_update(); // 0x23604
            self.enemies_update(); // 0x23420
            self.build_sprite_list(); // 0x13FA6
        }
        match self.flow.sub {
            0 => self.flow_ending_start_a(),
            2 => self.flow_ending_master(),
            4 => self.flow_ending_wait(),
            6 => self.flow_ending_names(),
            8 => self.flow_ending_sleep(),
            _ => {}
        }
    }

    /// 0xA69F2, the first part of sub 0: the clear of all layers (0xAE0: the sprites and scroll1 now, scroll2 and scroll3 in the next two
    /// frames, `flow_attract_clear`), the heroes are out, the flags and the name entry blocks. The pass takes the CPU through the next video frame
    /// (the recording of `flow_ending`: no task runs in it, the layer clear goes on) and its rest comes in the frame after.
    fn flow_ending_start_a(&mut self) {
        self.flow_attract_clear(); // jsr 0xAE0
        // the recording (flow_end-jack, records 1725 and 1726): the pass has cleared the first CLEAR2_FIRST cells of scroll2 when the first
        // frame ends, scroll2 is done and scroll3 cleared up to CLEAR3_FIRST in the next one, the rest of scroll3 in the third
        for c in self.vram.scroll2.iter_mut().take(CLEAR2_FIRST) {
            (c.code, c.attr) = (0x5800, 0);
        }
        self.flow.front.clear3_cut = CLEAR3_FIRST;
        self.brightness_full_current(); // jsr 0x8932: the banks of the current mask
        for p in self.players.iter_mut() {
            p.b.live = 0; // clr.b $3274(a5), $33F4(a5), $3574(a5)
        }
        self.flow.b4d7 = 1;
        self.flow.b4db = 1;
        self.flow.b507 = 1;
        self.flow.b5e4 = 1;
        for i in 0..3 {
            if self.flow.started & (1 << i) != 0 {
                self.flow_name_entry_setup(i); // jsr 0x15AE2 with a6 = the player
            }
        }
        self.flow.ending.phase = 1;
        self.flow_lag(1);
    }

    /// 0xA69F2 from the pools on: the lists, the pools, palette set 0, the cameras and the timer, then sub 2.
    fn flow_ending_start_b(&mut self) {
        // 0xFBE8 clears the flags of the spawn driver but not the list pointers G.$674 / G.$678 (they stay as they were, nothing reads them here)
        let lists = (self.spawn.l1, self.spawn.l2);
        self.title_pools_reset(); // 0x49A8, 0x4A18, 0x4A46, 0x4C48, 0x4C76, 0x12BE4, 0xFBE8
        (self.spawn.l1, self.spawn.l2) = lists;
        // the pool inits write the pool tag into every slot (0x4C5C: +$82 = 4 in pool A, 0x4C8A: 8 in pool B)
        self.enemies.slots.iter_mut().for_each(|e| e.m.sb(0x82, 4));
        self.props.slots.iter_mut().for_each(|p| p.m.sb(0x82, 8));
        self.flow_scene_palettes(0); // moveq #0,d0; jsr 0x89E6
        self.brightness_full(); // 0x892C
        self.cam.s2.cam_y = 0;
        self.cam.s2.cam_x = 0;
        self.cam.s3.cam_y = 0;
        self.cam.s3.cam_x = 0;
        self.flow.ending.timer = 0x14F0;
        self.flow.sub += 2;
    }

    /// 0xA6ABA: sub 2 makes the master of the scenes (pool B 82, 0x4860; a full pool makes none).
    fn flow_ending_master(&mut self) {
        let _ = self.prop_spawn(82, 0);
        self.flow.sub += 2;
    }

    /// 0xA6ADA: sub 4.
    fn flow_ending_wait(&mut self) {
        self.flow.ending.timer = self.flow.ending.timer.wrapping_sub(1);
        if self.flow.ending.timer == 0 {
            self.flow.sub += 2;
        }
    }

    /// 0xA6AE8: sub 6 waits while any player is entering a name (0xEFC2: the join state 0xA of a player).
    fn flow_ending_names(&mut self) {
        if !self.flow_name_entry_busy() {
            self.flow.sub += 2;
        }
    }

    /// 0xA6AF6: sub 8, the sleep of 180 frames (0xA8C); the pass after it is `flow_ending_mode` with `sleeping`.
    fn flow_ending_sleep(&mut self) {
        self.flow.ending.sleeping = true;
        self.flow.wait = 180;
        self.flow.resume = true;
    }

    /// A game frame that keeps the CPU through `n` more video frames: no task runs in them (`Flow::lag`).
    pub fn flow_lag(&mut self, n: u8) {
        self.flow.lag = n;
    }

    /// 0xEFC2: true (carry) while a player is in the name entry (join state 0xA).
    pub fn flow_name_entry_busy(&self) -> bool {
        self.flow.join.iter().any(|j| j.state == 0xA)
    }

    /// 0x15AE2: the setup of the name entry block of player `i` (the table of the players 0x15AF2: the 32 byte blocks at 0xFF0EE4,
    /// 0xFF0F04 and 0xFF0F24). TODO(M2-22): the name entry itself (the join state 0xA, 0x15BE4 each frame) is not ported yet; the
    /// blocks are not kept, so a player never enters his name and the ending goes on at once.
    pub(super) fn flow_name_entry_setup(&mut self, _i: usize) {}
}
