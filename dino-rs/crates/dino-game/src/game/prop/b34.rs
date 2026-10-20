//! Pool B class 34 (handler 0x81B3E): a caption box of the lab monitor's talk (made by B30 through 0x81B24). It prints one tall font text
//! (0x84F72) and is freed in the next frame. Variant 0 picks the text by the hero furthest right (his name id, texts 0x81BCA[0..3]); variant
//! n > 0 the text n + 3. D3e B34, D5 M8-16.

use super::Prop;
use crate::game::Game;

/// 0x81BCA: the text pointers (four by hero, then the talk's next lines).
const TEXTS: u32 = 0x81BCA;

impl Game {
    /// 0x81B24: a caption box of the variant `var` (a full pool B makes none).
    pub(super) fn prop_b34_spawn(&mut self, var: u16) {
        let Some(i) = self.props.alloc() else { return };
        let p = &mut self.props.slots[i];
        p.b.live = 1;
        p.b.k20 = 34 * 4;
        p.b.var26 = var;
    }

    /// 0x81B3E (mode table 0x81B4A): 0 the print 0x81B52 (mode 4 at once), 4 and 6 the free 0x81B76.
    pub(super) fn prop_b34(&mut self, p: &mut Prop, i: usize) {
        if p.b.mode != 0 {
            return self.prop_free(p, i);
        }
        p.b.mode = 4;
        let k = if p.b.var26 != 0 {
            p.b.var26.wrapping_add(3)
        } else {
            // 0x81B7C: the hero furthest right (live, X not below the best so far; P1 when none).
            let mut best = 0usize;
            let mut x = 0i16;
            for n in 0..3 {
                let b = &self.players[n].b;
                if b.live != 0 && x <= b.xi() {
                    x = b.xi();
                    best = n;
                }
            }
            self.players[best].b.id as u16
        };
        let t = &self.assets.frontend.text;
        let s = t.slice_from(t.u32(TEXTS, k as u32 * 4));
        self.text_tall(s);
    }
}
