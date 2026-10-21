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

/// What the master 82 still has to do at the start of the next frame (the 68000 was in the middle of it when the video frame ended).
#[derive(Clone, Copy, Debug)]
pub enum Cont {
    /// State 0 (0x8AB8E): the four pages, the sub state and the timer after the clear and the palettes (`b82_pages`).
    Pages(crate::game::Handle),
    /// State 0x16 at its end (0x8B032): the rest of the clear of scroll3, the cameras, the sub state and the timer (`b82_still_end_b`).
    StillEnd(crate::game::Handle),
}

/// The state of the ending that the original keeps in RAM outside the object pools.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ending {
    /// G.-$6FFA: frames left of the show (0x14F0 at the start, sub 4 counts it down).
    pub timer: u16,
    /// Sub 8 has started its sleep of 180 frames (`jsr 0xA8C`): the next run goes on after it.
    pub sleeping: bool,
    /// 1 while the second part of sub 0 is still to run (the frame after the lag of the first).
    pub phase: u8,
    /// The master 82 has not finished a state that overran the video frame: its rest comes at the start of the next video frame that runs
    /// a task, before the pass of that frame (`Cont`).
    pub cont: Option<Cont>,
    /// The game over that follows the ending has not shown its ranking yet (the first frame of the ranking writes `CUT_AFTER_ENDING` words).
    pub ranking_follows: bool,
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
            self.flow.ending.ranking_follows = true;
            self.flow_attract_clear(); // jmp 0xAE0: the sprites and scroll1 now, scroll2 and scroll3 in the next two frames (the recording)
            self.flow_lag(1);
            return;
        }
        match self.flow.ending.cont.take() {
            Some(Cont::Pages(h)) => self.b82_pages(h), // the rest of the state 0 of the master 82
            Some(Cont::StillEnd(h)) => self.b82_still_end_b(h),
            None => {}
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
                self.name_setup(i); // jsr 0x15AE2 with a6 = the player (`name_entry.rs`)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// The jump table 0xA69E6: sub 0 clears and sets up in two parts (the clear keeps the CPU through a lag frame), sub 2 makes the master, sub 4 counts
    /// G.-$6FFA down from 0x14F0 (5360 frames), sub 6 waits for the name entry, sub 8 sleeps 180 frames and goes to the mode 0x14 with the clear 0xAE0.
    #[test]
    fn the_sub_states_of_the_ending_follow_each_other() {
        let Some(a) = assets() else { return };
        let mut g = Game::new(a);
        g.flow.mode = 0x1C;
        g.flow.started = 1;
        g.players[0].b.live = 1;
        g.flow_ending_mode(); // sub 0, part one
        assert_eq!((g.flow.sub, g.flow.ending.phase, g.flow.lag, g.players[0].b.live), (0, 1, 1, 0));
        assert!(g.flow.b4d7 == 1 && g.flow.b4db == 1 && g.flow.b507 == 1 && g.flow.b5e4 == 1);
        g.flow_ending_mode(); // part two (the video frame after the lag) and the pass of sub 2 in the same frame: the master
        assert_eq!((g.flow.sub, g.flow.ending.timer), (4, 0x14F0));
        assert!(g.props.slots.iter().any(|p| p.b.live != 0 && p.b.k20 == 82 * 4), "the master 82");
        for _ in 0..0x14F0 {
            g.flow_ending_mode();
        }
        assert_eq!((g.flow.sub, g.flow.ending.timer), (6, 0));
        g.flow.join[0].state = 0xA; // a hero in the name entry: sub 6 waits
        g.flow_ending_mode();
        assert_eq!(g.flow.sub, 6);
        g.flow.join[0].state = 0xC;
        g.flow_ending_mode();
        assert_eq!(g.flow.sub, 8);
        g.flow_ending_mode(); // the sleep
        assert!(g.flow.ending.sleeping && g.flow.wait == 180 && g.flow.resume);
        g.flow_ending_mode(); // after it: the game over
        assert_eq!((g.flow.mode, g.flow.sub, g.flow.lag), (0x14, 0, 1));
        assert!(g.flow.ending.ranking_follows);
    }

    /// 0xAC06 / 0xAC26 are one pass of the game task: the sleeps of the clear 0xAE0 inside it do not count G.$0 / G.$4DA (`resume`), only the pass of 0xAC06 does (the
    /// next tick after the wait of the first clear); in a game clock recording the two ticks before it are in its record (`join_next`), the ticks of mode 0x18 are not
    /// (M10-20: m523_ep1_2p_full 30612 .. 30616, ep8_mustapha 68866 .. 68868). The end of the pass starts the attract task and gathers the next 5 ticks in one record.
    #[test]
    fn the_game_over_exit_is_one_pass_of_the_game_task() {
        let Some(a) = assets() else { return };
        let mut g = Game::new(a);
        g.flow.mode = 0x14;
        g.flow.in_game = 1;
        g.flow.cont.exit_wait = 4; // the continue mode has set the mode 0x14 and the wait of its clear
        for want in [(3, true, false), (2, true, true), (1, false, true)] {
            g.flow_game_over();
            assert_eq!((g.flow.cont.exit_wait, g.flow.resume, g.flow.join_next), want, "wait ticks of the mode 0x14");
        }
        g.flow_game_over(); // the pass of 0xAC06: the mode 0x18 and the clear
        assert_eq!((g.flow.mode, g.flow.cont.exit_wait, g.flow.resume), (0x18, 4, true));
        for want in [3, 2, 1] {
            g.flow_game_over();
            assert_eq!((g.flow.cont.exit_wait, g.flow.resume, g.flow.join_next), (want, true, false), "wait ticks of the mode 0x18");
        }
        g.flow_game_over(); // the end of 0xAC26: the game task ends, the attract task starts
        assert_eq!((g.flow.in_game, g.flow.attract_join), (0, 5));
    }
}
