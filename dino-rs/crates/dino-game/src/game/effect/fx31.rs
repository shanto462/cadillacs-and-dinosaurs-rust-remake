//! Pool C class 31 (+$20 = 0x7C), handler 0x96800: the muzzle flash of a gun shot. The flash is made by the class 33 spark
//! (`fx33.rs`, 0x967C4 `fx_spawn_muzzle`), one per record of the spark's table, at the spark's position plus the record's
//! offsets. Its picture is the animation of +$26 (high byte: 0 handgun starburst, 1 rifle cone, 2 shotgun flare, 3 Uzi burst),
//! its velocity is the word pair of the table 0x96986 (selector: +$27, the low byte of +$26) at the index of its direction (+$22 * 4).
//! The picture is loaded with the object loader 0x120E (the boxes and the extra bytes +$4C..+$4F too, `fx_start_obj`) and ticked with 0x15F4.

use super::*;

/// 0x96884: the four flash animations (effect table).
const MUZZLE_ANIMS: u32 = 0x96884;
/// 0x96986: four pointers (longs) by the selector (+$27, 0, 4, 8, 12); each points to (vx, vy) words by direction (+$22 * 4).
const MUZZLE_VELOCITY: u32 = 0x96986;

impl Game {
    /// 0x967C4: one flash at `pos` (the shooter's X, Y, Z high words, copied from the spark) moved by `off` (X offset in the high word, Y offset in
    /// the low word), with the word `sel` (+$26) and the direction `dir` (+$22, from the spark's scatter row). Nothing when pool C is full.
    pub(super) fn fx_spawn_muzzle(&mut self, pos: (i16, i16, i16), sel: u16, off: u32, dir: u8) {
        let Some(j) = self.effects.alloc() else { return };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = FX_MUZZLE;
        set_word(&mut e.b.x, pos.0.wrapping_add((off >> 16) as u16 as i16));
        set_word(&mut e.b.y, pos.1.wrapping_add(off as u16 as i16));
        set_word(&mut e.b.z, pos.2);
        e.b.var26 = sel;
        e.b.dir = dir;
    }

    /// 0x96800: mode 0 (init, falls into the flight) and 2; 4 and 6 free the slot (0x9687E).
    pub(super) fn fx_muzzle(&mut self, i: usize) {
        let fx = self.fx_data();
        match self.effects.slots[i].b.mode {
            0 => {
                let assets = self.assets;
                let e = &mut self.effects.slots[i];
                e.b.anim_table = MUZZLE_ANIMS;
                e.b.draw_flags = 0x10;
                let anim = e.b.var26 >> 8; // move.b $26(a6), d0 (the high byte)
                fx_start_obj(&mut e.b, fx, anim); // jsr 0x120E
                let sel = u32::from(e.b.var26 & 0xFF); // move.b $27(a6), d0
                let set = assets.enemy.t.u32(MUZZLE_VELOCITY, sel);
                let k = u32::from(e.b.dir) * 4; // move.b $22(a6), d0 ; lsl.w #2
                e.b.vx = assets.enemy.t.i16(set, k);
                e.b.vy = assets.enemy.t.i16(set, k + 2);
                e.b.mode = e.b.mode.wrapping_add(2);
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        // 0x96854: the picture ticks; the last picture frees the slot (0x4964)
        if fx_tick(&mut self.effects.slots[i].b, fx) {
            return self.fx_free(i);
        }
        let e = &mut self.effects.slots[i];
        // 0x96866 / 0x96878: X and Y move by the 8.8 velocities, no depth and no gravity
        e.b.x = e.b.x.wrapping_add((e.b.vx as i32) << 8);
        e.b.y = e.b.y.wrapping_add((e.b.vy as i32) << 8);
        self.draw_list_add_check(self.effects.handle(i)); // jmp 0x49CA
    }
}
