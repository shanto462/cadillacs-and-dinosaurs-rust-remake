//! Shared services for the pool B tile and palette props (T3-0, M3-16). Owner: enemy port.
//! Spec: D3e part 3 sections 2.2, 2.8, 2.9 and 7(b). The classes themselves come later: they pass
//! their own tables (nothing is extracted here).
//!
//! 1. **Palette cycler.** B60 (0x863AC), B61 (0x864DA) and B63 (0x869E6) share one countdown: `subq.b #1,+$A4`
//!    steps when the byte was 0 (`bcs`), then reloads `+$A5`. A reload of 8 is a step every 9 frames (B60, a 36
//!    frame cycle of 4 sets); a reload of 16 is every 17 frames (B61, B63). Each step copies one colour set per
//!    lane with the brightness nibble kept (`and.w #$F000 ; or.w (a0)+`, 0x86438). B71 (0x89746) and the
//!    lightning flashes (0x77FF6, 0x78170) use the same copy; their timers are their own (a word count, or a
//!    table of frame counts) and they call [`Game::prop_palette_copy`] directly.
//! 2. **Tile clock.** Readers such as 0x7862A (`move.b $7AE ; cmp.b $7AF ; beq` skips the rewrite) repaint when
//!    G.$7AE and G.$7AF differ. In Episode 8 area 1 the phase is `((gf + 60) / 24 mod 4) * 4` and G.$7AF is that
//!    value one frame later. The bytes are also the scroll2 fields `+$72` / `+$73`, stepped by `tile_anim_tick`
//!    (0x20956 stage 4 every 8 frames, 0x2097C stage 7 every 24). The function is the closed form the Episode 8
//!    windows measured. The phase 0, 4, 8, 12 is a byte index into the picture table (0x78646).
//! 3. **Burst.** B4 (0x767D8), B42 (0x83A0A), B52 (0x85178), B11 (0x789EE) and B14 (0x7B0BE): wait, write a tile
//!    block, spawn pool C class 21 (0x94640), play a sound, free on the next mode (0x4938). B12 var 0 is a block
//!    table (0x78B86, `tile_block.rs`), not a burst. B12 var 4 shatters on a hit and calls [`Game::prop_burst_fire`]
//!    itself, then waits 20 frames before it frees.
//! 4. **Hero reach.** Already the box test of `pass_10cd0` (0x128F8). D3e part 3 section 2.6's `Z + 8` is the jab
//!    summary, checked by the tests in `prop.rs`. No second rule.
//!
//! A class keeps the countdown on the prop (`+$A4`, `+$B0`, `+$A0`) and calls one step per update, the way
//! `BlockTable` does. Tests build synthetic tables (`AssetPack::default()`, `enemy.t.blobs`).
// A service for the scenery prop tasks of M3: nothing calls it before the first of them is ported.
#![allow(dead_code)]

use super::Prop;
use super::tile_block::{TileMode, TilePos};
use crate::game::Game;

/// G.$7AE for game frame `gf` (G.$0000). `7AE = ((gf + 60) / 24 mod 4) * 4`, unsigned 16-bit division
/// (the `divu` truncation). Episode 8 area 1: 0 at gf 1284, then 4, 8, 12, 0 at 1308, 1332, 1356, 1380.
pub fn prop_tile_clock(gf: u16) -> u8 {
    ((gf.wrapping_add(60) / 24) % 4 * 4) as u8
}

/// G.$7AF: the phase of the frame before. The reader 0x7862E branches away when the two bytes match,
/// and the step frame of 0x2097C returns before `move.b $72,$73` (0x20974), so the old phase lasts one frame.
pub fn prop_tile_clock_prev(gf: u16) -> u8 {
    prop_tile_clock(gf.wrapping_sub(1))
}

/// True on a frame the readers repaint: G.$7AE differs from G.$7AF (0x7862E `beq` not taken).
pub fn prop_tile_clock_edge(gf: u16) -> bool {
    prop_tile_clock(gf) != prop_tile_clock_prev(gf)
}

/// One palette range a cycler rewrites. `list` is the ROM address of `steps` longs, each the ROM address of
/// one colour set (words). B60's two lists are 0x86476 and 0x86488; a negative long is the wrap (0x86420)
/// and `steps` is the count the class already has, so the driver does not read the terminator.
#[derive(Clone, Copy, Debug)]
pub struct PaletteLane {
    /// Palette RAM address of the first colour (0x914A00 is scroll2 palette 80).
    pub dest: u32,
    /// Colours to copy. B60 writes 16, then 8 at 0x914A2E (palette 81, colours 7 to 14).
    pub count: u16,
    pub list: u32,
}

/// The countdown shared by B60, B61 and B63. `reload` is `+$A5` (8 or 16). `counter` and `index` are the
/// prop offsets of the byte countdown and the step word (`+$A4` and `+$B0` in those classes).
#[derive(Clone, Copy, Debug)]
pub struct PaletteCycler<'a> {
    pub reload: u8,
    pub steps: u16,
    pub lanes: &'a [PaletteLane],
    pub counter: usize,
    pub index: usize,
}

impl PaletteCycler<'_> {
    /// `+$A4` of B60 (0x863AC), B61 (0x864DA) and B63 (0x869E6).
    pub const COUNTER: usize = 0xA4;
    /// `+$B0`, the step word of B61 (0x86502) and B63 (0x86A0A). B60 walks two cursors instead; the same
    /// index stands in for both, because the two lists advance together.
    pub const INDEX: usize = 0xB0;

    /// Mode 0: `clr.b +$A4` and `clr.w` of the step (0x863EA, 0x864FE). The first running frame steps,
    /// because a countdown of 0 borrows.
    pub fn start(&self, p: &mut Prop) {
        p.m.sb(self.counter, 0);
        p.m.sw(self.index, 0);
    }
}

/// How long a burst waits, and whether the wait falls into the swap. The original routines differ by a
/// fall-through and by a `beq` against a `bne`, so the four shapes are the four classes.
#[derive(Clone, Copy, Debug)]
pub enum BurstWait {
    /// B4 (0x767EC) and B52 (0x8518C): `move.w #wait,+$A0` and fall into `subq.w #1 ; beq` the swap.
    /// `wait` 1 is consumed by that fall-through, so the first update opens the door. The frame the object
    /// spends un-updated after the wave spawns it is the "wait 1 frame" of the design.
    Open(u16),
    /// B14 var 0 (0x7B0E2): the same word countdown, but a zero count only advances to mode 4 (0x7B0F8 `rts`).
    /// The swap is the next call. `wait` 8 writes on the 9th call.
    Hold(u16),
    /// B11 (0x78A20): `move.b #wait,+$80` and return. Mode 2 is `subq.b #1 ; bne rts` (0x78A2C). `wait` 3
    /// swaps on the 5th call.
    Armed(u8),
    /// B42 (0x83A1E): mode 0 falls straight into the swap. No counter.
    Now,
}

/// One tile rectangle of a burst (column by column, as [`Game::prop_tile_block`]).
#[derive(Clone, Copy, Debug)]
pub struct BurstBlock {
    pub at: TilePos,
    pub cols: u16,
    pub rows: u16,
    pub src: u32,
    pub mode: TileMode,
}

/// An attribute-only rectangle (B11 0x78AF0: `and` then `or` on each cell).
#[derive(Clone, Copy, Debug)]
pub struct BurstAttr {
    pub at: TilePos,
    pub cols: u16,
    pub rows: u16,
    pub keep: u16,
    pub or_bits: u16,
}

/// Pool C class 21 made by 0x94640. `var` is d0 (B4 0x16, B11 2, B14 var 0 is 4, B42 0x26, B52 0x20),
/// `facing` is d1.
#[derive(Clone, Copy, Debug)]
pub struct BurstEffect {
    pub var: u16,
    pub facing: u8,
}

/// The sound of the swap.
#[derive(Clone, Copy, Debug)]
pub enum BurstSound {
    /// 0x9E60, the pan break sound (B4, B42, B52, B11).
    Break,
    /// 0xA63C with the three registers (B14 var 4: code 0x88, d1 0x32000, d2 0x1C000000).
    Request { code: u16, d1: u32, d2: u32 },
    /// No sound (B14 var 0 writes the hole and spawns the effect only).
    Quiet,
}

/// Tiles, effect and sound of one burst. The class owns the tables.
#[derive(Clone, Copy, Debug)]
pub struct BurstAction<'a> {
    pub tiles: &'a [BurstBlock],
    pub attrs: &'a [BurstAttr],
    pub effect: Option<BurstEffect>,
    pub sound: BurstSound,
}

/// One burst prop: the wait shape plus the action.
#[derive(Clone, Copy, Debug)]
pub struct BurstSpec<'a> {
    pub wait: BurstWait,
    pub action: BurstAction<'a>,
}

/// `+$A0`, the word wait of B4, B14 and B52.
const WAIT: usize = 0xA0;
/// `+$80`, the byte wait of B11.
const ARMED: usize = 0x80;

impl Game {
    /// G.$7AE and G.$7AF this frame: the live bytes (scroll2 `+$72` / `+$73`, written by `tile_anim_tick`), which the
    /// readers compare. [`prop_tile_clock`] is the closed form the Episode 8 windows measured, kept for the tests.
    pub fn prop_tile_phase(&self) -> (u8, u8) {
        (self.cam.s2.phase72, self.cam.s2.prev73)
    }

    /// 0x86438 / 0x8656C / 0x89964: `count` colours at palette RAM `dest` become `(old & 0xF000) | source`.
    /// The `or.w` merges the whole source word, so a source bit in the top nibble sticks (the sets in the
    /// ROM are 12-bit colours, and the brightness nibble of the map is what the player sees).
    pub fn prop_palette_copy(&mut self, dest: u32, set: u32, count: u16) {
        let first = dest.wrapping_sub(0x914000) / 2;
        let n = count as u32;
        assert!(
            dest >= 0x914000 && first.wrapping_add(n) <= 2048,
            "palette address {dest:#x} ({count} colours) is outside palette RAM"
        );
        for k in 0..n {
            let src = self.assets.enemy.t.u16(set, 2 * k);
            let w = &mut self.pal.words[(first + k) as usize];
            *w = (*w & 0xF000) | src;
        }
        self.pal.dirty = true;
    }

    /// One frame of a palette cycler (B60 0x8640C, B61 0x86542, B63 0x86A2E). Returns true when this
    /// frame wrote the sets. The class does its own stage test before the call (B60: stage 5 area 0,
    /// 0x863F4; B61 and B63: stage 6) and frees itself when the test fails.
    pub fn prop_palette_cycle(&mut self, p: &mut Prop, c: &PaletteCycler) -> bool {
        // subq.b #1,+$A4 ; bcs step. A byte that was 0 wraps to 0xFF and borrows.
        let n = p.m.b(c.counter).wrapping_sub(1);
        p.m.sb(c.counter, n);
        if n != 0xFF {
            return false;
        }
        p.m.sb(c.counter, c.reload); // move.b +$A5,+$A4
        let step = p.m.w(c.index);
        let next = step.wrapping_add(1);
        // B61 0x8657A: addq.w #1 ; cmpi.w #steps ; blt keep ; clr.w
        p.m.sw(c.index, if next >= c.steps { 0 } else { next });
        for lane in c.lanes {
            let set = self.assets.enemy.t.u32(lane.list, u32::from(step) * 4);
            self.prop_palette_copy(lane.dest, set, lane.count);
        }
        true
    }

    /// The swap itself (B4 0x76808, B42 0x83A2C, B52 0x851A2, B11 0x78A38, B14 0x7B0FA): the tile blocks,
    /// then pool C class 21, then the sound. B11 and B42 play the sound before the tiles; neither call uses
    /// `rand`, so the request is the same entry either way. Returns without an effect when pool C is empty
    /// (0x94646 `bcs`).
    pub fn prop_burst_fire(&mut self, p: &Prop, action: &BurstAction) {
        for b in action.tiles {
            self.prop_tile_block(b.at, b.cols, b.rows, b.src, b.mode);
        }
        for a in action.attrs {
            self.prop_tile_attr_block(a.at, a.cols, a.rows, a.keep, a.or_bits);
        }
        if let Some(fx) = action.effect {
            self.prop_burst_effect(p, fx);
        }
        match action.sound {
            BurstSound::Break => self.snd_break(p.b.xi()),
            BurstSound::Request { code, d1, d2 } => self.snd_request(code, d1, d2),
            BurstSound::Quiet => {}
        }
    }

    /// 0x94640: one pool C object, class word 0x54 (class 21), the prop's X Y Z, variant d0, facing d1.
    fn prop_burst_effect(&mut self, p: &Prop, fx: BurstEffect) {
        let Some(j) = self.effects.alloc() else {
            return;
        };
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = 0x54;
        e.b.set_xi(p.b.xi());
        e.b.set_yi(p.b.yi());
        e.b.set_zi(p.b.zi());
        e.b.var26 = fx.var;
        e.b.facing = fx.facing;
    }

    /// One update of a burst prop. True on the update that fires. Mode 6 and anything past it free the
    /// slot (0x4938, the last entry of each jump table). B4 and B42 and B52 set their own Y (0x60) in
    /// mode 0 before the fall-through: set it on the prop before the first call, the effect copies it.
    pub fn prop_burst_step(&mut self, p: &mut Prop, i: usize, spec: &BurstSpec) -> bool {
        match p.b.mode {
            0 => self.prop_burst_arm(p, spec),
            2 => self.prop_burst_wait(p, spec),
            4 => {
                self.prop_burst_fire(p, &spec.action);
                p.b.mode = 6; // addq.b #2 after the swap
                true
            }
            _ => {
                self.prop_free(p, i);
                false
            }
        }
    }

    /// Mode 0.
    fn prop_burst_arm(&mut self, p: &mut Prop, spec: &BurstSpec) -> bool {
        match spec.wait {
            BurstWait::Now => {
                p.b.mode = 4;
                self.prop_burst_fire(p, &spec.action);
                p.b.mode = 6;
                true
            }
            BurstWait::Open(w) | BurstWait::Hold(w) => {
                p.b.mode = 2;
                p.m.sw(WAIT, w);
                self.prop_burst_wait(p, spec) // the fall-through of 0x767F6 / 0x7B0E8 / 0x85190
            }
            BurstWait::Armed(w) => {
                p.b.mode = 2;
                p.m.sb(ARMED, w);
                false // 0x78A2A rts
            }
        }
    }

    /// Mode 2: the countdown. A zero count goes to mode 4, and `Open` falls into the swap.
    fn prop_burst_wait(&mut self, p: &mut Prop, spec: &BurstSpec) -> bool {
        let done = match spec.wait {
            BurstWait::Armed(_) => {
                let n = p.m.b(ARMED).wrapping_sub(1); // subq.b #1,+$80
                p.m.sb(ARMED, n);
                n == 0
            }
            BurstWait::Open(_) | BurstWait::Hold(_) => {
                let n = p.m.w(WAIT).wrapping_sub(1); // subq.w #1,+$A0
                p.m.sw(WAIT, n);
                n == 0
            }
            BurstWait::Now => true,
        };
        if !done {
            return false;
        }
        p.b.mode = 4;
        if matches!(spec.wait, BurstWait::Open(_)) {
            self.prop_burst_fire(p, &spec.action);
            p.b.mode = 6;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::snd::SndRequest;
    use dino_assets::AssetPack;
    use dino_assets::enemy::ByteTables;

    fn game(blobs: Vec<(u32, Vec<u8>)>) -> Game {
        let mut a = AssetPack::default();
        a.enemy.t = ByteTables { blobs };
        Game::new(Box::leak(Box::new(a)))
    }

    fn words(w: &[u16]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn longs(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    fn pal(g: &Game, addr: u32) -> u16 {
        g.pal.words[((addr - 0x914000) / 2) as usize]
    }

    /// The five measured frames of Episode 8 area 1, and G.$7AF one frame behind the step.
    #[test]
    fn the_tile_clock_at_the_measured_frames() {
        let samples = [(1284u16, 0u8), (1308, 4), (1332, 8), (1356, 12), (1380, 0)];
        for (gf, phase) in samples {
            assert_eq!(prop_tile_clock(gf), phase, "gf {gf}");
            assert!(
                prop_tile_clock_edge(gf),
                "gf {gf} is a step, G.$7AF still holds the old phase"
            );
            assert_eq!(
                prop_tile_clock_prev(gf),
                if phase == 0 { 12 } else { phase - 4 }
            );
            let next = gf.wrapping_add(1);
            assert_eq!(prop_tile_clock(next), phase, "the new phase holds");
            assert!(!prop_tile_clock_edge(next), "G.$7AF has caught up");
        }
        assert_eq!(
            prop_tile_clock(1307),
            0,
            "the step is at 1308, not the frame before"
        );
        let mut g = game(vec![]);
        (g.cam.s2.phase72, g.cam.s2.prev73) = (8, 4);
        assert_eq!(g.prop_tile_phase(), (8, 4), "the readers see the live bytes");
    }

    /// B60 (0x863AC): reload 8, four sets, a step every 9 frames. A06 (`r1:e6a0-lamps`, peeks every 4
    /// frames from a step) and the colour 5 numbers of D3e part 3 section 2.2. The sets here are synthetic
    /// stand-ins for the lists 0x86476 / 0x86488, with those measured low 12 bits in colour 0 and colour 5.
    #[test]
    fn the_b60_cycle_matches_a06() {
        // Colour 0, then four fillers, colour 5, then the rest. Brightness is not in the set.
        let set = |c0: u16, c5: u16, tag: u16| -> Vec<u16> {
            let mut w = vec![tag; 16];
            w[0] = c0;
            w[5] = c5;
            w
        };
        let sets = [
            set(0x0321, 0x0CBA, 0x0100),
            set(0x0661, 0x0FFF, 0x0200),
            set(0x0541, 0x0EDC, 0x0300),
            set(0x0431, 0x0DCB, 0x0400),
        ];
        // Palette 81, colours 7 to 14: eight colours, a different value per step, so the second lane is visible.
        let lane_b: Vec<u16> = (0..4).flat_map(|s| vec![0x0100 + 0x10 * s; 8]).collect();
        let mut blobs = vec![
            (0xA10000, longs(&[0xA11000, 0xA11020, 0xA11040, 0xA11060])),
            (0xA12000, longs(&[0xA13000, 0xA13010, 0xA13020, 0xA13030])),
            (0xA13000, words(&lane_b)),
        ];
        for (k, s) in sets.iter().enumerate() {
            blobs.push((0xA11000 + 0x20 * k as u32, words(s)));
        }
        let mut g = game(blobs);
        // Full brightness already in the map, plus one word whose nibble is not F (the mask must keep it).
        for addr in (0x914A00..0x914A20).step_by(2) {
            let i = ((addr - 0x914000) / 2) as usize;
            g.pal.words[i] = 0xF0AB;
        }
        g.pal.words[((0x914A00 - 0x914000) / 2) as usize + 1] = 0xA000;
        for addr in (0x914A2E..0x914A3E).step_by(2) {
            g.pal.words[((addr - 0x914000) / 2) as usize] = 0xE000;
        }
        let lanes = [
            PaletteLane {
                dest: 0x914A00,
                count: 16,
                list: 0xA10000,
            },
            PaletteLane {
                dest: 0x914A2E,
                count: 8,
                list: 0xA12000,
            },
        ];
        let cyc = PaletteCycler {
            reload: 8,
            steps: 4,
            lanes: &lanes,
            counter: PaletteCycler::COUNTER,
            index: PaletteCycler::INDEX,
        };
        let mut p = Prop::default();
        cyc.start(&mut p);

        let mut c0 = Vec::new();
        let mut c5 = Vec::new();
        let mut b0 = Vec::new();
        for _ in 0..53 {
            g.prop_palette_cycle(&mut p, &cyc);
            c0.push(pal(&g, 0x914A00));
            c5.push(pal(&g, 0x914A0A));
            b0.push(pal(&g, 0x914A2E));
        }
        // A06, sampled every 4 frames: F321, F321, F321, F661, F661, F541, F541, F431, F431, F321, F321, F321, F661, F661.
        let a06 = [0u16, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52];
        let want = [
            0xF321, 0xF321, 0xF321, 0xF661, 0xF661, 0xF541, 0xF541, 0xF431, 0xF431, 0xF321, 0xF321,
            0xF321, 0xF661, 0xF661,
        ];
        let got: Vec<u16> = a06.iter().map(|&t| c0[t as usize]).collect();
        assert_eq!(got, want, "palette 80 colour 0");
        let want5 = [
            0xFCBA, 0xFCBA, 0xFCBA, 0xFFFF, 0xFFFF, 0xFEDC, 0xFEDC, 0xFDCB, 0xFDCB, 0xFCBA, 0xFCBA,
            0xFCBA, 0xFFFF, 0xFFFF,
        ];
        let got5: Vec<u16> = a06.iter().map(|&t| c5[t as usize]).collect();
        assert_eq!(got5, want5, "palette 80 colour 5");
        // Steps at 0, 9, 18, 27, 36, 45. Each set holds for 9 frames. Frame 36 is set 0 again.
        for t in 0..9 {
            assert_eq!(c0[t], 0xF321, "frame {t}");
            assert_eq!(b0[t], 0xE100, "lane B frame {t}");
        }
        assert_eq!(c0[9], 0xF661);
        assert_eq!(c0[36], c0[0], "36 frame cycle");
        assert_eq!(c0[45], c0[9]);
        assert_eq!(
            p.m.w(PaletteCycler::INDEX),
            2,
            "six steps leave the index at 2 (the next set)"
        );
        // Colour 1 of set 0 is the tag 0x0100, and its destination nibble was 0xA000, not F.
        assert_eq!(
            pal(&g, 0x914A02),
            0xA200,
            "the top nibble is kept, the low 12 bits are the current set"
        );
    }

    /// `or.w` of a source word that itself has a top-nibble bit: the bit merges, it does not replace.
    #[test]
    fn the_palette_or_merges_the_source_nibble() {
        let mut g = game(vec![(0xA14000, words(&[0x1321]))]);
        g.pal.words[((0x914A00 - 0x914000) / 2) as usize] = 0xA000;
        g.prop_palette_copy(0x914A00, 0xA14000, 1);
        assert_eq!(pal(&g, 0x914A00), 0xB321);
    }

    fn burst_game() -> Game {
        // The pan byte of 0x9E60 (table 0xA66A). X 0 reads byte 0.
        let codes: Vec<u16> = (0..4).map(|k| 0x1110 + k).collect();
        game(vec![(0xA66A, vec![0x10; 48]), (0xB40000, words(&codes))])
    }

    fn action() -> BurstAction<'static> {
        const TILES: &[BurstBlock] = &[BurstBlock {
            at: TilePos(0x90C040),
            cols: 2,
            rows: 2,
            src: 0xB40000,
            mode: TileMode::Codes,
        }];
        BurstAction {
            tiles: TILES,
            attrs: &[],
            effect: Some(BurstEffect {
                var: 0x16,
                facing: 0,
            }),
            sound: BurstSound::Break,
        }
    }

    fn cell(g: &Game, addr: u32) -> u16 {
        g.vram.scroll2[((addr - 0x90C000) / 4) as usize].code
    }

    fn fired(g: &Game) -> bool {
        cell(g, 0x90C040) == 0x1110 && g.w.sounds.iter().any(|&s| s == 0xF6 || s == 0x6F)
    }

    /// B4 (0x767D8): wait 1 falls through, so the first update swaps, spawns C21 var 0x16 and plays the
    /// break sound. The next update frees.
    #[test]
    fn a_wait_of_one_opens_on_the_first_update() {
        let mut g = burst_game();
        let i = g.props.alloc().unwrap();
        let mut p = std::mem::take(&mut g.props.slots[i]);
        p.b.live = 1;
        p.b.set_xi(0);
        p.b.set_yi(0x60);
        p.b.set_zi(0x40);
        let spec = BurstSpec {
            wait: BurstWait::Open(1),
            action: action(),
        };
        assert!(g.prop_burst_step(&mut p, i, &spec));
        assert_eq!(p.b.mode, 6);
        assert_eq!(
            (
                cell(&g, 0x90C040),
                cell(&g, 0x90C044),
                cell(&g, 0x90C080),
                cell(&g, 0x90C084)
            ),
            (0x1110, 0x1111, 0x1112, 0x1113)
        );
        let fx = g.effects.slots.iter().find(|e| e.b.live != 0).unwrap();
        assert_eq!((fx.b.k20, fx.b.var26, fx.b.facing), (0x54, 0x16, 0));
        assert_eq!((fx.b.xi(), fx.b.yi(), fx.b.zi()), (0, 0x60, 0x40));
        assert_eq!(
            g.w.sounds,
            vec![0xF6],
            "the first break after a clear toggle is 0xF6"
        );
        assert!(!g.prop_burst_step(&mut p, i, &spec), "mode 6 frees");
        assert_eq!(p.b.live, 0);
        g.props.slots[i] = p;
        assert_eq!(g.props.free_count(), 24);
    }

    /// The other three shapes: B42 fires at once, a longer `Open` waits the extra frames, B14's hold and
    /// B11's armed wait hand the swap to the next mode.
    #[test]
    fn the_other_burst_shapes_wait_as_their_routines_do() {
        let shapes = [
            (BurstWait::Now, 1usize),
            (BurstWait::Open(2), 2),
            (BurstWait::Hold(8), 9),
            (BurstWait::Armed(3), 5),
        ];
        for (wait, fire_at) in shapes {
            let mut g = burst_game();
            let i = g.props.alloc().unwrap();
            let mut p = std::mem::take(&mut g.props.slots[i]);
            p.b.live = 1;
            let spec = BurstSpec {
                wait,
                action: action(),
            };
            for call in 1..fire_at {
                assert!(!g.prop_burst_step(&mut p, i, &spec), "{wait:?} call {call}");
                assert!(!fired(&g), "{wait:?} has not opened");
            }
            assert!(
                g.prop_burst_step(&mut p, i, &spec),
                "{wait:?} opens on call {fire_at}"
            );
            assert!(fired(&g));
            assert_eq!(p.b.mode, 6);
            assert!(!g.prop_burst_step(&mut p, i, &spec), "the next call frees");
            assert_eq!(p.b.live, 0);
        }
    }

    /// B14 var 4's sound is a plain request, and a full pool C drops the effect (0x94646) without dropping
    /// the tiles or the sound.
    #[test]
    fn a_request_sound_and_a_full_effect_pool() {
        let mut g = burst_game();
        let i = g.props.alloc().unwrap();
        let mut p = std::mem::take(&mut g.props.slots[i]);
        p.b.live = 1;
        while g.effects.alloc().is_some() {}
        let action = BurstAction {
            tiles: action().tiles,
            attrs: &[],
            effect: Some(BurstEffect { var: 6, facing: 1 }),
            sound: BurstSound::Request {
                code: 0x88,
                d1: 0x32000,
                d2: 0x1C00_0000,
            },
        };
        let spec = BurstSpec {
            wait: BurstWait::Now,
            action,
        };
        assert!(g.prop_burst_step(&mut p, i, &spec));
        assert!(fired(&g) || cell(&g, 0x90C040) == 0x1110);
        assert!(
            g.effects.slots.iter().all(|e| e.b.k20 != 0x54),
            "no class 21 when the pool is empty"
        );
        assert_eq!(
            g.snd_take_requests(),
            vec![SndRequest {
                code: 0x88,
                d1: 0x32000,
                d2: 0x1C00_0000
            }]
        );
    }
}
