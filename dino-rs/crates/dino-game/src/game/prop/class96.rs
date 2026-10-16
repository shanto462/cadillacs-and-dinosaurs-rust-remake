//! Pool B class 96 (`+$20 = 0x180`, handler 0x92382): area music object.
//! Spawned by camera mode 8 (0x20E78 / 0x9236C) during a sub-area transition.
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x92396: advance mode to 2, request music fade (0xA5EA: sound 0xFF07, d1 = 0x0444_0000, d2 = 0), set +$80 = 0x3C, and fall into mode 2 (which decrements +$80 to 0x3B). |
//! | 2 | 0x923A6: decrement +$80. When it hits 0 (59 frames after the fade frame), advance mode to 4, read music code from table 0xA582 at (stage * 4 + area), and queue it (sound code, d1 = 0, d2 = 0). |
//! | 4 | 0x923DA: free object (0x4938). |

use super::Prop;
use crate::game::Game;

mod f {
    /// word at +$80: countdown timer until area music byte is queued.
    pub const TIMER: usize = 0x80;
}

impl Game {
    /// 0x92382 pool B class 96 (jump table 0x9238E: mode 0 -> 0x92396, 2 -> 0x923A6, 4 -> 0x923DA).
    pub fn prop_class96(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                self.hook_a5ea(); // 0x9239A jsr 0xA5EA: the music fade
                p.m.sw(f::TIMER, 0x3C);
                self.prop_class96_wait(p); // falls through into 0x923A6
            }
            2 => self.prop_class96_wait(p),
            _ => self.prop_free(p, i), // 0x923DA: jmp 0x4938
        }
    }

    /// 0x923A6: decrement +$80 timer. On reaching 0, queue area music and advance to mode 4.
    fn prop_class96_wait(&mut self, p: &mut Prop) {
        let t = p.m.w(f::TIMER).wrapping_sub(1);
        p.m.sw(f::TIMER, t);
        if t == 0 {
            p.b.mode = 4;
            let code = self.assets.text.u8(0xA582, self.w.stage as u32 * 4 + self.cam.s2.area as u32);
            self.snd_request(code as u16, 0, 0); // 0x923D2: jsr 0xA63C
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn test_game() -> Game {
        let mut a = AssetPack::default();
        a.text.blobs.push((0xA582, vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]));
        Game::new(Box::leak(Box::new(a)))
    }

    #[test]
    fn class96_timing_and_music_queue() {
        let mut g = test_game();
        g.w.stage = 1;
        g.cam.s2.area = 1;

        // Allocate pool B class 96 object as hook_9236c does
        let i = g.props.alloc().expect("prop slot");
        let h = g.props.handle(i);
        {
            let b = g.body_mut(h);
            b.live = 1;
            b.k20 = 0x180;
        }

        // Frame 0: mode 0 runs: fades music (0xFF07), sets timer 0x3C, decrements to 0x3B (59)
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2);
        assert_eq!(g.props.slots[i].m.w(0x80), 59);
        let reqs = g.snd_take_requests();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].code, 0xFF07);
        assert_eq!(reqs[0].d1, 0x0444_0000);
        assert_eq!(reqs[0].d2, 0);

        // Frames 1..=58: timer decrements from 58 down to 1, mode stays 2, no sound
        for expected in (1..59).rev() {
            g.pool_b_update();
            assert_eq!(g.props.slots[i].b.mode, 2);
            assert_eq!(g.props.slots[i].m.w(0x80), expected);
            assert!(g.snd_take_requests().is_empty());
        }

        // Frame 59 (59 frames after the fade frame): timer hits 0, mode becomes 4, area music queued
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 4);
        assert_eq!(g.props.slots[i].m.w(0x80), 0);
        let expected_music = g.assets.text.u8(0xA582, 1 * 4 + 1) as u16;
        let reqs = g.snd_take_requests();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].code, expected_music);
        assert_eq!(reqs[0].d1, 0);
        assert_eq!(reqs[0].d2, 0);

        // Frame 60: mode 4 runs and frees the object
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0);
    }
}
