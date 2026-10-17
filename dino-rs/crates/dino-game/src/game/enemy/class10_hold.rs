//! Class 10: the machine of sub 4 (0x3C60C). `+$AC` = 0 is the grab and throw (0x3C62C, held
//! player action 0x40, `+$C7` = 0); `+$AC` = 2 is the stomp on a downed, dizzy or dying player
//! that the player code claimed (0x3C864, `+$C7` = 1). Two of the position tests of the original
//! carry a bias of 0x400 that makes them fail for any real position (0x3C74A, 0x3C9C8); they are
//! kept as they are.

use super::class10::f;
use super::{Enemy, off::*};
use crate::game::{Game, Handle};

impl Game {
    /// 0x3C60C.
    pub(super) fn c10_hold(&mut self, e: &mut Enemy, h: Handle) {
        match e.m.w(f::HOLD_STATE) {
            0 => self.c10_hold0(e, h),
            2 => self.c10_claim(e, h),
            v => panic!("class 10 hold machine {v:#x}"),
        }
    }

    /// 0x3C61E: the grab machine starts.
    pub(super) fn c10_3c61e(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0204_0000);
        e.m.sw(f::HOLD_STATE, 0);
        self.c10_hold0(e, h);
    }

    /// 0x3C856: the stomp machine starts.
    pub(super) fn c10_3c856(&mut self, e: &mut Enemy, h: Handle) {
        e.b.set_state(0x0204_0000);
        e.m.sw(f::HOLD_STATE, 2);
        self.c10_claim(e, h);
    }

    // ------------------------------------------------------------------ grab and throw

    /// 0x3C62C.
    fn c10_hold0(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.act == 0 {
            e.b.act = 2; // 0x3C63E
        }
        match e.b.step {
            0 => self.c10_hs0(e, h),
            2 => self.c10_hs2(e, h),
            4 => self.c10_hs4(e, h),
            6 => self.c10_hs6(e, h),
            8 => self.c10_hs8(e, h),
            s => panic!("class 10 grab step {s:#x}"),
        }
        self.enemy_floor_collision(e);
        if e.b.wall == 0x10 {
            e.b.set_state(0x0200_1200);
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x3C67A: walk to the player's side.
    fn c10_hs0(&mut self, e: &mut Enemy, h: Handle) {
        e.b.step = 2;
        self.c10_3d04a(e);
        self.c10_3cc8a(e);
        self.enemy_set_speed(&mut e.b, 8);
        e.m.sw(f::HOLD_TIMER, 0x5A);
        e.m.sb(f::HOLD_T, 0);
        e.m.sb(f::T80, 0);
        self.c10_hs2(e, h);
    }

    /// 0x3C69E: approach; once close, look at the player every 5 frames for the moment to grab.
    fn c10_hs2(&mut self, e: &mut Enemy, h: Handle) {
        if e.m.b(f::HOLD_T) != 0 {
            let w = e.m.w(f::HOLD_TIMER);
            e.m.sw(f::HOLD_TIMER, w.wrapping_sub(1));
            if w == 0 {
                return self.c10_3bff2(e, h); // 0x3C722
            }
        }
        if e.b.airborne != 0 {
            return e.b.set_state(0x0200_0E00);
        }
        self.c10_3cbc0(e);
        if !self.c10_3ceac(e) {
            e.m.sb(f::T80, 0);
            self.enemy_move_dir(&mut e.b);
            self.enemy_step(&mut e.b);
            return;
        }
        // 0x3C6D0
        e.m.sb(f::HOLD_T, 1);
        let b = e.m.b(f::T80);
        e.m.sb(f::T80, b.wrapping_sub(1));
        if b != 0 {
            return;
        }
        e.m.sb(f::T80, 5);
        let tp = self.c10_target(e);
        let (live, mode_sub, action, kind, air) = {
            let p = &self.players[tp];
            (p.b.live, ((p.b.mode as u16) << 8) | p.b.sub as u16, p.action, p.b.kind, p.b.airborne)
        };
        if live != 1 || mode_sub != 0x200 || action == 0x40 || kind == 0xC || air != 0 {
            return self.c10_3bc78(e, h);
        }
        if self.c10_3c726(e, action) {
            self.c10_3c766(e, h);
        }
    }

    /// 0x3C726: may the thug grab now? (`d0` = the player's action class). Returns the carry.
    fn c10_3c726(&mut self, e: &Enemy, action: u8) -> bool {
        let tp = self.c10_target(e);
        let near = |g: &Game| -> bool {
            // The 0x400 bias of the original makes this window sit 0x400 px to the left.
            let d0 = e.b.xi().wrapping_sub(g.players[tp].b.xi()).wrapping_add(0x400).wrapping_add(0x30) as u16;
            d0 < 0x60
        };
        if action == 0 || action == 0x21 {
            return near(self);
        }
        if action == 0x10 {
            let d1 = self.players[tp].b.facing & 1;
            if d1 == e.b.facing {
                return near(self);
            }
        }
        false
    }

    /// 0x3C766: take the player (held state 0x02060000, action 0x40, not yet damaged by the hold).
    fn c10_3c766(&mut self, e: &mut Enemy, h: Handle) {
        e.b.step = 4;
        self.enemy_anim(&mut e.b, 0x10);
        let tp = self.c10_target(e);
        e.m.sw(f::HOLD_TIMER, 0x61);
        e.b.partner = Game::player_handle(tp);
        e.b.kind = 0xC;
        let p = &mut self.players[tp];
        p.b.partner = h;
        p.b.set_state(0x0206_0000);
        p.action = 0x40;
        p.b.kind = 0xE;
        p.bc7 = 0;
        p.b.hp_prev = p.b.hp;
        self.c10_hs4(e, h);
    }

    /// 0x3C7AC: hold for 0x61 frames (the player may break free: his action is no longer 0x40).
    fn c10_hs4(&mut self, e: &mut Enemy, h: Handle) {
        let tp = self.c10_target(e);
        if self.players[tp].action != 0x40 {
            return self.c10_3bc78(e, h);
        }
        let w = e.m.w(f::HOLD_TIMER);
        e.m.sw(f::HOLD_TIMER, w.wrapping_sub(1));
        if w != 0 {
            return;
        }
        // 0x3C7C4
        e.b.step = 6;
        self.enemy_anim(&mut e.b, 0x11);
        self.c10_hs6(e, h);
    }

    /// 0x3C7D2: the wind up of the throw.
    fn c10_hs6(&mut self, e: &mut Enemy, h: Handle) {
        if !self.enemy_step(&mut e.b) {
            return;
        }
        // 0x3C7DE
        e.b.step = 8;
        self.enemy_anim(&mut e.b, 0x12);
        let tp = self.c10_target(e);
        if self.players[tp].action == 0x40 {
            let mut d0 = -0x4Ci16;
            if e.b.facing != 0 {
                d0 = -d0;
            }
            let x = d0.wrapping_add(e.b.xi());
            let y = 0x10i16.wrapping_add(e.b.yi());
            let p = &mut self.players[tp];
            p.b.set_state(0x0202_0000);
            p.b.act = 8;
            p.b.set_xi_keep(x);
            p.b.set_yi_keep(y);
            e.b.partner = Game::player_handle(tp);
            self.enemy_begin_attack(&mut e.b, 0x28);
            self.enemy_hit_held_player(e, h);
        }
        self.c10_hs8(e, h);
    }

    /// 0x3C84A: the throw animation, then the next decision.
    fn c10_hs8(&mut self, e: &mut Enemy, h: Handle) {
        if self.enemy_step(&mut e.b) {
            self.c10_3bc78(e, h);
        }
    }

    // ------------------------------------------------------------------ stomp on a downed player

    /// 0x3C864.
    fn c10_claim(&mut self, e: &mut Enemy, h: Handle) {
        if e.b.act == 0 {
            e.b.act = 2; // 0x3C876
        }
        let tp = self.c10_target(e);
        if self.players[tp].slow == 0 {
            return self.c10_3c916(e);
        }
        match e.b.step {
            0 => self.c10_cs0(e, h),
            2 => self.c10_cs2(e, h),
            4 => self.c10_cs4(e, h),
            6 => self.c10_cs6(e, h),
            8 => self.c10_cs8(e),
            s => panic!("class 10 stomp step {s:#x}"),
        }
        self.enemy_draw_check(&mut e.b, h);
        self.enemy_push_hurtable(h);
    }

    /// 0x3C8AA: walk towards the player.
    fn c10_cs0(&mut self, e: &mut Enemy, h: Handle) {
        e.b.step = 2;
        self.c10_3ccc0(e);
        self.enemy_anim(&mut e.b, 3);
        self.enemy_set_speed(&mut e.b, 8);
        self.c10_cs2(e, h);
    }

    /// 0x3C8C4.
    fn c10_cs2(&mut self, e: &mut Enemy, h: Handle) {
        self.c10_3cc2c(e);
        if !self.c10_3ceda(e) {
            self.enemy_move_dir(&mut e.b);
            self.enemy_walk_anim(&mut e.b);
            return;
        }
        e.b.step = 4;
        self.enemy_anim(&mut e.b, 0);
        self.c10_cs4(e, h);
    }

    /// 0x3C8E8: near the player: wait for the claim (both flags) of a lying player.
    fn c10_cs4(&mut self, e: &mut Enemy, h: Handle) {
        if !self.c10_3ceda(e) {
            return self.c10_cs0(e, h);
        }
        let tp = self.c10_target(e);
        let p = &self.players[tp];
        if ((p.b.mode as u16) << 8 | p.b.sub as u16) != 0x200 {
            return self.c10_3c916(e);
        }
        if p.action != 0x21 {
            return;
        }
        if p.b7e == 0 || p.b7f == 0 {
            return self.c10_3c916(e);
        }
        // 0x3C928
        e.b.step = 6;
        self.c10_3ccc0(e);
        self.enemy_anim(&mut e.b, 3);
        self.enemy_set_speed(&mut e.b, 8);
        self.c10_cs6(e, h);
    }

    /// 0x3C942.
    fn c10_cs6(&mut self, e: &mut Enemy, h: Handle) {
        self.c10_3cc2c(e);
        if !self.c10_3c9c8(e) {
            self.enemy_move_dir(&mut e.b);
            self.enemy_walk_anim(&mut e.b);
            return;
        }
        // 0x3C958: the stomp grab: effect, animation 8, the player is held (`+$C7` = 1).
        e.b.step = 8;
        self.c10_3c9fc(e, h);
        self.enemy_anim(&mut e.b, 8);
        // `sub.w ; bcs`: camera + 0xC0 < X (unsigned) -> facing 0, else 1.
        e.b.facing = if (self.cam.s2.cam_x.wrapping_add(0xC0)) < e.b.xi() as u16 { 0 } else { 1 };
        let tp = self.c10_target(e);
        e.b.partner = Game::player_handle(tp);
        e.b.kind = 0xC;
        let f = e.b.facing;
        let p = &mut self.players[tp];
        p.b.partner = h;
        p.b.set_state(0x0206_0000);
        p.b.facing = f;
        p.action = 0x40;
        p.b.kind = 0xE;
        p.bc7 = 1;
        p.b.hp_prev = p.b.hp;
        self.c10_cs8(e);
    }

    /// 0x3C9B8: hold while the player stays in the held action.
    fn c10_cs8(&mut self, e: &mut Enemy) {
        let tp = self.c10_target(e);
        if self.players[tp].action != 0x40 {
            self.c10_3c916(e);
        }
    }

    /// 0x3C9FC: an effect (pool C class 3) at the thug's position, owner = the thug. The effects
    /// port runs its handler; until then the object is freed on its first update.
    fn c10_3c9fc(&mut self, e: &Enemy, h: Handle) {
        let Some(i) = self.effects.alloc() else { return };
        let b = &mut self.effects.slots[i].b;
        b.live = 1;
        b.k20 = 0xC;
        b.set_xi_keep(e.b.xi());
        b.set_yi_keep(e.b.yi());
        b.set_zi_keep(e.b.zi());
        b.var26 = 0;
        self.effects.slots[i].owner58 = h;
        self.effects.slots[i].owner_class = e.b.k20;
    }

    /// 0x125C0: the thug hurts the player it holds (`+$70`, which the caller already put into the
    /// hurt state 0x02020000 / action 8). Damage = the attack record `+$58` scaled by 0x6422, the
    /// hurt reaction of the record, hit stop on both, the hit sound; then the weapon drop (0x1177A)
    /// and the power factor (0x11738).
    pub(super) fn enemy_hit_held_player(&mut self, e: &mut Enemy, me: Handle) {
        if self.w.scripted != 0 {
            return; // tst.b $4dd(a5)
        }
        let Some(crate::game::Slot::Player(pi)) = crate::game::slot_of(e.b.partner) else { return };
        let rec = self.assets.player.enemy_attacks.get(e.b.atk as usize).copied().unwrap_or_default();
        {
            let p = &mut self.players[pi];
            p.b.attacker = me;
            p.b.live = 2;
            p.b.set_state(0x0202_0000);
            p.b7a = 0;
            p.hits_t = 0x3C;
            p.hits = p.hits.wrapping_add(1);
            p.slow = 0;
            p.combo = 0;
            p.hit_serial = e.b.serial;
            p.b.hp_prev = p.b.hp;
        }
        let r = self.rand();
        let (hp, chr) = (self.players[pi].b.hp, self.players[pi].b.id);
        let dmg = crate::game::player_combat::enemy_damage(&self.assets.player, rec.damage as u8, self.w.level, e.m.w(W84), chr, hp, r);
        let p = &mut self.players[pi];
        let nhp = (p.b.hp as i16).wrapping_sub(dmg as i16);
        p.b.hp = if nhp > 0 { nhp as u16 } else { 0 };
        p.stun_refill = 0x78;
        p.b.stun = p.b.stun.saturating_sub(rec.stun);
        p.b.knock = 0;
        p.b.reaction = rec.reaction;
        p.b.act = rec.reaction;
        p.b.step = 0;
        p.b.hitstop = rec.hit_stop;
        e.b.hitstop = rec.hit_stop;
        p.b.b9a = rec.b7;
        if rec.sound != 0 {
            let (ax, vf) = (e.b.xi(), e.b.facing);
            self.snd_hit(rec.sound, ax, vf);
        }
        self.player_drop_check(pi);
        self.player_power_factor(pi);
    }
}
