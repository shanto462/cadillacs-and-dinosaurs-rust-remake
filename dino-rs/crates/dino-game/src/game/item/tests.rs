//! Unit tests of the pool E rules that do not need ROM data beyond the small tables built here.

use super::small::{FoodEffect, food_effect};
use super::*;
use dino_assets::AssetPack;
use dino_assets::enemy::ByteTables;
use dino_assets::text::TextData;

/// A game with the given ROM byte tables (`enemy.t` and `text` blobs by address).
fn game(enemy: Vec<(u32, Vec<u8>)>, text: Vec<(u32, Vec<u8>)>) -> Game {
    let mut a = AssetPack::default();
    a.enemy.t = ByteTables { blobs: enemy, ..Default::default() };
    a.text = TextData { blobs: text };
    Game::new(Box::leak(Box::new(a)))
}

/// A drop table (0xB6D12 for enemies) whose row `row` holds `code` in every column.
fn drop_table(row: usize, code: u8) -> Vec<(u32, Vec<u8>)> {
    let mut t = vec![0u8; 32 * 4];
    t[32 * row..32 * (row + 1)].fill(code);
    vec![(0xB6D12, t)]
}

fn enemy_with_drop(row: u8) -> Enemy {
    let mut e = Enemy::default();
    e.b.live = 1;
    e.b.set_xi(0x123);
    e.b.set_yi(0);
    e.b.set_zi(0x45);
    e.m.sb(super::super::enemy::off::DROP, row);
    e
}

/// The pool E object the drop created (slot 23 is the first allocation).
fn spawned(g: &Game) -> &Item {
    &g.items.slots[23]
}

#[test]
fn drop_codes_choose_weapon_food_or_treasure() {
    // (code, class word, variant): weapon type = code - 1 (var = type * 2), food var = code - 0x13,
    // treasure var = code - 0x27.
    for (code, class_word, var) in [(1u8, 0x00u16, 0u16), (0x13, 0x00, 0x24), (0x14, 0x10, 1), (0x27, 0x10, 0x14), (0x28, 0x24, 1), (0x37, 0x24, 0x10)] {
        let mut g = game(drop_table(1, code), vec![]);
        let mut e = enemy_with_drop(1);
        g.item_drop_from_enemy(&mut e);
        let it = spawned(&g);
        assert_eq!((it.b.live, it.b.k20, it.b.var26, it.b.act), (1, class_word, var, 2), "code {code:#x}");
        assert_eq!((it.b.xi(), it.b.zi()), (0x123, 0x45));
        assert_eq!(e.m.b(super::super::enemy::off::DROP), 0, "the row byte is cleared: one drop per enemy");
        assert_eq!(g.items.free_count(), 23);
    }
}

#[test]
fn drop_row_zero_and_code_zero_drop_nothing() {
    // Row 0 of the table is never rolled; a zero code in the row means "nothing".
    let mut g = game(drop_table(1, 0), vec![]);
    let mut e = enemy_with_drop(0);
    g.item_drop_from_enemy(&mut e);
    assert_eq!(g.items.free_count(), 24);
    assert_eq!(g.w.rng, 0, "no rand() call without a row");
    let mut e = enemy_with_drop(1);
    g.item_drop_from_enemy(&mut e);
    assert_eq!(g.items.free_count(), 24);
    assert_ne!(g.w.rng, 0, "one rand() call to pick the column");
    assert_eq!(e.m.b(super::super::enemy::off::DROP), 0);
}

#[test]
fn drop_roll_uses_one_rand_call_per_row() {
    let mut a = game(drop_table(1, 0x14), vec![]);
    let mut b = game(vec![], vec![]);
    let mut e = enemy_with_drop(1);
    a.item_drop_from_enemy(&mut e);
    b.rand();
    assert_eq!(a.w.rng, b.w.rng);
}

/// Code 0x0F: an ammo box when a gun lies on screen, else the fallback row 0xB64D2 (one more rand()).
#[test]
fn drop_ammo_code_needs_a_gun_on_screen() {
    let mut fallback = vec![0x34u8; 32]; // row 0: DIME
    fallback[0] = 0x35;
    let mut tables = drop_table(1, 0x0F);
    tables.push((0xB64D2, fallback));
    // No gun: fallback row, treasure var code - 0x27 = 0x0D or 0x0E (either way a treasure).
    let mut g = game(tables.clone(), vec![]);
    let mut e = enemy_with_drop(1);
    g.item_drop_from_enemy(&mut e);
    assert_eq!(spawned(&g).b.k20, 0x24);
    let mut two = game(vec![], vec![]);
    two.rand();
    two.rand();
    assert_eq!(g.w.rng, two.w.rng, "the fallback rolls again");
    // A rifle (weapon class, kind 0x205, type flag set in 0x24CC6) lying inside the screen.
    let mut flags = vec![0u8; 0x20];
    flags[0] = 1; // var 0
    tables.push((0x24CC6, flags));
    let mut g = game(tables, vec![]);
    g.cam.s2.cam_x = 0x100;
    let gun = &mut g.items.slots[5];
    gun.b.live = 1;
    gun.b.k20 = 0;
    gun.b.var26 = 0;
    gun.b.set_xi(0x180);
    gun.m.sw(o::KIND, 0x205);
    let mut e = enemy_with_drop(1);
    g.item_drop_from_enemy(&mut e);
    assert_eq!(spawned(&g).b.k20, 0x08, "an ammo box");
    assert_eq!(spawned(&g).b.var26, 0);
    // The gun left of the camera does not count (x + 0x18 < camera X).
    g.items.slots[5].b.set_xi(0xE0);
    g.items.slots[23] = Item::default();
    assert!(!g.item_gun_on_screen());
}

/// 0x48D4 clears +$00..+$27 and nothing else; slots come back in LIFO order.
#[test]
fn free_clears_only_the_header_and_slots_are_reused_last_in_first_out() {
    let mut g = game(vec![], vec![]);
    let i = g.items.alloc().unwrap();
    assert_eq!(i, 23, "the first allocation is the last slot");
    let j = g.items.alloc().unwrap();
    assert_eq!(j, 22);
    let mut it = std::mem::take(&mut g.items.slots[i]);
    it.b.live = 1;
    it.b.k20 = 0x14;
    it.b.var26 = 6;
    it.b.set_xi(0x321);
    it.b.anim_table = 0x2995E;
    it.m.sw(o::KIND, 0x20A);
    g.item_free(&mut it, i);
    assert_eq!((it.b.live, it.b.k20, it.b.var26, it.b.x), (0, 0, 0, 0));
    assert_eq!((it.b.anim_table, it.m.w(o::KIND)), (0x2995E, 0x20A), "stale fields survive");
    assert_eq!(g.items.alloc(), Some(23), "freed last, allocated first");
}

#[test]
fn food_heals_by_the_table_and_caps_at_100() {
    assert_eq!(food_effect(32, 0x30), FoodEffect::Heal(0x50));
    assert_eq!(food_effect(100, 1), FoodEffect::Heal(0x64));
    assert_eq!(food_effect(48, 0x63), FoodEffect::Heal(0x64));
    assert_eq!(food_effect(16, 0x64), FoodEffect::Score, "at full HP the food is worth points");
    assert_eq!(food_effect(100, 0x64), FoodEffect::Score);
}

/// Eating at full HP: the points come from the second word of the table row (an offset into the
/// BCD score table 0x10292): BARBECUE 10000, GUM 1000.
#[test]
fn food_at_full_hp_scores_the_table_offset() {
    let mut score = vec![0u8; 0x90];
    for (k, v) in [(0x38usize, 0x1000u32), (0x4C, 0x10000)] {
        score[k..k + 4].copy_from_slice(&v.to_be_bytes());
    }
    // 0x28B10: BARBECUE (heal 100, score offset 0x4C), ..., GUM is entry 13 (heal 16, offset 0x38).
    let mut food = vec![0u8; 0x48];
    food[0..4].copy_from_slice(&[0, 100, 0, 0x4C]);
    food[48..52].copy_from_slice(&[0, 16, 0, 0x38]);
    let mut g = game(vec![(0x28B10, food)], vec![(0x10292, score)]);
    for (var, expect) in [(1u16, 0x10000u32), (13, 0x1000)] {
        let p = Game::player_handle(0);
        g.players[0].b.live = 1;
        g.players[0].b.hp = 0x64;
        g.players[0].score = 0;
        let mut it = Item::default();
        it.b.live = 1;
        it.b.k20 = class::FOOD * 4;
        it.b.var26 = var;
        it.b.partner = p;
        it.b.who = 0;
        // item_eat needs the item in its slot for the HUD message; the message itself is not checked.
        let h = g.items.handle(3);
        g.items.slots[3] = it.clone();
        let mut it = std::mem::take(&mut g.items.slots[3]);
        g.item_eat(&mut it, h);
        assert_eq!(g.players[0].score, expect, "food {var}");
        assert_eq!(it.b.hp, 0x63, "the HUD message shows 99");
    }
}

/// 0x24CE6: an item that lands near an edge hops back in; only the first bounce halves vy; the
/// sixth contact stops it.
#[test]
fn land_check_bounces_off_the_screen_edges() {
    let mut g = game(vec![], vec![]);
    g.cam.s2.cam_x = 0x500;
    let mut it = Item::default();
    // Inside the screen: stays (carry set), nothing changes.
    it.b.set_xi(0x600);
    it.b.vy = -0x300;
    assert!(g.item_land_check(&mut it));
    assert_eq!((it.b.vx, it.b.vy, it.b.airborne), (0, -0x300, 0));
    // Left edge (x <= cam + 0x10): back in with vx 0x180.
    it.b.set_xi(0x510);
    assert!(!g.item_land_check(&mut it));
    assert_eq!((it.b.vx, it.b.vy, it.b.airborne, it.m.b(o::B81)), (0x180, 0x180, 1, 1));
    // Second contact: vy is only reversed (QUIRK: the halving happens on the first bounce only).
    it.b.vy = -0x200;
    assert!(!g.item_land_check(&mut it));
    assert_eq!((it.b.vy, it.m.b(o::B81)), (0x200, 2));
    // Right edge (x >= cam + 0x170): back in with vx -0x100.
    it.b.set_xi(0x670);
    it.m.sb(o::B81, 4);
    assert!(!g.item_land_check(&mut it));
    assert_eq!((it.b.vx, it.m.b(o::B81)), (-0x100, 5));
    // The sixth bounce is refused (carry set): the item stays where it is.
    assert!(g.item_land_check(&mut it));
}

/// 0x28AB8..: a gun that is already full scores 1000 when the box refills it.
#[test]
fn ammo_box_refills_a_gun_and_pays_for_a_full_one() {
    let mut score = vec![0u8; 0x90];
    score[0x38..0x3C].copy_from_slice(&0x1000u32.to_be_bytes());
    // 0x2855A row of weapon type 1 (var 2): max 6, kind 0x204, +$A2 = 0.
    let mut rows = vec![0u8; 0x80];
    rows[8..16].copy_from_slice(&[0, 6, 2, 4, 0, 0, 0, 0]);
    let mut g = game(vec![(0x2855A, rows)], vec![(0x10292, score)]);
    let gun_h = g.items.handle(7);
    {
        let gun = &mut g.items.slots[7];
        gun.b.live = 1;
        gun.b.k20 = 0;
        gun.b.var26 = 2;
        gun.b.hp = 2;
        gun.m.sw(o::KIND, 0x204);
    }
    g.players[0].b.live = 1;
    g.players[0].item = gun_h;
    g.players[0].item_type = 0x204;
    assert!(g.item_ammo_refill(0, 0));
    assert_eq!(g.items.slots[7].b.hp, 6);
    assert_eq!(g.players[0].score, 0, "a gun that needed ammo costs nothing");
    // Full already: refilled to the same value and 1000 points.
    assert!(g.item_ammo_refill(0, 0));
    assert_eq!(g.players[0].score, 0x1000);
    // No gun in the hand: the box is not used.
    g.players[0].item_type = 0;
    assert!(!g.item_ammo_refill(0, 0));
}

/// The fuse of an explosive barrel (0x28E5A..): the stage table gives the blink count and the
/// period; the last stage blinks out into the explosion.
#[test]
fn explosive_barrel_fuse_counts_down_its_stages() {
    let mut g = game(vec![(0x28F24, vec![20, 10, 5, 2, 2, 3, 3, 6])], vec![]);
    let h = g.items.handle(2);
    let mut it = Item::default();
    it.b.live = 1;
    it.b.k20 = class::CONTAINER * 4;
    it.b.var26 = 2;
    it.m.sw(o::W5C, 1);
    it.b.hp = 50;
    it.b.hp_prev = 50;
    // A hit lights the fuse: stage 3 (blink count table[3] = 2, period table[3] = 6).
    it.b.hp = 40;
    assert!(!g.container_fuse(&mut it, h));
    assert_eq!((it.m.b(o::B6), it.m.b(o::B7), it.m.b(o::B8)), (3, 2, 6));
    assert_eq!(it.b.hp_prev, 40);
    // The blink period runs out: next blink; the blinks of the stage run out: next stage.
    it.m.sb(o::B8, 1);
    it.m.sb(o::B7, 0);
    assert!(!g.container_fuse(&mut it, h));
    assert_eq!((it.m.b(o::B6), it.m.b(o::B7), it.m.b(o::B8)), (2, 5, 3));
    // Stage 0 with nothing left: it explodes.
    it.m.sb(o::B6, 0);
    it.m.sb(o::B7, 0);
    it.m.sb(o::B8, 1);
    assert!(g.container_fuse(&mut it, h));
}

/// M4-18: the weapon sounds carry the registers of their routines (0x9C76, 0x9CD8, 0x9D9C, 0xA442, 0x9E08): d1 = the mode
/// (3, or 4 when the weapon's +$24 is set; 5 / 6 for the bazooka) in the high word, the middle byte (0x20; 0x12 for the bazooka)
/// and the pan byte of the weapon's X, d2 a constant per routine. Synthetic tables (the pan table counts 0, 1, 2 ...).
#[test]
fn weapon_sounds_send_d1_and_d2() {
    let mut g = game(vec![(0xA66A, (0..0x30).collect()), (0x9E5C, vec![0x69, 0xEB, 0xEC, 0]), (0xA498, vec![0, 0x81, 1, 2, 1, 3])], vec![]);
    g.cam.s2.cam_x = 0;
    g.item_snd_9c76(0x30, 0); // pan index 6
    g.item_snd_9cd8(0x30, 1);
    g.item_snd_9d9c(0x30, 0);
    let mut gun = Item::default();
    gun.b.set_xi(0x30);
    gun.b.partner = Game::player_handle(0);
    g.players[0].voice = 1; // the second player's bank
    g.item_snd_a442(&gun);
    gun.b.facing = 1;
    g.item_snd_9e08(&gun);
    let r: Vec<_> = g.w.snd_reqs.iter().map(|r| (r.code, r.d1, r.d2)).collect();
    assert_eq!(
        r,
        vec![
            (0xEA, 0x3_2006, 0x1500_0000),
            (0xE7, 0x4_2006, 0x1500_0000),
            (0xDC, 0x3_2006, 0x0240_0000),
            (0x0102, 0x3_2006, 0x1500_0000),
            (0xEB, 0x6_1206, 0x00C4_1240),
        ]
    );
}

/// 0x9F54 (eating) has d1 = mode 7 and the pan of the food, d2 = 0; the statue break 0xA396 (0x89) has the pan only.
#[test]
fn food_and_statue_sounds_send_d1_and_d2() {
    let mut g = game(vec![(0xA66A, (0..0x30).collect()), (0x9FAA, vec![0, 0x7C, 0, 0xFD, 0, 0xFE])], vec![]);
    g.cam.s2.cam_x = 0;
    let mut food = Item::default();
    food.b.set_xi(0x28); // pan index 5
    food.b.partner = Game::player_handle(0);
    g.players[0].voice = 1;
    g.item_snd_food(&food);
    g.item_snd_a396(0x28);
    let r: Vec<_> = g.w.snd_reqs.iter().map(|r| (r.code, r.d1, r.d2)).collect();
    assert_eq!(r, vec![(0xFD, 0x7_0005, 0), (0x89, 5, 0)]);
}
