//! Extractor: turns the user's own `dino.zip` into an [`AssetPack`].
//!
//! It only reads bytes. No original code is executed or emulated: tables found by reverse
//! engineering the 68000 program are decoded here into plain Rust types.

pub mod boss;
pub mod effect;
pub mod enemy;
pub mod frontend;
pub mod gfx;
pub mod player;
pub mod rom;
pub mod script;
pub mod sound;
pub mod sprites;
pub mod stage;
pub mod terrain;
pub mod text;

use dino_assets::{AssetPack, TILES8_COUNT, TILES32_COUNT};
use rom::Rom;
use std::path::Path;

/// Frame records to decode on demand, on top of the registered animation tables. The exactness
/// harness (`dino-game --load-state`) lists the frame pointers that the objects of a MAME snapshot
/// hold; the extractor decodes them here so the game still never reads ROM bytes. Addresses that
/// are not valid frame records are skipped (the loader then reports them).
#[derive(Clone, Debug, Default)]
pub struct ExtraFrames {
    /// 10-byte records of ordinary objects (end up in `SpriteData::frames`).
    pub sprite: Vec<u32>,
    /// 10-byte records of effect objects (`EffectData::frames`).
    pub effect: Vec<u32>,
    /// 8-byte raw-list records (`EffectData::raw_frames`).
    pub raw: Vec<u32>,
}

pub fn extract(rom_path: &Path) -> Result<AssetPack, String> {
    extract_with(rom_path, &ExtraFrames::default())
}

/// Like [`extract`], and also decodes the frame records in `extra`.
pub fn extract_with(rom_path: &Path, extra: &ExtraFrames) -> Result<AssetPack, String> {
    let rom = Rom::load(rom_path)?;
    let (obj_palette_bank, obj_palette_sets) = sprites::obj_palettes(&rom);
    Ok(AssetPack {
        tiles16: gfx::decode_tiles16(&rom.gfx),
        tiles32: gfx::decode_tiles32(&rom.gfx, 0, TILES32_COUNT),
        tiles8: gfx::decode_tiles8(&rom.gfx, TILES8_COUNT),
        stages: stage::extract(&rom),
        sprites: sprites::extract(&rom, extra),
        obj_palette_bank,
        obj_palette_sets,
        player: player::extract(&rom),
        enemy: enemy::extract(&rom),
        boss: boss::extract(&rom),
        effects: effect::extract(&rom, extra),
        text: text::extract(&rom),
        terrain: terrain::extract(&rom),
        frontend: frontend::extract(&rom),
        script: script::extract(&rom),
        sound: sound::extract(&rom),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Needs the user's ROM set at `../dino.zip` (skipped otherwise). A frame record that no
    /// registered table reaches is decoded on demand; an address that is not a frame record is skipped.
    /// The candidate is a frame record of a placeholder class (`game/placeholder.rs`) whose animation table is not
    /// registered yet: 0x86D00 (class 65, BARREL). If the base set has it, the test fails, so pick a new candidate
    /// (cargo run -p dino-game -- --list-placeholders).
    #[test]
    fn extra_frames_are_decoded_on_demand() {
        let path = Path::new("../../../dino.zip");
        if !path.exists() {
            eprintln!("extra frames test skipped: user's dino.zip is absent");
            return;
        }
        let base = extract(path).unwrap();
        let addr = 0x86D00u32;
        assert!(!base.sprites.frame_by_addr.contains_key(&addr), "frame record {addr:#x} is registered: pick a new candidate");
        let extra = ExtraFrames { sprite: vec![addr, 0x10, 0x56C61], effect: vec![0x9252C, 0x10], raw: vec![0x10] };
        let pack = extract_with(path, &extra).unwrap();
        let id = pack.sprites.frame_by_addr[&addr];
        assert_eq!(pack.sprites.frame(id).rom_addr, addr);
        assert!(!pack.sprites.frame_by_addr.contains_key(&0x10));
        assert!(!pack.sprites.frame_by_addr.contains_key(&0x56C61));
        assert!(pack.effects.frame_by_addr.contains_key(&0x9252C));
        assert!(!pack.effects.frame_by_addr.contains_key(&0x10));
        assert!(!pack.effects.raw_by_addr.contains_key(&0x10));
    }
}
