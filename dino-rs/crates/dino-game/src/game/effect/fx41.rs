//! Pool C class 41 (`+$20` = 0xA4, handler 0x97800): the dust under the Cadillac's wheels and the smoke of a damaged car (M6-22). The car makes it
//! with 0x977D2 (`prop_car_puff` in `car.rs`: dust with the palette 0x8E, or smoke). It picks one of four drifts at random (16.16 speeds in X and
//! Y, table 0x978CE by `rand() & 0x18`), runs its animation (table 0x9786E) and is freed at the animation's end. D3f class 41.

use super::*;

/// 0x97814: the animation table; 0x97834: the drifts (four records of two longs, X then Y).
const PUFF_ANIMS: u32 = 0x9786E;
const DRIFTS: u32 = 0x978CE;

impl Game {
    /// 0x97800 (mode table 0x9780C: 0 the init 0x97814 falling into the run 0x97848, 2 the run, 4 and 6 the free 0x97868).
    pub(super) fn fx_car_puff(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                // 0x97814: the table, no shadow, animation 0 (0x120E), the drift `rand() & 0x18`.
                let fx = self.fx_data();
                let e = &mut self.effects.slots[i];
                e.b.anim_table = PUFF_ANIMS;
                e.b.draw_flags = 0x10;
                fx_start_obj(&mut e.b, fx, 0); // jsr 0x120E
                let r = (self.rand() & 0x18) as u32;
                let t = &self.assets.enemy.t;
                let (dx, dy) = (t.u32(DRIFTS, r), t.u32(DRIFTS, r + 4));
                let e = &mut self.effects.slots[i];
                e.set_l58(dx);
                e.set_l5c(dy);
                e.b.mode = e.b.mode.wrapping_add(2);
                self.fx_car_puff_run(i);
            }
            2 => self.fx_car_puff_run(i),
            _ => self.fx_free(i),
        }
    }

    /// 0x97848: the animation (0x15F4); at its end the free (0x4964); else the drift and the draw check (0x49CA).
    fn fx_car_puff_run(&mut self, i: usize) {
        let fx = self.fx_data();
        if fx_tick(&mut self.effects.slots[i].b, fx) {
            return self.fx_free(i);
        }
        let e = &mut self.effects.slots[i];
        let (dx, dy) = (e.l58(), e.l5c());
        e.b.x = e.b.x.wrapping_add(dx as i32);
        e.b.y = e.b.y.wrapping_add(dy as i32);
        self.draw_list_add_check(self.effects.handle(i));
    }
}

impl Effect {
    /// The long `+$58` (the words `+$58` and `+$5A`).
    fn l58(&self) -> u32 {
        ((self.owner58 as u32) << 16) | self.owner_class as u32
    }
    fn set_l58(&mut self, v: u32) {
        self.owner58 = (v >> 16) as u16;
        self.owner_class = v as u16;
    }
    /// The long `+$5C` (the words `+$5C` and `+$5E`; `+$5E` / `+$5F` are the bytes `b5e` / `b5f`).
    fn l5c(&self) -> u32 {
        ((self.w5c as u32) << 16) | ((self.b5e as u32) << 8) | self.b5f as u32
    }
    fn set_l5c(&mut self, v: u32) {
        self.w5c = (v >> 16) as u16;
        self.b5e = (v >> 8) as u8;
        self.b5f = v as u8;
    }
}
