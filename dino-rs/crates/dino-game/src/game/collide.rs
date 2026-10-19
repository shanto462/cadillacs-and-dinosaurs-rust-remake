//! Hit box test, ported from 0x128F8 and its four table-shape variants (0x12916 / 0x129A8 /
//! 0x12A5C / 0x12B0E). All arithmetic is 16-bit wrapping and the compare is unsigned, exactly
//! as the 68000 code does it (touching edges count as a hit).

use super::body::Body;
use dino_assets::{HitBox, SpriteData};

pub fn boxes(s: &SpriteData, b: &Body, index: u8) -> Vec<HitBox> {
    // The 68000 has a 24-bit address bus: pool E objects keep flag bits in the top byte of the
    // table pointer (a container has 0xC0100000) and the box lookup ignores them.
    let table = b.box_table & 0xFF_FFFF;
    // A table outside the bank (the Bludge's tongue, class 0: table 0x337EA with the box index 0xC5 that its raw frame record gives) has no box here: no hit.
    if b.multi_box { s.hitboxes.multi(table, index) } else { s.hitboxes.try_single(table, index).into_iter().collect() }
}

fn axis(a0: i16, asz: i16, b0: i16, bsz: i16) -> bool {
    (a0.wrapping_sub(b0).wrapping_add(asz) as u16) <= (asz.wrapping_add(bsz) as u16)
}

/// Left edge of a box on the X axis; facing mirrors the box about the object's X.
pub fn left(b: &Body, h: &HitBox) -> i16 {
    b.xi().wrapping_add(if b.facing != 0 { h.xoff.wrapping_neg().wrapping_sub(h.xsize) } else { h.xoff })
}

fn overlap(a: &Body, ha: &HitBox, b: &Body, hb: &HitBox) -> bool {
    axis(a.zi().wrapping_add(ha.zoff), ha.zsize, b.zi().wrapping_add(hb.zoff), hb.zsize)
        && axis(left(a, ha), ha.xsize, left(b, hb), hb.xsize)
        && axis(a.yi().wrapping_add(ha.yoff), ha.ysize, b.yi().wrapping_add(hb.yoff), hb.ysize)
}

/// One pair of boxes (z, x, y; the same test as `boxes_hit`).
pub fn box_hits(a: &Body, ha: &HitBox, b: &Body, hb: &HitBox) -> bool {
    overlap(a, ha, b, hb)
}

/// Attacker box `ia` (usually +$49) against defender box `ib` (usually +$48).
/// Returns the first overlapping pair, which the hit spark code uses.
pub fn boxes_hit(s: &SpriteData, a: &Body, ia: u8, b: &Body, ib: u8) -> Option<(HitBox, HitBox)> {
    let bb = boxes(s, b, ib);
    for ha in boxes(s, a, ia) {
        for hb in &bb {
            if overlap(a, &ha, b, hb) {
                return Some((ha, *hb));
            }
        }
    }
    None
}
