//! Pool B class 12, the hall double door and the hall window of Episode 1 area 1 (0x78B52).
//!
//! `+$26` selects the routine (pointer table 0x78B66): var 0 is 0x78B6E, var 4 is 0x78FB8.
//! Any other variant frees the slot. The window does not draw a sprite (no `jsr 0x49CA`).

use super::tile_block::{BlockTable, TileMode, TilePos};
use super::Prop;
use crate::game::Game;

/// 0x78B86: three pictures of 9 x 9 cells at scroll2 0x90F288. The pointer table is 0x78BDA
/// (0x78BE6, 0x78D2A, 0x78E6E). Each picture is one block every 4 updates. The third picture
/// (the black doorway) is the last, and the next update frees the slot (0x78FB2).
const HALL_DOOR: BlockTable = BlockTable {
    table: 0x78BDA,
    count: 3,
    at: TilePos::at(0x90F288),
    cols: 9,
    rows: 9,
    mode: TileMode::PairsMerge(0xFFC0),
    period: 4,
    looped: false,
};

/// 0x79098, as cell addresses. The routine starts at the attribute word 0x90FE16 and writes
/// six cells (`and #$FFE0 ; or #$10`). 0x790CE and 0x790F2 are after the `rts` and have no caller.
const WINDOW_ATTR: &[u32] = &[0x90FE14, 0x90FE18, 0x90FE50, 0x90FE54, 0x90FE58, 0x90FE5C];

/// 0x79152: ten records of 16 bytes for 0x93362 (vx, ax, vy, ay, vz, az, var, pad).
const SHARDS: u32 = 0x79152;
const SHARD_COUNT: u32 = 10;

/// 0x79200: two hit boxes. Index 1 is the window (0x78FC6 stores body box 1).
const BOX: u32 = 0x79200;

impl Game {
    /// 0x78B52.
    pub fn prop_b12(&mut self, p: &mut Prop, i: usize) {
        match p.b.var26 {
            0 => self.prop_b12_door(p, i),
            4 => self.prop_b12_window(p, i),
            _ => self.prop_free(p, i),
        }
    }

    /// 0x78B6E. Mode 0 (0x78B76) sets mode 2, `+$A0 = 1` and `+$A2 = 0`, then returns. It does
    /// not fall into mode 2. Mode 2 is 0x78B86. Mode 4 and mode 6 are 0x78FB2 (`jmp 0x4938`).
    fn prop_b12_door(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                HALL_DOOR.start(p, 1);
                p.b.mode = 2;
            }
            2 => {
                if self.prop_block_table_step(p, &HALL_DOOR) {
                    p.b.mode = 4; // 0x78BA6, the same update as the last picture
                }
            }
            _ => self.prop_free(p, i),
        }
    }

    /// 0x78FB8. Mode 0 (0x78FC0) falls into mode 2 (0x78FEA). Mode 4 is the break (0x7900E).
    /// Mode 6 (0x791F2) counts `+$A0` down from 0x14 and frees on 0.
    fn prop_b12_window(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.visible = 1;
                p.hit_init(BOX, 8);
                p.b.mode = 2;
                self.prop_b12_window_idle(p, i);
            }
            2 => self.prop_b12_window_idle(p, i),
            4 => self.prop_b12_window_break(p),
            6 => {
                let n = p.m.b(0xA0).wrapping_sub(1); // subq.b #1,+$A0
                p.m.sb(0xA0, n);
                if n == 0 {
                    self.prop_free(p, i);
                }
            }
            _ => self.prop_free(p, i),
        }
    }

    /// 0x78FEA. Sub 0 (0x78FFC) arms when G.$506 is set and does not register that frame.
    /// Sub 2 (0x79008) is `jmp 0x4BFA`. The sub stays 2 after G.$506 clears.
    fn prop_b12_window_idle(&mut self, p: &mut Prop, i: usize) {
        match p.b.sub {
            0 => {
                if self.w.no_clamp != 0 {
                    p.b.sub = 2;
                }
            }
            2 => self.prop_register_hit(self.props.handle(i)),
            _ => {}
        }
    }

    /// 0x7900E, in this order: codes (0x7903A), six attributes (0x79098), the move and the ten
    /// shards (0x7912C), then sound 0x37 (0xA63C), then `+$A0 = 0x14` and mode 6.
    fn prop_b12_window_break(&mut self, p: &mut Prop) {
        self.prop_tile_block(TilePos::at(0x90FDCC), 3, 7, 0x7906E, TileMode::Codes);
        for &cell in WINDOW_ATTR {
            self.prop_tile_attr(TilePos::at(cell), 0xFFE0, 0x10);
        }
        // 0x7912C writes the high words. A stage spawn is integral, so this is the same.
        p.b.set_xi(0x38A);
        p.b.set_yi(0x60);
        p.b.set_zi(0x30);
        self.prop_b12_shards(p);
        self.snd_request(0x37, 0x7_2020, 0x1000_0000);
        p.m.sb(0xA0, 0x14);
        p.b.mode = 6;
    }

    /// 0x93362, ten times (0x79144 `moveq #$9,d5` then `dbra`). A failed alloc sets carry and
    /// returns without advancing the record, so the next success reads the same record.
    fn prop_b12_shards(&mut self, p: &Prop) {
        let mut off = 0u32;
        for _ in 0..SHARD_COUNT {
            let Some(j) = self.effects.alloc() else {
                continue;
            };
            let vx = self.assets.enemy.t.i16(SHARDS, off);
            let ax = self.assets.enemy.t.i16(SHARDS, off + 2);
            let vy = self.assets.enemy.t.i16(SHARDS, off + 4);
            let ay = self.assets.enemy.t.i16(SHARDS, off + 6);
            let vz = self.assets.enemy.t.i16(SHARDS, off + 8);
            let az = self.assets.enemy.t.i16(SHARDS, off + 10);
            let var = self.assets.enemy.t.u16(SHARDS, off + 12);
            let e = &mut self.effects.slots[j].b;
            e.live = 1;
            e.k20 = 0x28;
            e.set_xi(p.b.xi());
            e.set_yi(p.b.yi());
            e.set_zi(p.b.zi());
            e.vx = vx;
            e.ax = ax;
            e.vy = vy;
            e.ay = ay;
            e.vz = vz;
            e.az = az;
            e.var26 = var;
            off += 16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::snd::SndRequest;
    use dino_assets::enemy::ByteTables;
    use dino_assets::stage::Cell;
    use dino_assets::AssetPack;

    fn words(w: &[u16]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn longs(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    /// Pointers at 0x78BDA and three 9 x 9 pictures. Picture `n`, cell `k`: code `0x1000*(n+1)+k`,
    /// attribute low bits `0x15`. Not the ROM pictures.
    fn door_blob() -> Vec<u8> {
        let mut b = longs(&[0x78BE6, 0x78D2A, 0x78E6E]);
        for n in 0..3u16 {
            for k in 0..81u16 {
                b.extend_from_slice(&(0x1000 * (n + 1) + k).to_be_bytes());
                b.extend_from_slice(&0x0015u16.to_be_bytes());
            }
        }
        assert_eq!(b.len(), 984);
        b
    }

    fn shard_blob() -> Vec<u8> {
        let mut w = Vec::new();
        for i in 0..10u16 {
            // vx, ax, vy, ay, vz, az, var, pad
            w.extend_from_slice(&[100 + i, 10, 20, 0xFFE0u16, 30, 0, 4 + i, 0]);
        }
        words(&w)
    }

    fn game() -> Game {
        let codes: Vec<u16> = (0..21).map(|k| 0x2100 + k).collect();
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables {
            blobs: vec![
                (0x78BDA, door_blob()),
                (0x7906E, words(&codes)),
                (0x79152, shard_blob()),
                (0x79200, words(&[0, 0, 0, 0, 0, 0, 0xFFD0, 80, 0, 48, 0xFFD0, 136])),
            ], ..Default::default()
        };
        Game::new(Box::leak(Box::new(a)))
    }

    fn cell(g: &Game, addr: u32) -> (u16, u16) {
        let c = g.vram.scroll2[((addr - 0x90C000) / 4) as usize];
        (c.code, c.attr)
    }

    fn paint(g: &mut Game, at: u32, n: usize, attr: u16) {
        for k in 0..n {
            g.vram.scroll2[((at - 0x90C000) / 4) as usize + k] = Cell { code: 0x1111, attr };
        }
    }

    fn live<'a>(g: &'a Game, var: u16) -> Option<&'a Prop> {
        g.props.slots.iter().find(|s| s.b.live != 0 && s.b.k20 == 12 * 4 && s.b.var26 == var)
    }

    /// Picture 0 on update 2, picture 1 on update 6, picture 2 and mode 4 on update 10, free on 11.
    #[test]
    fn the_hall_door_goes_black_on_the_tenth_update() {
        let mut g = game();
        paint(&mut g, 0x90F288, 9 * 16, 0xABCD);
        g.spawn_prop(12, 0, Some((0x60, 0x290, 0x120))).unwrap();
        g.pool_b_update();
        let p = live(&g, 0).unwrap();
        assert_eq!((p.b.mode, p.m.b(0xA0), p.m.w(0xA2)), (2, 1, 0));
        assert_eq!(cell(&g, 0x90F288).0, 0x1111, "mode 0 does not write");

        g.pool_b_update();
        let at = TilePos::at(0x90F288);
        assert_eq!(cell(&g, at.0), (0x1000, 0xABD5));
        assert_eq!(cell(&g, at.right(8).down(8).0), (0x1000 + 80, 0xABD5));
        assert_eq!(cell(&g, at.down(9).0).0, 0x1111, "row 9 is outside the 9-row block");
        assert_eq!(live(&g, 0).unwrap().b.mode, 2);

        for _ in 0..3 {
            g.pool_b_update();
        }
        assert_eq!(cell(&g, at.0).0, 0x1000, "picture 0 holds for the period");
        g.pool_b_update();
        assert_eq!(cell(&g, at.0).0, 0x2000);
        assert_eq!(cell(&g, at.right(8).down(8).0).0, 0x2000 + 80);

        for _ in 0..3 {
            g.pool_b_update();
        }
        assert_eq!(cell(&g, at.0).0, 0x2000);
        g.pool_b_update();
        assert_eq!(cell(&g, at.0), (0x3000, 0xABD5));
        assert_eq!(cell(&g, at.right(8).down(8).0).0, 0x3000 + 80);
        assert_eq!(live(&g, 0).unwrap().b.mode, 4, "the last picture and mode 4 are one update");
        g.pool_b_update();
        assert!(live(&g, 0).is_none());
        assert_eq!(cell(&g, at.0).0, 0x3000, "the black picture stays after the free");
    }

    #[test]
    fn an_unknown_variant_frees_the_slot() {
        let mut g = game();
        g.spawn_prop(12, 2, None).unwrap();
        g.pool_b_update();
        assert!(g.props.slots.iter().all(|s| s.b.live == 0 || s.b.k20 != 12 * 4));
    }

    /// Mode 0 falls into sub 0. G.$506 = 0 never arms it, so a later hit mode is not reached.
    #[test]
    fn the_window_waits_for_the_script_flag() {
        let mut g = game();
        g.spawn_prop(12, 4, Some((0x370, 0x30, 0x40))).unwrap();
        g.pool_b_update();
        let p = live(&g, 4).unwrap();
        assert_eq!(p.b.state(), 0x0200_0000);
        assert_eq!(
            (p.b.visible, p.b.body_box, p.b.attack_box, p.b.box_table, p.b.hp, p.b.hp_prev),
            (1, 1, 0, BOX, 8, 8)
        );
        assert!(g.combat.get(0x671C).is_empty());
        for _ in 0..30 {
            g.pool_b_update();
        }
        assert_eq!(live(&g, 4).unwrap().b.state(), 0x0200_0000);
        assert_eq!(cell(&g, 0x90FDCC).0, 0);
    }

    /// The arming update sets sub 2 and does not register. The next update does, and stays
    /// registered after G.$506 clears. The frame reset 0x105E8 is `combat_lists_reset`.
    #[test]
    fn the_window_registers_on_the_update_after_the_flag() {
        let mut g = game();
        let h = g.spawn_prop(12, 4, Some((0x370, 0x30, 0x40))).unwrap();
        g.pool_b_update();
        g.w.no_clamp = 1;
        g.pool_b_update();
        assert_eq!(live(&g, 4).unwrap().b.state(), 0x0202_0000);
        assert!(g.combat.get(0x671C).is_empty(), "0x78FFC returns before 0x4BFA");
        g.pool_b_update();
        assert_eq!(g.combat.get(0x671C), &[h]);
        g.w.no_clamp = 0;
        g.combat_lists_reset();
        g.pool_b_update();
        assert_eq!(live(&g, 4).unwrap().b.sub, 2);
        assert_eq!(g.combat.get(0x671C), &[h]);
    }

    /// Mode 4 is the hit of the frame before. Tiles, six attributes, the move, ten shards, then
    /// the sound. Twenty mode-6 updates later the slot frees and the tiles stay.
    #[test]
    fn the_window_breaks_in_the_original_order() {
        let mut g = game();
        paint(&mut g, 0x90FD00, 0x80, 0xFFFF);
        g.spawn_prop(12, 4, Some((0x370, 0x30, 0x40))).unwrap();
        g.pool_b_update();
        live_mut(&mut g).b.mode = 4;
        g.pool_b_update();

        let at = TilePos::at(0x90FDCC);
        for col in 0..3 {
            for row in 0..7 {
                let (code, attr) = cell(&g, at.right(col).down(row).0);
                assert_eq!(code, 0x2100 + (col * 7 + row) as u16, "col {col} row {row}");
                let marked = WINDOW_ATTR.contains(&at.right(col).down(row).0);
                assert_eq!(attr, if marked { 0xFFF0 } else { 0xFFFF }, "col {col} row {row}");
            }
        }
        assert_eq!(cell(&g, 0x90FDC8), (0x1111, 0xFFFF), "the cell before the block is untouched");
        assert_eq!(WINDOW_ATTR.iter().filter(|a| cell(&g, **a).1 == 0xFFF0).count(), 6);

        let p = live(&g, 4).unwrap();
        assert_eq!((p.b.xi(), p.b.yi(), p.b.zi(), p.b.mode, p.m.b(0xA0)), (0x38A, 0x60, 0x30, 6, 0x14));
        let mut shards: Vec<_> = g.effects.slots.iter().filter(|e| e.b.live != 0).map(|e| e.b.clone()).collect();
        shards.sort_by_key(|b| b.vx);
        assert_eq!(shards.len(), 10);
        for (i, b) in shards.iter().enumerate() {
            let i = i as i16;
            assert_eq!(b.k20, 0x28);
            assert_eq!((b.xi(), b.yi(), b.zi()), (0x38A, 0x60, 0x30));
            assert_eq!((b.vx, b.ax, b.vy, b.ay, b.vz, b.az, b.var26), (100 + i, 10, 20, -32, 30, 0, (4 + i) as u16));
        }
        assert_eq!(g.snd_take_requests(), vec![SndRequest { code: 0x37, d1: 0x7_2020, d2: 0x1000_0000 }]);

        for _ in 0..19 {
            g.pool_b_update();
        }
        assert_eq!(live(&g, 4).unwrap().b.mode, 6);
        g.pool_b_update();
        assert!(live(&g, 4).is_none());
        assert_eq!(cell(&g, at.0).0, 0x2100);
    }

    /// Pool C empty (0x93362 carry): no shard, and the tiles, the move and the sound still run.
    #[test]
    fn an_empty_pool_still_breaks_the_window() {
        let mut g = game();
        while g.effects.free_count() > 0 {
            g.effects.alloc().unwrap();
        }
        paint(&mut g, 0x90FDCC, 1, 0xFFFF);
        g.spawn_prop(12, 4, Some((1, 2, 3))).unwrap();
        live_mut(&mut g).b.mode = 4;
        g.pool_b_update();
        assert!(g.effects.slots.iter().all(|e| e.b.k20 != 0x28 || e.b.live == 0));
        assert_eq!(cell(&g, 0x90FDCC).0, 0x2100);
        assert_eq!(g.w.sounds, vec![0x37]);
        assert_eq!(live(&g, 4).map(|p| (p.b.mode, p.b.xi())), Some((6, 0x38A)));
    }

    fn live_mut(g: &mut Game) -> &mut Prop {
        g.props.slots.iter_mut().find(|s| s.b.live != 0 && s.b.k20 == 12 * 4).unwrap()
    }
}
