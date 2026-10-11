//! Native port of the game logic. Each module names the original routines it ports, so the
//! Rust can be read side by side with `re/ghidra/decomp.c` and `re/specs/*/SPEC.md`.
//!
//! Conventions
//! * One `tick()` = one video frame of the original. Everything that counts frames is exact.
//! * Integer types and wrapping arithmetic follow the 68000 code (u16/i16 with `wrapping_*`,
//!   16.16 fixed point positions, 8.8 velocities). Do not "clean up" overflow behaviour.
//! * Data (tables, frames, maps) comes from `dino_assets::AssetPack`; code constants are written
//!   in Rust with the address of the instruction they come from.
//! * Objects are named by their original 16-bit RAM address (`Handle`). That is how the original
//!   links objects (target player, last attacker, grab partner) and how it breaks depth-sort ties.
//!   It is only a name: there is no memory image behind it.

pub mod body;
pub mod camera;
pub mod collide;
pub mod combat;
pub mod draw;
pub mod effect;
pub mod enemy;
pub mod enemy_combat;
pub mod flow;
pub mod hud;
pub mod item;
pub mod palette;
pub mod player;
pub mod player_combat;
pub mod prop;
pub mod rank;
pub mod spawn;
pub mod vram;

use body::Body;
use camera::Camera;
use dino_assets::AssetPack;
use draw::SpriteList;
use palette::Fade;
use vram::{PaletteRam, Vram};

/// An object's original RAM address (low 16 bits; RAM is at 0xFF0000). 0 = none.
pub type Handle = u16;

/// Players: 3 fixed objects of 0x180 bytes.
pub const PLAYERS: (Handle, u16, usize) = (0xB274, 0x180, 3);
/// Pool A (enemies, bosses, dinosaurs), allocator 0x483C.
pub const POOL_A: (Handle, u16, usize) = (0xC8F4, 0xE0, 24);
/// Pool B (controllers, props), allocator 0x4860.
pub const POOL_B: (Handle, u16, usize) = (0xB6F4, 0xC0, 24);
/// Pool E (items, food, weapons, containers, projectiles), allocator 0x4818.
pub const POOL_E: (Handle, u16, usize) = (0x8874, 0xC0, 24);
/// Pool C (effects, shots), allocator 0x4884.
pub const POOL_C: (Handle, u16, usize) = (0x9A74, 0x80, 48);
/// Pool D (special), allocator 0x48A8.
pub const POOL_D: (Handle, u16, usize) = (0xDDF4, 0x80, 16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Player(usize),
    Enemy(usize),
    Prop(usize),
    Item(usize),
    Effect(usize),
    Special(usize),
}

pub fn slot_of(h: Handle) -> Option<Slot> {
    let pools = [
        (PLAYERS, Slot::Player as fn(usize) -> Slot),
        (POOL_A, Slot::Enemy),
        (POOL_B, Slot::Prop),
        (POOL_E, Slot::Item),
        (POOL_C, Slot::Effect),
        (POOL_D, Slot::Special),
    ];
    for ((base, stride, n), make) in pools {
        if h >= base && h < base + stride * n as u16 && (h - base) % stride == 0 {
            return Some(make(((h - base) / stride) as usize));
        }
    }
    None
}

/// A fixed-size object pool with the original free list: a stack of slots. The init routine
/// pushes slot 0 first, so the first allocation returns the last slot (23 for a 24-slot pool),
/// and freeing pushes the slot back (LIFO).
pub struct Pool<T> {
    pub slots: Vec<T>,
    free: Vec<usize>,
    base: Handle,
    stride: u16,
}

impl<T: Default> Pool<T> {
    pub fn new((base, stride, n): (Handle, u16, usize)) -> Pool<T> {
        Pool { slots: (0..n).map(|_| T::default()).collect(), free: (0..n).collect(), base, stride }
    }
}

impl<T> Pool<T> {
    /// Allocators 0x4818 / 0x483C / 0x4860 / 0x4884 / 0x48A8. `None` = pool empty (carry set).
    pub fn alloc(&mut self) -> Option<usize> {
        self.free.pop()
    }

    /// Pushes the slot back. The caller clears exactly the fields the original free routine clears.
    pub fn release(&mut self, i: usize) {
        self.free.push(i);
    }

    /// Free slot count ($65F4 / $65F6 / $65F8 / $65FA / $65FC).
    pub fn free_count(&self) -> usize {
        self.free.len()
    }

    pub fn handle(&self, i: usize) -> Handle {
        self.base + self.stride * i as u16
    }
}

/// Globals of the original (work RAM at 0xFF8000 + offset), named. Modules add the ones they use.
#[derive(Default)]
pub struct World {
    /// G.$0000 frame counter.
    pub frame: u16,
    /// G.$14 state of rand() (0x119C).
    pub rng: u16,
    /// G.$4D9 stage (0-7). The area lives in the camera (G.$786).
    pub stage: u8,
    /// G.$4DC vertical scene (stage 6 elevator): sprites use camera Y 0.
    pub vertical_scene: u8,
    /// G.$4DD scripted control of the players.
    pub scripted: u8,
    /// G.$4D4 stage clear flag, set by camera modes 6 / 0x10 / 0x13.
    pub stage_clear: u8,
    /// G.$4EF active players - 1 (selects the camera push threshold).
    pub players_minus_one: u8,
    /// G.$28 CPS-B layer control value (0x12D6 normal, 0x06D6 = scroll1 above sprites).
    pub layer_ctrl: u16,
    /// G.$26 bit 0: scroll2 rowscroll enabled (stage 2).
    pub video_ctrl: u16,
    /// CPS-B priority masks for tile groups 0..3 (written by 0x8D92).
    pub priority_masks: [u16; 4],
    /// G.$16 / $17 / $18 cooked inputs of players 1-3 (active high: 1 R, 2 L, 4 D, 8 U, 0x10 attack, 0x20 jump).
    pub input: [u8; 3],
    /// G.$1A system inputs (start buttons, coins), active high.
    pub system: u8,
    /// G.$67BE global attack serial counter.
    pub attack_serial: u8,
    /// G.$5EE level (difficulty row, 0..31) and G.$5F0 rank.
    pub level: u16,
    pub rank: u16,
    /// Sound requests made this frame (sound ids as the original passes them to the sound queue).
    pub sounds: Vec<u16>,
}

pub struct Game {
    pub assets: &'static AssetPack,
    pub w: World,
    pub vram: Vram,
    pub pal: PaletteRam,
    pub fade: Fade,
    pub cam: Camera,
    pub sprites: SpriteList,
    /// Draw list for this frame: (depth << 16) | handle, see 0x49C0 / 0x49CA.
    pub draw_list: Vec<u32>,
    pub combat: combat::Lists,
    pub players: [player::Player; 3],
    pub enemies: Pool<enemy::Enemy>,
    pub props: Pool<prop::Prop>,
    pub items: Pool<item::Item>,
    pub effects: Pool<effect::Effect>,
    pub specials: Pool<effect::Special>,
    pub spawn: spawn::Spawn,
    pub rank: rank::Rank,
    pub hud: hud::Hud,
    pub flow: flow::Flow,
}

impl Game {
    pub fn new(assets: &'static AssetPack) -> Game {
        Game {
            assets,
            w: World { layer_ctrl: 0x12D6, ..Default::default() },
            vram: Vram::default(),
            pal: PaletteRam::default(),
            fade: Fade::default(),
            cam: Camera::default(),
            sprites: SpriteList::default(),
            draw_list: Vec::with_capacity(128),
            combat: combat::Lists::default(),
            players: Default::default(),
            enemies: Pool::new(POOL_A),
            props: Pool::new(POOL_B),
            items: Pool::new(POOL_E),
            effects: Pool::new(POOL_C),
            specials: Pool::new(POOL_D),
            spawn: Default::default(),
            rank: Default::default(),
            hud: Default::default(),
            flow: Default::default(),
        }
    }

    /// rand() at 0x119C. Shared by all combat and AI code: call order matters.
    pub fn rand(&mut self) -> u16 {
        let s = if self.w.rng == 0 { 0x01C3 } else { self.w.rng };
        let t = s.wrapping_mul(3);
        let lo = (s as u8).wrapping_add((t >> 8) as u8);
        self.w.rng = (t & 0xFF00) | lo as u16;
        self.w.rng
    }

    /// Queue a sound request (the original's sound queue at $3A, flushed in vblank).
    pub fn sound(&mut self, id: u16) {
        self.w.sounds.push(id);
    }

    /// Common header of any object.
    pub fn body(&self, h: Handle) -> &Body {
        match slot_of(h).unwrap_or_else(|| panic!("bad handle {h:#06x}")) {
            Slot::Player(i) => &self.players[i].b,
            Slot::Enemy(i) => &self.enemies.slots[i].b,
            Slot::Prop(i) => &self.props.slots[i].b,
            Slot::Item(i) => &self.items.slots[i].b,
            Slot::Effect(i) => &self.effects.slots[i].b,
            Slot::Special(i) => &self.specials.slots[i].b,
        }
    }

    pub fn body_mut(&mut self, h: Handle) -> &mut Body {
        match slot_of(h).unwrap_or_else(|| panic!("bad handle {h:#06x}")) {
            Slot::Player(i) => &mut self.players[i].b,
            Slot::Enemy(i) => &mut self.enemies.slots[i].b,
            Slot::Prop(i) => &mut self.props.slots[i].b,
            Slot::Item(i) => &mut self.items.slots[i].b,
            Slot::Effect(i) => &mut self.effects.slots[i].b,
            Slot::Special(i) => &mut self.specials.slots[i].b,
        }
    }

    pub fn player_handle(i: usize) -> Handle {
        PLAYERS.0 + PLAYERS.1 * i as u16
    }

    /// One video frame: the in-game task (0xAAF6), then the other tasks in slot order.
    pub fn tick(&mut self) {
        self.w.sounds.clear();
        self.flow_frame();
        self.run_fade_task();
        self.w.frame = self.w.frame.wrapping_add(1);
    }

    /// 0xAAF6 game_frame: the in-game per-frame order.
    pub fn game_frame(&mut self) {
        self.rank_tick(); // 0x186CC
        self.objects_update(); // 0xE90E
        self.collision(); // 0x10546
        self.hud_update(); // 0x16BEA
        self.build_sprite_list(); // 0x13FA6
        self.combat_lists_reset(); // 0x105E8
        self.time_limit(); // 0x103FE, 0x1037E
        self.spawn_script_tick(); // 0xFBF4
    }

    /// 0xE90E objects_update.
    fn objects_update(&mut self) {
        self.enemy_census(); // 0x3242E
        self.players_update(); // 0x18834
        self.combat.items_reset(); // $69EE / $69F0
        self.pool_e_update(); // 0x233C6 items
        self.enemies_update(); // 0x23420 pool A
        self.screen_shake(); // 0x222D6
        self.pool_b_update(); // 0x23604 props / controllers
        self.pool_c_update(); // 0x237B2 effects
        self.held_players_update(); // 0x188DA
        // 0x670C, 0x100DA, 0x1667E: not ported yet.
        self.background_update(); // 0x2042E
        // 0x12C90: not ported yet.
    }

    /// 0x49C0: add to the draw list if the on-screen flag is set.
    pub fn draw_list_add_if_visible(&mut self, h: Handle) {
        let b = self.body(h);
        if b.visible != 0 {
            let key = ((b.zi() as u16 as u32) << 16) | h as u32;
            self.draw_list.push(key);
        }
    }

    /// 0x49CA: set the on-screen flag (0x16FA) and add to the draw list if on screen.
    pub fn draw_list_add_check(&mut self, h: Handle) {
        let (cx, cy) = if self.w.vertical_scene != 0 { (0, 0) } else { (self.cam.s2.cam_x, self.cam.s2.cam_y) };
        let b = self.body_mut(h);
        let dx = b.xi().wrapping_sub(cx as i16).wrapping_add(0x100) as u16;
        let dy = b.yi().wrapping_add(b.zi()).wrapping_sub(cy as i16).wrapping_add(0x40) as u16;
        b.visible = (dx <= 0x380 && dy <= 0x1C0) as u8;
        self.draw_list_add_if_visible(h);
    }

    /// 0x13FA6 build_sprite_list: sort by (depth, handle) ascending; first drawn = on top.
    fn build_sprite_list(&mut self) {
        let cam_y = if self.w.vertical_scene != 0 { 0 } else { self.cam.s2.cam_y as i16 };
        let cam = draw::DrawCamera::new(self.cam.s2.cam_x as i16, cam_y);
        self.sprites.clear();
        self.draw_list.sort_unstable();
        let list = std::mem::take(&mut self.draw_list);
        for &key in &list {
            self.draw_object(key as u16, cam); // 0x141F8
        }
        self.shadow_pass(&list, cam); // 0x14EC0
        self.draw_list = list;
        self.draw_list.clear();
    }

}
