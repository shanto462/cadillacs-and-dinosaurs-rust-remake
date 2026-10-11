//! Hit box test, ported from 0x128F8 and its four table-shape variants (0x12916 / 0x129A8 /
//! 0x12A5C / 0x12B0E). All arithmetic is 16-bit wrapping and the compare is unsigned, exactly
//! as the 68000 code does it (touching edges count as a hit).

use super::body::Body;
use dino_assets::{HitBox, SpriteData};

pub fn boxes(s: &SpriteData, b: &Body, index: u8) -> Vec<HitBox> {
    if b.multi_box { s.hitboxes.multi(b.box_table, index) } else { vec![s.hitboxes.single(b.box_table, index)] }
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
