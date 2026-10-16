//! Pool B class 18 (`+$20 = 0x48`, handler 0x7B65E): the respawn air strike (M2-16, D2b 2.5).
//!
//! The player respawn (0x18C5E) creates one object with the hero's index and id. It is invisible.
//! Every 12 frames it makes a pool E class 8 bomb from the next record of the list 0x7B7A6
//! (X offset from the camera, depth offset, variant). After the 5th record (end word negative)
//! it frees itself. The bombs, explosions, white blast, shake and sounds are the pool E side
//! (`item/blast.rs`).
//!
//! | mode | sub | what |
//! | --- | --- | --- |
//! | 0 | | 0x7B672: list pointer, X = camera X, floor lookup 0x12FE4 and terrain 0x12CAC, start X = X + 0x40, start height = ground + 0x120; vertical scene forces height 0x120. Mode 2 and the first bomb in this frame. |
//! | 2 | 0 | 0x7B6E4: one bomb for the current record, then wait or free |
//! | 2 | 2 | 0x7B78E: `$80` counts down as a byte; a borrow writes `$4 = $200` (mode 2, sub 0) |
//! | 4, 6 | | 0x7B7A0: free (0x4938) |

use super::Prop;
use crate::game::item::{class, put_hi};
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// long: pointer into the record list 0x7B7A6.
    pub const LIST: usize = 0xA0;
    /// word: start X of every bomb (X after the terrain push, plus 0x40).
    pub const START_X: usize = 0xA6;
    /// word: start height of every bomb (ground + 0x120).
    pub const START_Y: usize = 0xA8;
    /// byte: wait of 10 between bombs (`subq.b`, borrow after 11 waits).
    pub const TIMER: usize = 0x80;
}

/// The table of this class (extracted exactly: `TABLES` of `dino-extract/src/enemy.rs`).
mod rom {
    /// 5 records of 3 words, then a negative end word.
    pub const LIST: u32 = 0x7B7A6;
}

impl Game {
    /// 0x7B65E pool B class 18 (jump table 0x7B66A: mode 0 -> 0x7B672, 2 -> 0x7B6DC, 4 and 6 -> 0x7B7A0).
    pub fn prop_b18(&mut self, p: &mut Prop, h: Handle, i: usize) {
        match p.b.mode {
            0 => {
                self.prop_b18_init(p, h);
                self.prop_b18_frame(p, h, i); // falls into 0x7B6DC
            }
            2 => self.prop_b18_frame(p, h, i),
            _ => self.prop_free(p, i), // 0x7B7A0: jmp 0x4938
        }
    }

    /// 0x7B672.
    fn prop_b18_init(&mut self, p: &mut Prop, h: Handle) {
        p.m.sl(f::LIST, rom::LIST);
        put_hi(&mut p.b.x, self.cam.s2.cam_x as i16); // 0x7B67A move.w $744(a5),$8(a6)
        self.terrain_floor_lookup(&mut p.b); // 0x7B680 jsr 0x12FE4
        put_hi(&mut p.b.y, p.b.floor.wrapping_add(p.b.floor2));
        put_hi(&mut p.b.z, (self.cam.s2.cam_y as i16).wrapping_sub(p.b.floor));
        let _ = self.terrain_player(&mut p.b, h); // 0x7B69E jsr 0x12CAC
        p.m.sw(f::START_X, (p.b.xi() as u16).wrapping_add(0x40));
        p.m.sw(f::START_Y, (p.b.yi() as u16).wrapping_add(0x120));
        if self.w.vertical_scene != 0 {
            // 0x7B6C4
            put_hi(&mut p.b.y, 0);
            p.b.floor = 0;
            p.b.floor2 = 0;
            p.m.sw(f::START_Y, 0x120);
        }
        p.b.mode += 2; // 0x7B6D8 addq.b #2,$4(a6)
    }

    /// 0x7B6DC.
    fn prop_b18_frame(&mut self, p: &mut Prop, h: Handle, i: usize) {
        if p.b.sub != 0 {
            // 0x7B78E: subq.b #1,$80(a6) ; bcs 0x7B798
            let (t, borrow) = p.m.b(f::TIMER).overflowing_sub(1);
            p.m.sb(f::TIMER, t);
            if borrow {
                p.b.mode = 2;
                p.b.sub = 0; // 0x7B798 move.w #$200,$4(a6)
            }
            return;
        }
        self.prop_b18_spawn(p, h, i);
    }

    /// 0x7B6E4: one bomb for the current record.
    fn prop_b18_spawn(&mut self, p: &mut Prop, h: Handle, i: usize) {
        let rec = p.m.l(f::LIST);
        let xoff = self.assets.enemy.t.i16(rec, 0);
        let x = (self.cam.s2.cam_x as i16).wrapping_add(xoff);
        put_hi(&mut p.b.x, x);
        self.terrain_floor_lookup(&mut p.b); // 0x7B6F2
        put_hi(&mut p.b.y, p.b.floor.wrapping_add(p.b.floor2));
        let cam_y = if self.w.vertical_scene != 0 { 0 } else { self.cam.s2.cam_y as i16 };
        put_hi(&mut p.b.z, cam_y.wrapping_sub(p.b.floor).wrapping_add(0x20));
        let _ = self.terrain_player(&mut p.b, h); // 0x7B71E
        let rec = p.m.l(f::LIST);
        let dz = self.assets.enemy.t.i16(rec, 2);
        let z = p.b.zi().wrapping_add(dz);
        put_hi(&mut p.b.z, z); // 0x7B72C add.w d0,$10(a6)
        if let Some(j) = self.items.alloc() {
            // 0x7B730 jsr 0x4818 ; bcs 0x7B776
            let n = &mut self.items.slots[j];
            n.b.live = 1;
            n.b.who = p.b.who;
            n.b.id = p.b.id;
            n.b.k20 = class::PROJECTILE * 4;
            put_hi(&mut n.b.x, p.m.w(f::START_X) as i16);
            put_hi(&mut n.b.y, p.m.w(f::START_Y) as i16);
            put_hi(&mut n.b.z, p.b.zi());
            n.m.sw(0xA0, p.b.xi() as u16);
            n.m.sw(0xA2, p.b.yi() as u16);
            n.m.sw(0xA4, p.b.zi() as u16);
            n.b.var26 = self.assets.enemy.t.u16(rec, 4);
        }
        let next = rec.wrapping_add(6);
        p.m.sl(f::LIST, next);
        if self.assets.enemy.t.i16(next, 0) < 0 {
            return self.prop_free(p, i); // 0x7B77E bmi 0x7B7A0
        }
        p.m.sb(f::TIMER, 0x0A);
        p.b.sub += 2;
    }
}

#[cfg(test)]
mod tests {
    use super::f;
    use super::rom;
    use crate::game::item::class;
    use crate::game::Game;
    use dino_assets::AssetPack;

    /// Documented record list of D2b 2.5 (X offset, depth offset, variant). Built as words, not as a ROM copy.
    const RECORDS: [(i16, i16, u16); 5] = [(320, 64, 0), (192, 32, 0), (64, 48, 0), (128, 0, 0), (256, 16, 1)];

    fn blob() -> Vec<u8> {
        let mut b = Vec::new();
        for (x, z, v) in RECORDS {
            b.extend_from_slice(&x.to_be_bytes());
            b.extend_from_slice(&z.to_be_bytes());
            b.extend_from_slice(&v.to_be_bytes());
        }
        b.extend_from_slice(&(-1i16).to_be_bytes());
        b
    }

    fn game() -> Game {
        let mut a = AssetPack::default();
        a.enemy.t.blobs = vec![(rom::LIST, blob())];
        a.terrain.shapes = vec![[0; 8]; 64];
        Game::new(Box::leak(Box::new(a)))
    }

    fn strike(g: &mut Game) -> usize {
        let h = g.spawn_prop(18, 0, None).unwrap();
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
        g.props.slots[i].b.who = 0;
        g.props.slots[i].b.id = 2;
        i
    }

    fn tick(g: &mut Game, i: usize) {
        let h = g.props.handle(i);
        let mut p = std::mem::take(&mut g.props.slots[i]);
        if p.b.live != 0 {
            g.prop_b18(&mut p, h, i);
        }
        g.props.slots[i] = p;
    }

    fn bombs(g: &Game) -> Vec<(i16, i16, i16, u16, i16, i16)> {
        g.items
            .slots
            .iter()
            .filter(|it| it.b.live != 0 && it.b.k20 == class::PROJECTILE * 4)
            .map(|it| (it.b.xi(), it.b.yi(), it.b.zi(), it.b.var26, it.m.w(0xA0) as i16, it.m.w(0xA2) as i16))
            .collect()
    }

    /// D2b S2: five bombs at R, R+12 ... R+48, start X = camera X + 0x40, start Y = ground + 0x120,
    /// depths camera Y - floor + 0x20 plus the record, 5th variant 1, then the controller frees.
    #[test]
    fn five_bombs_follow_the_record_list() {
        let mut g = game();
        g.cam.s2.cam_x = 897;
        g.cam.s2.cam_y = 256;
        let i = strike(&mut g);
        let mut born = Vec::new();
        for f in 0..50 {
            tick(&mut g, i);
            let n = bombs(&g).len();
            if n > born.len() {
                born.push(f);
            }
        }
        assert_eq!(born, vec![0, 12, 24, 36, 48]);
        assert_eq!(g.props.slots[i].b.live, 0, "freed after the 5th record");
        let mut b = bombs(&g);
        b.sort_by_key(|t| t.4); // target X
        let start_x = 897i16.wrapping_add(0x40);
        let start_y = 0x120i16;
        let base_z = 256i16.wrapping_add(0x20);
        let mut want: Vec<(i16, i16, i16, u16, i16, i16)> = RECORDS
            .iter()
            .map(|&(xoff, dz, var)| (start_x, start_y, base_z.wrapping_add(dz), var, 897i16.wrapping_add(xoff), 0))
            .collect();
        want.sort_by_key(|t| t.4);
        assert_eq!(b, want);
        assert_eq!(bombs(&g).iter().filter(|t| t.3 == 1).count(), 1);
        assert_eq!(g.props.slots[i].m.w(f::START_X) as i16, start_x);
        assert_eq!(g.props.slots[i].m.w(f::START_Y) as i16, start_y);
    }

    /// The bomb copies +$02 / +$03 of the controller (0x7B740).
    #[test]
    fn the_bomb_keeps_the_player_index_and_id() {
        let mut g = game();
        let i = strike(&mut g);
        tick(&mut g, i);
        let it = g.items.slots.iter().find(|it| it.b.live != 0).unwrap();
        assert_eq!((it.b.who, it.b.id, it.b.k20), (0, 2, class::PROJECTILE * 4));
    }

    /// A full pool E still consumes the record (0x7B736 bcs 0x7B776) and the controller ends.
    #[test]
    fn a_full_pool_still_walks_the_list() {
        let mut g = game();
        while g.items.alloc().is_some() {}
        let i = strike(&mut g);
        for _ in 0..50 {
            tick(&mut g, i);
        }
        assert_eq!(g.props.slots[i].b.live, 0);
        assert!(g.items.slots.iter().all(|it| it.b.k20 != class::PROJECTILE * 4 || it.b.live == 0));
    }

    /// Vertical scene (G.$4DC): start height 0x120, camera Y treated as 0 for the bomb depth.
    #[test]
    fn the_vertical_scene_uses_a_flat_floor() {
        let mut g = game();
        g.w.vertical_scene = 1;
        g.cam.s2.cam_x = 100;
        g.cam.s2.cam_y = 256;
        let i = strike(&mut g);
        tick(&mut g, i);
        let it = g.items.slots.iter().find(|it| it.b.live != 0).unwrap();
        assert_eq!(it.b.yi(), 0x120);
        assert_eq!(it.b.zi(), 0i16.wrapping_add(0x20).wrapping_add(64));
    }

    /// With the ROM table (skipped without `dino.zip`): every record byte is extracted.
    #[test]
    fn it_reads_the_extracted_table() {
        let path = std::path::Path::new("../../../dino.zip");
        if !path.exists() {
            return;
        }
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads")));
        let t = &assets.enemy.t;
        for (k, &(x, z, v)) in RECORDS.iter().enumerate() {
            let off = (k as u32) * 6;
            assert_eq!((t.i16(rom::LIST, off), t.i16(rom::LIST, off + 2), t.u16(rom::LIST, off + 4)), (x, z, v));
        }
        assert!(t.i16(rom::LIST, 30) < 0, "end word");
        let mut g = Game::new(assets);
        g.cam.s2.cam_x = 897;
        g.cam.s2.cam_y = 256;
        let i = strike(&mut g);
        tick(&mut g, i);
        let it = g.items.slots.iter().find(|it| it.b.live != 0 && it.b.k20 == class::PROJECTILE * 4);
        assert!(it.is_some(), "the first bomb is made from the ROM list");
    }
}
