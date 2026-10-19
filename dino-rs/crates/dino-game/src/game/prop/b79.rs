//! Pool B class 79 (handler 0x8A3D0): the starter of the Tyrog scene (Episode 6, area 2, script record at trigger 0x0F80). It plays sound 0x21,
//! makes the four actors of the scene (pool A class 75: the crate, variant 0, and the three guards, variants 1 and 4) and shows the captions
//! 0x50 to 0x53 one after the other, then frees itself. The crate makes the creature (class 76), which makes Tyrog (class 4). D3d 3.1, 3.2.

use super::Prop;
use crate::game::Game;

impl Game {
    /// 0x8A3D0 (mode table 0x8A3DC): 0 the init 0x8A3E4 (falls into 2), 2 the scene 0x8A3F0, 4 and 6 the free 0x8A57C.
    pub(super) fn prop_b79(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                p.b.live = 2;
                self.prop_b79_scene(p, i);
            }
            2 => self.prop_b79_scene(p, i),
            _ => self.prop_free(p, i),
        }
    }

    /// 0x8A3F0 by `+$05` (table 0x8A3FE): 0 the sound 0x21 (falls into 2); 2 the actors and caption 0x50; 6, 0xA, 0xE the captions 0x51, 0x52,
    /// 0x53; 4, 8, 0xC, 0x10 the waits `+$A0`; 0x12 the free.
    fn prop_b79_scene(&mut self, p: &mut Prop, i: usize) {
        match p.b.sub {
            0 | 2 => {
                if p.b.sub == 0 {
                    self.snd_request(0x21, 0, 0); // 0x8A412
                    p.b.sub = 2;
                }
                // 0x8A42A: the crate at the starter, a guard 0x30 left, two guards (variant 4) 0x30 right / 0x20 nearer and 0x10 left / 0x20 deeper.
                let (x, y, z) = (p.b.xi(), p.b.yi(), p.b.zi());
                for (var, dx, dz) in [(0u16, 0i16, 0i16), (1, -0x30, 0), (4, 0x30, -0x20), (4, -0x10, 0x20)] {
                    self.prop_b79_actor(var, x.wrapping_add(dx), y, z.wrapping_add(dz));
                }
                self.prop_caption_spawn(0x50); // 0x86D2E
                p.m.sw(0xA0, 0x79);
                p.b.sub = p.b.sub.wrapping_add(2);
            }
            6 | 0xA | 0xE => {
                p.b.sub = p.b.sub.wrapping_add(2);
                let (wait, code) = match p.b.sub {
                    8 => (0xDD, 0x51),
                    0xC => (0x79, 0x52),
                    _ => (0x79, 0x53),
                };
                p.m.sw(0xA0, wait);
                self.prop_caption_spawn(code);
            }
            4 | 8 | 0xC | 0x10 => {
                // 0x8A56C
                let t = p.m.w(0xA0).wrapping_sub(1);
                p.m.sw(0xA0, t);
                if t == 0 {
                    p.b.sub = p.b.sub.wrapping_add(2);
                }
            }
            _ => self.prop_free(p, i), // 0x8A568: bra 0x8A57C
        }
    }

    /// One actor of 0x8A42A: pool A class 75 (0x483C), the variant, direction 0, the position.
    fn prop_b79_actor(&mut self, var: u16, x: i16, y: i16, z: i16) {
        let Some(j) = self.enemies.alloc() else { return };
        let n = &mut self.enemies.slots[j];
        n.b.live = 1;
        n.b.k20 = 75 * 4;
        n.b.var26 = var;
        n.b.dir = 0;
        let put = |v: &mut i32, w: i16| *v = (*v & 0xFFFF) | ((w as i32) << 16);
        put(&mut n.b.x, x);
        put(&mut n.b.y, y);
        put(&mut n.b.z, z);
    }
}
