//! Unit tests of the class 13 rules that need no recording (made-up tables, no ROM data): the step counter of the jump on entrances, the
//! turn limits, the guard (0x3DFC6) with its thrown away random number, the pause and close in tables with their byte wrap, the far gate. The state
//! machine itself is checked against MAME (`re/harness/m601_check.sh`, the recordings of D3a H5 to H10).

use super::class13::{f::*, t};
use super::{Enemy, off::*};
use crate::game::Game;
use dino_assets::{AnimTable, AssetPack, Frame, FrameDraw};

/// A game with the given byte tables, one speed table for every speed id, and one animation table (ROM address 0, every id = one frame).
fn game_with(blobs: Vec<(u32, Vec<u8>)>) -> Game {
    let mut a = AssetPack::default();
    a.enemy.t.blobs = blobs;
    a.enemy.speed_ptrs = vec![0x1000; 0x40];
    a.enemy.speeds.insert(0x1000, [(0, 0); 32]);
    a.sprites.frames = vec![Frame {
        rom_addr: 0,
        stride: 10,
        w0: 0,
        w1: 0x8000,
        draw: FrameDraw::Comp { main: 0, overlay: None },
        param_a: 0,
        param_b: 0,
        kind: 0,
        duration: 5,
        last: true,
        body_box: 0,
        attack_box: 0,
        attach: 0,
        extra: [0; 4],
        next: 0,
    }];
    a.sprites.anim_tables.insert(0, AnimTable { rom_addr: 0, stride: 10, anims: vec![Some(0); 0x40] });
    Game::new(Box::leak(Box::new(a)))
}

/// A poacher of variant `var` (the high byte of `+$26`) that targets player 1.
fn poacher(g: &mut Game, var: u16) -> Enemy {
    g.players[0].b.live = 1;
    let mut e = Enemy::default();
    e.b.var26 = var << 8;
    e.m.sw(TARGET, Game::player_handle(0));
    e
}

/// The steps loop of the jump on entrances: `d1 = 0; do { d1 += 1; d0 -= 0x3800 } while d0 >= 0`.
#[test]
fn the_steps_count_0x3800_units() {
    assert_eq!(Game::enemy_c13_steps(0), 1);
    assert_eq!(Game::enemy_c13_steps(0x37FF), 1);
    assert_eq!(Game::enemy_c13_steps(0x3800), 2);
    assert_eq!(Game::enemy_c13_steps(0x30_0000), (0x30_0000i32 / 0x3800 + 1) as u16);
    assert_eq!(Game::enemy_c13_steps(-5), 1, "a negative start still counts the first step");
}

/// 0x3F2E8 / 0x3F32E: the new direction may differ from the old one by at most one / two steps.
#[test]
fn the_turn_limits_are_one_and_two_steps() {
    let mut g = game_with(Vec::new());
    let mut e = Enemy::default();
    for (old, new, want1, want2) in [(0u8, 0x10u8, 1u8, 2u8), (0, 0x14, 0x1F, 0x1E), (0, 1, 1, 1), (0, 2, 1, 2), (0x1E, 0x1E, 0x1E, 0x1E), (0x10, 0, 0x11, 0x12)] {
        e.m.sb(OLD_DIR, old);
        e.b.dir = new;
        g.enemy_c13_3f2e8(&mut e);
        assert_eq!(e.b.dir, want1, "one step: old {old:#x} new {new:#x}");
        e.b.dir = new;
        g.enemy_c13_3f32e(&mut e);
        assert_eq!(e.b.dir, want2, "two steps: old {old:#x} new {new:#x}");
    }
}

/// 0x3E4DC: the pause at a point is the table 0x7435A by the level plus the variant byte asr 5 (a byte add: it wraps inside the low byte).
#[test]
fn the_pause_at_a_point_uses_the_level_and_the_variant_row() {
    let mut g = game_with(vec![(t::PAUSE_AT_POINT, (0..0x100u32).map(|v| v as u8).collect())]);
    let mut e = poacher(&mut g, 0);
    g.w.level = 7;
    e.m.sb(ARRIVED, 0);
    g.enemy_c13_3e4dc(&mut e);
    assert_eq!(e.m.b(STILL_T), 0, "nothing to do before the point is reached");
    e.m.sb(ARRIVED, 1);
    g.enemy_c13_3e4dc(&mut e);
    assert_eq!((e.m.b(STILL_T), e.m.b(ARRIVED)), (7, 0));
    // the variant 0x40 asr 5 = 2, the variant 0xE0 as a signed byte asr 5 = -1
    for (var, level, want) in [(0x40u16, 7u16, 9u8), (0xE0, 7, 6), (0xE0, 0, 0xFF)] {
        let mut e = poacher(&mut g, var);
        g.w.level = level;
        e.m.sb(ARRIVED, 1);
        g.enemy_c13_3e4dc(&mut e);
        assert_eq!(e.m.b(STILL_T), want, "variant {var:#x} level {level}");
    }
}

/// 0x3F5E2: the close in timer is the table by level plus a random add, at least 1.
#[test]
fn the_close_in_timer_is_at_least_one() {
    let mut adds = Vec::new();
    for _ in 0..16 {
        adds.extend(0xFFF1u16.to_be_bytes()); // -15 for every roll
    }
    let mut g = game_with(vec![(t::CLOSE_IN, vec![0x0E; 0x60]), (t::CLOSE_IN_ADD, adds)]);
    let mut e = poacher(&mut g, 0);
    g.enemy_c13_3f5e2(&mut e);
    assert_eq!(e.m.w(TIMER), 1, "14 - 15 is below 1");
    assert_eq!((e.m.b(BUSY), e.b.act), (1, 0x16));
}

/// 0x40118: the far gate reads the table by aggression * 8 + variant * 32 + the X class stored by the LAST decision (a byte add).
#[test]
fn the_far_gate_reads_the_stored_class() {
    let mut table = vec![0u8; 0xC0];
    table[(3 << 3) + (2 << 5) + 5] = 0x40; // aggression 3, variant 2, class 5: every roll below 32 passes
    let mut g = game_with(vec![(t::FAR_GATE, table), (t::FAR_DECISION, vec![8; 0x300]), (0x32988, vec![0; 0x30]), (0x3FC84, vec![0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14, 0, 0x14])]);
    g.rank.aggression = 3;
    let mut e = poacher(&mut g, 2);
    e.m.sb(DIST, 5);
    let rng = g.w.rng;
    g.enemy_c13_40118(&mut e);
    assert_ne!(g.w.rng, rng, "the gate draws a number");
    assert_eq!(e.b.act, 0x46, "the gate opened: the far decision ran (the made-up table says 8: wait)");
    // another class: the table byte is 0, the gate stays shut
    let mut e = poacher(&mut g, 2);
    e.m.sb(DIST, 4);
    g.enemy_c13_40118(&mut e);
    assert_eq!(e.b.act, 0);
    // the target in a hit reaction: no draw at all
    g.players[0].b.set_state(0x0202_0000);
    e.m.sb(DIST, 5);
    let rng = g.w.rng;
    g.enemy_c13_40118(&mut e);
    assert_eq!(g.w.rng, rng);
}

/// 0x3DFC6: the guard. Never in the air, in another sub, committed, or facing the same way as the target; a negative counter is a pause.
/// A free guard draws one number and throws it away (`moveq #0,d0` before the compare): the table byte alone decides.
#[test]
fn the_guard_draws_one_number_and_the_table_decides() {
    let mut count = vec![0u8; 0x63];
    count[0x21] = 0x28; // variant 2: the base byte
    for k in 1..=0x20 {
        count[0x21 + k] = k as u8;
    }
    let mut chance = vec![0u8; 24];
    chance[8 + 3] = 1; // variant 2, aggression 3
    let blobs = vec![(t::GUARD_CHANCE, chance), (t::GUARD_COUNT, count), (t::GUARD_AGAIN, vec![0xFF; 0x19]), (t::GUARD_PAUSE, vec![0, 0x28].repeat(24))];
    let mut g = game_with(blobs);
    g.rank.aggression = 3;
    g.players[0].b.facing = 1;
    let mut e = poacher(&mut g, 2);
    e.b.facing = 0;
    // nothing is drawn when a precondition fails
    let rng = g.w.rng;
    e.b.set_yi(1);
    assert!(!g.enemy_c13_3dfc6(&mut e));
    e.b.set_yi(0);
    e.m.sb(COMMIT, 1);
    assert!(!g.enemy_c13_3dfc6(&mut e));
    e.m.sb(COMMIT, 0);
    e.m.sb(COUNT, 0xF0);
    assert!(!g.enemy_c13_3dfc6(&mut e));
    e.m.sb(COUNT, 0);
    g.players[0].b.facing = 0;
    assert!(!g.enemy_c13_3dfc6(&mut e));
    assert_eq!(g.w.rng, rng);
    g.players[0].b.facing = 1;
    // aggression 0: the chance byte is 0: one number drawn, no block
    g.rank.aggression = 0;
    let mut probe = game_with(Vec::new());
    probe.w.rng = g.w.rng;
    probe.rand();
    assert!(!g.enemy_c13_3dfc6(&mut e));
    assert_eq!((g.w.rng, e.m.b(LUNGE)), (probe.w.rng, 0));
    // aggression 3: block, two numbers drawn, the counter is the base byte plus the random add
    g.rank.aggression = 3;
    probe.w.rng = g.w.rng;
    probe.rand();
    let r = probe.rand() & 0x1F;
    assert!(g.enemy_c13_3dfc6(&mut e));
    assert_eq!((g.w.rng, e.m.b(LUNGE), e.m.b(COUNT)), (probe.w.rng, 1, 0x28 + 1 + r as u8));
    // guarding (counter above 0): the keep guarding byte 0xFF beats every roll: blocked again, the counter stays
    let c = e.m.b(COUNT);
    e.m.sb(LUNGE, 0);
    assert!(g.enemy_c13_3dfc6(&mut e));
    assert_eq!((e.m.b(COUNT), e.m.b(LUNGE)), (c, 1));
}

/// 0x401CE: the death sets `+$80` to 0x3C, gives the score and counts down; the last frame frees the object.
#[test]
fn the_death_blinks_for_sixty_frames() {
    let mut g = game_with(vec![(0x325F8, vec![0; 0x28]), (0x10292, vec![0; 0x90])]);
    let i = g.enemies.alloc().unwrap();
    let h = g.enemies.handle(i);
    let mut e = poacher(&mut g, 0);
    e.b.live = 1;
    e.b.mode = 4;
    for n in 0..0x3Cu16 {
        g.enemy_class13(&mut e, h);
        assert_eq!(e.m.w(T80), 0x3B - n, "frame {n}");
    }
    assert_eq!(e.b.live, 1);
    g.enemy_class13(&mut e, h);
    assert_eq!(e.b.live, 0, "freed when the counter has run out");
}
