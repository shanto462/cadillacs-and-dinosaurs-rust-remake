//! Pool A class 97 (`+$20 = 0x184`, handler 0x69EA2): the stand ins of the heroes in the clear scene of Episode 8 (M3-27). The script action 0x42
//! (0x1E650, `script.rs`) makes one for every character: at the position of each hero who plays (0x69E32) and, for the characters nobody plays, at
//! X 0x10E0 with the depth of the table 0x69E9A (0x69E66). The variant is twice the character (0 Jack, 2 Mustapha, 4 Hannah, 6 Mess); the animation
//! table is the long 0x6A010 + 4 * character (0x6A020, 0x6A08E, 0x6A0FC, 0x6A16A).
//!
//! G.-$70A9 (`script.b70a9`) holds a bit for each character that is in place (the heroes who play from the start). Sub states of mode 2 (table 0x69EE6):
//!
//! | sub | what |
//! | --- | --- |
//! | 0 | 0x69EF4: a character who is not in place takes the animation 1; then the walk of sub 2 at once |
//! | 2 | 0x69F0C: while all four bits are not set a stand in who is not in place walks right 3 pixels a frame to X 0x1180, then (0x69F3A) is in place: his bit is set, animation 0 |
//! | 4 | 0x69F50: when all four are in place and the boss (G.$4E0) has `+$A6` set: one pool A 93 piece (0x69F6A) and `+$80` = 0x78 |
//! | 6 | 0x69F84: `+$80` runs down (a byte); at the borrow the dialogue of the boss (pool B 50 variant 7, 0x84490) and `+$80` = 0xB4 |
//! | 8 | 0x69F9C: at the next borrow G.$7A9 = 1 (the camera go flag) and animation 1 |
//! | 0xA | 0x69FB6: walks right 3 pixels a frame (the three heroes' X words follow) to X 0x1200 |
//! | 0xC | 0x69FEC: the stand in of Mustapha (variant 2) makes the aura (pool C 39, 0x974FA); all walk on for ever, dragging the heroes |

use super::{Enemy, off::ATTACKING};
use crate::game::{Game, Handle};

const ANIM_PTRS: u32 = 0x6A010;
const ABSENT_Z: u32 = 0x69E9A;
const SPAWN_X: i16 = 0x10E0;

impl Game {
    /// 0x69E32: a stand in at the object `hero` (a1) for the character `id` (d0): the position words and the facing byte of the hero. A full
    /// pool makes none.
    pub fn enemy_stand_in_present(&mut self, hero: usize, id: u8) {
        let Some(i) = self.enemies.alloc() else { return };
        let (x, y, z, f) = {
            let b = &self.players[hero].b;
            (b.xi(), b.yi(), b.zi(), b.facing)
        };
        let e = &mut self.enemies.slots[i];
        e.b.live = 1;
        e.b.k20 = 0x184;
        e.b.var26 = 2 * id as u16;
        e.b.set_xi(x);
        e.b.set_yi(y);
        e.b.set_zi(z);
        e.b.facing = f;
    }

    /// 0x69E66: a stand in of a character nobody plays: X 0x10E0, Y 0, facing 1, the depth from the table 0x69E9A.
    pub fn enemy_stand_in_absent(&mut self, id: u8) {
        let Some(i) = self.enemies.alloc() else { return };
        let z = self.assets.frontend.text.i16(ABSENT_Z, 2 * id as u32);
        let e = &mut self.enemies.slots[i];
        e.b.live = 1;
        e.b.k20 = 0x184;
        e.b.var26 = 2 * id as u16;
        e.b.set_zi(z);
        e.b.set_xi(SPAWN_X);
        e.b.set_yi(0);
        e.b.facing = 1;
    }

    /// 0x69EA2 class 97 (jump table 0x69EAE: mode 0 -> 0x69EB6, 2 -> 0x69ED2, 4 and 6 -> 0x6A00A).
    pub fn enemy_class97(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.mode {
            0 => {
                e.b.mode += 2;
                e.b.anim_table = self.assets.frontend.text.u32(ANIM_PTRS, 2 * e.b.var26 as u32); // (a0,d0.w) with d0 = 2 * variant = 4 * character
                self.enemy_anim(&mut e.b, 0);
                self.c97_frame(e, h);
            }
            2 => self.c97_frame(e, h),
            _ => {
                let i = self.enemy_index(h);
                self.enemy_free_pool_a(e, i); // 0x6A00A: jmp 0x48FC
            }
        }
    }

    /// 0x69ED2: the sub state, then the draw check.
    fn c97_frame(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.sub {
            0 => self.c97_sub0(e),
            2 => self.c97_walk_in(e),
            4 => self.c97_wait_boss(e),
            6 => self.c97_delay_a(e),
            8 => self.c97_delay_b(e),
            0xA => self.c97_walk_off(e),
            0xC => self.c97_walk_on(e),
            _ => {}
        }
        self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
    }

    /// The bit of the character in G.-$70A9.
    fn c97_in_place(&self, e: &Enemy) -> bool {
        self.flow.script.b70a9 & (1 << (e.b.var26 >> 1)) != 0
    }

    /// 0x69EF4.
    fn c97_sub0(&mut self, e: &mut Enemy) {
        e.b.sub += 2;
        if !self.c97_in_place(e) {
            self.enemy_anim(&mut e.b, 1);
        }
        self.c97_walk_in(e); // falls into 0x69F0C
    }

    /// 0x69F0C: the walk in of a character nobody plays.
    fn c97_walk_in(&mut self, e: &mut Enemy) {
        if self.flow.script.b70a9 != 0xF {
            if self.c97_in_place(e) {
                return;
            }
            if e.b.xi() < 0x1180 {
                e.b.set_xi(e.b.xi().wrapping_add(3));
                self.enemy_step(&mut e.b);
                return;
            }
        }
        // 0x69F3A: in place
        e.b.sub += 2;
        self.flow.script.b70a9 |= 1 << (e.b.var26 >> 1);
        self.enemy_anim(&mut e.b, 0);
        self.c97_wait_boss(e); // falls into 0x69F50
    }

    /// 0x69F50: all four are in place and the boss has made his mark (`+$A6`).
    fn c97_wait_boss(&mut self, e: &mut Enemy) {
        if self.flow.script.b70a9 != 0xF {
            return;
        }
        let a6 = match crate::game::slot_of(self.spawn.g.killer) {
            Some(crate::game::Slot::Enemy(k)) => self.enemies.slots[k].m.b(0xA6),
            _ => 0,
        };
        if a6 == 0 {
            return;
        }
        e.b.sub += 2; // 0x69F66
        if let Some(j) = self.enemies.alloc() {
            let n = &mut self.enemies.slots[j];
            n.b.live = 1;
            n.b.k20 = 0x174;
            e.m.sb(0x80, 0x78);
        }
        self.c97_delay_a(e);
    }

    /// 0x69F84: `+$80` (a byte) runs down; at the borrow the dialogue of the boss and the next delay.
    fn c97_delay_a(&mut self, e: &mut Enemy) {
        let b = e.m.b(0x80);
        e.m.sb(0x80, b.wrapping_sub(1));
        if b != 0 {
            return;
        }
        e.b.sub += 2;
        let _ = self.prop_spawn(50, 7); // jsr 0x84490: the dialogue box of the boss (pool B 50, variant 7)
        e.m.sb(0x80, 0xB4);
        self.c97_delay_b(e); // falls into 0x69F9C
    }

    /// 0x69F9C.
    fn c97_delay_b(&mut self, e: &mut Enemy) {
        let b = e.m.b(0x80);
        e.m.sb(0x80, b.wrapping_sub(1));
        if b != 0 {
            return;
        }
        self.cam.s2.go = 1; // move.b #1,$7A9(a5)
        e.b.sub += 2;
        self.enemy_anim(&mut e.b, 1);
        self.c97_walk_off(e); // falls into 0x69FB6
    }

    /// 0x69FB6: walks right to X 0x1200, the three heroes' X words with him.
    fn c97_walk_off(&mut self, e: &mut Enemy) {
        if e.b.xi() >= 0x1200 {
            e.b.sub += 2; // 0x69FDA
            if e.b.var26 == 2 {
                self.fx_spawn_mutation_aura(&e.b); // jsr 0x974FA
            }
            self.c97_walk_on(e);
            return;
        }
        self.c97_step_right(e);
    }

    /// 0x69FEC: the same walk, without end.
    fn c97_walk_on(&mut self, e: &mut Enemy) {
        self.c97_step_right(e);
    }

    /// `addi.w #3,$8(a6)`, the X words of the three heroes, the animation step.
    fn c97_step_right(&mut self, e: &mut Enemy) {
        e.b.set_xi(e.b.xi().wrapping_add(3));
        let x = e.b.xi();
        for p in self.players.iter_mut() {
            p.b.x = (p.b.x & 0xFFFF) | ((x as u16 as i32) << 16); // move.w d0,$327C(a5), $33FC, $357C
        }
        self.enemy_step(&mut e.b);
    }

    /// 0x974FA: a pool C object of the class 39 (`+$20` = 0x9C) at X + 8, Y + 0x40 and the depth of the stand in.
    fn fx_spawn_mutation_aura(&mut self, b: &crate::game::body::Body) {
        let Some(j) = self.effects.alloc() else { return };
        let f = &mut self.effects.slots[j];
        f.b.live = 1;
        f.b.k20 = 0x9C;
        f.b.set_xi(b.xi().wrapping_add(8));
        f.b.set_yi(b.yi().wrapping_add(0x40));
        f.b.set_zi(b.zi());
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

    /// The stand in of a hero who plays waits in place; when all four characters are in place and the boss has `+$A6` set it makes one pool A 93, waits
    /// 0x78 frames, makes the dialogue of the boss (pool B 50, variant 7), waits 0xB4 frames more, sets the camera go flag and walks right 3 pixels a frame,
    /// dragging the heroes' X (the stand in of Mustapha makes the aura at X 0x1200).
    #[test]
    fn the_stand_ins_wait_for_the_boss_then_walk_off() {
        let Some(a) = assets() else { return };
        let mut g = Game::new(a);
        g.players[0].b.live = 1;
        g.players[0].b.set_xi(0x1100);
        g.players[0].b.set_zi(0x40);
        g.flow.script.b70a9 = 0b0001; // Jack plays
        g.enemy_stand_in_present(0, 0);
        for id in 1..4 {
            g.enemy_stand_in_absent(id);
        }
        let boss = 0; // a free slot: the stand ins only read its `+$A6`
        g.spawn.g.killer = g.enemies.handle(boss);
        let n_stand: Vec<usize> = (0..24).filter(|&i| g.enemies.slots[i].b.k20 == 0x184 && g.enemies.slots[i].b.live != 0).collect();
        assert_eq!(n_stand.len(), 4);
        let stand = |g: &Game, var: u16| g.enemies.slots.iter().position(|e| e.b.live != 0 && e.b.k20 == 0x184 && e.b.var26 == var).unwrap();
        assert_eq!(g.enemies.slots[stand(&g, 6)].b.xi(), 0x10E0, "an absent character starts at 0x10E0");
        // the absent characters walk to 0x1180 (96 pixels at 3 a frame), then wait for the boss
        for _ in 0..60 {
            g.enemies_update();
        }
        assert_eq!(g.flow.script.b70a9, 0xF, "all four are in place");
        assert_eq!(g.enemies.slots[stand(&g, 0)].b.sub, 4, "waiting for the boss");
        g.enemies.slots[boss].m.sb(0xA6, 1);
        g.enemies_update();
        let jack = stand(&g, 0);
        assert_eq!(g.enemies.slots[jack].b.sub, 6);
        assert!(g.enemies.slots.iter().any(|e| e.b.live != 0 && e.b.k20 == 0x174), "a pool A 93 piece");
        for _ in 0..0x79 {
            g.enemies_update();
        }
        assert_eq!(g.enemies.slots[jack].b.sub, 8);
        assert!(g.props.slots.iter().any(|p| p.b.k20 == 200), "the dialogue (pool B 50)");
        assert_eq!(g.cam.s2.go, 0);
        for _ in 0..0xB5 {
            g.enemies_update();
        }
        assert_eq!((g.cam.s2.go, g.enemies.slots[jack].b.sub), (1, 0xA));
        let x = g.enemies.slots[jack].b.xi();
        g.enemies_update();
        assert_eq!(g.enemies.slots[jack].b.xi(), x + 3);
        assert_eq!(g.players[0].b.xi(), x + 3, "the hero follows");
    }
}
