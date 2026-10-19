//! Pool C class 40 (+$20 = 0xA0, handler 0x97674, table 0x9770A): the debris of Fessenden form 2's death (spawned by 0x9764E in the quake step).
//! The init (0x97688) gives the facing by the slot (d7 & 1), the animation of the variant, vx (`rand & 0x3FF`) - 0x200, vy `rand & 0x3F0`,
//! gravity -8; it flies (0x189C) and is freed below the floor (Y < 0, 0x4964); drawn (0x49C0) except one update in four (`G.$0 + d7`).
//! Owner: effects.

use super::*;
use crate::game::item::put_hi;

impl Game {
    /// 0x9764E: a debris piece (variant `var`) at (x, y, z); a full pool C makes none.
    pub(crate) fn fx40_spawn(&mut self, x: i16, y: i16, z: i16, var: u16) {
        let Some(i) = self.effects.alloc() else { return };
        let e = &mut self.effects.slots[i];
        e.b.live = 1;
        e.b.k20 = 0xA0;
        put_hi(&mut e.b.x, x);
        put_hi(&mut e.b.y, y);
        put_hi(&mut e.b.z, z);
        e.b.var26 = var;
    }

    /// 0x97674 (mode table 0x97680): 0 the init (falls into 2), 2 the flight, 4 and 6 the free.
    pub(super) fn fx40(&mut self, i: usize) {
        let d7 = 47 - i as u16;
        match self.effects.slots[i].b.mode {
            0 => {
                let fx = self.fx_data();
                let e = &mut self.effects.slots[i];
                e.b.anim_table = 0x9770A;
                e.b.draw_flags = 0x10;
                e.b.visible = 1;
                e.b.facing = (d7 & 1) as u8;
                let var = e.b.var26;
                fx_start(&mut self.effects.slots[i].b, fx, var); // 0x15E4
                let vx = ((self.rand() & 0x3FF) as i16).wrapping_sub(0x200);
                let vy = (self.rand() & 0x3F0) as i16;
                let e = &mut self.effects.slots[i];
                e.b.vx = vx;
                e.b.vy = vy;
                e.b.ay = -8;
                e.b.mode = e.b.mode.wrapping_add(2);
                self.fx40_run(i, d7);
            }
            2 => self.fx40_run(i, d7),
            _ => self.fx_free(i),
        }
    }

    /// 0x976DA.
    fn fx40_run(&mut self, i: usize, d7: u16) {
        let fx = self.fx_data();
        fx_tick(&mut self.effects.slots[i].b, fx); // 0x15F4
        self.effects.slots[i].b.motion(); // 0x189C
        if self.effects.slots[i].b.yi() < 0 {
            return self.fx_free(i); // 0x4964
        }
        if self.w.frame.wrapping_add(d7) & 3 != 0 {
            self.draw_list_add_if_visible(self.effects.handle(i)); // 0x49C0
        }
    }
}
