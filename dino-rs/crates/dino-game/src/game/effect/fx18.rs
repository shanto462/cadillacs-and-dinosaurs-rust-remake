//! Pool C class 18 (+$20 = 0x48), handler 0x942C6: the splash of the Bludge's acid ball. Made by the acid ball (0x3B1E8) at its position with its wading byte
//! (+$53). Mode 0 (0x942DC) shows it (+$01 = 1), loads the frame list 0x9430E (4 frames of 7 ticks) and goes on in the mode 2 (0x942F0): the animation
//! ticks, the end moves to the mode 4, else it is drawn (0x49C0). Modes 4 and 6 free it (0x4964).

use super::*;

/// 0x9430E: the frames of the splash.
const SPLASH: u32 = 0x9430E;

impl Game {
    /// 0x3B1E8: one pool C object of the class 18 at the position of the acid ball. A full pool makes none.
    pub fn fx_spawn_acid(&mut self, x: i16, y: i16, z: i16, wading: u8) {
        let Some(i) = self.fx_alloc() else { return };
        let b = &mut self.effects.slots[i].b;
        b.live = 1;
        b.k20 = FX_ACID;
        set_word(&mut b.x, x);
        set_word(&mut b.y, y);
        set_word(&mut b.z, z);
        b.wading = wading;
    }

    /// 0x942C6.
    pub(super) fn fx_acid(&mut self, i: usize) {
        let fx = self.fx_data();
        let h = self.effects.handle(i);
        let b = &mut self.effects.slots[i].b;
        match b.mode {
            0 => {
                b.mode += 2;
                b.visible = 1;
                fx_load(b, fx, fx.frame_at(SPLASH)); // jsr 0x1236
            }
            2 => {}
            _ => return self.fx_free(i), // 0x94304
        }
        if fx_tick(&mut self.effects.slots[i].b, fx) {
            self.effects.slots[i].b.mode += 2;
        } else {
            self.draw_list_add_if_visible(h);
        }
    }
}
