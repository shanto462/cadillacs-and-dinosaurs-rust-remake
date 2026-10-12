//! Pool B class 86 (`+$20 = 0x158`, handler 0x8B428): the Cadillac that drives through the ranking screen
//! (M1-07; the same handler runs the car of the ending, whose other parts are not ported here). The ranking
//! routine creates it last, at X camera - 0x80, Y camera + 0x20, depth G.$74C.
//!
//! Mode 0 (0x8B43C) makes it present (+$02 = 0xFF, +$63 = 0xFF, +$7D = 0xFF, +$01 = 1), clears the vertical
//! scene flag G.$4DC, starts animation 0 of the table 0x8B5E6 with 12 pixels a frame to the right (+$14 =
//! 0xC00), creates the two heroes (pool A class 17 with the variants 0 and 1, +$AC = this car) and queues the
//! music 0x80 (0xA49E); it does not move in that frame. Mode 2 (0x8B4FC), sub state 0: the car moves
//! (0x189C), its animation steps (0x121E) and it goes into the draw list (0x49C0); when it is 0xA0 or more
//! right of the scroll2 camera it queues the ambient sound 0x70 (0xA4B2), slows to 0x830 and goes to sub state
//! 2 (the same without the test: the car keeps pace with the camera). Mode 4 and 6 (0x8B55C) queue the stop
//! code 0xFF00 (0xA5D2) and free it.

use super::Prop;
use crate::game::Game;

/// Offsets of this class.
mod f {
    /// byte: frames to go in sub state 2 (counted down, unused: the instruction after it is a nop).
    pub const TIMER: usize = 0x80;
    /// long: the object this one is a part of (the heroes read the car through it).
    pub const OWNER: usize = 0xAC;
}

const ANIM_TABLE: u32 = 0x8B5E6;

impl Game {
    /// 0x8B428 pool B class 86 (jump table 0x8B434: mode 0 -> 0x8B43C, 2 -> 0x8B4FC, 4 and 6 -> 0x8B55C).
    pub fn prop_class86(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => self.prop_c86_init(p, i),
            2 => self.prop_c86_run(p, i),
            _ => {
                self.sound(0xFF00); // 0xA5D2
                self.prop_free(p, i); // jmp 0x4938
            }
        }
    }

    /// 0x8B43C.
    fn prop_c86_init(&mut self, p: &mut Prop, i: usize) {
        p.b.mode = 2;
        p.m.sb(crate::game::enemy::off::ATTACKING, 0xFF); // +$7D
        p.b.b63 = 0xFF;
        p.b.who = 0xFF;
        p.b.visible = 1;
        self.w.vertical_scene = 0; // clr.b $4DC(a5)
        p.b.anim_table = ANIM_TABLE;
        let sprites = &self.assets.sprites;
        p.b.start_anim(sprites, 0); // jsr 0x120E
        p.b.vx = 0xC00;
        (p.b.ax, p.b.vy, p.b.ay, p.b.vz, p.b.az) = (0, 0, 0, 0, 0);
        // the two heroes: +$00 = 1, +$20 = 0x44, position words and direction of the car, +$26 = 0 / 1
        let me = self.props.handle(i);
        for var in 0..2u16 {
            let Some(k) = self.enemies.alloc() else { continue };
            let e = &mut self.enemies.slots[k];
            e.b.live = 1;
            e.b.k20 = 0x44;
            e.b.set_xi(p.b.xi());
            e.b.set_yi(p.b.yi());
            e.b.set_zi(p.b.zi());
            e.b.var26 = var;
            e.b.dir = p.b.dir;
            e.m.sl(f::OWNER, me as u32);
        }
        self.sound(0x80); // 0xA49E
    }

    /// 0x8B4FC.
    fn prop_c86_run(&mut self, p: &mut Prop, i: usize) {
        if p.b.sub == 0 {
            // 0x8B50E: until the car is 0xA0 right of the camera
            let d0 = self.cam.s2.cam_x.wrapping_add(0xA0) as i16;
            if d0 <= p.b.xi() {
                p.b.sub += 2;
                self.sound(0x70); // 0xA4B2
                p.b.vx = 0x830;
                p.m.sb(f::TIMER, 0x3C);
            }
        } else {
            p.m.sb(f::TIMER, p.m.b(f::TIMER).wrapping_sub(1)); // 0x8B544
        }
        p.b.motion(); // jsr 0x189C
        p.b.tick_anim(&self.assets.sprites); // jsr 0x121E
        // jmp 0x49C0: in the draw list when the on-screen flag is set (`+$01` stays 1 here)
        if p.b.visible != 0 {
            let h = self.props.handle(i);
            self.draw_list.push(((p.b.zi() as u16 as u32) << 16) | h as u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;
    use std::path::Path;
    use std::sync::OnceLock;

    fn assets() -> Option<&'static AssetPack> {
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// The first update makes the two heroes and queues the music without moving; then the car moves 12
    /// pixels a frame until it is 0xA0 right of the scroll2 camera, where it queues 0x70 once and slows to
    /// 0x830 (8.19 pixels a frame); mode 4 queues the stop code and frees it.
    #[test]
    fn the_cadillac_drives_in_and_keeps_pace() {
        if assets().is_none() {
            return;
        }
        let mut g = Game::new(assets().unwrap());
        let i = g.props.alloc().unwrap();
        g.props.slots[i].b.live = 1;
        g.props.slots[i].b.k20 = 0x158;
        g.props.slots[i].b.set_xi(-0x80);
        g.props.slots[i].b.set_yi(0x60);
        g.pool_b_update();
        assert_eq!(g.w.sounds, [0x80]);
        let heroes: Vec<u16> = g.enemies.slots.iter().filter(|e| e.b.live != 0).map(|e| e.b.var26).collect();
        assert_eq!(heroes, [1, 0], "pool A slot 22 and 23");
        assert_eq!(g.props.slots[i].b.xi(), -0x80, "mode 0 does not move");
        g.w.sounds.clear();
        let mut ambient = None;
        for n in 1..=90 {
            g.draw_list.clear();
            g.pool_b_update();
            if g.w.sounds.contains(&0x70) {
                ambient.get_or_insert(n);
            }
        }
        // the test is before the move: 0xA0 + 0x80 = 0x120 pixels at 12 a frame, 24 moves, so in the 25th update
        assert_eq!(ambient, Some(25));
        assert_eq!(g.w.sounds.iter().filter(|&&s| s == 0x70).count(), 1);
        assert_eq!(g.props.slots[i].b.vx, 0x830);
        g.props.slots[i].b.mode = 4;
        g.w.sounds.clear();
        g.pool_b_update();
        assert_eq!((g.w.sounds.clone(), g.props.slots[i].b.live), (vec![0xFF00], 0));
    }
}
