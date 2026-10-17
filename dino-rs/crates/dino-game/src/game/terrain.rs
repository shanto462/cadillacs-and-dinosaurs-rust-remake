//! Terrain collision of every object: floor heights, walls and street edges, and the other-object
//! obstacle test (phase 1 task 1).
//!
//! Original routines (all use the object in A6, the globals G.$6AC / $6B0 / $6B2 and the scroll2
//! tilemap attributes):
//!
//! | address | routine | callers |
//! | --- | --- | --- |
//! | 0x12BE4 | `terrain_area_start`: floor list of the stage and area, G.$6B0 | area start (flow, camera) |
//! | 0x12C90 | `terrain_advance`: floor list pointer follows the camera | end of `objects_update` |
//! | 0x12CB4 | `terrain_collide`: floor + walls (5 passes) + obstacles + snap | enemies, props, items |
//! | 0x12CAC | `terrain_collide` + 0x1361E `terrain_ceiling` | players |
//! | 0x12D4E | `terrain_floor_only` | players when G.$4F0 is set, respawn |
//! | 0x12DAC | `terrain_effect_floor`: 5 floor lookups + snap | effects (phase 5) |
//! | 0x12E20 | `terrain_collide_free`: 0x12CB4 without the obstacle test | some enemy classes |
//! | 0x12E9A | `terrain_obstacles`: pushes against registered obstacles (containers) | inside 0x12CB4 |
//! | 0x12FE4 | `floor_lookup`: floor segment under X | all of the above |
//! | 0x1309C | `terrain_wall`: class of the scroll2 tile under the feet and its shape handler | 0x12CB4 |
//! | 0x13138.. | `shape_handler`: the shape handlers selected by word 0 of the shape (0x1370C) | 0x1309C |
//! | 0x136E4 | `scene_class`: the same lookup in the vertical (elevator) scene map | 0x1309C, 0x1361E |
//! | 0x4A76 / 0x4A96 | `terrain_obstacles_reset` / `item_register_obstacle`: the obstacle list | area start, containers |
//!
//! Coordinates: X = obj+$08, height Y = obj+$0C, depth Z = obj+$10 (integer words only; the
//! routines never touch the fractions except where noted). The map lookups use X and `Z + floor`
//! (the ground contact point) for walls and `Z + Y + head height` for the ceiling test.
//!
//! All terrain data (floor segments, shape descriptors, class flags) is extracted by
//! `dino-extract/src/terrain.rs`.
//!
//! Checked against the original: `re/harness/terrain_oracle.py` runs the 68000 routines (a small
//! interpreter) on random setups and the test `terrain_matches_the_68000_code` replays them (250 000
//! vectors, all routines, every shape handler). It found these quirks, kept on purpose:
//! * `ble` / `bgt` after `sub.w` compare the operands as signed numbers even when the 16-bit
//!   difference overflows (the street edge shapes with G.$6B0 + floor above 0x7FFF).
//! * The shape 0x10 variant 12 (0x134D8) shifts G.$6B0 + floor, the others the height above floor.
//! * The ceiling test in the vertical scene rotates the class twice (0x13642 / 0x13674): it is
//!   always class 0, so that scene has no ceiling.
//! * The obstacle list storage is shared by this frame's registrations and last frame's list.
//! * Shapes always set +$50 with `move.b` (not OR); only the final `or` adds the class flag.

use super::body::{Body, Fixed};
use super::{slot_of, Game, Handle, Slot};
use dino_assets::terrain::FloorSegment;

/// Size of the obstacle list storage G.$6A2A..$6A59 (24 words, one per pool E slot).
pub const OBSTACLE_SLOTS: usize = 24;

/// State of the terrain routines.
#[derive(Clone, Debug)]
pub struct Terrain {
    /// G.$6AC: the floor segment the lookup starts from, as an index into the list of the
    /// current stage and area (the original keeps the ROM address).
    pub seg: usize,
    /// G.$6B0: height of the lower level used by the street edge shapes (table 0x12C40).
    pub base: i16,
    /// G.$6A2A..$6A58: the obstacle list storage. Pushes (0x4A96) fill it from the top, slot 0 =
    /// G.$6A58 first. The list that 0x12E9A reads (last frame's) lives in the same words, so
    /// registrations of this frame overwrite it while it is being read: kept as in the original.
    pub buf: [Handle; OBSTACLE_SLOTS],
    /// The vertical scene collision map the original draws at 0x91C000 / 0x91C400 (32 columns x
    /// 16 rows of (code, attribute) cells, cell = column * 16 + row); empty outside that scene.
    pub scene: Vec<[u16; 2]>,
    /// G.$6A22: objects registered so far this frame.
    pub count: u16,
    /// G.$6A26: size of the list 0x12E9A reads (the registrations of the previous frame).
    pub prev: u16,
}

impl Default for Terrain {
    fn default() -> Self {
        Terrain { seg: 0, base: 0, buf: [0; OBSTACLE_SLOTS], scene: Vec::new(), count: 0, prev: 0 }
    }
}

// ------------------------------------------------------------------------------------------
// 68000 helpers
// ------------------------------------------------------------------------------------------

/// `move.w d0,$8(a6)` style write: only the integer word of a 16.16 coordinate.
fn put_hi(v: &mut Fixed, w: i16) {
    *v = ((w as i32) << 16) | (*v & 0xFFFF);
}

/// `add.w d0,$8(a6)`.
fn add_x(b: &mut Body, v: u16) {
    let x = b.xi().wrapping_add(v as i16);
    put_hi(&mut b.x, x);
}
/// `sub.w d0,$8(a6)`.
fn sub_x(b: &mut Body, v: u16) {
    let x = b.xi().wrapping_sub(v as i16);
    put_hi(&mut b.x, x);
}
/// `add.w d1,$10(a6)`.
fn add_z(b: &mut Body, v: u16) {
    let z = b.zi().wrapping_add(v as i16);
    put_hi(&mut b.z, z);
}
/// `sub.w d1,$10(a6)`.
fn sub_z(b: &mut Body, v: u16) {
    let z = b.zi().wrapping_sub(v as i16);
    put_hi(&mut b.z, z);
}

/// `lsr.w dN,dM`: the count is taken modulo 64, and counts of 16 or more clear the word.
fn lsr(v: u16, count: u16) -> u16 {
    let n = count & 63;
    if n >= 16 { 0 } else { v >> n }
}
/// `lsl.w dN,dM`.
fn lsl(v: u16, count: u16) -> u16 {
    let n = count & 63;
    if n >= 16 { 0 } else { v << n }
}

/// The height test of the street edge shapes: `d2 = G.$6B0 + floor; sub.w Y,d2; ble skip`, then
/// `lsr.w #2,d2`. `ble` after a subtraction is the true signed comparison of the operands (the
/// difference can overflow 16 bits), so the test is on the operands and the quarter is taken
/// from the wrapped difference. `None` = the branch to the "lower level" code was taken.
fn edge_rise(base: u16, b: &Body) -> Option<u16> {
    let top = base.wrapping_add(b.floor as u16);
    ((top as i16) > b.yi()).then(|| top.wrapping_sub(b.yi() as u16) >> 2)
}

// ------------------------------------------------------------------------------------------
// Floor
// ------------------------------------------------------------------------------------------

/// 0x12FE4 floor lookup for the object at X = `b.x`: takes the first of the 4 segments from `seg`
/// whose last X reaches the object (the fifth when none does), then sets +$53 (wading), and for a
/// flat segment +$52 and the floor +$54, for a slope the floor +$54 and (landing or following the
/// slope) the height +$0C. In the vertical scene (G.$4DC) the floor is 0.
pub fn floor_lookup(segs: &[FloorSegment], seg: usize, vertical: bool, b: &mut Body) {
    if vertical {
        // 0x13046
        b.wading = 0;
        b.b52 = 0;
        b.floor = 0;
        return;
    }
    if segs.is_empty() {
        return;
    }
    let x = b.xi();
    let at = |k: usize| segs[(seg + k).min(segs.len() - 1)];
    // The list ends with a segment that reaches 0x7FFF, so the scan never leaves it.
    let mut s = at(4);
    for j in 0..4 {
        let e = at(j);
        if e.x1 >= x {
            s = e;
            break;
        }
    }
    b.wading = s.wading;
    if s.slope == 0 {
        // 0x13038
        b.b52 = s.base as u8;
        b.floor = s.height;
        return;
    }
    // 0x13056 slope: height from the distance to the segment start.
    let d0 = (x.wrapping_sub(s.x0) as u16) >> 2;
    let d1 = s.base as u16;
    let h = if s.height == 0 { d0.wrapping_add(d1) } else { d1.wrapping_sub(1).wrapping_sub(d0) } as i16;
    if h < 0 {
        return; // bmi
    }
    b.floor = h;
    if b.airborne != 0 {
        if (h as u16) <= (b.yi() as u16) {
            return; // bls: still above the slope
        }
        b.airborne = 0; // landed
    }
    put_hi(&mut b.y, h);
}

/// 0x12CFE / 0x12D06 (tail of 0x12CB4, 0x12D4E, 0x12DAC, 0x12E20): keep the object on the floor
/// (+$54 + +$56). A grounded object whose floor rises is lifted, one whose floor drops starts to
/// fall (+$51 = 1); an airborne one that is below the floor lands.
pub fn floor_snap(b: &mut Body) {
    let floor = b.floor.wrapping_add(b.floor2);
    if b.airborne == 0 {
        let y = b.yi();
        if floor == y {
            return;
        }
        if floor > y {
            b.set_yi(floor); // move.w d0,$c ; clr.w $e
        } else {
            b.airborne = 1;
        }
    } else if floor > b.yi() {
        b.set_yi(floor);
        b.airborne = 0;
    }
}

// ------------------------------------------------------------------------------------------
// Terrain class lookups
// ------------------------------------------------------------------------------------------

/// Cell index in the scroll2 map (0x1309C / 0x1361E): the original's byte offset / 4. Columns come
/// from X, rows count down from the top of the 64 x 64 map: Y is flipped.
pub fn scroll2_cell(x: u16, y: u16) -> usize {
    let d2 = (x & 0x3F0) << 2;
    let d4 = 0x3F0u16 - (y & 0x3F0);
    let d3 = ((d4 & 0x300) << 4) + ((d4 & 0xF0) >> 2);
    ((d2 + d3) / 4) as usize
}

/// Cell index in the vertical scene map (0x136E4): 32 columns of 16 rows.
pub fn scene_cell(x: u16, y: u16) -> usize {
    let d2 = (x & 0x1F0) << 2;
    let d4 = (0xF0 - (y & 0xF0)) >> 2;
    ((d2 + d4) / 4) as usize
}

/// Terrain class of an attribute word: bits 10..15 (`rol.w #6`).
pub fn attr_class(attr: u16) -> u16 {
    attr.rotate_left(6) & 0x3F
}

// ------------------------------------------------------------------------------------------
// Shape handlers (0x13138 .. 0x1361A)
// ------------------------------------------------------------------------------------------

/// The shape handler of a terrain class. `sh` is the 8-word descriptor (0x1370C), `base` G.$6B0,
/// `d0` = X and `d1` = Z + floor as 0x1309C passes them. The handler pushes the object out of the
/// tile by changing +$08 / +$10 (and sets +$50 / +$56). Returns the carry flag: true (0x13138) asks
/// 0x12CB4 for another pass, false (0x13134) ends the wall test.
///
/// Word 0 picks the handler (byte offset into the table 0x1313E), word 1 the variant, words 2..4
/// are its parameters (flip flag, shift, offset), word 5 the "another pass" flag and the low byte
/// of word 7 the wall bits stored in +$50.
pub fn shape_handler(sh: &[u16; 8], base: i16, b: &mut Body, mut d0: u16, mut d1: u16) -> bool {
    let (w1, w2, w3, w4) = (sh[1], sh[2], sh[3], sh[4]);
    let again = sh[5] != 0; // tst.w $a(a0)
    let bits = sh[7] as u8; // move.b $f(a0),$50(a6)
    let base = base as u16; // G.$6B0
    let kind = if sh[0] == 0x20 { 0x14 } else { sh[0] };
    match kind {
        // 0x1315E: flat walls, one tile edge per variant.
        0x00 => match w1 {
            0 => {
                // 0x13178: push +Z to the tile edge
                d1 = !d1 & 0xF;
                d1 = d1.wrapping_add(1);
                add_z(b, d1);
                b.wall = bits;
                again
            }
            4 => {
                // 0x13192: push -Z
                d1 = (d1 & 0xF).wrapping_add(1);
                sub_z(b, d1);
                b.wall = bits;
                again
            }
            8 => {
                // 0x131AA: push -X
                d0 = (d0 & 0xF).wrapping_add(1);
                sub_x(b, d0);
                b.wall = bits;
                again
            }
            0xC => {
                // 0x131C6: push +X
                d0 = d0.wrapping_neg() & 0xF;
                d0 = d0.wrapping_add(1);
                add_x(b, d0);
                b.wall = bits;
                again
            }
            _ => false,
        },
        // 0x131E4: slanted walls (the wall depth depends on the other coordinate).
        0x04 => {
            match w1 {
                0 => {
                    // 0x131FE: push +Z
                    if w2 == 0 {
                        d0 = !d0;
                    }
                    d0 &= 0xF;
                    d0 = lsr(d0, w3).wrapping_add(w4).wrapping_add(1);
                    d1 &= 0xF;
                    if d0 >= d1 {
                        d0 -= d1;
                        add_z(b, d0);
                        b.wall = bits;
                    }
                }
                4 => {
                    // 0x13230: push -Z
                    if w2 != 0 {
                        d0 = !d0;
                    }
                    d0 &= 0xF;
                    d0 = lsr(d0, w3).wrapping_add(w4).wrapping_add(1);
                    d1 = !d1 & 0xF;
                    if d0 >= d1 {
                        d0 -= d1;
                        sub_z(b, d0);
                        b.wall = bits;
                    }
                }
                8 => {
                    // 0x13264: push -X
                    if w2 == 0 {
                        d1 = !d1;
                    }
                    d1 &= 0xF;
                    d1 = lsr(d1, w3).wrapping_add(w4).wrapping_add(1);
                    d0 = !d0 & 0xF;
                    if d1 >= d0 {
                        d1 -= d0;
                        sub_x(b, d1);
                        b.wall = bits;
                    }
                }
                0xC => {
                    // 0x13298: push +X
                    if w2 != 0 {
                        d1 = !d1;
                    }
                    d1 &= 0xF;
                    d1 = lsr(d1, w3).wrapping_add(w4).wrapping_add(1);
                    d0 &= 0xF;
                    if d1 >= d0 {
                        d1 -= d0;
                        add_x(b, d1);
                        b.wall = bits;
                    }
                }
                _ => {}
            }
            false
        }
        // 0x132CA: diagonal corners: push along the shorter way out (+$50 = 8 first).
        0x08 => {
            b.wall = 8;
            match w1 {
                0 => {
                    // 0x132EA
                    d0 &= 0xF;
                    d1 &= 0xF;
                    if d0 < d1 {
                        sub_x(b, d0.wrapping_add(1));
                    } else {
                        sub_z(b, d1.wrapping_add(1));
                    }
                }
                4 => {
                    // 0x1331C
                    d0 = !d0 & 0xF;
                    d1 &= 0xF;
                    if d0 < d1 {
                        add_x(b, d0.wrapping_add(1));
                    } else {
                        sub_z(b, d1.wrapping_add(1));
                    }
                }
                8 => {
                    // 0x13350
                    d1 = !d1 & 0xF;
                    d0 &= 0xF;
                    if d0 < d1 {
                        sub_x(b, d0.wrapping_add(1));
                    } else {
                        add_z(b, d1.wrapping_add(1));
                    }
                }
                0xC => {
                    // 0x13384
                    d0 = !d0 & 0xF;
                    d1 = !d1 & 0xF;
                    if d0 < d1 {
                        add_x(b, d0.wrapping_add(1));
                    } else {
                        add_z(b, d1.wrapping_add(1));
                    }
                }
                _ => return false,
            }
            again
        }
        // 0x133BA: street edge with a lower level (+$56 = G.$6B0 while below the edge line).
        0x0C => {
            if w1 != 0xC {
                // 0x133C4
                d1 &= 0xF;
                d1 = lsr(d1, w3).wrapping_add(w4).wrapping_add(1);
                d0 = !d0 & 0xF;
                if d1 <= d0 {
                    return false; // bls
                }
                d1 -= d0;
                if edge_rise(base, b).is_some_and(|q| d1 < q) {
                    sub_x(b, d1);
                    return false;
                }
                b.floor2 = base as i16;
            } else {
                // 0x1340A
                d1 &= 0xF;
                d1 = lsr(d1, w3).wrapping_add(w4).wrapping_add(1);
                d0 &= 0xF;
                if d1 < d0 {
                    return false; // bcs
                }
                d1 -= d0;
                if edge_rise(base, b).is_some_and(|q| d1 < q) {
                    add_x(b, d1);
                    return false;
                }
                b.floor2 = base as i16;
            }
            false
        }
        // 0x1344E: lower level edges.
        0x10 => match w1 {
            0 => {
                // 0x13468
                b.floor2 = base as i16;
                false
            }
            4 => {
                // 0x13472: push -Z
                d1 &= 0xF;
                if edge_rise(base, b).is_some_and(|q| q > d1) {
                    sub_z(b, d1.wrapping_add(1));
                    b.wall = bits;
                } else {
                    b.floor2 = base as i16;
                }
                false
            }
            8 => {
                // 0x134A8: push -X, another pass
                d0 &= 0xF;
                if edge_rise(base, b).is_some_and(|q| q > d0) {
                    sub_x(b, d0.wrapping_add(1));
                    return true;
                }
                b.floor2 = base as i16;
                false
            }
            0xC => {
                // 0x134D8: push +X, another pass. Unlike its neighbours it shifts G.$6B0 + floor,
                // not the height above the floor (the original compares without subtracting).
                let d2 = base.wrapping_add(b.floor as u16);
                d0 = d0.wrapping_neg() & 0xF;
                if (d2 as i16) > b.yi() && d2 >> 2 > d0 {
                    add_x(b, d0.wrapping_add(1));
                    return true;
                }
                b.floor2 = base as i16;
                false
            }
            _ => false,
        },
        // 0x1350A: nothing (class 0x3F, and word 0 = 0x20 which only the ceiling test uses).
        0x14 => false,
        // 0x1350E: sloped walls.
        0x18 => {
            if w1 != 0xC {
                // 0x13528
                if w2 == 0 {
                    d1 = !d1;
                }
                d1 &= 0xF;
                if d1 <= w4 {
                    // 0x1355E
                    d0 = (d0 & 0xF).wrapping_add(1);
                    sub_x(b, d0);
                    b.wall = bits;
                    return true;
                }
                d1 = lsl(d1 - w4, w3);
                d0 &= 0xF;
                if d0 >= d1 {
                    d0 = d0 - d1 + 1;
                    sub_x(b, d0);
                    b.wall = bits;
                }
            } else {
                // 0x13572
                if w2 == 0 {
                    d1 = !d1;
                }
                d1 &= 0xF;
                if d1 < w4 {
                    return false;
                }
                d1 = lsl(d1 - w4, w3);
                d0 &= 0xF;
                if d0 <= d1 {
                    d0 = d0.wrapping_sub(d1).wrapping_sub(1);
                    sub_x(b, d0);
                    b.wall = bits;
                }
            }
            false
        }
        // 0x135A8: sloped walls at a lower level edge (all variants share 0x135C2).
        0x1C => {
            d1 &= 0xF;
            if edge_rise(base, b).is_some_and(|q| d1 < q) {
                sub_z(b, d1.wrapping_add(1));
                return true;
            }
            // 0x135EA
            b.floor2 = base as i16;
            if w2 == 0 {
                d1 = !d1;
            }
            d1 &= 0xF;
            if d1 < w4 {
                return false;
            }
            d1 = lsl(d1 - w4, w3);
            d0 &= 0xF;
            if d0 <= d1 {
                d0 = d0.wrapping_sub(d1).wrapping_sub(1);
                sub_x(b, d0);
            }
            false
        }
        _ => false,
    }
}

// ------------------------------------------------------------------------------------------
// Game level routines
// ------------------------------------------------------------------------------------------

impl Game {
    /// The floor segment list of the current stage and area.
    fn terrain_segs(&self) -> &'static [FloorSegment] {
        let t = &self.assets.terrain;
        t.floor
            .get(self.w.stage as usize)
            .and_then(|a| a.get(self.cam.s2.area as usize))
            .map(|l| l.segs.as_slice())
            .unwrap_or(&[])
    }

    /// ROM address of the first segment of the current list (G.$6AC when `terrain.seg` is 0).
    pub fn terrain_list_addr(&self) -> u32 {
        let t = &self.assets.terrain;
        t.floor.get(self.w.stage as usize).and_then(|a| a.get(self.cam.s2.area as usize)).map(|l| l.addr).unwrap_or(0)
    }

    /// Number of segments of the current list.
    pub fn terrain_list_len(&self) -> usize {
        self.terrain_segs().len()
    }

    /// 0x12BE4: select the floor list of the stage and area and G.$6B0. In the vertical scene
    /// the original also draws the collision map at 0x91C000 / 0x91C400 (0x12C50).
    pub fn terrain_area_start(&mut self) {
        self.terrain.base = self.assets.terrain.floor_base[self.w.stage as usize & 7];
        self.terrain.seg = 0;
        if self.w.vertical_scene != 0 {
            self.terrain.scene = self.assets.terrain.scene_map.clone();
        }
    }

    /// 0x12C90: advance the floor list pointer by one segment while the next segment starts left
    /// of the camera (it never moves back). Last call of `objects_update`.
    pub fn terrain_advance(&mut self) {
        let segs = self.terrain_segs();
        let next = self.terrain.seg + 1;
        if next < segs.len() && segs[next].x0 < self.cam.s2.cam_x as i16 {
            self.terrain.seg = next;
        }
    }

    /// 0x12FE4 floor lookup for an object.
    pub fn terrain_floor_lookup(&self, b: &mut Body) {
        floor_lookup(self.terrain_segs(), self.terrain.seg, self.w.vertical_scene != 0, b);
    }

    /// Terrain class (0..63) under a point in scroll2 (0x1309C, 0x1361E) or in the vertical scene
    /// map (0x136E4): bits 10..15 of the tile attribute. 0 = nothing there.
    fn terrain_class(&self, x: i16, y: i16) -> u16 {
        let attr = if self.w.vertical_scene != 0 {
            self.terrain.scene.get(scene_cell(x as u16, y as u16)).map_or(0, |c| c[1])
        } else {
            self.vram.scroll2[scroll2_cell(x as u16, y as u16)].attr
        };
        attr_class(attr)
    }

    /// 0x1309C wall test: the class of the tile under the feet selects a flag byte (table 0x13B0C,
    /// stored in `flag` = G.$6B2) and a shape handler. Returns the carry flag (another pass).
    pub fn terrain_wall(&self, b: &mut Body, flag: &mut u8) -> bool {
        let d0 = b.xi();
        let d1 = b.zi().wrapping_add(b.floor);
        b.floor2 = 0; // clr.w $56(a6)
        let class = self.terrain_class(d0, d1) as usize;
        if class == 0 {
            return false;
        }
        let t = &self.assets.terrain;
        let st = self.w.stage as usize;
        let ar = self.cam.s2.area as usize;
        *flag = t.flags.get(st).and_then(|a| a.get(ar)).map(|f| f[class]).unwrap_or(0);
        shape_handler(&t.shapes[class], self.terrain.base, b, d0 as u16, d1 as u16)
    }

    /// 0x1361E ceiling test (players): the tile at the head. A class whose shape word 0 is 0x20
    /// pushes the object down to the tile edge and turns an upward velocity around.
    pub fn terrain_ceiling(&self, b: &mut Body) {
        let head = self.assets.terrain.head[b.id as usize & 3];
        let d1 = head.wrapping_add(b.yi()).wrapping_add(b.zi());
        let mut class = self.terrain_class(b.xi(), d1);
        if self.w.vertical_scene != 0 {
            // 0x13642 calls 0x136E4, which already returns the class, and 0x13674 rotates it by 6
            // again: the class is always 0 there, so the vertical scene has no ceiling.
            class = attr_class(class);
        }
        if self.assets.terrain.shapes[class as usize][0] == 0x20 {
            let d = (d1.wrapping_sub(b.zi()) as u16 & 0xF).wrapping_add(1);
            let y = b.yi().wrapping_sub(d as i16);
            put_hi(&mut b.y, y);
            if b.vy >= 0 {
                b.vy = !b.vy;
            }
        }
    }

    /// 0x12CB4: terrain collision of an object. Up to 5 passes of floor lookup and wall test (a
    /// handler that pushed the object asks for another pass), then the obstacle test, the wall
    /// flag byte and the floor snap. `me` = the object's handle (the obstacle test skips itself),
    /// `item` = a pool E object (they skip the obstacle test, tag +$82 = 0xC). Returns the
    /// obstacle the object ran into (the caller stores it at +$94).
    pub fn terrain_collide(&mut self, b: &mut Body, me: Handle, item: bool) -> Option<Handle> {
        b.wall = 0;
        let mut flag = 0u8;
        for _ in 0..5 {
            self.terrain_floor_lookup(b);
            if !self.terrain_wall(b, &mut flag) {
                break;
            }
        }
        let mut hit = None;
        if !item && b.state() >> 16 != 0x0202 && self.spawn.g.b4d5 == 0 {
            hit = self.terrain_obstacles(b, me); // 0x12E9A
        }
        b.wall |= flag;
        floor_snap(b);
        hit
    }

    /// 0x12E20: like 0x12CB4 without the obstacle test (called by enemy classes that are not
    /// ported yet).
    #[allow(dead_code)]
    pub fn terrain_collide_free(&mut self, b: &mut Body) {
        b.wall = 0;
        let mut flag = 0u8;
        for _ in 0..5 {
            self.terrain_floor_lookup(b);
            if !self.terrain_wall(b, &mut flag) {
                break;
            }
        }
        b.wall |= flag;
        floor_snap(b);
    }

    /// 0x12CAC: players: 0x12CB4, then the ceiling test 0x1361E.
    pub fn terrain_player(&mut self, b: &mut Body, me: Handle) -> Option<Handle> {
        let hit = self.terrain_collide(b, me, false);
        self.terrain_ceiling(b);
        hit
    }

    /// 0x12D4E: one floor lookup and the snap, no walls.
    pub fn terrain_floor_only(&mut self, b: &mut Body) {
        b.wall = 0;
        self.terrain_floor_lookup(b);
        floor_snap(b);
    }

    /// 0x12DAC (effects): one floor lookup 0x12FE4 (the code has no loop and no wall test 0x1309C; G.$6B4 is counted down once) and the snap.
    pub fn terrain_effect_floor(&mut self, b: &mut Body) {
        b.wall = 0;
        self.terrain_floor_lookup(b);
        floor_snap(b);
    }

    /// 0x4A76: empty both obstacle lists (area start).
    pub fn terrain_obstacles_reset(&mut self) {
        self.terrain.count = 0;
        self.terrain.prev = 0;
        self.terrain.buf = [0; OBSTACLE_SLOTS];
    }

    /// 0x4A96: register the object (a container) as an obstacle for the next frame and clear its
    /// +$AC flag, which 0x12E9A sets when something climbs on it.
    pub fn item_register_obstacle(&mut self, h: Handle) {
        if let Some(Slot::Item(i)) = slot_of(h) {
            self.items.slots[i].m.sb(0xAC, 0);
        }
        let t = &mut self.terrain;
        let n = t.count as usize;
        if n < OBSTACLE_SLOTS {
            t.buf[n] = h; // move.w a6,-(a4): the list grows downwards
        }
        t.count = t.count.wrapping_add(1);
    }

    /// Part of 0xE90E (0xE9B0): the list built this frame becomes the one 0x12E9A reads next.
    pub fn terrain_obstacles_swap(&mut self) {
        self.terrain.prev = self.terrain.count;
        self.terrain.count = 0;
    }

    /// `move.b #v,off(a1)` on a pool object (the obstacle test flags the obstacle).
    fn terrain_set_ram_byte(&mut self, h: Handle, off: usize, v: u8) {
        match slot_of(h) {
            Some(Slot::Item(i)) => self.items.slots[i].m.sb(off, v),
            Some(Slot::Prop(i)) => self.props.slots[i].m.sb(off, v),
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.sb(off, v),
            _ => {}
        }
    }

    /// 0x12E9A: push the object out of the registered obstacles (containers) or let it stand on
    /// them. Overlap uses the obstacle's body box (`+$44 + $48 * 12`, six words: Z offset, depth,
    /// X offset, width, Y offset, height) against the point (X, Y, Z) of the object. Standing above
    /// the top (or climbing it) copies the obstacle's floor; otherwise bit 7 of +$50 is set and the
    /// obstacle handle is returned (+$94 of the object) while the object is pushed out along X or
    /// Z, whichever way is shorter. Only the first overlapping obstacle counts.
    fn terrain_obstacles(&mut self, b: &mut Body, me: Handle) -> Option<Handle> {
        let n = self.terrain.prev as usize;
        for j in 0..n.min(OBSTACLE_SLOTS) {
            let h = self.terrain.buf[n - 1 - j]; // the list is read from its top pointer upwards
            if h == me {
                continue;
            }
            let Some(slot) = slot_of(h) else { continue };
            let o = match slot {
                Slot::Item(i) => &self.items.slots[i].b,
                Slot::Prop(i) => &self.props.slots[i].b,
                Slot::Enemy(i) => &self.enemies.slots[i].b,
                _ => continue,
            };
            let (ox, oy, oz) = (o.xi(), o.yi(), o.zi());
            let (ofloor, facing) = (o.floor, o.facing != 0);
            // The 68000 has a 24-bit address bus: the flag bits in the top byte of +$44 are ignored.
            let Some(bx) = self.assets.sprites.hitboxes.try_single(o.box_table & 0xFF_FFFF, o.body_box) else { continue };
            let (mx, my, mz) = (b.xi(), b.yi(), b.zi());
            // Z: (obstacle Z + box offset - Z + depth) must not exceed the depth (unsigned).
            let d0 = oz.wrapping_add(bx.zoff).wrapping_sub(mz).wrapping_add(bx.zsize);
            if d0 as u16 > bx.zsize as u16 {
                continue;
            }
            // X: the box is mirrored about the obstacle's X when it faces right.
            let mut xo = bx.xoff;
            let xw = bx.xsize;
            if facing {
                xo = xo.wrapping_neg().wrapping_sub(xw);
            }
            let d0 = ox.wrapping_add(xo).wrapping_add(xw).wrapping_sub(mx);
            if d0 as u16 > xw as u16 {
                continue;
            }
            let top = oy.wrapping_add(bx.yoff).wrapping_add(bx.ysize);
            if top <= my {
                // 0x12FB8: above the obstacle: stand on it.
                b.floor2 = top.wrapping_sub(ofloor).wrapping_sub(1);
                b.floor = ofloor;
                return None;
            }
            // Overlapping: distances to push out along Z (d4, distance d6) and X (d5, distance d0).
            let zlo = oz.wrapping_add(bx.zoff);
            let zhi = bx.zsize.wrapping_add(zlo);
            let center = ((zhi.wrapping_add(zlo)) as u16) >> 1;
            let (d4, d6) = if center >= mz as u16 {
                let d4 = zlo.wrapping_sub(mz);
                (d4, d4.wrapping_neg())
            } else {
                let d4 = zhi.wrapping_sub(mz);
                (d4, d4)
            };
            let left = ox.wrapping_add(xo);
            let right = xw.wrapping_add(left);
            let center = (right.wrapping_add(left) as u16) >> 1;
            let (d5, d0) = if center >= mx as u16 {
                let d5 = left.wrapping_sub(mx);
                (d5, d5.wrapping_neg())
            } else {
                let d5 = right.wrapping_sub(mx);
                (d5, d5)
            };
            b.wall |= 0x80; // bset.b #7,$50(a6)
            let rise = top.wrapping_sub(my) as u16; // obstacle top - Y
            let z_first = (d6 as u16) < (d0 as u16);
            // 0x12F98 (Z is the shorter way) or 0x12F7E (X): how high the obstacle is, compared
            // with the distance, decides between climbing it and being pushed out.
            let (d1, dist) = if z_first { ((rise >> 1) as i16, d6) } else { ((rise >> 2) as i16, d0) };
            if d1 <= dist {
                // 0x12FCA: low enough to climb: stand on top and flag the obstacle.
                self.terrain_set_ram_byte(h, 0xAC, 1);
                b.floor2 = d1.wrapping_add(my).wrapping_sub(ofloor);
                b.floor = ofloor;
            } else if z_first {
                put_hi(&mut b.z, mz.wrapping_add(d4));
            } else {
                put_hi(&mut b.x, mx.wrapping_add(d5));
            }
            return Some(h);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::terrain::{FloorList, TerrainData};
    use dino_assets::{AssetPack, HitBox, HitBoxBank};

    fn body(x: i16, y: i16, z: i16) -> Body {
        let mut b = Body::default();
        b.set_xi(x);
        b.set_yi(y);
        b.set_zi(z);
        b
    }

    fn shape(w: [u16; 8]) -> [u16; 8] {
        w
    }

    // ---- floor ----

    fn seg(slope: i16, x0: i16, x1: i16, base: i16, height: i16, wading: u8) -> FloorSegment {
        FloorSegment { slope, x0, x1, base, height, wading }
    }

    #[test]
    fn floor_lookup_flat_picks_first_segment_that_reaches_x() {
        let segs = [seg(0, 0, 99, 0x0105, 0x20, 0), seg(0, 100, 199, 7, 0x30, 0x80), seg(0, 200, 0x7FFF, 9, 0x40, 0)];
        let mut b = body(150, 0, 0);
        floor_lookup(&segs, 0, false, &mut b);
        assert_eq!((b.floor, b.b52, b.wading), (0x30, 7, 0x80));
        // The base word's low byte goes to +$52.
        let mut b = body(50, 0, 0);
        floor_lookup(&segs, 0, false, &mut b);
        assert_eq!((b.floor, b.b52), (0x20, 0x05));
        // From a later start the first segments are not looked at.
        let mut b = body(50, 0, 0);
        floor_lookup(&segs, 1, false, &mut b);
        assert_eq!(b.floor, 0x30);
    }

    #[test]
    fn floor_lookup_takes_the_fifth_segment_without_comparing() {
        // Four segments that all end left of X: the fifth is used even though it does not reach X.
        let segs = [seg(0, 0, 9, 0, 1, 0), seg(0, 10, 19, 0, 2, 0), seg(0, 20, 29, 0, 3, 0), seg(0, 30, 39, 0, 4, 0), seg(0, 40, 49, 0, 5, 0)];
        let mut b = body(100, 0, 0);
        floor_lookup(&segs, 0, false, &mut b);
        assert_eq!(b.floor, 5);
    }

    #[test]
    fn floor_lookup_slopes() {
        // Rising slope: height = base + (x - x0) / 4; the grounded object follows it.
        let segs = [seg(1, 100, 0x7FFF, 16, 0, 0)];
        let mut b = body(140, 0, 0);
        floor_lookup(&segs, 0, false, &mut b);
        assert_eq!((b.floor, b.yi()), (26, 26));
        // Falling slope (+8 != 0): height = base - 1 - (x - x0) / 4.
        let segs = [seg(1, 100, 0x7FFF, 16, 1, 0)];
        let mut b = body(140, 0, 0);
        floor_lookup(&segs, 0, false, &mut b);
        assert_eq!((b.floor, b.yi()), (5, 5));
        // A negative height is ignored (bmi).
        let mut b = body(240, 3, 0);
        floor_lookup(&segs, 0, false, &mut b);
        assert_eq!((b.floor, b.yi()), (0, 3));
        // Airborne above the slope stays up; below it lands (+$51 = 0).
        let segs = [seg(1, 100, 0x7FFF, 16, 0, 0)];
        let mut b = body(140, 40, 0);
        b.airborne = 1;
        floor_lookup(&segs, 0, false, &mut b);
        assert_eq!((b.floor, b.yi(), b.airborne), (26, 40, 1));
        let mut b = body(140, 20, 0);
        b.airborne = 1;
        floor_lookup(&segs, 0, false, &mut b);
        assert_eq!((b.floor, b.yi(), b.airborne), (26, 26, 0));
    }

    #[test]
    fn floor_lookup_vertical_scene_has_no_floor() {
        let segs = [seg(0, 0, 0x7FFF, 1, 0x20, 0x80)];
        let mut b = body(10, 0, 0);
        b.floor = 5;
        b.wading = 1;
        floor_lookup(&segs, 0, true, &mut b);
        assert_eq!((b.floor, b.wading, b.b52), (0, 0, 0));
    }

    #[test]
    fn floor_snap_follows_the_floor() {
        // Grounded: a higher floor lifts (the fraction is cleared), a lower one makes it fall.
        let mut b = body(0, 10, 0);
        b.y |= 0x8000;
        b.floor = 0x20;
        floor_snap(&mut b);
        assert_eq!((b.y, b.airborne), (0x20 << 16, 0));
        b.floor = 0x08;
        floor_snap(&mut b);
        assert_eq!((b.yi(), b.airborne), (0x20, 1));
        // Airborne: only lands when below the floor; +$56 counts.
        b.floor = 0x10;
        b.floor2 = 0x20;
        floor_snap(&mut b);
        assert_eq!((b.yi(), b.airborne), (0x30, 0));
        // The compare is signed.
        let mut b = body(0, -4, 0);
        b.airborne = 1;
        b.floor = -8;
        floor_snap(&mut b);
        assert_eq!((b.yi(), b.airborne), (-4, 1));
    }

    // ---- tile lookups ----

    #[test]
    fn scroll2_cell_index_matches_the_vram_layout() {
        use crate::game::vram::Vram;
        // Column = X / 16; row counts from the top: row = 63 - Y / 16.
        for (x, y) in [(0u16, 0u16), (0x10, 0x20), (0x3F0, 0x3F0), (0x123, 0x2A5), (0x7FF, 0x400), (0x555, 0x155)] {
            let col = (x >> 4) as usize & 0x3F;
            let row = 63 - ((y >> 4) as usize & 0x3F);
            assert_eq!(scroll2_cell(x, y), Vram::index2(col, row), "x {x:#x} y {y:#x}");
        }
    }

    #[test]
    fn scene_cell_index_is_32_columns_of_16_rows() {
        for (x, y) in [(0u16, 0u16), (0x10, 0x20), (0x1F0, 0xF0), (0x123, 0x85)] {
            let col = (x >> 4) as usize & 0x1F;
            let row = 15 - ((y >> 4) as usize & 0xF);
            assert_eq!(scene_cell(x, y), col * 16 + row, "x {x:#x} y {y:#x}");
        }
    }

    #[test]
    fn terrain_class_is_attribute_bits_10_to_15() {
        assert_eq!(attr_class(0x0000), 0);
        assert_eq!(attr_class(0x0400), 1);
        assert_eq!(attr_class(0xFC00), 0x3F);
        assert_eq!(attr_class(0x5400 | 0x03FF), 0x15);
    }

    // ---- shape handlers ----
    // d0 = X, d1 = Z + floor, as 0x1309C passes them. Tile coordinates are the low 4 bits.

    #[test]
    fn wall_shapes_push_to_the_tile_edge() {
        // Type 0 (flat walls). The bits (word 7) go to +$50 and word 5 asks for another pass.
        let w = |v1: u16, again: u16| shape([0, v1, 0, 0, 0x10, again, 0, 0x0004]);
        // push +Z: (~z & 15) + 1
        let mut b = body(0x105, 0, 0x203);
        assert!(!shape_handler(&w(0, 0), 0, &mut b, 0x105, 0x203));
        assert_eq!((b.zi(), b.wall), (0x203 + 13, 4));
        // push -Z: (z & 15) + 1
        let mut b = body(0x105, 0, 0x203);
        assert!(shape_handler(&w(4, 1), 0, &mut b, 0x105, 0x203));
        assert_eq!(b.zi(), 0x203 - 4);
        // push -X: (x & 15) + 1
        let mut b = body(0x105, 0, 0x203);
        shape_handler(&w(8, 0), 0, &mut b, 0x105, 0x203);
        assert_eq!(b.xi(), 0x105 - 6);
        // push +X: (-x & 15) + 1
        let mut b = body(0x105, 0, 0x203);
        shape_handler(&w(0xC, 0), 0, &mut b, 0x105, 0x203);
        assert_eq!(b.xi(), 0x105 + 12);
        // At the tile edge itself the push is 1 pixel on one side and 1 on the other.
        let mut b = body(0x110, 0, 0x200);
        shape_handler(&w(0xC, 0), 0, &mut b, 0x110, 0x200);
        assert_eq!(b.xi(), 0x111);
    }

    #[test]
    fn slanted_walls_depend_on_the_other_coordinate() {
        // Type 4 variant 0: push +Z by ((x or ~x) & 15 >> shift) + offset + 1 - (z & 15), when that is >= 0.
        let w = |flip: u16, shift: u16, off: u16| shape([4, 0, flip, shift, off, 0, 0, 0x0001]);
        let mut b = body(0x105, 0, 0x203);
        assert!(!shape_handler(&w(0, 0, 0), 0, &mut b, 0x105, 0x203));
        // flip = 0: d0 = ~0x105 & 15 = 10; + 0 + 1 = 11; - (z & 15 = 3) = 8
        assert_eq!((b.zi(), b.wall), (0x203 + 8, 1));
        // The shift halves the slope.
        let mut b = body(0x105, 0, 0x203);
        shape_handler(&w(0, 1, 0), 0, &mut b, 0x105, 0x203);
        assert_eq!(b.zi(), 0x203 + 3); // (10 >> 1) + 1 - 3
        // Nothing happens when the tile edge is behind the object (borrow).
        let mut b = body(0x105, 0, 0x20F);
        shape_handler(&w(0, 1, 0), 0, &mut b, 0x105, 0x20F);
        assert_eq!((b.zi(), b.wall), (0x20F, 0));
        // Variant 12: push +X from the z coordinate.
        let w = shape([4, 0xC, 1, 0, 0, 0, 0, 0x0002]);
        let mut b = body(0x105, 0, 0x203);
        shape_handler(&w, 0, &mut b, 0x105, 0x203);
        // flip != 0: d1 = ~0x203 & 15 = 12, +1 = 13, - (x & 15 = 5) = 8
        assert_eq!((b.xi(), b.wall), (0x105 + 8, 2));
    }

    #[test]
    fn corner_shapes_push_the_shorter_way() {
        let w = |v1: u16| shape([8, v1, 0, 0, 0, 1, 0, 0x0004]);
        // Variant 0: x & 15 = 2, z & 15 = 6: X is shorter, push -X by 3. +$50 = 8 always.
        let mut b = body(0x102, 0, 0x206);
        assert!(shape_handler(&w(0), 0, &mut b, 0x102, 0x206));
        assert_eq!((b.xi(), b.zi(), b.wall), (0x102 - 3, 0x206, 8));
        // x & 15 = 9, z & 15 = 6: Z is shorter, push -Z by 7.
        let mut b = body(0x109, 0, 0x206);
        shape_handler(&w(0), 0, &mut b, 0x109, 0x206);
        assert_eq!((b.xi(), b.zi()), (0x109, 0x206 - 7));
        // Variant 12 (both inverted): x & 15 = 2 -> 13, z & 15 = 6 -> 9: X is longer, push +Z by 10.
        let mut b = body(0x102, 0, 0x206);
        shape_handler(&w(0xC), 0, &mut b, 0x102, 0x206);
        assert_eq!((b.xi(), b.zi()), (0x102, 0x206 + 10));
        // Variant 4: ~x = 13, z = 6: push -Z? 13 >= 6 -> -Z by 7.
        let mut b = body(0x102, 0, 0x206);
        shape_handler(&w(4), 0, &mut b, 0x102, 0x206);
        assert_eq!(b.zi(), 0x206 - 7);
        // Variant 8: x = 2, ~z = 9: 2 < 9 -> push -X by 3.
        let mut b = body(0x102, 0, 0x206);
        shape_handler(&w(8), 0, &mut b, 0x102, 0x206);
        assert_eq!(b.xi(), 0x102 - 3);
    }

    #[test]
    fn street_edge_shapes_use_the_lower_level_height() {
        // Type 0x10 variant 0 only sets +$56 = G.$6B0.
        let mut b = body(0x105, 0, 0x203);
        b.floor2 = 0x77;
        assert!(!shape_handler(&shape([0x10, 0, 0, 0, 0, 0, 0, 0]), 0x40, &mut b, 0x105, 0x203));
        assert_eq!(b.floor2, 0x40);
        // Variant 4: pushes -Z when the object is low enough ((6B0 + floor - Y) / 4 > z & 15).
        let w = shape([0x10, 4, 0, 0, 0, 0, 0, 0x0008]);
        let mut b = body(0x105, 4, 0x203); // (0x40 - 4) / 4 = 15 > 3
        shape_handler(&w, 0x40, &mut b, 0x105, 0x203);
        assert_eq!((b.zi(), b.wall, b.floor2), (0x203 - 4, 8, 0));
        // Too high: +$56 gets the lower level instead.
        let mut b = body(0x105, 0x40, 0x203); // 0 >= ... not low enough
        shape_handler(&w, 0x40, &mut b, 0x105, 0x203);
        assert_eq!((b.zi(), b.floor2), (0x203, 0x40));
        // Variant 8 asks for another pass.
        let w = shape([0x10, 8, 0, 0, 0, 0, 0, 0]);
        let mut b = body(0x105, 4, 0x203);
        assert!(shape_handler(&w, 0x40, &mut b, 0x105, 0x203));
        assert_eq!(b.xi(), 0x105 - 6);
        // Variant 12 compares and shifts G.$6B0 + floor, without Y (a quirk of the original).
        let w = shape([0x10, 0xC, 0, 0, 0, 0, 0, 0]);
        let mut b = body(0x105, 4, 0x203);
        b.floor = 0x40; // (0x40 + 0x40) >> 2 = 32 > (-x & 15) = 11
        assert!(shape_handler(&w, 0x40, &mut b, 0x105, 0x203));
        assert_eq!(b.xi(), 0x105 + 12);
    }

    #[test]
    fn edge_with_offset_shapes() {
        // Type 0xC (variant != 12): push -X by d1 where d1 = (z & 15 >> shift) + offset + 1 - (~x & 15),
        // only when that is positive and below the height above the floor / 4.
        let w = shape([0xC, 0, 0, 0, 3, 0, 0, 0]);
        let mut b = body(0x10C, 0x20, 0x20F); // z & 15 = 15, +3 +1 = 19; ~x & 15 = 3; d1 = 16
        shape_handler(&w, 0x40, &mut b, 0x10C, 0x20F);
        // (0x40 + 0 - 0x20) / 4 = 8 <= 16: +$56 = G.$6B0, no push.
        assert_eq!((b.xi(), b.floor2), (0x10C, 0x40));
        let mut b = body(0x10C, 0x00, 0x20F);
        shape_handler(&w, 0x400, &mut b, 0x10C, 0x20F); // 0x400 / 4 = 0x100 > 16: push
        assert_eq!(b.xi(), 0x10C - 16);
    }

    #[test]
    fn sloped_wall_shapes() {
        // Type 0x18 (variant != 12): d1 = (~z & 15) - offset; when it is above the offset the push
        // is d0 - (d1 << shift) + 1 and ends the test; otherwise a full push and another pass.
        let w = shape([0x18, 0, 0, 1, 2, 0, 0, 0x0004]);
        let mut b = body(0x105, 0, 0x203);
        // ~z & 15 = 12 > 2: d1 = (12 - 2) << 1 = 20; x & 15 = 5 < 20: borrow, nothing happens.
        assert!(!shape_handler(&w, 0, &mut b, 0x105, 0x203));
        assert_eq!((b.xi(), b.wall), (0x105, 0));
        let mut b = body(0x10F, 0, 0x20D);
        // ~z & 15 = 2 <= offset 2: push -X by (x & 15) + 1 = 16, wall bits, another pass.
        assert!(shape_handler(&w, 0, &mut b, 0x10F, 0x20D));
        assert_eq!((b.xi(), b.wall), (0x10F - 16, 4));
        // The ceiling class 0x20 and class 0x3F (0x14) do nothing.
        let mut b = body(0x105, 0, 0x203);
        assert!(!shape_handler(&shape([0x20, 0, 0, 0, 0, 1, 0, 0xFF]), 0, &mut b, 0x105, 0x203));
        assert!(!shape_handler(&shape([0x14, 0, 0, 0, 0, 1, 0, 0xFF]), 0, &mut b, 0x105, 0x203));
        assert_eq!((b.xi(), b.zi(), b.wall), (0x105, 0x203, 0));
    }

    #[test]
    fn shifts_use_the_68000_register_rules() {
        assert_eq!(lsr(0x8000, 15), 1);
        assert_eq!(lsr(0xFFFF, 16), 0);
        assert_eq!(lsr(0xFFFF, 64), 0xFFFF); // the count is taken modulo 64
        assert_eq!(lsl(1, 15), 0x8000);
        assert_eq!(lsl(1, 16), 0);
    }

    // ---- Game level ----

    fn assets_with(tables: impl FnOnce(&mut AssetPack)) -> &'static AssetPack {
        let mut a = AssetPack::default();
        a.terrain = TerrainData {
            floor: vec![vec![FloorList { addr: 0x13D92, segs: vec![seg(0, 0, 0x7FFF, 0, 0x20, 0)] }; 4]; 8],
            floor_base: [0x20; 8],
            shapes: vec![[0; 8]; 64],
            flags: vec![vec![[0; 64]; 4]; 8],
            head: [0x50; 4],
            drop_x: [0; 12],
            drop_z: [0; 3],
            scene_map: vec![[0; 2]; 512],
        };
        tables(&mut a);
        Box::leak(Box::new(a))
    }

    #[test]
    fn collide_runs_walls_then_snaps_to_the_floor() {
        // Class 5 under the feet is a wall that pushes +X by 12 (type 0, variant 12, bits 4) and a
        // flag byte 1 for the area.
        let a = assets_with(|a| {
            a.terrain.shapes[5] = [0, 0xC, 0, 0, 0x10, 0, 0, 0x0004];
            a.terrain.flags[0][0][5] = 1;
        });
        let mut g = Game::new(a);
        // Tile (col 8, row from Y): put class 5 where X = 0x85, Z + floor = 0x20 + 0x20.
        let cell = scroll2_cell(0x85, 0x40);
        g.vram.scroll2[cell].attr = 5 << 10;
        let mut b = body(0x85, 0, 0x20);
        let hit = g.terrain_collide(&mut b, 0x1234, false);
        assert_eq!(hit, None);
        // floor lookup: flat 0x20; wall: x = 0x85 + ((-0x85 & 15) + 1) = 0x85 + 12; wall bits 4 | flag 1.
        assert_eq!((b.xi(), b.floor, b.wall), (0x85 + 12, 0x20, 4 | 1));
        // The object stood at height 0 above a floor at 0x20: grounded objects are lifted.
        assert_eq!(b.yi(), 0x20);
    }

    #[test]
    fn collide_allows_a_second_pass_when_the_shape_asks() {
        // Two walls in a row: the first (class 5) pushes +X and asks for another pass, the second
        // (class 6, one tile to the right) pushes +X again.
        let a = assets_with(|a| {
            a.terrain.shapes[5] = [0, 0xC, 0, 0, 0, 1, 0, 0x0004];
            a.terrain.shapes[6] = [0, 0xC, 0, 0, 0, 0, 0, 0x0008];
        });
        let mut g = Game::new(a);
        g.vram.scroll2[scroll2_cell(0x85, 0x40)].attr = 5 << 10;
        g.vram.scroll2[scroll2_cell(0x95, 0x40)].attr = 6 << 10;
        let mut b = body(0x85, 0x20, 0x20);
        g.terrain_collide(&mut b, 0, false);
        // 0x85 -> 0x91 (second pass sees column 9) -> 0x91 + ((-0x91 & 15) + 1) = 0xA1
        assert_eq!(b.xi(), 0xA1);
        assert_eq!(b.wall, 8);
    }

    #[test]
    fn floor_only_and_effect_variants_skip_the_walls() {
        let a = assets_with(|a| {
            a.terrain.shapes[5] = [0, 0xC, 0, 0, 0, 0, 0, 0x0004];
        });
        let mut g = Game::new(a);
        g.vram.scroll2[scroll2_cell(0x85, 0x40)].attr = 5 << 10;
        let mut b = body(0x85, 0x20, 0x20);
        b.wall = 0xFF;
        b.floor2 = 3;
        g.terrain_floor_only(&mut b);
        assert_eq!((b.xi(), b.wall, b.floor2), (0x85, 0, 3)); // +$56 is not cleared
        let mut b = body(0x85, 0x20, 0x20);
        g.terrain_effect_floor(&mut b);
        assert_eq!((b.xi(), b.floor), (0x85, 0x20));
        // 0x12E20 walks like 0x12CB4.
        let mut b = body(0x85, 0x20, 0x20);
        g.terrain_collide_free(&mut b);
        assert_eq!(b.xi(), 0x85 + 12);
    }

    #[test]
    fn ceiling_pushes_a_jumping_player_down() {
        // Class 0x15 has word 0 = 0x20: the head tile. head = 0x50, y = 0x30, z = 0x40 -> Y = 0xC0.
        let a = assets_with(|a| a.terrain.shapes[0x15] = [0x20, 0, 0, 0, 0, 0, 0, 0]);
        let mut g = Game::new(a);
        g.vram.scroll2[scroll2_cell(0x85, 0xC0)].attr = 0x15 << 10;
        let mut b = body(0x85, 0x30, 0x40);
        b.vy = 0x120;
        g.terrain_ceiling(&mut b);
        // y -= ((0x50 + 0x30) & 15) + 1 = 1; an upward velocity turns into ~v.
        assert_eq!(b.yi(), 0x2F);
        assert_eq!(b.vy, !0x120);
        // Falling objects keep their velocity.
        let mut b = body(0x85, 0x30, 0x40);
        b.vy = -5;
        g.terrain_ceiling(&mut b);
        assert_eq!((b.yi(), b.vy), (0x2F, -5));
        // No ceiling class: nothing.
        let mut b = body(0x185, 0x30, 0x40);
        g.terrain_ceiling(&mut b);
        assert_eq!(b.yi(), 0x30);
    }

    #[test]
    fn vertical_scene_reads_the_scene_map() {
        let a = assets_with(|a| {
            a.terrain.shapes[7] = [0, 8, 0, 0, 0, 0, 0, 0x0001]; // push -X
        });
        let mut g = Game::new(a);
        g.w.vertical_scene = 1;
        g.terrain.scene = vec![[0; 2]; 512];
        g.terrain.scene[scene_cell(0x45, 0x80)] = [0, 7 << 10];
        let mut b = body(0x45, 0x00, 0x80);
        g.terrain_collide(&mut b, 0, false);
        assert_eq!(b.xi(), 0x45 - 6);
        // No floor in the vertical scene.
        assert_eq!(b.floor, 0);
    }

    #[test]
    fn floor_pointer_follows_the_camera() {
        let a = assets_with(|a| {
            a.terrain.floor[0][0] = FloorList {
                addr: 0x13DB0,
                segs: vec![seg(0, 0, 99, 0, 1, 0), seg(0, 100, 199, 0, 2, 0), seg(0, 200, 0x7FFF, 0, 3, 0)],
            };
        });
        let mut g = Game::new(a);
        g.terrain_area_start();
        g.cam.s2.cam_x = 250;
        g.terrain_advance();
        assert_eq!(g.terrain.seg, 1); // one segment per call
        g.terrain_advance();
        assert_eq!(g.terrain.seg, 2);
        g.terrain_advance(); // never past the last segment
        assert_eq!(g.terrain.seg, 2);
        g.cam.s2.cam_x = 0;
        g.terrain_advance(); // and never back
        assert_eq!(g.terrain.seg, 2);
        assert_eq!(g.terrain_list_addr(), 0x13DB0);
    }

    // ---- obstacles ----

    /// A game with one container (pool E slot 3) registered as an obstacle: Z 0x40 + [-8, +8],
    /// X 0x100 + [-16, +16], height 0x30.
    fn obstacle_game(facing: u8) -> (Game, Handle) {
        let a = assets_with(|a| {
            let mut w = vec![0i16; 6 * 4];
            // box 1 at table 0x100000 + 12: zo, zs, xo, xw, yo, ys
            w[6..12].copy_from_slice(&[-8, 16, -16, 32, 0, 0x30]);
            a.sprites.hitboxes = HitBoxBank { base: 0x100000, words: w };
        });
        let mut g = Game::new(a);
        let h = g.items.handle(3);
        let it = &mut g.items.slots[3];
        it.b.live = 1;
        it.b.set_xi(0x100);
        it.b.set_yi(0);
        it.b.set_zi(0x40);
        it.b.facing = facing;
        it.b.box_table = 0xC010_0000; // the flag bits in the top byte are ignored
        it.b.body_box = 1;
        it.b.floor = 0;
        g.item_register_obstacle(h);
        g.terrain_obstacles_swap();
        (g, h)
    }

    #[test]
    fn obstacle_pushes_along_the_shorter_axis() {
        let (mut g, h) = obstacle_game(0);
        // Z overlap [0x38, 0x48], X overlap [0xF0, 0x110]. An enemy near the left edge (x 0xF4):
        // X distance 4 (to 0xF0 - 0xF4 = -4), Z distance 8: push in X, the obstacle's height / 4 = 12 > 4.
        let mut b = body(0xF4, 0, 0x40);
        let hit = g.terrain_obstacles(&mut b, 0x5555);
        assert_eq!(hit, Some(h));
        assert_eq!(b.wall & 0x80, 0x80);
        assert_eq!((b.xi(), b.zi()), (0xF0, 0x40));
        // Near the front (z 0x39): Z is shorter: push -Z.
        let mut b = body(0x100, 0, 0x39);
        g.terrain_obstacles(&mut b, 0x5555);
        assert_eq!((b.xi(), b.zi()), (0x100, 0x38));
        // Outside the box: nothing.
        let mut b = body(0x200, 0, 0x40);
        assert_eq!(g.terrain_obstacles(&mut b, 0x5555), None);
        assert_eq!(b.wall, 0);
        // The obstacle never pushes itself.
        let mut b = body(0xF4, 0, 0x40);
        assert_eq!(g.terrain_obstacles(&mut b, h), None);
    }

    #[test]
    fn obstacle_top_is_a_floor_and_low_ones_are_climbed() {
        let (mut g, h) = obstacle_game(0);
        // Above the top (y >= 0x30): stand on it: +$54 = obstacle floor, +$56 = top - floor - 1.
        let mut b = body(0x100, 0x30, 0x40);
        assert_eq!(g.terrain_obstacles(&mut b, 0x5555), None);
        assert_eq!((b.floor, b.floor2, b.wall), (0, 0x2F, 0));
        // Climbable: the shorter distance is not larger than height / 4: a tall rise pushes out,
        // a small one (y = 0x2E, rise 2 -> 0) lets the object climb and flags the obstacle.
        let mut b = body(0xF4, 0x2E, 0x40);
        assert_eq!(g.terrain_obstacles(&mut b, 0x5555), Some(h));
        assert_eq!(g.items.slots[3].m.b(0xAC), 1);
        assert_eq!((b.floor, b.floor2), (0, 0 + 0x2E - 0)); // d1 = 2 >> 2 = 0, + y 0x2E - floor 0
        assert_eq!(b.xi(), 0xF4);
    }

    #[test]
    fn mirrored_obstacle_boxes() {
        // Facing right: the box is mirrored about the obstacle's X: [-(xo + xw), -xo] = [-16, 16]
        // for xo = -16, xw = 32 -> -(-16) - 32 = -16: the same box here; use a lopsided one.
        let a = assets_with(|a| {
            let mut w = vec![0i16; 12];
            w[6..12].copy_from_slice(&[-8, 16, 0, 32, 0, 0x30]); // X from 0 to +32
            a.sprites.hitboxes = HitBoxBank { base: 0x100000, words: w };
        });
        let mut g = Game::new(a);
        let h = g.items.handle(0);
        let it = &mut g.items.slots[0];
        it.b.live = 1;
        it.b.set_xi(0x100);
        it.b.set_zi(0x40);
        it.b.box_table = 0x100000;
        it.b.body_box = 1;
        it.b.facing = 1; // mirrored: X from -32 to 0
        g.item_register_obstacle(h);
        g.terrain_obstacles_swap();
        let mut b = body(0xF0, 0, 0x40);
        assert_eq!(g.terrain_obstacles(&mut b, 0), Some(h));
        let mut b = body(0x110, 0, 0x40);
        assert_eq!(g.terrain_obstacles(&mut b, 0), None);
    }

    #[test]
    fn obstacle_list_is_one_frame_late_and_shares_its_storage() {
        let (mut g, h) = obstacle_game(0);
        // This frame's registration only becomes visible after the swap.
        let mut g2 = Game::new(g.assets);
        g2.item_register_obstacle(h);
        assert_eq!((g2.terrain.count, g2.terrain.prev), (1, 0));
        g2.terrain_obstacles_swap();
        assert_eq!((g2.terrain.count, g2.terrain.prev), (0, 1));
        // The storage is shared: a registration of the new frame overwrites the old list's slot
        // while that list is still being read.
        let other = g.items.handle(5);
        g.item_register_obstacle(other);
        assert_eq!(g.terrain.buf[0], other);
        assert_eq!(g.terrain.prev, 1);
        g.terrain_obstacles_reset();
        assert_eq!((g.terrain.count, g.terrain.prev, g.terrain.buf[0]), (0, 0, 0));
    }

    #[test]
    fn items_skip_the_obstacle_test_and_so_does_state_0202() {
        let (mut g, _h) = obstacle_game(0);
        let mut b = body(0xF4, 0, 0x40);
        g.terrain_collide(&mut b, 0x5555, true);
        assert_eq!(b.wall & 0x80, 0);
        let mut b = body(0xF4, 0, 0x40);
        b.set_state(0x0202_0000);
        g.terrain_collide(&mut b, 0x5555, false);
        assert_eq!(b.wall & 0x80, 0);
        let mut b = body(0xF4, 0, 0x40);
        g.terrain_collide(&mut b, 0x5555, false);
        assert_eq!(b.wall & 0x80, 0x80);
    }

    #[test]
    fn hit_box_bank_rejects_stale_pointers() {
        let bank = HitBoxBank { base: 0x100000, words: vec![1; 12] };
        assert!(bank.try_single(0x100000, 1).is_some());
        assert!(bank.try_single(0x100000, 2).is_none());
        assert!(bank.try_single(0, 0).is_none());
        let _ = HitBox::default();
    }

    // ---- differential test against the original code ----

    /// `re/harness/terrain_oracle.py` runs the original 68000 routines on random setups and writes
    /// the results to `re/out/terrain_vectors.txt` (not committed: it derives from the ROM). Every
    /// vector is replayed here on the port. Skipped when the file or the ROM set is missing.
    #[test]
    fn terrain_matches_the_68000_code() {
        use std::collections::HashMap;
        use std::path::Path;
        let (vectors, rom) = (Path::new("../../../re/out/terrain_vectors.txt"), Path::new("../../../dino.zip"));
        if !vectors.exists() || !rom.exists() {
            eprintln!("terrain_matches_the_68000_code: skipped (run re/harness/terrain_oracle.py)");
            return;
        }
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let text = std::fs::read_to_string(vectors).unwrap();
        let (mut failures, mut total) = (Vec::new(), 0);
        let mut per_entry: HashMap<String, (u32, u32)> = HashMap::new();
        for line in text.lines() {
            let (inp, out) = line.split_once(" | ").unwrap();
            let kv = |s: &str| -> HashMap<String, String> {
                s.split(' ').filter_map(|t| t.split_once('=')).map(|(k, v)| (k.to_string(), v.to_string())).collect()
            };
            let (i, o) = (kv(inp), kv(out));
            let hx = |m: &HashMap<String, String>, k: &str| u32::from_str_radix(&m[k], 16).unwrap();
            let dec = |k: &str| i[k].parse::<u32>().unwrap();
            let cells = |k: &str| -> Vec<(usize, u16)> {
                if i[k] == "-" {
                    return Vec::new();
                }
                i[k].split(',').map(|c| {
                    let (a, b) = c.split_once(':').unwrap();
                    (usize::from_str_radix(a, 16).unwrap(), u16::from_str_radix(b, 16).unwrap())
                }).collect()
            };
            let mut g = Game::new(assets);
            g.w.stage = dec("stage") as u8;
            g.cam.s2.area = dec("area") as u8;
            g.w.vertical_scene = dec("vert") as u8;
            g.spawn.g.b4d5 = dec("b4d5") as u8;
            g.terrain.seg = dec("seg") as usize;
            g.terrain.base = dec("base") as i16;
            g.cam.s2.cam_x = hx(&i, "cam") as u16;
            for (c, a) in cells("s2") {
                g.vram.scroll2[c].attr = a;
            }
            g.terrain.scene = vec![[0; 2]; 512];
            for (c, a) in cells("scene") {
                g.terrain.scene[c][1] = a;
            }
            let mut b = Body::default();
            b.x = hx(&i, "x") as i32;
            b.y = hx(&i, "y") as i32;
            b.z = hx(&i, "z") as i32;
            b.vy = hx(&i, "vy") as u16 as i16;
            b.airborne = hx(&i, "air") as u8;
            b.floor = hx(&i, "fl") as u16 as i16;
            b.floor2 = hx(&i, "fl2") as u16 as i16;
            b.wall = hx(&i, "wall") as u8;
            b.b52 = hx(&i, "b52") as u8;
            b.wading = hx(&i, "wad") as u8;
            b.id = hx(&i, "id") as u8;
            b.set_state(hx(&i, "st"));
            b.facing = hx(&i, "fac") as u8;
            let me: Handle = 0xB274;
            let mut handles: Vec<Handle> = Vec::new();
            let mut slots = Vec::new();
            if i["obs"] != "-" {
                for ob in i["obs"].split(';') {
                    let f: Vec<u32> = ob.split(',').map(|v| u32::from_str_radix(v, 16).unwrap()).collect();
                    let it = &mut g.items.slots[f[0] as usize];
                    it.b.live = 1;
                    it.b.set_xi(f[1] as u16 as i16);
                    it.b.set_yi(f[2] as u16 as i16);
                    it.b.set_zi(f[3] as u16 as i16);
                    it.b.facing = f[4] as u8;
                    it.b.box_table = f[5];
                    it.b.body_box = f[6] as u8;
                    it.b.floor = f[7] as u16 as i16;
                    it.m.sb(0xAC, 0x55);
                    handles.push(g.items.handle(f[0] as usize));
                    slots.push(f[0] as usize);
                }
            }
            if dec("melist") != 0 {
                handles.insert(0, me);
            }
            let n = handles.len();
            g.terrain.prev = n as u16;
            for (j, h) in handles.iter().enumerate() {
                g.terrain.buf[n - 1 - j] = *h;
            }
            let (entry, tag) = (i["entry"].as_str(), dec("tag"));
            let mut w94 = 0xBEEFu16;
            let mut carry = 0;
            let mut flag = 0u8;
            match entry {
                "collide" => w94 = g.terrain_collide(&mut b, me, tag == 0xC).unwrap_or(w94),
                "player" => w94 = g.terrain_player(&mut b, me).unwrap_or(w94),
                "floor_only" => g.terrain_floor_only(&mut b),
                "free" => g.terrain_collide_free(&mut b),
                "effect" => g.terrain_effect_floor(&mut b),
                "lookup" => g.terrain_floor_lookup(&mut b),
                "wall" => carry = g.terrain_wall(&mut b, &mut flag) as u32,
                "ceiling" => g.terrain_ceiling(&mut b),
                "advance" => g.terrain_advance(),
                e => panic!("unknown entry {e}"),
            }
            let mut got: Vec<(&str, u32)> = vec![
                ("x", b.x as u32), ("y", b.y as u32), ("z", b.z as u32), ("vy", b.vy as u16 as u32),
                ("wall", b.wall as u32), ("air", b.airborne as u32), ("b52", b.b52 as u32), ("wad", b.wading as u32),
                ("fl", b.floor as u16 as u32), ("fl2", b.floor2 as u16 as u32), ("obst94", w94 as u32),
            ];
            if entry == "wall" {
                got.push(("carry", carry));
                got.push(("flag", flag as u32));
            }
            if entry == "advance" {
                // G.$6AC is a ROM address in the original: list address + 12 * segment.
                got.push(("ptr", hx(&i, "addr") + 12 * g.terrain.seg as u32));
            }
            let mut bad = Vec::new();
            for (k, v) in got {
                if hx(&o, k) != v {
                    bad.push(format!("{k}: original {:x} port {v:x}", hx(&o, k)));
                }
            }
            let ac: Vec<u32> = slots.iter().map(|&s| g.items.slots[s].m.b(0xAC) as u32).collect();
            let want: Vec<u32> = if o["ac"] == "-" { Vec::new() } else { o["ac"].split(',').map(|v| u32::from_str_radix(v, 16).unwrap()).collect() };
            if ac != want {
                bad.push(format!("obstacle +$AC: original {want:?} port {ac:?}"));
            }
            total += 1;
            let e = per_entry.entry(entry.to_string()).or_default();
            e.0 += 1;
            if !bad.is_empty() {
                e.1 += 1;
                if failures.len() < 8 {
                    failures.push(format!("{}\n    {}", line, bad.join(", ")));
                }
            }
        }
        let mut summary: Vec<_> = per_entry.into_iter().collect();
        summary.sort();
        eprintln!("terrain vectors: {total}; (entry, vectors, mismatches) {summary:?}");
        assert!(failures.is_empty(), "{} mismatches, first ones:\n{}", summary.iter().map(|e| e.1.1).sum::<u32>(), failures.join("\n"));
    }
}
