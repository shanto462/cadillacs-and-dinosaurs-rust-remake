//! Pool C class 39 (+$20 = 0x9C), handler 0x97530: the mutation aura of the Episode 8 clear scene. The stand in of Mustapha (pool A class 97,
//! `enemy/class97.rs`) makes it at X 0x1200 (0x974FA: X + 8, Y + 0x40, its depth). It moves right 6 pixels a frame with its picture (the frame
//! records at 0x975AC, loaded with 0x160A, ticked with 0x15F4) and at X 0x12D0 leaves a blast (pool E class 1 kind 4, `+$5C` = 1, 0x27DAA) and the
//! pool B class 46 (0x84188) and frees itself. D5 errata (M8-36).

use super::*;

/// 0x975AC: the first frame record of the aura (0x160A loads it directly).
const AURA_FRAME: u32 = 0x975AC;

impl Game {
    /// 0x97530 (mode table 0x9753E): 0 the init 0x97546 (falls into the move), 2 the move 0x97572, 4 the end 0x9758C, 6 the free 0x975A6.
    pub(super) fn fx_aura(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                // 0x97546. `+$7D` = 0xFF is not kept: no list or pass reads a pool C object's `+$7D`.
                let fx = self.fx_data();
                let e = &mut self.effects.slots[i];
                e.b.mode = e.b.mode.wrapping_add(2);
                fx_load(&mut e.b, fx, fx.frame_at(AURA_FRAME));
                e.b.draw_flags = 0x10;
                e.off_x = (e.off_x as u16 & 0xFF00 | 0xFF) as i16; // move.b #$FF, $63: the low byte of the word +$62 in pool C
                e.b.box_table = 0x103000;
                e.b.multi_box = false; // clr.b $4A
                self.fx_aura_move(i);
            }
            2 => self.fx_aura_move(i),
            4 => self.fx_aura_end(i),
            _ => self.fx_free(i), // 0x975A6: jmp 0x4964
        }
    }

    /// 0x97572: X + 6 (the word); at X 0x12D0 or more (signed) the end, else the picture and the draw (0x49CA).
    fn fx_aura_move(&mut self, i: usize) {
        let b = &mut self.effects.slots[i].b;
        let x = b.xi().wrapping_add(6);
        set_word(&mut b.x, x);
        if x >= 0x12D0 {
            return self.fx_aura_end(i);
        }
        let fx = self.fx_data();
        fx_tick(&mut self.effects.slots[i].b, fx); // jsr 0x15F4, the carry is ignored
        self.draw_list_add_check(self.effects.handle(i)); // jmp 0x49CA
    }

    /// 0x9758C: a blast at the aura (0x27DAA, d3 = 1), the pool B class 46 (0x84188), the free (0x4964).
    fn fx_aura_end(&mut self, i: usize) {
        let b = &self.effects.slots[i].b;
        let (x, y, z) = (b.xi(), b.yi(), b.zi());
        self.c84_blast(x, y, z, 1);
        let _ = self.prop_spawn(46, 0); // 0x84188: 0x4860, live 1, class word 0xB8 (the variant is the cleared word of the slot)
        self.fx_free(i);
    }
}
