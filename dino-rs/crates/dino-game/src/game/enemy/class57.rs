//! Pool A class 57 (`+$20 = 0xE4`, handler 0x57D94): the puff of a hunter's gun in the opening movie's jungle (M1-10;
//! the class has other users in the game, the variant 0 plays the sound 0xE7 first). The hunter (class 11) creates it with
//! the variant 1 at his position + 0x2B / 0x44.
//!
//! Mode 0 (0x57DA8) sets up the object (live 2, the table 0x57E0E, `+$80` and `+$83` clear, no shadow, the terrain, the
//! animation 1 for a variant other than 0, else the sound 0x9CD8 and the animation 0) and goes on in mode 2 (0x57DF0),
//! which ends the object when the frame has the flag `+$2C` (mode 4), steps the animation and draws. Mode 4 and 6 free it
//! (0x57E08).

use super::Enemy;
use crate::game::{Game, Handle};

/// The animation table of the puff.
const TABLE: u32 = 0x57E0E;

impl Game {
    /// 0x57D94 class 57 (jump table 0x57DA0: mode 0 -> 0x57DA8, 2 -> 0x57DF0, 4 and 6 -> 0x57E08).
    pub fn enemy_class57(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                e.b.live = 2;
                e.b.anim_table = TABLE;
                e.m.sw(0x80, 0);
                e.m.sb(0x83, 0);
                e.b.draw_flags = 0x10;
                if let Some(o) = self.terrain_collide(&mut e.b, h, false) { // jsr 0x12CB4
                    e.m.sw(0x94, o); // 0x12CB4 leaves the obstacle in +$94
                }
                if e.b.var26 != 0 {
                    self.enemy_anim(&mut e.b, 1);
                } else {
                    self.item_snd_9cd8(e.b.xi(), e.b.facing); // jsr 0x9CD8: the fixed code 0xE7 of a pool A object, d1 / d2 as the item row
                    self.enemy_anim(&mut e.b, 0);
                }
                e.b.mode += 2; // 0x57DEC, then the frame at once
                self.enemy_c57_frame(e, h);
            }
            2 => self.enemy_c57_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x57E08: jmp 0x48FC
            }
        }
    }

    /// 0x57DF0: the flag `+$2C` of the frame ends it, the animation step, the draw list.
    fn enemy_c57_frame(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.param_a != 0 {
            e.b.mode += 2;
        }
        self.enemy_step(&mut e.b);
        self.enemy_draw_check(&mut e.b, h);
    }
}
