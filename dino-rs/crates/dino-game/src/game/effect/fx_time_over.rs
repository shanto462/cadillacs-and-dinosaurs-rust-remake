//! Pool C class 28 (`+$20 = 0x70`, handler 0x96488): the words TIME and OVER of the stage time out (D4b section 1.5,
//! D3f class 28). The time limit routine 0x103FE makes two with 0x96450 (`Game::fx_spawn_time_over`): variant 0, TIME,
//! comes first, variant 1, OVER, second (the pool hands out slot 47 and then 46). Each is a raw sprite list of one
//! record (0x96546 TIME, 0x96550 OVER), drawn in screen space over everything (draw flags 0xD0).
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x9649C: mode 2, visible, draw flags 0xD0, the hold counter `+$60` = 0xB4, and for TIME X 0x188, for OVER X 0xFFF8, both Y 0x80 and depth 0, X speed 0xE000 (TIME, -32 pixels a frame) or 0x2000 (OVER, +32). The frame ends here: no move, no picture. |
//! | 2 | 0x964FC: TIME is done when X is 0xC0 or less, OVER when it is 0x140 or more (signed word compares, tested before the move), else it moves (0x189C) and goes into the draw list (0x49C0). When done it becomes mode 4 and goes on in the same frame. |
//! | 4 | 0x9652A: with the lock G.$4EB clear (a respawn gave the clock its time back, 0x104A8) it frees itself; else `+$60` counts down, at the borrow (after 0xB5 updates) it frees itself, else it goes into the draw list. |
//! | 6 | 0x96540: free (0x4964). |
//!
//! So TIME stops at X 0xA8 (0x188 - 7 * 0x20) and OVER at 0x158 (-8 + 11 * 0x20): the two words do not meet at 0xC0 as
//! D3f writes (it has the speed 4 pixels a frame wrong too; MAME and the listing agree on 32). The words stay for 0xB5
//! frames after they have stopped, or until the clock is topped up.

use super::{raw_load, set_word};
use crate::game::Game;

/// The raw records of the two words (instruction constants).
const TIME: u32 = 0x96546;
const OVER: u32 = 0x96550;

impl Game {
    /// 0x96450: the two words of TIME OVER. A full pool stops it (carry set): the second word is not made when the first
    /// failed.
    pub fn fx_spawn_time_over(&mut self) -> bool {
        for var in 0..2u16 {
            let Some(i) = self.fx_alloc() else { return false };
            let b = &mut self.effects.slots[i].b;
            b.live = 1;
            b.k20 = super::FX_TIME_OVER;
            b.var26 = var;
        }
        true
    }

    /// 0x96488 pool C class 0x70 (jump table 0x96494: mode 0 -> 0x9649C, 2 -> 0x964FC, 4 -> 0x9652A, 6 -> 0x96540).
    pub(super) fn fx_time_over(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => self.fx_time_over_init(i),
            2 => self.fx_time_over_move(i),
            4 => self.fx_time_over_hold(i),
            _ => self.fx_free(i),
        }
    }

    /// 0x9649C.
    fn fx_time_over_init(&mut self, i: usize) {
        let fx = self.fx_data();
        let e = &mut self.effects.slots[i];
        e.b.mode += 2;
        e.b.visible = 1;
        e.b.draw_flags = 0xD0;
        e.owner60 = 0xB4; // the hold counter (+$60 is the owner of the flames in other classes)
        let (record, x, vx) = if e.b.var26 == 0 { (TIME, 0x188, 0xE000u16) } else { (OVER, -8, 0x2000) };
        raw_load(&mut e.b, fx, fx.raw_at(record));
        set_word(&mut e.b.x, x);
        set_word(&mut e.b.y, 0x80);
        set_word(&mut e.b.z, 0);
        e.b.vx = vx as i16;
    }

    /// 0x964FC.
    fn fx_time_over_move(&mut self, i: usize) {
        let h = self.effects.handle(i);
        let b = &mut self.effects.slots[i].b;
        let done = if b.var26 == 0 { b.xi() <= 0xC0 } else { b.xi() >= 0x140 };
        if !done {
            b.motion(); // jsr 0x189C
            self.draw_list_add_if_visible(h); // jmp 0x49C0
        } else {
            b.mode += 2; // 0x96526, and on into 0x9652A
            self.fx_time_over_hold(i);
        }
    }

    /// 0x9652A.
    fn fx_time_over_hold(&mut self, i: usize) {
        if self.rank.time_lock == 0 {
            return self.fx_free(i); // tst.b $4eb(a5) ; beq 0x96540
        }
        let h = self.effects.handle(i);
        let e = &mut self.effects.slots[i];
        let (left, borrow) = e.owner60.overflowing_sub(1); // subq.w #1,$60(a6) ; bcs
        e.owner60 = left;
        if borrow {
            self.fx_free(i);
        } else {
            self.draw_list_add_if_visible(h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use dino_assets::effect::{RAW_FRAME, RawFrame};

    /// A game with two raw records standing in for the ROM's (made up: no ROM bytes in a test), each one looping on itself.
    fn game() -> Game {
        let mut a = AssetPack::default();
        for (k, addr) in [TIME, OVER].into_iter().enumerate() {
            a.effects.raw_frames.push(RawFrame {
                rom_addr: addr,
                list_addr: 0,
                sprites: Vec::new(),
                param_a: 0,
                param_b: 0,
                kind: 0,
                duration: 2,
                last: true,
                next: RAW_FRAME | k as u32,
            });
            a.effects.raw_by_addr.insert(addr, RAW_FRAME | k as u32);
        }
        Game::new(Box::leak(Box::new(a)))
    }

    /// The updates of a pair, as the pool C update runs them, with the x of both words after each frame.
    fn run(g: &mut Game, frames: usize) -> Vec<(i16, i16)> {
        (0..frames)
            .map(|_| {
                g.draw_list.clear();
                g.pool_c_update();
                (g.effects.slots[47].b.xi(), g.effects.slots[46].b.xi())
            })
            .collect()
    }

    /// With the ROM data (skipped without `dino.zip`): the two raw records are extracted, each one a list of sprites that loops on
    /// itself (the "last" flag with the link back to the record), and TIME is left of OVER once both have stopped.
    #[test]
    fn the_real_records_are_extracted() {
        let path = std::path::Path::new("../../../dino.zip");
        if !path.exists() {
            return;
        }
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads")));
        let fx = &assets.effects;
        for addr in [TIME, OVER] {
            let id = fx.raw_at(addr);
            let r = fx.raw(id);
            assert!(!r.sprites.is_empty() && r.last && r.next == id, "{addr:#x}");
            assert_eq!(r.duration, 2);
        }
        let mut g = Game::new(assets);
        g.rank.time_lock = 1;
        g.fx_spawn_time_over();
        run(&mut g, 14);
        let (t, o) = (&g.effects.slots[47].b, &g.effects.slots[46].b);
        assert!(t.xi() < o.xi() && o.xi() - t.xi() == 0x158 - 0xA8);
        assert_eq!(g.draw_list.len(), 2);
    }

    /// 0x96450: TIME first (the pool gives slot 47), OVER second (46), variants 0 and 1, class 0x70.
    #[test]
    fn the_spawn_makes_time_then_over() {
        let mut g = game();
        assert!(g.fx_spawn_time_over());
        let (t, o) = (&g.effects.slots[47].b, &g.effects.slots[46].b);
        assert_eq!((t.live, t.k20, t.var26, o.live, o.k20, o.var26), (1, 0x70, 0, 1, 0x70, 1));
        // a pool with one slot left makes TIME only
        let mut g = game();
        for _ in 0..47 {
            g.effects.alloc();
        }
        assert!(!g.fx_spawn_time_over());
        assert_eq!(g.effects.free_count(), 0, "TIME took the last slot");
    }

    /// The motion of D3f class 28 as MAME shows it (m515_s21_timeover_life): the first update only starts the words (no
    /// move, no picture), then 32 pixels a frame until TIME is at 0xC0 or less (0xA8, 7 moves) and OVER at 0x140 or more
    /// (0x158, 11 moves); a word that is done goes into mode 4 in the same frame.
    #[test]
    fn the_words_fly_in_and_stop() {
        let mut g = game();
        g.rank.time_lock = 1;
        g.fx_spawn_time_over();
        let xs = run(&mut g, 14);
        assert_eq!(xs[0], (0x188, -8), "the init frame");
        assert_eq!(xs[1], (0x168, 0x18));
        assert_eq!(xs[7], (0xA8, 0xD8), "TIME has made its 7th move");
        assert_eq!(xs[8], (0xA8, 0xF8));
        assert_eq!(xs[11], (0xA8, 0x158), "OVER its 11th");
        assert_eq!(xs[13], (0xA8, 0x158));
        let (t, o) = (&g.effects.slots[47].b, &g.effects.slots[46].b);
        assert_eq!((t.mode, o.mode), (4, 4));
        assert_eq!((t.visible, t.draw_flags, t.xi() as u16 >> 15, t.yi(), t.zi()), (1, 0xD0, 0, 0x80, 0));
        assert_eq!((t.timer, o.timer), (2, 2), "the record's duration");
        // TIME is found done in its 9th update (OVER in the 13th), which is already a hold update: the counter drops by 1
        assert_eq!(g.effects.slots[47].owner60, 0xB4 - 6, "TIME: held since its 8th update");
        assert_eq!(g.effects.slots[46].owner60, 0xB4 - 2, "OVER: held since its 12th update");
    }

    /// The pictures: in the draw list while moving and holding, but not in the first update.
    #[test]
    fn they_are_drawn_from_the_second_update() {
        let mut g = game();
        g.rank.time_lock = 1;
        g.fx_spawn_time_over();
        g.draw_list.clear();
        g.pool_c_update();
        assert!(g.draw_list.is_empty());
        g.draw_list.clear();
        g.pool_c_update();
        assert_eq!(g.draw_list.len(), 2);
    }

    /// The hold: 0xB5 updates after the stop, then the borrow frees the word; and a clear lock (the top up of a respawn)
    /// frees it in the next update, even the one that has just stopped.
    #[test]
    fn the_words_stay_for_a_while_and_go_with_the_lock() {
        let mut g = game();
        g.rank.time_lock = 1;
        g.fx_spawn_time_over();
        run(&mut g, 12);
        // OVER's first hold update is the 13th (counter 0xB3 after it): the counter reaches 0 in the 192nd, the 193rd frees it
        let mut kept = 0;
        while g.effects.slots[46].b.live != 0 {
            run(&mut g, 1);
            kept += 1;
            assert!(kept < 400);
        }
        assert_eq!(kept, 0xB5, "0xB4 steps to 0 and one more for the borrow, counted from the 13th update");
        assert_eq!(g.effects.slots[47].b.live, 0, "TIME went 4 frames before");
        // the lock
        let mut g = game();
        g.rank.time_lock = 1;
        g.fx_spawn_time_over();
        run(&mut g, 10);
        g.rank.time_lock = 0;
        run(&mut g, 1);
        assert_eq!(g.effects.slots[47].b.live, 0, "TIME was in mode 4");
        assert_eq!(g.effects.slots[46].b.live, 1, "OVER is still flying: the lock is looked at in mode 4 only");
        run(&mut g, 2);
        assert_eq!(g.effects.slots[46].b.live, 0);
    }
}
