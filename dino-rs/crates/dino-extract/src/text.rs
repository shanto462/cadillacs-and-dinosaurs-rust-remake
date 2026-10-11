//! Text, HUD and flow tables (owner: flow / HUD port). Copies the regions listed in
//! `dino_assets::text::TABLES`.

use crate::rom::Rom;
use dino_assets::text::{TABLES, TextData};

pub fn extract(rom: &Rom) -> TextData {
    TextData { blobs: TABLES.iter().map(|&(a, n, _)| (a, (a..a + n).map(|x| rom.u8(x)).collect())).collect() }
}
