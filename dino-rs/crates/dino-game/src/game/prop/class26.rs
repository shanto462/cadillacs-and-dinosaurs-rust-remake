//! Pool B class 26 (`+$20 = 0x68`, handler 0x7EC48): the typewriter of the episode title (M2-26). The original makes it with 0x7EC20 in the intro task of a
//! stage; it types a big font string into scroll1 one item per frame (`+$86`) and frees itself one frame after the end marker. The routine itself is
//! `flow/text.rs` (`typer_step`, the state `+4` and sub state `+5` of the object); this is the slot that holds it in pool B (the free list, the class,
//! the mode bytes the recordings show).

use super::{Game, Prop};

impl Game {
    /// 0x7EC48 for the object in pool B slot `i`.
    pub(super) fn prop_class26(&mut self, p: &mut Prop, i: usize) {
        let Some(k) = self.flow.typers.iter().position(|t| t.slot == Some(i)) else {
            return self.prop_free(p, i); // no text: nothing to type
        };
        let mut t = self.flow.typers.remove(k);
        self.typer_step(&mut t);
        let (state, sub) = t.state_sub();
        if t.live {
            p.b.live = 2; // 0x7EC62: move.b #2,0(a6) in the first update
        }
        p.b.mode = state;
        p.b.sub = sub;
        if t.live {
            self.flow.typers.insert(k, t);
        } else {
            self.prop_free(p, i); // 0x7EEB4
        }
    }
}
