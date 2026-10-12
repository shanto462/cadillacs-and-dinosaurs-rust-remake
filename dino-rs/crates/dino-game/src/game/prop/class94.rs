//! Pool B class 94 (`+$20 = 0x178`, handler 0x921DA): the scroll of the checker board of a profile
//! card (M1-06). Created by the card handlers with the card number as variant (unused).
//!
//! Mode 0 (0x921EE) sets the life to 0x128 frames. Every update of mode 2 (0x92204) takes one off;
//! while it is not 0 the scroll3 camera Y (G.$7E8) goes down by 2 (295 updates: 590 pixels, the board
//! slides by), at 0 the object frees itself. Mode 4 and 6 free it at once (0x9221A).

use super::Prop;
use crate::game::Game;

/// Offsets of this class.
mod f {
    /// word: frames left (0x128).
    pub const LIFE: usize = 0xA0;
    /// long: cleared (not used).
    pub const L_AA: usize = 0xAA;
}

impl Game {
    /// 0x921DA pool B class 94 (jump table 0x921E6: mode 0 -> 0x921EE, 2 -> 0x92204, 4 and 6 -> 0x9221A).
    pub fn prop_class94(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2; // 0x921EE
                p.m.sw(f::LIFE, 0x128);
                p.m.sl(f::L_AA, 0);
            }
            2 => {
                let life = p.m.w(f::LIFE).wrapping_sub(1); // 0x92204
                p.m.sw(f::LIFE, life);
                if life == 0 {
                    p.b.mode = 2; // 0x92214 (then the free routine clears it)
                    self.prop_free(p, i); // jmp 0x4938
                } else {
                    self.cam.s3.cam_y = self.cam.s3.cam_y.wrapping_sub(2); // subi.w #2,$7E8(a5)
                }
            }
            _ => self.prop_free(p, i), // 0x9221A
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

    /// The first update only sets the life; then 295 updates take 2 off G.$7E8 each (590 pixels) and the
    /// 296th frees the object.
    #[test]
    fn the_board_scrolls_295_times_and_the_object_frees_itself() {
        if assets().is_none() {
            return;
        }
        let mut g = Game::new(assets().unwrap());
        g.flow_profile_spawn_b(94, 0);
        g.pool_b_update();
        assert_eq!(g.cam.s3.cam_y, 0, "the first update does not scroll");
        for n in 1..=295 {
            g.pool_b_update();
            assert_eq!(g.cam.s3.cam_y, (-2 * n as i32) as u16, "update {n}");
            assert!(g.props.slots[23].b.live != 0);
        }
        g.pool_b_update();
        assert_eq!(g.cam.s3.cam_y, (-590i32) as u16);
        assert_eq!(g.props.slots[23].b.live, 0, "freed in the 296th update");
    }
}
