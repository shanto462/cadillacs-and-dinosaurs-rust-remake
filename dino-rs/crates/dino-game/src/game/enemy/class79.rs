//! The dinosaur hunter, class 79 (+$20 = 0x13C, handler 0x5F5D0..0x5FF2E, 2,398 bytes): HAMMER T. (variant 0, name 0x1E), BLK ELMER (variant 2, name 0x1F) and
//! WRENCH T. (variant 4, name 0x20) who walk up to a sleeping dinosaur (Shivat, Rock Hopper, Mack: `+$C0` negative), swing at it three times and poke it
//! awake with the third swing (helper 0x533FA, `enemy_hunter_poke`). D3a section 2 and 5, D3b 1.8. M6-06.
//!
//! +$04 mode: 0 (sub 0 the target scan and the init, sub 2 the walk in, sub 4 the entry into the AI), 2 the AI (sub 0 the frame 0x5F7F6, sub 2 the generic hit
//! reaction 0x2A606, sub 4 nothing, sub 6 held 0x2B35A; kill-all 0x326F8 first, the far test 0x173C last), 4 death (0x5FA8E), 6 free (0x5FADA). The AI
//! states (+$06, table 0x5F842): 0 wait and check the target, 2 walk (step 0 in Z, step 2 in X, step 4 in Z again), 4 stand and look (animations 3, 4, the alert
//! spark 0xD), 6 the swing, 8 walk away to the left. Own bytes: +$76 the dinosaur, +$A0 the counter of the scan and the "target seen awake" flag, +$80 word timer.
//! The hunter has no attack on heroes apart from the swing (attack 5), registers in the hurtable list (0x4BEA) and is counted by the waves (+$7D = 0).
//!
//! Owner: enemy port (M6-06).

use super::{off::*, Enemy};
use crate::game::{slot_of, Game, Handle, Slot};

/// The speed ids by variant and level (rows of 8 words, `dino-extract` TABLES): 0x5FB50.
const SPEEDS: u32 = 0x5FB50;
/// Own byte of the class: the scan counter / "has seen the target awake" flag.
const SEEN: usize = 0xA0;
/// The dinosaur's bytes the hunter reads and writes: the hunt flag (negative = asleep, 0 awake neutral, 1 hostile) and the claim flag.
const HUNT: usize = 0xC0;
const CLAIM: usize = 0xC1;

impl Game {
    /// 0x5F5D0: class 79. Mode 0 sub 0 / 2 / 4, mode 2, mode 4 (death), mode 6 frees (0x5FADA).
    pub(super) fn enemy_class79(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        match e.b.mode {
            0 => match e.b.sub {
                0 => self.c79_init(e, h, i),
                2 => self.c79_walk_in(e, h),
                4 => {
                    // 0x5F7C4: the entry into the AI, then the draw check once more
                    self.c79_enter_ai(e, h);
                    self.enemy_draw_check(&mut e.b, h);
                }
                _ => {}
            },
            2 => {
                self.enemy_kill_all_check(e); // 0x326F8
                match e.b.sub {
                    0 => self.c79_alive(e, h),
                    2 => self.enemy_hit_reaction(e, h), // 0x2A606
                    6 => self.enemy_grabbed(e, h),      // 0x2B35A
                    _ => {}                             // 0x5FA8C: rts
                }
                self.enemy_despawn_if_far(e, i); // jmp 0x173C
            }
            4 => self.c79_death(e, h),
            _ => self.enemy_free_pool_a(e, i), // 0x5FADA: jmp 0x48FC
        }
    }

    /// `+$C0` of the dinosaur (pool A object `t`); other objects read 0.
    fn c79_hunt_byte(&self, t: Handle) -> u8 {
        match slot_of(t) {
            Some(Slot::Enemy(i)) => self.enemies.slots[i].m.b(HUNT),
            _ => 0,
        }
    }

    /// X and Z words of the dinosaur.
    fn c79_target_xz(&self, e: &Enemy) -> (i16, i16) {
        let b = self.body(e.m.w(TARGET));
        (b.xi(), b.zi())
    }

    /// 0x5F6CA: the target scan. Pool A in slot order for the classes 24, 29 and 38 whose `+$C0` is negative: the first one found is kept, the next unclaimed one
    /// ends the scan with the nearer in X (ties to the later one); else the first. The chosen one gets `+$C1` = 1. +$A0 counts what was found. True (carry) when
    /// there is none.
    fn c79_scan(&mut self, e: &mut Enemy) -> bool {
        e.m.sb(SEEN, 0);
        let mut first: Option<usize> = None;
        for i in 0..self.enemies.slots.len() {
            let o = &self.enemies.slots[i];
            if o.b.live == 0 || !matches!(o.b.k20, 0x60 | 0x74 | 0x98) || (o.m.b(HUNT) as i8) >= 0 {
                continue;
            }
            match first {
                None => {
                    first = Some(i);
                    e.m.sb(SEEN, 1);
                }
                Some(a1) => {
                    if o.m.b(CLAIM) != 0 {
                        continue;
                    }
                    let x = e.b.xi();
                    let d1 = self.enemies.slots[a1].b.xi().wrapping_sub(x).wrapping_abs();
                    let d2 = o.b.xi().wrapping_sub(x).wrapping_abs();
                    let pick = if d2 > d1 { a1 } else { i };
                    return self.c79_take(e, pick);
                }
            }
        }
        match first {
            Some(a1) => self.c79_take(e, a1),
            None => true,
        }
    }

    /// 0x5F742 / 0x5F74E: `+$76` = the dinosaur, `+$C1` = 1; false (carry clear).
    fn c79_take(&mut self, e: &mut Enemy, i: usize) -> bool {
        e.m.sw(TARGET, self.enemies.handle(i));
        self.enemies.slots[i].m.sb(CLAIM, 1);
        false
    }

    /// 0x5F5F8: mode 0 sub 0. No dinosaur asleep: free at once (the next frame the pool update frees it, as `bcs 0x5FADA`: `jmp 0x48FC` now). Else the name, the
    /// animation table and the HP of the variant (the byte +$26), the shared init, the walk in.
    fn c79_init(&mut self, e: &mut Enemy, h: Handle, i: usize) {
        if self.c79_scan(e) {
            return self.enemy_free_pool_a(e, i); // 0x5FADA
        }
        e.b.sub = e.b.sub.wrapping_add(2);
        let (id, w84, table) = match e.b.var26 >> 8 {
            0 => (0x1E, 0x78, 0x5FB80),
            2 => (0x1F, 0x7C, 0x5FCBA),
            4 => (0x20, 0x80, 0x5FDF4),
            v => panic!("class 79 variant {v:#x}: the table 0x5F66A has three entries"),
        };
        e.b.id = id;
        e.m.sw(W84, w84);
        e.b.anim_table = table;
        self.enemy_set_hp(&mut e.b); // jmp 0x31424
        self.enemy_anim(&mut e.b, 0);
        e.b.box_table = 0x103000;
        e.b.multi_box = false;
        e.b.stun = 10;
        for off in [POISE_T, WAIT, ATTACKING, B87] {
            e.m.sb(off, 0);
        }
        e.b.reaction = 0;
        e.b.b63 = 0;
        e.b.b72 = 0xB4;
        e.m.sb(SEEN, 0);
        e.b.who = 0xFF;
        self.enemy_snap_to_floor(e); // 0x3140C
        e.m.sb(ATTACKING, 0);
        self.c79_walk_in(e, h); // bra 0x5F75C
    }

    /// 0x5F75C: mode 0 sub 2: state 0 (0x5F77A) the speed, animation 1, the side toward the screen centre, 60 frames of walking (+$80); state 2 (0x5F7AE) counts
    /// them down and then sub 4. Then the draw check and the hurtable list.
    fn c79_walk_in(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.act == 0 {
            self.enemy_anim(&mut e.b, 1);
            self.c79_speed(e); // bsr 0x5FB30
            let (dir, facing) = if (self.cam.s2.cam_x as i16).wrapping_add(0xC0).wrapping_sub(e.b.xi()) >= 0 { (8, 1) } else { (0x18, 0) };
            e.b.dir = dir;
            e.b.facing = facing;
            e.m.sw(T80, 0x3C);
            e.b.act = e.b.act.wrapping_add(2);
        }
        if e.b.act == 2 {
            let w = e.m.w(T80);
            e.m.sw(T80, w.wrapping_sub(1));
            if w == 0 {
                e.b.sub = e.b.sub.wrapping_add(2); // `subq.w #1,$80(a6); bcc`: a borrow only from 0
            }
            self.enemy_move_dir(&mut e.b); // jsr 0x1862
            self.enemy_step(&mut e.b); // jmp 0x121E
        }
        self.c79_end(e, h); // jsr 0x49CA, jmp 0x4BEA
    }

    /// 0x5F836: the draw check (0x49CA) and the hurtable list (0x4BEA).
    fn c79_end(&mut self, e: &mut Enemy, h: Handle) {
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x5FB30: the walking speed id by the variant and the level (table 0x5FB50, rows of 8 words by (level & 0x1C) / 2), set through 0x17A4.
    fn c79_speed(&mut self, e: &mut Enemy) {
        let row = (e.b.var26 >> 8) as u32 * 8;
        let col = ((self.w.level & 0x1C) >> 1) as u32;
        let id = self.assets.enemy.t.u16(SPEEDS, row + col);
        self.enemy_set_speed(&mut e.b, id);
    }

    /// 0x5F84C: back to the AI: mode 2, sub 0, state 0, animation 1, `+$7D` = 0 and a 2 frame wait when none runs; then the draw check and the hurtable list. This is
    /// also the recover entry of the hit reaction (the three names in the table 0x2B56C).
    pub(super) fn c79_enter_ai(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 1);
        e.m.sb(ATTACKING, 0);
        if e.m.b(WAIT) == 0 {
            e.m.sb(WAIT, 2);
        }
        self.c79_end(e, h); // bra 0x5F836
    }

    /// 0x5F7F6: the AI frame. The poise refill, the check whether the dinosaur woke up neutral (`+$C0` = 0: 0x5FA5A once), the state, the floor, the draw and the list.
    fn c79_alive(&mut self, e: &mut Enemy, h: Handle) {
        let p = e.m.b(POISE_T);
        if p != 0 {
            e.m.sb(POISE_T, p - 1);
            if p == 1 {
                e.b.stun = 10;
            }
        }
        if e.m.b(SEEN) == 0 && self.c79_hunt_byte(e.m.w(TARGET)) == 0 {
            self.c79_target_awake(e, h); // bsr 0x5FA5A
            e.m.sb(SEEN, 1);
        }
        match e.b.act {
            0 => self.c79_state0(e, h),
            2 => self.c79_state2(e, h),
            4 => self.c79_state4(e, h),
            6 => self.c79_swing(e, h),
            8 => {
                // 0x5FA80: walk away
                self.enemy_move_dir(&mut e.b);
                self.enemy_step(&mut e.b);
            }
            _ => {}
        }
        self.terrain_collide_free(&mut e.b); // 0x12E20
        self.c79_end(e, h);
    }

    /// 0x5FA5A: the dinosaur is awake: it is hostile now (`+$C1`... `+$C0` = 1); look for another sleeper: found, the AI starts again (0x5F84C); none: state 8, walk left.
    fn c79_target_awake(&mut self, e: &mut Enemy, h: Handle) {
        if let Some(Slot::Enemy(i)) = slot_of(e.m.w(TARGET)) {
            self.enemies.slots[i].m.sb(HUNT, 1);
        }
        if !self.c79_scan(e) {
            return self.c79_enter_ai(e, h); // bcc 0x5F84C
        }
        e.b.set_state(0x0200_0800);
        e.b.dir = 0x18; // 0x5FB0A
        e.b.facing = 0;
        self.enemy_anim(&mut e.b, 1);
    }

    /// 0x5FB18: faces the dinosaur: right when its X is not smaller.
    fn c79_face_target(&self, e: &mut Enemy) {
        let (tx, _) = self.c79_target_xz(e);
        e.b.facing = u8::from(tx.wrapping_sub(e.b.xi()) >= 0);
    }

    /// 0x5F870: state 0. The wait +$7A, then: the dinosaur is no longer asleep: 0x5FA5A; else the speed, animation 1, state 2 and the same frame the walk.
    fn c79_state0(&mut self, e: &mut Enemy, h: Handle) {
        let w = e.m.b(WAIT);
        if w != 0 {
            e.m.sb(WAIT, w - 1);
            if w - 1 != 0 {
                return; // 0x5F882
            }
        }
        if (self.c79_hunt_byte(e.m.w(TARGET)) as i8) >= 0 {
            return self.c79_target_awake(e, h); // bpl 0x5FA5A
        }
        self.c79_speed(e);
        self.enemy_anim(&mut e.b, 1);
        e.b.set_state(0x0200_0200);
        e.m.sb(ATTACKING, 0);
        self.c79_face_target(e); // bsr 0x5FB18
        self.c79_state2(e, h); // falls into 0x5F8AE
    }

    /// 0x5FAE0: the depth test: his Z minus the dinosaur's plus 8 below 8 (unsigned).
    fn c79_depth_near(&self, e: &Enemy) -> bool {
        let (_, tz) = self.c79_target_xz(e);
        (e.b.zi().wrapping_sub(tz).wrapping_add(8) as u16) < 8
    }

    /// 0x5FAF4: the X test: the dinosaur's X minus his plus 0x28 below 0x50 (unsigned), within 40 px.
    fn c79_x_near(&self, e: &Enemy) -> bool {
        let (tx, _) = self.c79_target_xz(e);
        (tx.wrapping_sub(e.b.xi()).wrapping_add(0x28) as u16) < 0x50
    }

    /// The walk in Z (steps 0 and 4 of state 2): the stuck check (0x324EE) or the depth reached ends the step; direction 0x10 (toward the camera... smaller Z)
    /// when the dinosaur's Z is smaller, else 0.
    fn c79_walk_z(&mut self, e: &mut Enemy) -> bool {
        if Self::enemy_stuck_check(e) {
            return true; // bcs 0x5F8F4
        }
        let (_, tz) = self.c79_target_xz(e);
        e.b.dir = if tz.wrapping_sub(e.b.zi()) < 0 { 0x10 } else { 0 };
        if self.c79_depth_near(e) {
            return true;
        }
        self.enemy_move_dir(&mut e.b); // jsr 0x1862
        self.enemy_step(&mut e.b); // jmp 0x121E
        false
    }

    /// 0x5F8AE: state 2, the walk: step 0 in Z, step 2 in X (direction 8 right, 0x18 left, 40 px), step 4 in Z again, then the state 4 (or 0x5FA5A when the
    /// dinosaur has woken up on the way).
    fn c79_state2(&mut self, e: &mut Enemy, h: Handle) {
        match e.b.step {
            0 => {
                if self.c79_walk_z(e) {
                    e.b.step = e.b.step.wrapping_add(2); // 0x5F8F4
                }
            }
            2 => {
                let (tx, _) = self.c79_target_xz(e);
                e.b.dir = if tx.wrapping_sub(e.b.xi()) >= 0 { 8 } else { 0x18 };
                if self.c79_x_near(e) {
                    e.b.step = e.b.step.wrapping_add(2); // 0x5F924
                } else {
                    self.enemy_move_dir(&mut e.b);
                    self.enemy_step(&mut e.b);
                }
            }
            4 => {
                if self.c79_walk_z(e) {
                    // 0x5F95C
                    if (self.c79_hunt_byte(e.m.w(TARGET)) as i8) >= 0 {
                        // bpl 0x5FA5A: a tail jump, the caller is the AI frame
                        return self.c79_target_awake(e, h);
                    }
                    e.b.set_state(0x0200_0400);
                    e.m.sw(T80, 0x1E);
                    self.enemy_anim(&mut e.b, 3);
                }
            }
            _ => {}
        }
    }

    /// 0x5F97E: state 4. Step 0: look (animation 3 for 30 frames), then animation 4, the alert spark 0xD above his head; step 2 (and the same frame): 30 frames,
    /// then the state 6 and the swing animation 2.
    fn c79_state4(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.step == 0 {
            let w = e.m.w(T80);
            e.m.sw(T80, w.wrapping_sub(1));
            if w != 0 {
                self.enemy_step(&mut e.b); // jmp 0x121E
                return;
            }
            e.b.step = e.b.step.wrapping_add(2);
            self.enemy_anim(&mut e.b, 4);
            e.m.sw(T80, 0x1E);
            self.c79_spark(e, h, e.b.xi(), e.b.yi().wrapping_add(0x50), e.b.zi(), 0xD); // jsr 0x92422
        }
        let w = e.m.w(T80);
        e.m.sw(T80, w.wrapping_sub(1));
        if w != 0 {
            self.enemy_step(&mut e.b); // jmp 0x121E
            return;
        }
        e.b.set_state(0x0200_0600); // 0x5F9D6
        self.enemy_anim(&mut e.b, 2);
    }

    /// 0x92422 with the hunter as the caller (its facing): the body is copied into its slot for the call.
    fn c79_spark(&mut self, e: &Enemy, h: Handle, x: i16, y: i16, z: i16, spark: u16) {
        let i = self.enemy_index(h);
        let saved = std::mem::replace(&mut self.enemies.slots[i].b, e.b.clone());
        self.fx_spawn_spark_at(h, x, y, z, spark);
        self.enemies.slots[i].b = saved;
    }

    /// 0x5F9E6: state 6, the swing (animation 2). The dinosaur must still be asleep (else 0x5FA5A). The frame flag +$2D: negative on the third swing, which pokes the
    /// dinosaur (0x533FA); any flag starts the swing attack 0x5 (new serial, the flag cleared, the slash effect 0x92450, the sound 0xA1A0). At the end of the animation 0x5FA5A.
    fn c79_swing(&mut self, e: &mut Enemy, h: Handle) {
        if (self.c79_hunt_byte(e.m.w(TARGET)) as i8) >= 0 {
            return self.c79_target_awake(e, h); // bpl 0x5FA5A
        }
        if (e.b.param_b as i8) < 0 {
            self.enemy_hunter_poke(e, h); // jsr 0x533FA
        }
        if e.b.param_b != 0 {
            self.enemy_begin_attack(&mut e.b, 5);
            e.b.param_b = 0;
            let dx: i16 = if e.b.facing != 0 { 0x58 } else { -0x58 };
            self.fx_spawn_spark_jitter(e.b.xi().wrapping_add(dx), e.b.yi().wrapping_add(0x30), e.b.zi().wrapping_sub(1), 5); // 0x5FA36, jmp 0x92450
            // 0xA1A0: the code 0x3B, or 0xAB when the bit 4 of G.$43B was clear (it is flipped), the pan of the X
            let code = if self.snd_toggle_b(4) { 0x3B } else { 0xAB };
            self.snd_request(code, self.snd_pan_x(e.b.xi()) as u32, 0);
        }
        if self.enemy_step(&mut e.b) {
            self.c79_target_awake(e, h); // bcs 0x5FA5A
        }
    }

    /// 0x5FA8E: mode 4, the death. Sub 0: the claim on the dinosaur is let go (when it is still there), the kill score, 120 frames; sub 2: counts them down, drawn on
    /// every other frame; then mode 6.
    fn c79_death(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.sub == 0 {
            if let Some(Slot::Enemy(i)) = slot_of(e.m.w(TARGET))
                && self.enemies.slots[i].b.live != 0 {
                    self.enemies.slots[i].m.sb(CLAIM, 0);
                }
            let b = e.b.clone();
            self.enemy_award_kill_score(&b); // jsr 0x325E6
            e.m.sw(T80, 0x78);
            e.b.sub = e.b.sub.wrapping_add(2);
        }
        let w = e.m.w(T80);
        if w == 0 {
            e.b.mode = e.b.mode.wrapping_add(2);
            return;
        }
        e.m.sw(T80, w - 1);
        if (w - 1) & 1 != 0 {
            self.enemy_draw_check(&mut e.b, h); // jmp 0x49CA
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn sleeper(g: &mut Game, class: u16, x: i16, claimed: u8) -> Handle {
        let i = g.enemies.alloc().unwrap();
        let s = &mut g.enemies.slots[i];
        s.b.live = 2;
        s.b.k20 = class * 4;
        s.b.set_xi(x);
        s.m.sb(HUNT, 0xFF);
        s.m.sb(CLAIM, claimed);
        g.enemies.handle(i)
    }

    /// 0x5F6CA: the first sleeper found is taken without a claim test; a second unclaimed one decides by the X distance (a tie goes to the later one) and the
    /// chosen one is claimed. Nothing asleep: carry (the hunter frees itself).
    #[test]
    fn the_scan_picks_the_nearer_sleeper_and_claims_it() {
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        let mut hunter = Enemy::default();
        hunter.b.set_xi(0x400);
        assert!(g.c79_scan(&mut hunter), "no sleeper: carry");
        // the allocator hands out the high slots first, so the one made first is found LAST in the slot order
        let far = sleeper(&mut g, 24, 0x100, 0);
        let near = sleeper(&mut g, 38, 0x3F0, 0);
        assert!(!g.c79_scan(&mut hunter));
        assert_eq!(hunter.m.w(TARGET), near);
        assert_eq!(g.enemies.slots[g.enemy_index(near)].m.b(CLAIM), 1);
        assert_eq!(g.enemies.slots[g.enemy_index(far)].m.b(CLAIM), 0);
        // a claimed second one is not considered: the first found is taken even if it is claimed
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        let a = sleeper(&mut g, 24, 0x100, 1);
        let _b = sleeper(&mut g, 29, 0x3F0, 1);
        assert!(!g.c79_scan(&mut hunter));
        assert_eq!(hunter.m.w(TARGET), g.enemies.handle(g.enemy_index(a).min(g.enemy_index(_b))));
        // an awake dinosaur (+$C0 >= 0) is no target
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        let w = sleeper(&mut g, 24, 0x300, 0);
        let wi = g.enemy_index(w);
        g.enemies.slots[wi].m.sb(HUNT, 1);
        assert!(g.c79_scan(&mut hunter));
    }
}
