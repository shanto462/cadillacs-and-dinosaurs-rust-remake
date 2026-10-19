//! Pool B class 95 (handler 0x92220): the Episode 3 sky blend. The stage script places it at the road triggers (variants 0 .. 0xC by camera
//! X); it fades 16 colours of palette RAM toward a target set of the ROM, one step per RGB nibble every `+$A9` frames, for 16 rounds, keeping
//! the brightness nibble, then frees itself. No sprite, body, drop or hit. D3e part 2 section 3 ("All road animation"), A204.

use super::Prop;
use crate::game::Game;

/// 0x922D0: per variant / 2 the target colours (a ROM long) and the palette RAM address (a long); 0x922C8: the period by variant / 2.
const TARGETS: u32 = 0x922D0;
const PERIODS: u32 = 0x922C8;

impl Game {
    /// 0x92220 (mode table 0x9222C): 0 the init 0x92234, 2 the blend 0x9226E, 4 and 6 the free 0x922C2.
    pub(super) fn prop_b95(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                // 0x92234: `+$A0` = the targets, `+$A4` = the palette address, 16 rounds, the period, the first round in the next frame.
                p.b.mode = p.b.mode.wrapping_add(2);
                let k = (p.b.var26 as u32).wrapping_mul(4);
                let t = &self.assets.enemy.t;
                p.m.sl(0xA0, t.u32(TARGETS, k));
                p.m.sl(0xA4, t.u32(TARGETS, k + 4));
                p.m.sb(0xA8, 0x10);
                p.m.sb(0xA9, t.u8(PERIODS, (p.b.var26 >> 1) as u32));
                p.m.sb(0xAA, 1);
            }
            2 => {
                let t = p.m.b(0xAA).wrapping_sub(1);
                p.m.sb(0xAA, t);
                if t != 0 {
                    return;
                }
                p.m.sb(0xAA, p.m.b(0xA9));
                self.prop_b95_round(p.m.l(0xA4), p.m.l(0xA0));
                let n = p.m.b(0xA8).wrapping_sub(1);
                p.m.sb(0xA8, n);
                if n == 0 {
                    self.prop_free(p, i); // 0x922C2: jmp 0x4938
                }
            }
            _ => self.prop_free(p, i),
        }
    }

    /// 0x9227A: one round over 16 colours: each of the three low nibbles (blue, green, red) moves one step toward the target's; the top
    /// (brightness) nibble stays.
    fn prop_b95_round(&mut self, dest: u32, target: u32) {
        let first = (dest.wrapping_sub(0x914000) / 2) as usize;
        for k in 0..16usize {
            let Some(&cur) = self.pal.words.get(first + k) else { break };
            let want = self.assets.enemy.t.u16(target, 2 * k as u32);
            self.pal.words[first + k] = blend_step(cur, want);
        }
        self.pal.dirty = true;
    }
}

/// The nibble steps of 0x92286 .. 0x922B2: for each of the three low nibbles, one toward the target's (signed compare of 0..15), then the
/// word is turned back by the fourth `ror`.
pub(super) fn blend_step(cur: u16, want: u16) -> u16 {
    let (mut d2, mut d3) = (cur, want);
    for _ in 0..3 {
        let (mut d0, d1) = (d2 & 0xF, d3 & 0xF);
        if d0 > d1 {
            d0 -= 1;
        } else if d0 < d1 {
            d0 += 1;
        }
        d2 = (d2 & 0xFFF0) | d0;
        d2 = d2.rotate_right(4);
        d3 = d3.rotate_right(4);
    }
    d2.rotate_right(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_rgb_nibble_moves_one_step_and_the_brightness_stays() {
        assert_eq!(blend_step(0xF000, 0x0FFF), 0xF111);
        assert_eq!(blend_step(0x1A3C, 0x5A2F), 0x1A2D);
        assert_eq!(blend_step(0x8555, 0x0555), 0x8555);
    }
}
