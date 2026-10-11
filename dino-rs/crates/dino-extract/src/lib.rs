//! Extractor: turns the user's own `dino.zip` into an [`AssetPack`].
//!
//! It only reads bytes. No original code is executed or emulated: tables found by reverse
//! engineering the 68000 program are decoded here into plain Rust types.

pub mod gfx;
pub mod rom;
pub mod sprites;
pub mod stage;

use dino_assets::{AssetPack, TILES8_COUNT, TILES32_FIRST};
use rom::Rom;
use std::path::Path;

pub fn extract(rom_path: &Path) -> Result<AssetPack, String> {
    let rom = Rom::load(rom_path)?;
    let (obj_palette_bank, obj_palette_sets) = sprites::obj_palettes(&rom);
    Ok(AssetPack {
        tiles16: gfx::decode_tiles16(&rom.gfx),
        tiles32: gfx::decode_tiles32(&rom.gfx, TILES32_FIRST, 1024),
        tiles8: gfx::decode_tiles8(&rom.gfx, TILES8_COUNT),
        stages: stage::extract(&rom),
        sprites: sprites::extract(&rom),
        obj_palette_bank,
        obj_palette_sets,
    })
}
