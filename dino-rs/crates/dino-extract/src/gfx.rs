//! Tile decoding from the original 4 bits-per-pixel planar format.
//!
//! One pixel row of 8 pixels takes 4 bytes: byte `p` holds bitplane `p` (weight `1 << p`)
//! and the most significant bit is the leftmost pixel. Pen 15 is transparent.
//! * 16x16 tile: 128 bytes, row = 8 bytes (pixels 0-7, then 8-15).
//! * 32x32 tile: 512 bytes, row = 16 bytes.
//! * 8x8 tile: 64 bytes, row = 8 bytes, but only one 4-byte half is used (two glyphs share a cell).

#[inline]
fn plane_pixel(group: &[u8], x: usize) -> u8 {
    let bit = 7 - (x & 7);
    ((group[0] >> bit) & 1)
        | (((group[1] >> bit) & 1) << 1)
        | (((group[2] >> bit) & 1) << 2)
        | (((group[3] >> bit) & 1) << 3)
}

/// Decodes every 16x16 tile to one byte per pixel (0x8000 tiles x 256 pixels = 8 MB).
pub fn decode_tiles16(raw: &[u8]) -> Vec<u8> {
    let count = raw.len() / 128;
    let mut out = vec![0u8; count * 256];
    for (src, dst) in raw.as_chunks::<128>().0.iter().zip(out.chunks_exact_mut(256)) {
        for y in 0..16 {
            for x in 0..16 {
                dst[y * 16 + x] = plane_pixel(&src[y * 8 + (x / 8) * 4..][..4], x);
            }
        }
    }
    out
}

/// One pixel of a 32x32 tile.
pub fn pixel32(raw: &[u8], code: u32, x: usize, y: usize) -> u8 {
    let base = (code as usize * 512) % raw.len();
    plane_pixel(&raw[base + y * 16 + (x / 8) * 4..][..4], x)
}

/// One pixel of an 8x8 tile; `half` picks the left (0) or right (1) 4-byte group.
pub fn pixel8(raw: &[u8], code: u32, half: usize, x: usize, y: usize) -> u8 {
    let base = (code as usize * 64) % raw.len();
    plane_pixel(&raw[base + y * 8 + half * 4..][..4], x)
}

/// Decodes the scroll3 tiles `first..first + count` (1024 indices each).
pub fn decode_tiles32(raw: &[u8], first: u16, count: u16) -> Vec<u8> {
    let mut out = Vec::with_capacity(count as usize * 1024);
    for code in first as u32..first as u32 + count as u32 {
        for y in 0..32 {
            for x in 0..32 {
                out.push(pixel32(raw, code, x, y));
            }
        }
    }
    out
}

/// Decodes scroll1 tiles: both 8x8 halves of codes `0..count`.
pub fn decode_tiles8(raw: &[u8], count: u16) -> Vec<u8> {
    let mut out = Vec::with_capacity(count as usize * 128);
    for code in 0..count as u32 {
        for half in 0..2 {
            for y in 0..8 {
                for x in 0..8 {
                    out.push(pixel8(raw, code, half, x, y));
                }
            }
        }
    }
    out
}
