//! The data model shared by `dino-extract` (producer) and `dino-game` (consumer).
//!
//! Rule of the project: anything that is *data* in the original program (tiles, palettes,
//! sprite frames, hitboxes, maps, stat tables, scripts) is extracted into these types.
//! Anything that is *code* in the original is ported by hand to native Rust in `dino-game`.
//!
//! Original ROM addresses are kept as identifiers (`rom_addr`, `AnimTable::rom_addr`, ...) so the
//! ported code can name the same tables the 68000 code names, and so every value can be traced
//! back to `re/ghidra/decomp.c`.

use std::collections::HashMap;

pub mod effect;
pub mod enemy;
pub mod frontend;
pub mod player;
pub mod sprites;
pub mod stage;
pub mod terrain;
pub mod text;
pub use sprites::*;
pub use stage::StageData;

/// Screen size of the original game in pixels.
pub const SCREEN_W: u32 = 384;
pub const SCREEN_H: u32 = 224;

/// Pixel index that is never drawn.
pub const TRANSPARENT: u8 = 15;

/// Scroll3 (32x32) codes in use are 0x1C00..0x1FFF (SPEC stage 3); the bank mapper hides others.
pub const TILES32_FIRST: u16 = 0x1C00;
/// Scroll1 (8x8) codes in use are 0x000..0xFFF.
pub const TILES8_COUNT: u16 = 0x1000;

/// Everything the game needs, extracted from the ROM set in memory.
#[derive(Default)]
pub struct AssetPack {
    /// 16x16 tiles, 256 palette indices per tile (row-major), indexed by the original tile code.
    pub tiles16: Vec<u8>,
    /// 32x32 scroll3 tiles for codes `TILES32_FIRST..TILES32_FIRST + 1024`, 1024 indices each.
    pub tiles32: Vec<u8>,
    /// 8x8 scroll1 tiles: for code c and half h (column parity), 64 indices at `(c * 2 + h) * 64`.
    pub tiles8: Vec<u8>,
    /// Background maps, palettes and camera tables.
    pub stages: StageData,
    /// Sprite compositions, frames, animation tables, hitboxes and attack records.
    pub sprites: SpriteData,
    /// ROM palettes (16 colours each) from the OBJ palette bank, by ROM palette number.
    pub obj_palette_bank: Vec<[u16; 16]>,
    /// For each (stage, area): the ROM palette number loaded into OBJ palette slot 0..31.
    pub obj_palette_sets: HashMap<(u8, u8), [u16; 32]>,
    /// Player movement and combat tables (`player.rs`).
    pub player: player::PlayerData,
    /// Enemy, stage-script and item tables (`enemy.rs`).
    pub enemy: enemy::EnemyData,
    /// Draw-path lists (shadows, splashes) and effect animations (`effect.rs`).
    pub effects: effect::EffectData,
    /// HUD and flow strings and tables (`text.rs`).
    pub text: text::TextData,
    /// Terrain collision tables (`terrain.rs`).
    pub terrain: terrain::TerrainData,
    /// Front end scenes: picture pages, scene palettes, ROM tables (`frontend.rs`).
    pub frontend: frontend::FrontendData,
}

impl AssetPack {
    pub fn tile16_count(&self) -> u32 {
        (self.tiles16.len() / 256) as u32
    }
}

/// Converts one 16-bit colour word 0xBRGB (brightness, red, green, blue nibbles) to RGBA.
/// Brightness scales all channels; the game fades by rewriting only the brightness nibble.
pub fn colour(word: u16) -> [u8; 4] {
    let bright = 0x0f + (((word >> 12) as u32) << 1);
    let ch = |n: u16| ((n as u32 & 0xf) * 0x11 * bright / 0x2d) as u8;
    [ch(word >> 8), ch(word >> 4), ch(word), 255]
}
