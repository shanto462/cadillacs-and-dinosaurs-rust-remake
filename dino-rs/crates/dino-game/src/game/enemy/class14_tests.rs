//! Unit tests of the class 14 rules that need no recording (made-up tables, no ROM data): the
//! three-object init, the guard, the turn limit, the library windows and slots. The state machine
//! itself is checked against MAME (`re/harness/README.md`, scenarios `c14a` .. `c14h`).

use super::class14::f;
use super::{Enemy, off::*};
use crate::game::{Game, Handle};
use dino_assets::AssetPack;

fn game_with(blobs: Vec<(u32, Vec<u8>)>) -> Game {
    let mut a = AssetPack::default();
    a.enemy.t.blobs = blobs;
    Game::new(Box::leak(Box::new(a)))
}

/// A class 14 object about to run its init (sub 0) in pool A slot `i`.
fn arm(g: &mut Game, i: usize, var: u16) -> Handle {
    let e = &mut g.enemies.slots[i];
    e.b.live = 1;
    e.b.k20 = 0x38;
    e.b.var26 = var;
    e.b.set_xi(0x1234);
    e.b.set_yi(0x56);
    e.b.set_zi(0x78);
    e.b.dir = 0x0A;
    e.m.sb(DROP, 7);
    g.enemies.handle(i)
}

fn run(g: &mut Game, i: usize) {
    let h = g.enemies.handle(i);
    let mut e = std::mem::take(&mut g.enemies.slots[i]);
    g.enemy_class14(&mut e, h);
    g.enemies.slots[i] = e;
}

/// Takes every pool A slot, then gives two back so that the next two allocations return `next[0]` and
/// `next[1]` in this order.
fn pool_with(g: &mut Game, next: [usize; 2]) {
    for _ in 0..24 {
        g.enemies.alloc();
    }
    g.enemies.release(next[1]);
    g.enemies.release(next[0]);
}

/// 0x40224: whichever of the three pool A slots runs the init, the middle handle becomes the class 14
/// object (+$A8 / +$AA = the other two, sub 2) and the other two become class 60 parts (variant 0 and
/// 0x200, +$A0 = the master as a sign extended long).
#[test]
fn the_init_makes_the_middle_object_the_master() {
    for (this, next) in [(21usize, [22usize, 23usize]), (22, [21, 23]), (23, [21, 22]), (10, [20, 5])] {
        let mut g = game_with(Vec::new());
        pool_with(&mut g, next);
        let h = arm(&mut g, this, 0x0204);
        run(&mut g, this);
        let mut hs = [h, g.enemies.handle(next[0]), g.enemies.handle(next[1])];
        hs.sort();
        let [lo, mid, hi] = hs;
        let m = &g.enemies.slots[g.enemy_index(mid)];
        assert_eq!((m.b.live, m.b.k20, m.b.sub, m.b.var26), (1, 0x38, 2, 0x0204), "this {this}");
        assert_eq!((m.b.xi(), m.b.yi(), m.b.zi(), m.b.dir), (0x1234, 0x56, 0x78, 0x0A));
        assert_eq!((m.m.b(DROP), m.m.w(f::PART_A), m.m.w(f::PART_B)), (7, lo, hi));
        for (p, var) in [(lo, 0u16), (hi, 0x200)] {
            let p = &g.enemies.slots[g.enemy_index(p)];
            assert_eq!((p.b.live, p.b.k20, p.b.var26, p.b.anim_table), (1, 0xF0, var, 0x6FB6A));
            assert_eq!(p.m.l(0xA0), 0xFFFF_0000 | mid as u32, "this {this}");
        }
        assert_eq!(g.enemies.free_count(), 0, "two slots were taken");
    }
}

/// With fewer than two free slots the object frees itself (0x41E90) and takes nothing.
#[test]
fn the_init_needs_two_free_slots() {
    let mut g = game_with(Vec::new());
    for _ in 0..23 {
        g.enemies.alloc();
    }
    let i = 0;
    let h = arm(&mut g, i, 0);
    assert_eq!(g.enemies.free_count(), 1);
    run(&mut g, i);
    assert_eq!(g.enemies.slots[i].b.live, 0);
    assert_eq!(g.enemies.free_count(), 2, "the slot of the object went back, nothing else was taken");
    let _ = h;
}

/// 0x412B0: the new direction may differ from the old one by at most two steps.
#[test]
fn the_turn_is_limited_to_two_steps() {
    let mut g = game_with(Vec::new());
    let mut e = Enemy::default();
    for (old, new, want) in [(0u8, 0x10u8, 2u8), (0, 0x14, 0x1E), (0, 3, 2), (0, 2, 2), (0x1E, 0x1E, 0x1E), (0x10, 0, 0x12)] {
        e.m.sb(f::OLD_DIR, old);
        e.b.dir = new;
        g.c14_412b0(&mut e);
        assert_eq!(e.b.dir, want, "old {old:#x} new {new:#x}");
    }
}

/// 0x4150A: the jump kick speed is the number of 0x3800 steps in the distance (at least 1).
#[test]
fn the_leap_speed_counts_steps_of_0x3800() {
    assert_eq!(Game::c14_leap_steps(0), 1);
    assert_eq!(Game::c14_leap_steps(0x37FF), 1);
    assert_eq!(Game::c14_leap_steps(0x3800), 2); // bpl: a result of 0 is not negative
    assert_eq!(Game::c14_leap_steps(0x50_0000), (0x50_0000i32 / 0x3800 + 1) as i16);
}

/// A made-up table set: windows of 8 px per X step and 4 px per Z step.
fn class_tables() -> Vec<(u32, Vec<u8>)> {
    let w = |v: &[u16]| v.iter().flat_map(|x| x.to_be_bytes()).collect::<Vec<u8>>();
    let mut t = w(&[0, 8, 16, 24, 32, 40, 48, 56]);
    t.extend(w(&[0, 16, 32, 48, 64, 80, 96, 112]));
    t.extend(w(&[0, 4, 8, 12]));
    t.extend(w(&[0, 8, 16, 24]));
    vec![(0x32988, t)]
}

/// 0x3293C: the class is the highest window the target is outside of.
#[test]
fn the_distance_class_is_the_highest_window_missed() {
    let mut g = game_with(class_tables());
    g.players[0].b.live = 1;
    let mut e = Enemy::default();
    e.m.sw(TARGET, Game::player_handle(0));
    e.b.set_xi(100);
    e.b.set_zi(100);
    for (tx, tz, want) in [(100i16, 100i16, (0u16, 0u16)), (130, 100, (3, 0)), (70, 100, (3, 0)), (100, 109, (0, 2)), (400, 100, (7, 0)), (100, 90, (0, 2))] {
        g.players[0].b.set_xi(tx);
        g.players[0].b.set_zi(tz);
        assert_eq!(g.lib_distance_class(&e), want, "target at {tx},{tz}");
    }
}

/// The slot routines put the point on the side of the target that the object is on: a target to the right
/// of the object (biased signed compare) gives `tx - |dx|`, a target to the left `tx + |dx|`.
#[test]
fn a_slot_point_is_on_the_side_of_the_object() {
    let mut g = game_with(Vec::new());
    g.players[0].b.live = 1;
    g.players[0].b.set_xi(0x200);
    let mut e = Enemy::default();
    e.m.sw(TARGET, Game::player_handle(0));
    for (x, dx, want) in [(0x100i16, 0x50i16, 0x1B0u16), (0x100, -0x50, 0x1B0), (0x300, 0x50, 0x250), (0x300, -0x50, 0x250)] {
        e.b.set_xi(x);
        e.m.sw(DEST_X, 0x200u16.wrapping_add(dx as u16)); // what the slot routines write first
        g.lib_slot_mirror(&mut e, dx);
        assert_eq!(e.m.w(DEST_X), want, "object at {x:#x}, offset {dx}");
    }
}

/// 0x404CA: the guard. Nothing while a block timer is negative; a block when the chance table beats the
/// draw (here: always), which sets the timer and asks for the back step; while blocking, the keep-blocking
/// table decides, and the end of the block starts a pause (the timer goes negative).
#[test]
fn the_guard_blocks_by_its_tables() {
    let mut blobs = vec![(0x74298, vec![0x10; 0x10]), (0x742EA, vec![0; 0x10])];
    let mut dur = vec![0x1E];
    dur.extend(vec![4u8; 0x41]);
    blobs.push((0x742A8, dur));
    blobs.push((0x742FA, 60u16.to_be_bytes().iter().copied().cycle().take(0x20).collect()));
    let mut g = game_with(blobs);
    g.players[0].b.live = 1;
    g.players[0].b.facing = 1;
    let mut e = Enemy::default();
    e.m.sw(TARGET, Game::player_handle(0));
    e.b.facing = 0;
    // a pause (negative timer): no block
    e.m.sb(f::COOL, 0xF0);
    assert!(!g.c14_404ca(&mut e));
    // timer 0: block (the table 0x10 beats any draw 0..15), block time 0x1E + 4, back step requested
    e.m.sb(f::COOL, 0);
    assert!(g.c14_404ca(&mut e));
    assert_eq!((e.m.b(f::COOL), e.m.b(f::BLOCK)), (0x22, 1));
    // blocking, keep-blocking chance 0: the block ends with a pause of 60 frames
    e.m.sb(f::BLOCK, 0);
    assert!(!g.c14_404ca(&mut e));
    assert_eq!(e.m.b(f::COOL), 0xC4);
    // in the air, in another state or facing the same way: never
    e.m.sb(f::COOL, 0);
    e.b.set_yi(1);
    assert!(!g.c14_404ca(&mut e));
    e.b.set_yi(0);
    e.b.act = 0x22;
    assert!(!g.c14_404ca(&mut e));
    e.b.act = 6;
    g.players[0].b.facing = 0;
    assert!(!g.c14_404ca(&mut e));
}

/// 0x9D44 alternates 0xE3 / 0x64 with the toggle bit 7 of G.$43A, 0x9732 0xFC / 0x7B with bit 2.
#[test]
fn the_knife_sounds_toggle() {
    // the pan table 0xA66A (synthetic: counts 0, 1, 2 ...), read for the pan byte of the object's X
    let mut g = game_with(vec![(0xA66A, (0..0x30).collect())]);
    g.cam.s2.cam_x = 0x300;
    g.snd_9d44(0x300 + 0x18, 0);
    g.snd_9d44(0x300 + 0x18, 1);
    g.snd_9d44(0x300 + 0x18, 0);
    g.snd_9732(0x300 + 0x18);
    g.snd_9732(0x300 + 0x18);
    assert_eq!(g.w.sounds, vec![0xE3, 0x64, 0xE3, 0xFC, 0x7B]);
    // d1 = (3 or 4 by facing) << 16 | 0x2000 | pan (3 at X + 0x18), d2 = 0x2400000; 0x9732 d1 = pan, d2 = 0
    let got: Vec<(u32, u32)> = g.w.snd_reqs.iter().map(|r| (r.d1, r.d2)).collect();
    assert_eq!(got, vec![(0x3_2003, 0x0240_0000), (0x4_2003, 0x0240_0000), (0x3_2003, 0x0240_0000), (3, 0), (3, 0)]);
}
