//! Pool C class 39 (`+$20` = 0x9C, handler 0x97530, frame chain 0x975AC): the flare of the clear scene of Episode 8 (M9-04). The stand in of Mustapha makes it (0x974FA, called by
//! the stand in's last step 0x69FEC, `enemy/class97.rs`) at X + 8, Y + 0x40 of the stand in. It flies right 6 pixels a frame, its 16 frames (2 ticks each) turn round; at
//! X 0x12D0 it makes a blast (pool E class 1, kind 4, 0x27DAA) and the wall break (pool B class 46, 0x84188) and is gone. Owner: effects.

use super::*;

/// 0x975AC: the first frame of the chain (the frames link on to each other and round).
const FLARE_FRAMES: u32 = 0x975AC;
/// The X at which the flare hits the wall (0x97578).
const WALL_X: i16 = 0x12D0;

impl Game {
    /// 0x97530 (mode table 0x9753E): 0 the init 0x97546 (falls into 2), 2 the flight 0x97572, 4 the hit 0x9758C, 6 the free 0x975A6.
    pub(super) fn fx39(&mut self, i: usize) {
        match self.effects.slots[i].b.mode {
            0 => {
                // 0x97546: the chain (0x160A), no shadow, `+$63` 0xFF, the box table 0x103000
                let fx = self.fx_data();
                let e = &mut self.effects.slots[i];
                e.b.mode = e.b.mode.wrapping_add(2);
                fx_load(&mut e.b, fx, fx.frame_at(FLARE_FRAMES));
                e.b.draw_flags = 0x10;
                e.b.b63 = 0xFF;
                e.off_x = (e.off_x & !0xFF) | 0xFF; // move.b #$FF,$63(a6): the low byte of the word +$62
                e.b.box_table = 0x103000;
                e.b.multi_box = false;
                self.fx39_fly(i);
            }
            2 => self.fx39_fly(i),
            4 => self.fx39_hit(i),
            _ => self.fx_free(i), // 0x975A6: jmp 0x4964
        }
    }

    /// 0x97572: X + 6 (the word); from X 0x12D0 on the hit (0x9758C), else the animation step (0x15F4) and the draw check (0x49CA).
    fn fx39_fly(&mut self, i: usize) {
        let e = &mut self.effects.slots[i];
        e.b.set_xi_keep(e.b.xi().wrapping_add(6));
        if e.b.xi() >= WALL_X {
            return self.fx39_hit(i);
        }
        let fx = self.fx_data();
        fx_tick(&mut self.effects.slots[i].b, fx);
        self.draw_list_add_check(self.effects.handle(i));
    }

    /// 0x9758C: the blast at the flare (d3 = 1), the wall break (class 46), the free.
    fn fx39_hit(&mut self, i: usize) {
        let (x, y, z) = {
            let b = &self.effects.slots[i].b;
            (b.xi(), b.yi(), b.zi())
        };
        self.item_blast_at(x, y, z, 1); // jsr 0x27DAA
        self.prop_spawn_b46(); // jsr 0x84188
        self.fx_free(i); // jmp 0x4964
    }

    /// 0x27DAA: a blast (pool E class 1, kind 4) at (x, y, z) with `+$5C` = `d3` (a full pool makes none).
    pub(crate) fn item_blast_at(&mut self, x: i16, y: i16, z: i16, d3: u16) {
        let Some(j) = self.items.alloc() else { return };
        let n = &mut self.items.slots[j];
        n.b.live = 1;
        n.b.k20 = 4;
        n.b.set_xi_keep(x);
        n.b.set_yi_keep(y);
        n.b.set_zi_keep(z);
        n.b.var26 = 4;
        n.m.sw(0x5C, d3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn rom() -> Option<&'static AssetPack> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../dino.zip");
            let path = if path.exists() { path } else { std::path::Path::new("../../../dino.zip") };
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    fn flare(g: &mut Game, x: i16) -> usize {
        let i = g.effects.alloc().expect("a free slot");
        let e = &mut g.effects.slots[i];
        e.b.live = 1;
        e.b.k20 = 0x9C;
        e.b.set_xi(x);
        e.b.set_yi(0x40);
        e.b.set_zi(0x100);
        i
    }

    /// The flare flies 6 pixels a frame (the first update already moves it), is in the draw list while it flies, and at X 0x12D0 makes the blast (pool E 1 kind 4,
    /// `+$5C` 1) and the wall break (pool B 46) and is freed (MAME ep8_jack: X 0x120E at the first record, 0x12CE at the last, gone at the next).
    #[test]
    fn the_flare_flies_and_breaks_the_wall() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut g = Game::new(a);
        let i = flare(&mut g, 0x1208);
        let mut xs = Vec::new();
        for _ in 0..40 {
            g.pool_c_update();
            if g.effects.slots[i].b.live == 0 {
                break;
            }
            xs.push(g.effects.slots[i].b.xi());
        }
        assert_eq!(xs.first(), Some(&0x120E));
        assert_eq!(xs.last(), Some(&0x12CE));
        assert!(xs.windows(2).all(|w| w[1] - w[0] == 6));
        assert_eq!(g.effects.slots[i].b.live, 0, "the flare is gone at 0x12D4");
        let blast = g.items.slots.iter().find(|n| n.b.live != 0 && n.b.k20 == 4).expect("the blast");
        assert_eq!((blast.b.xi(), blast.b.yi(), blast.b.zi(), blast.b.var26, blast.m.w(0x5C)), (0x12D4, 0x40, 0x100, 4, 1));
        assert!(g.props.slots.iter().any(|p| p.b.live != 0 && p.b.k20 == 46 * 4), "the wall break");
    }
}
