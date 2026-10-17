//! Pool B classes 36, 37 and 38: the sewer waterfall (0x8217E, 0x82372, 0x8279C).
//!
//! End of Episode 8 area 1. The level list spawns them when the camera reaches the triggers
//! (B36 var 0 at X 0xC20 trigger 0xA70, B36 var 4 at X 0xD20 trigger 0xB70, B37 and B38 at
//! X 0xD20 trigger 0xA70). Script 22 walks the heroes off the edge while these objects run.
//! The slow fade is camera mode 9 (0x2120E), not a call from these classes. The cave is the
//! next area.
//!
//! Each class: mode 0 only advances to mode 2. Mode 2 frees on the next update once
//! `(X + 0x100) < camX` (signed `blt` against G.$744). Otherwise, on a frame G.$7AE differs
//! from G.$7AF, it rewrites scroll2 code words. The attribute word stays. Mode 4 is
//! `jmp 0x4938`. No sprite, no random number, no sound.
//!
//! The picture tables are ROM data (`dino-extract` `TABLES`). 0x90C038 and 0x90C438 are the
//! scroll2 destinations of B36, not tables.

use super::tile_block::TilePos;
use super::Prop;
use crate::game::Game;

/// 0x821D6 and the same shape in the B36 var 4 writer (0x8223E): one code word, then this many
/// bytes added to the video address. The last add is the original's and writes nothing after it.
const B36_EDGE: &[u16] = &[4, 0x3BC, 4, 0];
const B36_SHAFT: &[u16] = &[4, 0x38, 4, 4, 0x38, 4, 4, 0x34, 4, 4, 4, 0x34, 4, 4, 4, 0x34, 4, 4, 4, 0x34];

impl Game {
    /// 0x8217E pool B class 36, the water over the edge (two blocks).
    pub fn prop_b36(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => p.b.mode = p.b.mode.wrapping_add(2), // 0x82192
            2 => {
                // 0x82198. Var 0 is the lip at 0x90C038 (0x821C0, 4 words). Any other var is the
                // shaft block at 0x90C438 (0x82228, 20 words). The spawn list uses 0 and 4.
                if self.prop_waterfall_repaint(p) {
                    if p.b.var26 == 0 {
                        let pic = self.prop_clock_pic(0x821E8); // 0x821CC
                        self.prop_code_gaps(0x90C038, pic, B36_EDGE);
                    } else {
                        let pic = self.prop_clock_pic(0x8229C); // 0x82234
                        self.prop_code_gaps(0x90C438, pic, B36_SHAFT);
                    }
                }
            }
            _ => self.prop_free(p, i), // 0x8236C
        }
    }

    /// 0x82372 pool B class 37, two strips of 16 code words. Vars 0, 4 and 8 pick the row
    /// (Z 0x300, 0x200, 0x100): the destination long at 0x823EE and the picture table at 0x823FA.
    pub fn prop_b37(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => p.b.mode = p.b.mode.wrapping_add(2), // 0x82386
            2 => {
                if self.prop_waterfall_repaint(p) {
                    self.prop_b37_write(p); // 0x823AE
                }
            }
            _ => self.prop_free(p, i), // 0x82796
        }
    }

    /// 0x8279C pool B class 38, 80 code words a step. Vars 0 and 1 are the top row (Z 0x300),
    /// 2 and 3 the middle (Z 0x200), 4 and 5 the bottom (Z 0x100). An even var writes the 80
    /// words downward (0x827E2). An odd var writes the same picture backwards, five columns of
    /// 16 going up (0x8280E).
    pub fn prop_b38(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => p.b.mode = p.b.mode.wrapping_add(2), // 0x827B0
            2 => {
                if self.prop_waterfall_repaint(p) {
                    self.prop_b38_write(p); // 0x827D8
                }
            }
            _ => self.prop_free(p, i), // 0x830C0
        }
    }

    /// 0x82198, 0x8238C and 0x827B6. `(X + #$100) < camX` is a signed `blt`: the mode advances and
    /// the next update frees the object. Otherwise the frame repaints when G.$7AE differs from
    /// G.$7AF (`cmp.b $7AF; beq` skips the write). The bytes are the ones `tile_anim_tick` left,
    /// which runs after the props.
    fn prop_waterfall_repaint(&self, p: &mut Prop) -> bool {
        let x = p.b.xi().wrapping_add(0x100);
        if x < self.cam.s2.cam_x as i16 {
            p.b.mode = p.b.mode.wrapping_add(2); // 0x821BA
            return false;
        }
        let (now, prev) = self.prop_tile_phase();
        now != prev
    }

    /// A long of a table, the low 24 bits (the address the 68000 uses).
    fn prop_rom24(&self, table: u32, off: u32) -> u32 {
        self.assets.enemy.t.u32(table, off) & 0x00FF_FFFF
    }

    /// Picture for this frame's G.$7AE. The phase byte is the offset of the long (0, 4, 8, 12).
    fn prop_clock_pic(&self, table: u32) -> u32 {
        let (phase, _) = self.prop_tile_phase();
        self.prop_rom24(table, u32::from(phase))
    }

    /// `move.w (a1)+,(a0)` then `adda.w gap,a0`, once per entry (0x821D6, 0x8223E).
    fn prop_code_gaps(&mut self, mut addr: u32, src: u32, gaps: &[u16]) {
        for (k, gap) in gaps.iter().enumerate() {
            let code = self.assets.enemy.t.u16(src, 2 * k as u32);
            self.prop_tile_code(TilePos::at(addr), code);
            addr = addr.wrapping_add(u32::from(*gap));
        }
    }

    /// `moveq #n-1,d5` / `move.w (a1)+,(a0)` / `addq.w #4,a0` / `dbra` (0x823D2, 0x82802).
    fn prop_code_down(&mut self, addr: u32, src: u32, n: u16) {
        for k in 0..n {
            let code = self.assets.enemy.t.u16(src, 2 * u32::from(k));
            let at = addr.wrapping_add(4 * u32::from(k));
            self.prop_tile_code(TilePos::at(at), code);
        }
    }

    /// 0x823AE.
    fn prop_b37_write(&mut self, p: &Prop) {
        let var = u32::from(p.b.var26);
        let dest = self.prop_rom24(0x823EE, var);
        let pics = self.prop_rom24(0x823FA, var);
        let pic = self.prop_clock_pic(pics);
        self.prop_code_down(dest, pic, 16); // 0x823D2
        // 0x823DC: the second strip starts at the saved destination plus 0x3C0, and a1 continues.
        self.prop_code_down(dest.wrapping_add(0x3C0), pic.wrapping_add(32), 16);
    }

    /// 0x827D8. `+$26` is 0..5 (the spawn word, not a byte offset). `btst.b #0` picks the writer,
    /// then `lsl.w #2` indexes the destination longs at 0x82880 and the picture tables at 0x82898.
    fn prop_b38_write(&mut self, p: &Prop) {
        let off = u32::from(p.b.var26) << 2;
        let dest = self.prop_rom24(0x82880, off);
        let pics = self.prop_rom24(0x82898, off);
        let pic = self.prop_clock_pic(pics);
        if p.b.var26 & 1 == 0 {
            self.prop_code_down(dest, pic, 0x50); // 0x82802, 80 words
            return;
        }
        // 0x8282E: a1 starts at the picture plus 0xA0. Five columns (`adda.w #$40,a2`), each 16
        // `move.w -(a1),(a0)` / `subq.w #4,a0`.
        let mut src = pic.wrapping_add(0xA0);
        let mut col = dest;
        for _ in 0..5 {
            let mut addr = col;
            for _ in 0..16 {
                src = src.wrapping_sub(2);
                let code = self.assets.enemy.t.u16(src, 0);
                self.prop_tile_code(TilePos::at(addr), code);
                addr = addr.wrapping_sub(4);
            }
            col = col.wrapping_add(0x40);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::stage::Cell;
    use dino_assets::AssetPack;
    use std::path::Path;

    fn game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut pack = AssetPack::default();
        pack.enemy.t.blobs = blobs;
        Game::new(Box::leak(Box::new(pack)))
    }

    fn code(g: &Game, addr: u32) -> u16 {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].code
    }

    fn attr(g: &Game, addr: u32) -> u16 {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].attr
    }

    fn paint(g: &mut Game, addr: u32, n: usize) {
        let i0 = ((addr - 0x90C000) / 4) as usize;
        for k in 0..n {
            g.vram.scroll2[i0 + k] = Cell { code: 0xAAAA, attr: 0x00FF };
        }
    }

    fn be32(v: u32) -> [u8; 4] {
        v.to_be_bytes()
    }

    fn words(ws: &[u16]) -> Vec<u8> {
        let mut v = Vec::with_capacity(ws.len() * 2);
        for w in ws {
            v.extend_from_slice(&w.to_be_bytes());
        }
        v
    }

    /// Pointer table of `n` longs at `table`, pictures of `words_n` words packed just after it.
    /// Phase byte 0, 4, 8, 12 selects picture 0, 1, 2, 3. Word `k` of picture `p` is `base + p * words_n + k`.
    fn pics(table: u32, words_n: usize, base: u16) -> (u32, Vec<u8>) {
        let mut data = Vec::new();
        let pic0 = table + 16;
        for p in 0..4u32 {
            data.extend_from_slice(&be32(pic0 + p * (words_n as u32) * 2));
        }
        for p in 0..4u16 {
            let row: Vec<u16> = (0..words_n as u16).map(|k| base + p * words_n as u16 + k).collect();
            data.extend_from_slice(&words(&row));
        }
        (table, data)
    }

    fn slot(_g: &Game, h: crate::game::Handle) -> usize {
        match crate::game::slot_of(h).unwrap() {
            crate::game::Slot::Prop(i) => i,
            _ => unreachable!(),
        }
    }

    #[test]
    fn b36_var0_writes_four_codes_on_the_clock_edge_only() {
        let (table, data) = pics(0x821E8, 4, 0x1000);
        let mut g = game(vec![(table, data), pics(0x8229C, 20, 0x2000)]);
        paint(&mut g, 0x90C038, 1);
        paint(&mut g, 0x90C03C, 1);
        paint(&mut g, 0x90C3F8, 2);
        let h = g.spawn_prop(36, 0, Some((0x100, 0, 0x300))).unwrap();
        let i = slot(&g, h);
        g.cam.s2.phase72 = 4;
        g.cam.s2.prev73 = 0;
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2, "mode 0 only advances");
        assert_eq!(code(&g, 0x90C038), 0xAAAA, "no write on the first update");

        g.pool_b_update();
        // phase 4 is picture 1: words 0x1004..0x1007
        assert_eq!(code(&g, 0x90C038), 0x1004);
        assert_eq!(code(&g, 0x90C03C), 0x1005);
        assert_eq!(code(&g, 0x90C3F8), 0x1006);
        assert_eq!(code(&g, 0x90C3FC), 0x1007);
        assert_eq!(attr(&g, 0x90C038), 0x00FF, "the attribute stays");

        g.cam.s2.prev73 = 4;
        paint(&mut g, 0x90C038, 1);
        g.pool_b_update();
        assert_eq!(code(&g, 0x90C038), 0xAAAA, "equal phases do not write");
        assert_eq!(g.props.slots[i].b.mode, 2);
    }

    #[test]
    fn b36_var4_writes_twenty_codes_and_the_camera_frees_it() {
        let mut g = game(vec![pics(0x821E8, 4, 1), pics(0x8229C, 20, 0x3000)]);
        paint(&mut g, 0x90C438, 0x40);
        let h = g.spawn_prop(36, 4, Some((0x100, 0, 0x300))).unwrap();
        let i = slot(&g, h);
        g.cam.s2.phase72 = 0;
        g.cam.s2.prev73 = 12;
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(code(&g, 0x90C438), 0x3000);
        assert_eq!(code(&g, 0x90C43C), 0x3001);
        // after +4 and +0x38 the third word is at 0x90C438+4+0x38
        assert_eq!(code(&g, 0x90C474), 0x3002);
        assert_eq!(g.props.slots[i].b.mode, 2);

        // X + 0x100 == camX stays (blt is strict). One more and the next update frees.
        g.cam.s2.cam_x = 0x200;
        g.cam.s2.prev73 = 0;
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 2, "equal is not past the camera");
        g.cam.s2.cam_x = 0x201;
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.mode, 4, "the cull advances the mode");
        assert_eq!(code(&g, 0x90C438), 0x3000, "the cull frame does not write");
        g.pool_b_update();
        assert_eq!(g.props.slots[i].b.live, 0, "mode 4 is the free");
    }

    #[test]
    fn b37_writes_two_strips_of_sixteen() {
        // Destinations and picture-table pointers, then one phase table and one 32-word picture.
        let mut head = Vec::new();
        head.extend_from_slice(&be32(0x90D000));
        head.extend_from_slice(&be32(0x90E000));
        head.extend_from_slice(&be32(0x90F000));
        head.extend_from_slice(&be32(0x82406));
        head.extend_from_slice(&be32(0x82406));
        head.extend_from_slice(&be32(0x82406));
        // 0x823EE + 24 = 0x82406. Phase longs: picture at 0x82416 for every phase.
        let mut phase = Vec::new();
        for _ in 0..4 {
            phase.extend_from_slice(&be32(0x82416));
        }
        let pic: Vec<u16> = (0..32).collect();
        let mut g = game(vec![(0x823EE, head), (0x82406, phase), (0x82416, words(&pic))]);
        paint(&mut g, 0x90D000, 16);
        paint(&mut g, 0x90D3C0, 16);
        let h = g.spawn_prop(37, 0, Some((0xD20, 0, 0x300))).unwrap();
        g.cam.s2.phase72 = 8;
        g.cam.s2.prev73 = 4;
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(g.props.slots[slot(&g, h)].b.mode, 2);
        assert_eq!(code(&g, 0x90D000), 0);
        assert_eq!(code(&g, 0x90D000 + 15 * 4), 15);
        assert_eq!(code(&g, 0x90D3C0), 16);
        assert_eq!(code(&g, 0x90D3C0 + 15 * 4), 31);
        assert_eq!(attr(&g, 0x90D03C), 0x00FF);
    }

    #[test]
    fn b38_even_writes_eighty_down_and_odd_writes_them_backwards() {
        let (table, data) = pics(0x828B0, 80, 0);
        // Six destination longs and six picture-table pointers (vars 0..5, byte offset var*4).
        let mut dirs = Vec::new();
        for dest in [0x90DEC0u32, 0x90D43C, 0x90EEC0, 0x90E43C, 0x90FEC0, 0x90F43C] {
            dirs.extend_from_slice(&be32(dest));
        }
        for _ in 0..6 {
            dirs.extend_from_slice(&be32(table));
        }
        let mut g = game(vec![(0x82880, dirs), (table, data)]);
        paint(&mut g, 0x90DEC0, 80);
        let _h = g.spawn_prop(38, 0, Some((0xD20, 0, 0x300))).unwrap();
        g.cam.s2.phase72 = 0;
        g.cam.s2.prev73 = 4;
        g.pool_b_update();
        g.pool_b_update();
        assert_eq!(code(&g, 0x90DEC0), 0);
        assert_eq!(code(&g, 0x90DEC0 + 79 * 4), 79);

        paint(&mut g, 0x90D400, 0x80);
        let h = g.spawn_prop(38, 1, Some((0xD20, 0, 0x200))).unwrap();
        g.cam.s2.phase72 = 12;
        g.cam.s2.prev73 = 8;
        g.pool_b_update();
        g.pool_b_update();
        // phase 12 is picture 3: words 240..319. The first cell is the last word.
        assert_eq!(code(&g, 0x90D43C), 319);
        assert_eq!(code(&g, 0x90D43C - 15 * 4), 304);
        // second column starts at dest + 0x40 with the next 16 words, still reversed
        assert_eq!(code(&g, 0x90D43C + 0x40), 303);
        assert_eq!(code(&g, 0x90D43C + 0x100 - 15 * 4), 240);
        assert_eq!(g.props.slots[slot(&g, h)].b.var26, 1);
    }

    fn rom() -> Option<&'static AssetPack> {
        use std::sync::OnceLock;
        static A: OnceLock<Option<&'static AssetPack>> = OnceLock::new();
        *A.get_or_init(|| {
            let path = ["../../../dino.zip", "../../dino.zip", "../dino.zip"]
                .into_iter()
                .map(Path::new)
                .find(|p| p.exists())?;
            Some(&*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// The extracted pictures are long enough for every phase the clock asks for.
    #[test]
    fn the_waterfall_tables_load_from_the_rom() {
        let Some(a) = rom() else { return };
        let g = Game::new(a);
        let t = &g.assets.enemy.t;
        let pic = t.u32(0x821E8, 0) & 0xFFFFFF;
        assert_eq!(pic, 0x821F8);
        let _ = t.u16(pic, 6);
        let pic4 = t.u32(0x8229C, 12) & 0xFFFFFF;
        let _ = t.u16(pic4, 38);
        assert_eq!(t.u32(0x823EE, 0), 0x90D000);
        assert_eq!(t.u32(0x823EE, 8), 0x90F000);
        let pt = t.u32(0x823FA, 4) & 0xFFFFFF;
        let p12 = t.u32(pt, 12) & 0xFFFFFF;
        let _ = t.u16(p12, 62);
        assert_eq!(t.u32(0x82880, 4), 0x90D43C);
        assert_eq!(t.u32(0x82880, 16), 0x90FEC0);
        let pt = t.u32(0x82898, 8) & 0xFFFFFF;
        let p8 = t.u32(pt, 8) & 0xFFFFFF;
        let _ = t.u16(p8, 158);
    }
}
