//! Extracts enemy, stage-script and item data. Formats: `re/specs/enemy/SPEC.md`.

use crate::rom::Rom;
use crate::sprites::OBJECT_ANIM_TABLES;
use dino_assets::enemy::*;
use dino_assets::AttackRecord;
use std::collections::HashMap;

/// Spawn list pointer tables (by stage, then 4 areas): list 1 (14-byte records), list 2 (16-byte).
const LIST1_TABLE: u32 = 0xFC2E;
const LIST2_TABLE: u32 = 0xFC4E;
/// Wave script tables by stage (word offsets per wave), used by pool B class 15 (0x7B336).
const WAVE_TABLE: u32 = 0x7B37C;
/// Trigger values at or above this never fire (list sentinels).
const SENTINEL: u16 = 0x9FFF;

/// Small tables copied as byte blobs: (ROM address, length, user).
const TABLES: &[(u32, u32, &str)] = &[
    (0x4FA30, 0xC0, "Hopper direction steps (0x4F374)"),
    (0x4F3DA, 0xE, "Hopper frame X offsets (0x4F3B4)"),
    // M5-01 monster services: dispatch, reaction mapping and colour flashes.
    (0x11CC4, 0x20, "monster hit reaction mapping"),
    (0x11CE4, 8, "monster guard dispatch"),
    (0x4F4C8, 0x14, "Hopper colour flash delays"),
    (0x4F4DC, 8, "Hopper colour pairs by stage"),
    (0x49448, 0x14, "Shivat colour flash delays"),
    (0x4945C, 4, "Shivat colour pairs"),
    (0x53956, 0x14, "Mack colour flash delays"),
    (0x5396A, 4, "Mack colour pairs"),
    // Scene actors (M3-20): the Cadillac's rider rows (0x42350) and offsets (0x42374); the crouching men's variants (0x4D992).
    (0x42350, 0x3C, "Cadillac rider rows and offsets (class 16)"),
    (0x4D992, 0x1C, "crouching men variants: name, +$84, animation (class 27)"),
    // Pool B class 20, GO picture (0x7BB68).
    (0x7BCAC, 14, "GO scroll1 palette 10 colours"),
    (0x7BCEA, 40, "GO glyph cells"),
    (0x7BD12, 40, "GO erase cells"),
    // Pool B class 47 (0x84282), manhole hole patch tile codes (M3-21).
    (0x842F6, 24, "B47 manhole hole patch tile codes (0x84282)"),
    // Pool B class 57 (0x85B64), the floor labels of the lift: 56 labels of 3 glyph words (0x85BF6) and the 12 glyphs (L, B and the digits) of 4 cells (0x85D6C).
    (0x85BF6, 0x150, "B57 lift floor labels, 3 glyph numbers each (0x85B64)"),
    (0x85D6C, 0xC0, "B57 lift label glyphs, 2 x 2 scroll2 cells each (0x85D46)"),
    // Pool B class 64 (0x86B64), the cage posts of the lift: the position offsets by variant (0x86BBE) and the frame record pointers (0x86BD4).
    (0x86BBE, 0x10, "B64 cage post offsets from the scroll1 position, 4 x (dx, dy) (0x86B64)"),
    (0x86BD4, 0x10, "B64 cage post frame record pointers (0x86B64)"),
    // Pool B class 35 (0x81FDE), manhole cover hero offsets (M3-21).
    (0x820FE, 0x70, "B35 manhole cover hero offsets (0x81FDE)"),
    // Waterfall pictures (M3-22). 0x90C038 and 0x90C438 are scroll2 destinations, not tables.
    (0x821E8, 0x3C, "B36 var 0 picture pointers and 4 pictures of 4 code words (0x821C0)"),
    (0x8229C, 0xC8, "B36 var 4 picture pointers and 4 pictures of 20 code words (0x82228)"),
    (0x823EE, 0x3A0, "B37 destinations, picture tables and 12 pictures of 32 code words (0x823AE)"),
    (0x82880, 0x838, "B38 destinations, picture tables and 12 pictures of 80 code words (0x827D8)"),
    // Pool B class 2 (0x7542C), swamp water phase words, records and tile codes (M6-15).
    (0x754FE, 0xF6, "B2 water phase and records (0x7542C)"),
    (0x75746, 0xCB0, "B2 water tile codes (0x7542C)"),
    // Pool B class 27 (0x7EEBA), yard picture pointers and the four pictures (M6-15).
    (0x7F1D4, 0x550, "B27 yard pictures (0x7EEBA)"),
    // Pool B class 7 (0x76CD2), near pictures (var 0) and far-layer pictures (var 4) (M6-16).
    (0x76D64, 0x22, "B7 var 0 picture offsets and tiles (0x76CD2)"),
    (0x76DF6, 0x18, "B7 var 4 far-layer pictures (0x76CD2)"),
    // Pool B class 8 (0x76E0E), swamp water surface phase, records and tile codes (M8-05).
    (0x76ECE, 0xEA, "B8 water phase and records (0x76E0E)"),
    (0x770F6, 0xD80, "B8 water tile codes (0x76E0E)"),
    // Pool B class 39 (0x830C6), swamp water surface pictures (M8-05).
    (0x831BE, 0x4E0, "B39 swamp water pictures (0x830C6)"),
    // Pool B classes 61, 63, 41, 52, 54 (M8-17): Episode 7 lab and library events.
    (0x865BA, 0x36, "B61 lab lights A step pointers (0x864DA)"),
    (0x86AD8, 0x64, "B63 lab lights B step pointers (0x869E6)"),
    (0xBF292, 0x360, "B61/B63 lab lights palette colors (0x864DA, 0x869E6)"),
    (0x83944, 0xC0, "B41 console blinking lights picture tables and tile codes (0x8383A)"),
    (0x851DE, 0x8C, "B52 bookshelf burst tile codes (0x85178)"),
    (0x8526A, 0x8C, "B52 bookshelf burst tile attributes (0x85178)"),
    (0x8562E, 0x80, "B54 cage wall smash scroll1 block 1 tile codes (0x855A8)"),
    (0x856B2, 0x80, "B54 cage wall smash scroll1 block 2 tile codes (0x855A8)"),
    // Pool B class 5 (0x769E2), chimney smoke: four pictures of two scroll3 code words (0x76A3E).
    (0x76A5A, 16, "B5 chimney smoke pictures (0x769E2)"),
    // Pool B class 6 (0x76A70), interior flames. Destinations and picture bases, then the pictures
    // (0x76B4C through 0x76CCC, three variants of four pictures of 16 code words).
    (0x76AE6, 12, "B6 flame destinations, vars 0/4/8 (0x76A70)"),
    (0x76AF2, 12, "B6 flame picture bases (0x76A70)"),
    (0x76B4C, 0x180, "B6 flame pictures (0x76A70)"),
    // Pool B class 10 (0x785F0), foreground fire strips. Pointers and every word the writers read.
    (0x78664, 0x15A, "B10 var 0 fire pictures (0x7863E)"),
    (0x78898, 0x128, "B10 var 4 fire pictures (0x787F4)"),
    // Pool B class 9 (0x77E76), lightning: pause table, VRAM pointers, bolt tile pictures.
    (0x77F38, 16, "B9 pause table (0x77E76)"),
    (0x77F68, 12, "B9 bolt destination VRAM pointers (0x77F48)"),
    (0x77F74, 80, "B9 bolt pictures (0x77F74)"),
    // Pool B class 9 (0x77E76), lightning flashes: timing tables, palette tables and source palettes.
    (0x77FF6, 16, "B9 flash sequence 1 frame counts (0x77FD2)"),
    (0x78036, 64, "B9 flash sequence 1 palette table (0x78002)"),
    (0x78076, 208, "B9 flash sequence 1 source pointers (0x78002)"),
    (0x78170, 16, "B9 flash sequence 2 frame counts (0x78146)"),
    (0x781B0, 256, "B9 flash sequence 2 palette table (0x7817C)"),
    (0x782B0, 832, "B9 flash sequence 2 source pointers (0x7817C)"),
    (0x0BEE52, 0x0400, "B9 lightning sequence 2 foreground palettes 1 (0x7817C)"),
    (0x0C1492, 0x09C0, "B9 lightning sequence 2 foreground palettes 2 (0x7817C)"),
    (0x0C36D2, 0x0F60, "B9 lightning sequence 1 sky palettes (0x78002)"),
    // Pool B class 13 (0x79218), burning huts. Each blob is the pointer table and the four pictures.
    (0x79400, 0x2E0, "B13 var 0 fire pictures (0x79270)"),
    (0x79812, 0x250, "B13 var 2 fire pictures (0x796E6)"),
    (0x79B5E, 0x1C0, "B13 var 4 fire pictures (0x79A62)"),
    (0x79EAE, 0x2E0, "B13 var 6 fire pictures (0x79D1E)"),
    (0x7A21A, 0x310, "B13 var 8 fire pictures (0x7A18E)"),
    (0x7A5A4, 0x310, "B13 var A fire pictures (0x7A52A)"),
    (0x7A93A, 0x2B0, "B13 var C fire pictures (0x7A8B4)"),
    (0x7AC64, 0x310, "B13 var E fire pictures (0x7ABEA)"),
    (0x7AFEE, 0xD0, "B13 var 10 fire pictures (0x7AF74)"),
    // score
    (0x10292, 0x90, "BCD score values (0x10036), indexed by byte offset"),
    (0x325F8, 0x28, "kill score offset by name id (0x325E6)"),
    // damage enemy -> player (0x6422)
    (0x2C3AE, 0x27 * 32, "damage scale by name id and level"),
    (0x2C896, 0x100, "random damage reduction by HP band"),
    (0x2C88E, 8, "character factor"),
    (0x2C996, 0x20, "low-HP factor"),
    // damage player -> enemy (0x2BA2A)
    (0xA320C, 0x100, "counter-hit factor by enemy attack id"),
    (0xA32FA, 0xA0, "weak-point attack ids and factors by +$84"),
    (0x2BECE, 0x4E0, "damage scale by +$84*8 + level (+$84 = name * 4, up to 0x98; the table ends at 0x2C3AE)"),
    (0x2BC4E, 0x240, "HP bar factor"),
    (0x2BB4E, 0x100, "in-bar HP factor"),
    // class 2 (FERRIS / GNEISS / DRIVER)
    (0x33B28, 0x40, "entrance 4 idle timers"),
    (0x33CDA, 0x30, "entrance 6 fade palette pointers"),
    (0x33FA4, 8, "ambush threshold by aggression"),
    (0x34186, 0x20, "pause at slot by level"),
    (0x34210, 8, "state E pause by aggression"),
    (0x34338, 0x60, "combo choice by variant"),
    (0x343E8, 0x30, "combo scripts"),
    (0x345D0, 0x20, "recovery index"),
    (0x345F0, 0x20, "recovery frames by level band"),
    (0x3524C, 0x404, "approach slots (+ the bytes after them, read by the slot-index quirk)"),
    (0x352F8, 0x20, "attack positions"),
    (0x3533C, 0x20, "attack side offset"),
    (0x353B6, 0x18, "attack chance at slot"),
    (0x3546A, 0x10, "reposition duration"),
    (0x354B2, 0x60, "reposition choice"),
    (0x35796, 0x60, "dash attack chance"),
    (0x35832, 0x30, "walk speed ids"),
    (0x35882, 0x30, "run speed ids"),
    (0x1033E, 0x40, "stage time by stage*8 + area*2 (0x1031A)"),
    // class 10 (HAMMER T. / BLK ELMER / WRENCH T.)
    (0x3BAA6, 0x40, "class 10 entrance 8 wait times"),
    (0x3BD84, 0x10, "class 10 pause times"),
    (0x3C0EE, 0x60, "class 10 attack kind by variant row and rand"),
    (0x3C1E6, 0x3C, "class 10 attack scripts, rows of 6"),
    (0x3C35A, 8, "class 10 wind up by aggression"),
    (0x3C3DE, 8, "class 10 run speed ids by level band"),
    (0x3C5D0, 0x20, "class 10 recovery by level"),
    (0x3CC04, 0x28, "class 10 slots (dx, dz)"),
    (0x3CD14, 0x20, "class 10 direction map"),
    (0x3CFCA, 0x60, "class 10 attack chance by variant row and level"),
    (0x3D120, 0x30, "class 10 walk speed ids by variant row and level band"),
    (0x32566, 0x80, "repeat count by level band and rand (0x32540)"),
    // class 20 (PUNK / THUG)
    (0x458E6, 0x40, "class 20 entrance 0 wait times"),
    (0x459E0, 0x40, "class 20 entrance 2 wait times"),
    (0x45E64, 0x20, "class 20 action pick by variant row and rand"),
    (0x45F54, 0x10, "class 20 hop counts"),
    (0x4634E, 8, "class 20 wait by aggression (state 0xA)"),
    (0x463F6, 0x58, "class 20 attack scripts, rows of 8"),
    (0x460F8, 0x10, "class 20 choice by rand"),
    (0x46AF6, 0x18, "class 20 slots (dx, dz)"),
    (0x46CF0, 0x10, "class 20 steering noise"),
    (0x46EC6, 0x40, "class 20 script row by variant and rand"),
    (0x46D98, 0x100, "class 20 attack chance by variant row and level (a variant above 3 or a level above 15 reads on into the code that follows, 0x46DD8: the original does the same)"),
    (0x46E22, 0x28, "class 20 offsets of the walk at the target by row"),
    (0x4697E, 0x20, "class 20 recovery index by rand"),
    (0x4699E, 0x20, "class 20 recovery frames by index and level band"),
    (0x47224, 0x20, "class 20 walk speed ids by variant row and aggression"),
    (0x45C62, 8, "class 20 fade destination"),
    (0x45C6E, 0x20, "class 20 fade source palette pointers by variant and step"),
    // class 14 (BLADE / RAZOR, 0x4020E) and its parts: classes 60 (0x580C6), 34 (0x5129E), 41 (0x55866)
    (0x40D88, 0x10, "class 14 pause after a block by aggression (read as bytes: the even bytes are 0xFF)"),
    (0x40EA8, 0x14, "class 14 attack animation by +$BB * 4"),
    (0x4162E, 0x14, "class 14 attack starts: +$BB, +$BC, animation, pad (4 bytes per kind)"),
    (0x413B8, 0x40, "class 14 side step speeds by rand"),
    (0x4177E, 0x10, "class 14 knives to throw by rand"),
    (0x417E2, 0x20, "class 14 wait random add by rand"),
    (0x410CC, 0x20, "class 14 pause after a dash by rand"),
    (0x406C0, 0x40, "class 14 flee timers: 0x406C0 positive, 0x406E0 negative, by rand & 0x1E"),
    (0x7441A, 0x40, "class 14 pause at a point by level"),
    (0x7445A, 0x40, "class 14 stand wait by level"),
    (0x74DA2, 0x40, "class 14 reload time by rand"),
    (0x74EA2, 0x40, "class 14 reload time add by level"),
    (0x73214, 0x200, "class 14 decision 1 by variant, rand and distance class"),
    (0x73414, 0x80, "class 14 chance to start decision 2 (near)"),
    (0x73494, 0x80, "class 14 chance to start decision 2 (same facing)"),
    (0x73514, 0x200, "class 14 decision 2 without knives"),
    (0x73714, 0x200, "class 14 decision 2 with one knife"),
    (0x73914, 0x200, "class 14 decision 2 with two knives"),
    (0x74298, 0x10, "class 14 guard: chance by aggression and variant"),
    (0x742A8, 0x42, "class 14 guard: duration by variant"),
    (0x742EA, 0x10, "class 14 guard: chance to keep blocking by aggression and variant"),
    (0x742FA, 0x20, "class 14 guard: pause after the block by aggression and variant"),
    (0x58178, 0x190, "class 60 part placement for BLADE: dx, dy, dz, animation by the frame byte"),
    (0x58308, 0x190, "class 60 part placement for RAZOR"),
    // class 13 (POACHER J / SKINNER / GUTTER, 0x3DDEA): the decision tables, the timers and the small tables of the states.
    (0x72A94, 0x300, "class 13 decision 1 by variant, rand and X class (0x4005C)"),
    (0x72D94, 0x180, "class 13 far gate by variant row, aggression and X class (0x40118); the index has no bound (variants of Episode 8 read past 6 rows), so the whole space up to the far decision table"),
    (0x72F14, 0x300, "class 13 far decision by variant, rand and X class (0x40156)"),
    (0x3E28E, 0x40, "class 13 flee timers: 0x3E28E positive, 0x3E2AE negative, by rand & 0x1E (0x3E1FA)"),
    (0x3EADA, 0x20, "class 13 pause after a rifle shot by rand (0x3EA2A)"),
    (0x3ECE2, 0x20, "class 13 pause after a dash by rand (0x3ECA8)"),
    (0x3EF1E, 0x38, "class 13 melee animation by +$BA * 8 + +$BB (0x3EE82)"),
    (0x3FA3E, 0x40, "class 13 melee starts: +$BA, +$BB, animation, pad (4 bytes per kind) (0x3F998)"),
    (0x3F636, 0x20, "class 13 close in timer random add by rand (0x3F5E2)"),
    (0x3F6E8, 0x20, "class 13 rifle shots by rand (0x3F6B2)"),
    (0x3F530, 0x40, "class 13 drift speeds by rand (0x3F4D2)"),
    (0x3FC84, 0x20, "class 13 wait after a hit target by rand (0x3FC60)"),
    (0x743BA, 0x60, "class 13 close in timer by level and variant (0x3F5E2)"),
    (0x7435A, 0x60, "class 13 pause at a point by level and variant (0x3E4DC)"),
    (0x74E22, 0x40, "class 13 melee step add by level (0x3F0F6)"),
    (0x3FBC4, 0x20, "class 13 kick speeds by X class: vx words, then vy words (0x3FB40)"),
    (0x3EE5E, 0x10, "class 13 guard counter after the hop back by aggression (0x3EE2C)"),
    (0x741D4, 0x18, "class 13 guard chance by aggression and variant (0x3DFC6)"),
    (0x741EC, 0x63, "class 13 guard counter: a base byte and 32 random adds per variant (0x3DFC6)"),
    (0x7424F, 0x19, "class 13 chance to keep guarding by aggression and variant (0x3E068)"),
    (0x74268, 0x30, "class 13 pause after a guard by aggression and variant (0x3E068)"),
    (0x3EFFC, 0xC, "class 13 fade destination palette by variant (0x3EFD0)"),
    (0x3F008, 0x30, "class 13 fade source palette pointers by variant and step (0x3EFD0)"),
    // class 15 (flying pieces, 0x41F1A): five rows of 20 bytes after a header of five offsets
    (0x42050, 0x6E, "class 15 piece rows: dx, dy, dz, vx, vy, vz, attack flag"),
    // shared pool A library 0x32xxx
    (0x32988, 0x30, "distance class tables (0x3293C)"),
    (0x32CF0, 0x1C, "six slots round the target: dx, dz (0x32C5E)"),
    (0x32DF8, 0x10, "three slots round the target: dx (0x32D90); 0x32D86 reads it with +$BD up to 6 (the slot of 0x32C5E): the code bytes after the table are read on purpose"),
    // pool B class 73 (bird spawner)
    (0x89B32, 16, "bird variants by random 0..7"),
    // class 55 (opening scene thugs)
    (0x5794A, 12, "scripted thug X step by variant"),
    (0x57956, 12, "scripted thug Z step by variant"),
    // HP
    (0x31454, 0x27 * 64, "HP by name id and level"),
    (0x31E52, 10 * 48, "boss HP"),
    // rank / level / aggression
    (0x18094, 0x20, "initial aggression + 1 by stage*4 + area"),
    (0x180B4, 0x46, "initial rank: thresholds and row offsets"),
    (0x180FA, 0x140, "initial rank rows"),
    (0x1823A, 0x40, "per-stage rank delta / max / min"),
    (0x182E4, 0x100, "aggression Markov tables (0x182E4, 0x18364)"),
    (0x184C4, 0x1E, "extra-life rank table"),
    (0x1860C, 0xC0, "level by rank and players"),
    (0x187B8, 0x7C, "rank-up periods and rank constants"),
    // hit reaction (0x2A606)
    (0x2B158, 0x1B4, "dizzy masks (0x2B158, 0x2B168, 0x2B180, 0x2B200, 0x2B28C)"),
    (0x2B604, 0x290, "lie-down time: random add and rows by +$84"),
    (0x2B8C4, 0x40, "counter-attack delay"),
    (0x2CB46, 0x9A, "throw / slide / knockback records"),
    (0x2CB80, 0x20, "weapon break masks"),
    (0x2E68C, 0xA0, "hit animation tables by +$84"),
    (0x2E724, 0xA0, "grabbed animation tables by +$84"),
    (0x2D0E0, 0x15AC, "grabbed enemy placement (0x2B35A): 4 pointers by the player's +$20, each a word table by the enemy's +$84 / 2 whose rows (dx, dy, animation, facing; 8 bytes) are indexed by the player's frame attach index"),
    // drops and items
    (0xB64D2, 0xFA0, "drop rows: fallback 0xB64D2, containers 0xB64F2, enemies 0xB6D12 (rows 0..0x3A; Episode 1 wave 4 uses 0x30, D4a IT-P1)"),
    (0x28B10, 0x48, "food heal and score offset"),
    (0x2A164, 0x11, "treasure score offsets (index 0..16: treasure 17 reads index 16, D4a IT-P1)"),
    // pool E: containers, hand positions, guns
    (0x29860, 0x0E, "container thrown attack id by variant"),
    (0x2986E, 0xF0, "container throw velocities: table of 4 pointers, then 4 x 7 records (vx, ax, vy, ay)"),
    (0x28F24, 8, "container fuse: blink count, blink period by stage"),
    (0x29552, 0x15A, "container debris: chance masks, then the piece lists 0x295F4 / 0x29650"),
    // M5-04 pool C class 21 (0x94672): the puff lists by variant, a word offset table then (ay, +$60, 12 byte records) per variant.
    (0x9484A, 0x962, "pool C class 21 puff lists by variant (0x94698)"),
    // M5-08 gunshot muzzle flash (class 31, 0x96800) and ricochet spark (class 33, 0x96B78): the velocity sets by selector (0x96986, to the end of the
    // velocity sets at 0x96A7E), the flash and spark records by kind with the scatter rows (0x96D2A to 0x96DAA).
    (0x96986, 0xF8, "gunshot flash velocities: 4 pointers by the selector, then (vx, vy) words by direction (0x967EA)"),
    (0x96D2A, 0x80, "ricochet records: 6 offsets by kind 4..9, 6 records of 14 bytes (0x96B78), scatter rows for facing (0x96D8A) and facing 0 (0x96D9A)"),
    // M5-07 burn death and flames: the pieces of the charred body (0x93DB4), the offsets of the torch glow by frame byte (0x29C26), the flame lists of the enemies by name id (0x2CA36).
    (0x93DB4, 0xA0, "pool C class 15 burn piece records (0x93D48)"),
    (0x29C26, 0x90, "torch glow offsets by the frame byte of the torch (0x29B6E)"),
    (0x2CA36, 0xE8, "flame lists of a burning enemy by name id (0x2AD98)"),
    // M5-06 pool C class 24 (0x959E6): the piece lists by variant, a word offset table then (ay, +$60, 8 byte records) per variant.
    (0x95B80, 0x2A8, "pool C class 24 piece lists by variant (0x95A0C)"),
    // M5-05 pool C class 19 (0x94338): the height of the dizzy stars over a head by the owner's class (a word per class).
    (0x94398, 0xCA, "pool C class 19 star height by owner class (0x94376)"),
    // M6-14 pool C class 5 (0x6192, 0x61F6): the variant byte by (rand & 1) + the owner's class word, and by the character id.
    (0x61E6, 0x10, "pool C class 5 bubble variant by (rand & 1) + owner class (0x61D8)"),
    (0x6244, 4, "pool C class 5 bubble variant by character id (0x6234)"),
    // M6-14 pool C class 29 (0x965F2): a word index by the variant byte, then dx, dy, vx, vy and the life byte.
    (0x9665E, 0x20, "pool C class 29 smoke ring parameter blocks (0x965F2)"),
    (0x24CC6, 0x20, "gun flag by weapon variant (0x24C3A)"),
    (0x2494A, 0x10, "gun-kind flag by item kind low nibble (0x248F6)"),
    (0x2CBA0, 0x20, "weapon attack tables (0x259C8)"),
    (0x2855A, 0x80, "ammo box refill by weapon variant: max ammo, kind, +$A2"),
    (0x122230, 0x600, "hand position of held items: 4 pointers, then (dx, dy) by character and attach index"),
    (0x120A20, 0x1810, "hand animation of held items: 4 pointers, then pointer rows by +$A2, then ids by attach index"),
    (0x26408, 0x200, "weapon tables: explosion chain offsets, throw vectors 0x26534.., hand offsets 0x265B4, depth speeds 0x265D4"),
    (0x28814, 0xE0, "bullet records (vx, animation, attack id, aim), pointer rows, vertical speeds, aim tables (class 3)"),
    (0x2A484, 0x18, "pellet records (vx, vy) by variant (class 10)"),
    (0x62AE, 0x28, "weapon reach box by character: 5 words (0x6248)"),
    (0x9E5C, 4, "bazooka sound by voice bank (0x9E08)"),
    (0xA498, 6, "shotgun sound by voice bank (0xA442)"),
    (0x9FAA, 12, "eating sound by voice bank, normal and Hannah rows (0x9F54)"),
    (0xA66A, 0x30, "pan byte by screen X / 8 (0x9E60 break sound, 0x9EA2, 0x9EDC, 0x9F12)"),
    // the stage clock (pool B class 31, 0x81046)
    (0x81194, 0x48, "clock: the slot template, 18 longs copied into the pool D slot (0x81088): the record, the sprite list of 5 digit and colon sprites"),
    (0x811CA, 0x14, "clock: tile code of the digits 0 to 9 (0x8114E)"),
    // pool B class 18 (respawn air strike, 0x7B65E)
    (0x7B7A6, 0x20, "air strike records: 5 x (X offset, depth offset, variant) then a negative end word"),
    // Episode 1 transition props (M3-17). Code words only; the classes read them in column order.
    (0x78A8E, 42, "B11 metal door codes, 3 columns of 7 (0x78A5A)"),
    (0x7B14A, 56, "B14 hole codes, 4 columns of 7 (0x7B108)"),
    (0x7B2E6, 72, "B14 wooden door codes, 4 columns of 9 (0x7B2A4)"),
    // pool B class 3 (mine wall, 0x763F6)
    (0x76666, 0x18, "prop B3: body box table (0x76432)"),
    (0x76530, 0x40, "prop B3 var 0: tile codes (0x76488)"),
    (0x766FA, 0x36, "prop B3 var 8: tile codes (0x766CC)"),
    // pool B class 1 (chain-link gate / fence, 0x75238)
    (0x75414, 0x18, "prop B1: body box table (0x7525E)"),
    (0x7535E, 0x9C, "prop B1: attribute rewrite table (0x7532E)"),
    // Episode 1 hall door and window (M3-18, class 12, 0x78B52). Read at run time from dino.zip.
    (0x78BDA, 984, "B12 hall door: 3 block pointers then 3 pictures of 9 x 9 pairs (0x78BAC)"),
    (0x7906E, 42, "B12 window codes, 3 columns of 7 (0x7903A)"),
    (0x79152, 160, "B12 glass shard records, 10 of 16 bytes (0x93362)"),
    (0x79200, 24, "B12 window hit boxes, 2 records of 6 words (0x78FD2)"),
    // pool B class 40 (steam pipes, 0x836A4, M5-14)
    (0x83786, 6, "prop B40: kind bytes by variant / 4 (0x83786)"),
    (0x8378C, 20, "prop B40: scroll2 base tile addresses (0x8378C)"),
    (0x837A0, 10, "prop B40: scroll2 column step offsets (0x837A0)"),
    (0x837E8, 22, "prop B40: tile codes (0x837E8)"),
    (0x8380A, 24, "prop B40: kind 0 body box table (0x8380A)"),
    (0x83822, 24, "prop B40: kind 1 body box table (0x83822)"),
    // Pool B class 28 (water bands, 0x7F72A): 5 variants of 4 phases.
    (0x7F7C8, 352, "B28 var 0: 4 phase pointers then 4 pictures of tile codes (0x7F786)"),
    (0x7F9BE, 688, "B28 var 2: 4 phase pointers then 4 pictures of tile codes (0x7F934)"),
    (0x7FCF4, 880, "B28 var 4: 4 phase pointers then 4 pictures of tile codes (0x7FC7A)"),
    (0x800AE, 448, "B28 var 6: 4 phase pointers then 4 pictures of tile codes (0x80070)"),
    (0x802C4, 448, "B28 var 8: 4 phase pointers then 4 pictures of tile codes (0x8027A)"),
    // Pool B class 29 (water bands, 0x8048A): 7 records of 3 rows x 16 columns.
    (0x80518, 2856, "B29 block tables: dest addrs, phase table ptrs, phase ptrs, tile codes (0x804CA)"),
    // Pool B class 43 (wall torches, 0x83ACA): 7 records of 2 x 2 flames, 4 phases.
    (0x83B4E, 92, "B43 torch tables: dest addrs, phase ptrs, tile codes (0x83B20)"),
    // Pool B class 0 (generic breakable wall panel / garage fixtures, 0x74FA2, M7-05).
    (0x75018, 9, "B0 size records (0x75006)"),
    (0x750B4, 0x12C, "B0 box tables and patch layouts (0x74FA2)"),
    // Pool B class 59 (foundry palette cycles, 0x861E8, M7-05).
    (0x8622A, 0x182, "B59 foundry palette tables (0x861E8)"),
    // Pool B class 42 (red sewer door burst, 0x83A0A, M8-22)
    (0x83A76, 48, "prop B42: tile codes (0x83A76)"),
    (0x83AA6, 20, "prop B42: scroll2 destination addresses (0x83AA6)"),
    // Pool B class 53 (fire glow, 0x85306, M8-22)
    (0x8534C, 8, "prop B53: G.$7B2 offset table (0x8534C)"),
    (0x8537A, 216, "prop B53: 27 palette table pointers and destinations (0x8537A)"),
    (0x85458, 336, "prop B53: 27 pointer tables (0x85458)"),
    // Pool B class 4 (door burst, 0x767D8, M8-02)
    (0x768CC, 56, "prop B4: var 4/8 tile codes (0x768CC)"),
    (0x7699A, 48, "prop B4: var 0xC tile codes (0x7699A)"),
    (0x76904, 12, "prop B4: destination pointers (0x76904)"),
    // Pool B class 60 (mine lamps, 0x863AC, M8-02)
    (0x86476, 18, "prop B60: palette list A pointers (0x86476)"),
    (0x86488, 18, "prop B60: palette list B pointers (0x86488)"),
    (0xBE052, 32, "prop B60: palette 80 step 0 (0xBE052)"),
    (0xBEA52, 96, "prop B60: palette 80 steps 1..3 (0xBEA52)"),
    (0x8649A, 64, "prop B60: palette 81 colour sets (0x8649A)"),
    // Pool B class 55 (rain streaks, 0x8574C, M8-02)
    (0x857D8, 150, "prop B55: 75 streak offsets (0x857D8)"),
    (0x8586E, 75, "prop B55: 75 streak phase offsets (0x8586E)"),
    (0x858D0, 64, "prop B55: streak glyphs (0x858D0)"),
    // Pool B class 56 (rain drops, 0x85916, M8-02)
    (0x85AE4, 32, "prop B56: drop depth table (0x85AE4)"),
    (0x85A14, 16, "prop B56: probability table 1 (0x85A14)"),
    (0x85A24, 32, "prop B56: variant table 1 (0x85A24)"),
    (0x85A44, 16, "prop B56: probability table 2 (0x85A44)"),
    (0x85A54, 32, "prop B56: variant table 2 (0x85A54)"),
    (0x85A74, 16, "prop B56: probability table 3 (0x85A74)"),
    (0x85A84, 32, "prop B56: variant table 3 (0x85A84)"),
    // Pool B classes 49 and 62 (M7-25): Episode 5 wall burst and fire palettes.
    (0x84424, 40, "B49 wall burst tile codes, 4 columns of 5 (0x843C8)"),
    (0x84454, 8, "B49 wall burst destinations by variant (0x843CC)"),
    (0x86690, 4, "B62 hardware toggle waits (0x8666A)"),
    (0x866EE, 44, "B62 palette dest offsets and counts (0x866BC)"),
    (0x86720, 1, "B62 palette reload (0x86612)"),
    (0x8672C, 0x5C, "B62 palette lanes, list pointer then dest (0x866A2)"),
    (0x86788, 0x11E, "B62 palette phase lists (0x866B8)"),
    (0x868A6, 0x140, "B62 inline colour sets (0x866D2)"),
    (0xBDA52, 32, "B62 scroll2 palette phase 0 set (0x867F0)"),
    (0xBDAD2, 32, "B62 scroll2 palette phase 0 set (0x867BC)"),
    (0xBDC92, 32, "B62 scroll2 palette phase 0 set (0x86788)"),
    (0xBDD52, 32, "B62 scroll2 palette phase 0 set (0x86824)"),
    (0xBDE12, 32, "B62 scroll2 palette phase 0 set (0x8680A)"),
    (0xC0E52, 0x1E0, "B62 scroll2 palette phases 1 to 5 (0x866D2)"),
    // Zeke class 82 (0x606D4, M7-10)
    (0x6137A, 160, "Zeke altitude transition table (0x606D4)"),
    (0x6141A, 10, "Zeke target heights (0x606D4)"),
    (0x61424, 8, "Zeke hit recoil physics (0x606D4)"),
    (0x6142C, 16, "Zeke attack masks (0x606D4)"),
    (0x6143C, 160, "Zeke attack choice table (0x606D4)"),
    (0x614DC, 40, "Zeke approach table (0x606D4)"),
    (0x61504, 40, "Zeke physics tables (0x606D4)"),
];

pub fn extract(rom: &Rom) -> EnemyData {
    let mut d = EnemyData::default();
    for st in 0..8u32 {
        d.stages.push(stage_script(rom, st));
    }
    d.hp = (0..0x27u32).map(|i| std::array::from_fn(|l| rom.u16(0x31454 + 64 * i + 2 * l as u32))).collect();
    d.boss_hp = (0..10u32)
        .map(|i| std::array::from_fn(|p| std::array::from_fn(|l| rom.u16(0x31E52 + 48 * i + 16 * p as u32 + 2 * l as u32))))
        .collect();
    d.attacks = (0..0x80u32)
        .map(|k| {
            let a = 0xA2EA4 + 8 * k;
            AttackRecord {
                damage: rom.u8(a) as i8,
                stun: rom.u8(a + 1),
                reaction: rom.u8(a + 2),
                hit_stop: rom.u8(a + 3),
                sound: rom.u8(a + 4),
                score: rom.u8(a + 5),
                spark: rom.u8(a + 6),
                b7: rom.u8(a + 7),
            }
        })
        .collect();
    // 0x17A4: speed table pointers at 0x17CE, each table 32 x (vx, vz).
    for i in 0..64u32 {
        let p = rom.u32(0x17CE + 4 * i);
        if !(0x120000..0x130000).contains(&p) {
            break;
        }
        d.speed_ptrs.push(p);
        d.speeds.entry(p).or_insert_with(|| std::array::from_fn(|k| (rom.i16(p + 4 * k as u32), rom.i16(p + 4 * k as u32 + 2))));
    }
    d.reverse_frames = reverse_frames(rom);
    d.t.blobs = TABLES
        .iter()
        .chain(crate::boss::TABLES.iter())
        .map(|&(a, n, _)| (a, (a..a + n).map(|x| rom.u8(x)).collect()))
        .collect();
    // Palette blocks pointed to by the entrance 6 fade table (16 colours each).
    let mut extra = Vec::new();
    for k in 0..12u32 {
        let p = rom.u32(0x33CDA + 4 * k);
        extra.push((p, (p..p + 32).map(|x| rom.u8(x)).collect()));
    }
    // class 20 fade (0x45C6E): 2 variants x 4 palette blocks of 16 colours.
    for k in 0..8u32 {
        let p = rom.u32(0x45C6E + 4 * k);
        extra.push((p, (p..p + 32).map(|x| rom.u8(x)).collect()));
    }
    // class 13 fade (0x3F008): 3 variants x 4 palette blocks of 16 colours.
    for k in 0..12u32 {
        let p = rom.u32(0x3F008 + 4 * k);
        extra.push((p, (p..p + 32).map(|x| rom.u8(x)).collect()));
    }
    // Pool B class 53: 27 palette sets x 3 variants (16 colours each = 32 bytes).
    for i in 0..27u32 {
        let tbl_ptr = rom.u32(0x8537A + 8 * i);
        for j in 0..3u32 {
            let p = rom.u32(tbl_ptr + 4 * j);
            extra.push((p, (p..p + 32).map(|x| rom.u8(x)).collect()));
        }
    }
    d.t.blobs.extend(extra);
    d
}

pub(crate) fn rec(rom: &Rom, a: u32, long: bool) -> SpawnRec {
    SpawnRec {
        trig: rom.u16(a),
        flags: rom.u16(a + 2),
        class: rom.u16(a + 4),
        x: rom.u16(a + 6),
        z: rom.u16(a + 8),
        var: rom.u16(a + 10),
        wc: rom.u16(a + 12),
        b5b: if long { rom.u8(a + 14) } else { 0 },
        b96: if long { rom.u8(a + 15) } else { 0 },
    }
}

/// A trigger list: records until (and including) the first sentinel.
fn list(rom: &Rom, start: u32, long: bool) -> Vec<SpawnRec> {
    let size = if long { 16 } else { 14 };
    let mut v = Vec::new();
    for k in 0..400 {
        let r = rec(rom, start + size * k, long);
        v.push(r);
        if r.trig >= SENTINEL {
            break;
        }
    }
    v
}

/// Wave format: { thr[3], (16-byte record)+, 16-byte terminator }* thr[3] end_word.
pub(crate) fn wave(rom: &Rom, start: u32) -> Wave {
    let mut w = Wave { addr: start, ..Default::default() };
    let mut a = start;
    for _ in 0..256 {
        let thr = [rom.u16(a), rom.u16(a + 2), rom.u16(a + 4)];
        a += 6;
        if (rom.u16(a) as i16) < 0 {
            w.final_thr = thr;
            w.end = rom.u16(a);
            return w;
        }
        let mut g = WaveGroup { thr, spawns: Vec::new() };
        while (rom.u16(a) as i16) >= 0 {
            g.spawns.push(rec(rom, a, true));
            a += 16;
        }
        a += 16;
        w.groups.push(g);
    }
    w
}

/// Records with a delay word until a negative delay (boss timed list).
pub(crate) fn delay_list(rom: &Rom, start: u32) -> Vec<SpawnRec> {
    let mut v = Vec::new();
    let mut a = start;
    for _ in 0..256 {
        let r = rec(rom, a, true);
        v.push(r);
        if (r.trig as i16) < 0 {
            break;
        }
        a += 16;
    }
    v
}

fn stage_script(rom: &Rom, st: u32) -> StageScript {
    let t1 = rom.u32(LIST1_TABLE + 4 * st);
    let t2 = rom.u32(LIST2_TABLE + 4 * st);
    let areas = (0..4)
        .map(|ar| {
            let (p1, p2) = (rom.u32(t1 + 4 * ar), rom.u32(t2 + 4 * ar));
            AreaScript { list1_addr: p1, list2_addr: p2, list1: list(rom, p1, false), list2: list(rom, p2, true) }
        })
        .collect();
    let base = rom.u32(WAVE_TABLE + 4 * st);
    let count = rom.u16(base) / 2;
    let waves = (0..count as u32).map(|k| wave(rom, base.wrapping_add(rom.i16(base + 2 * k) as i32 as u32))).collect();
    StageScript {
        areas,
        waves,
    }
}

/// 0x1380 steps an animation backwards: p = frame - 10; if the word at p is negative, the
/// target is p + 2 + i16(p + 2) and the carry is set. Precomputed for every frame of the
/// 10-byte object tables.
fn reverse_frames(rom: &Rom) -> HashMap<u32, (u32, bool)> {
    let mut m = HashMap::new();
    for &table in OBJECT_ANIM_TABLES {
        let mut first = u32::MAX;
        let mut i = 0;
        while 2 * i < first && i < 256 {
            let off = rom.u16(table + 2 * i);
            if off != 0 {
                first = first.min(off as u32);
                let mut a = table.wrapping_add(off as i16 as i32 as u32);
                for _ in 0..256 {
                    let p = a.wrapping_sub(10);
                    let prev = if (rom.u16(p) as i16) < 0 { (p + 2).wrapping_add(rom.i16(p + 2) as i32 as u32) } else { p };
                    m.insert(a, (prev, (rom.u16(p) as i16) < 0));
                    if rom.u8(a + 7) & 0x80 != 0 {
                        break;
                    }
                    a += 10;
                }
            }
            i += 1;
        }
    }
    m
}
