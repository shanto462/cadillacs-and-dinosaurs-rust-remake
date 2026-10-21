//! Shared pieces of the D3d boss handlers. Class handlers keep their own state dispatch,
//! counterattacks and transformations. These helpers do not run an invented common AI.
#![allow(dead_code)] // Called as the remaining D3d classes replace their placeholders.

use crate::game::enemy::{Enemy, off::*};
use crate::game::{Game, Handle};

/// Table selection in 0x52C50, 0x55748, 0x38BD6, 0x675AE and 0x695B0.
/// The gate uses G.$5EC (aggression), not G.$5EE (level).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossAi {
    Morgan,
    Morgue,
    Tyrog,
    Tyrog2,
    Fessenden,
}

impl BossAi {
    /// 0x51942 / 0x54156 / 0x36D0E / 0x65734 / 0x6804C.
    fn phase_tables(self) -> (u32, u32) {
        match self {
            Self::Morgan => (0x5196A, 0x5198A),
            Self::Morgue => (0x5418A, 0x541AA),
            Self::Tyrog => (0x36D36, 0x36D56),
            Self::Tyrog2 => (0x6575C, 0x6577C),
            Self::Fessenden => (0x68080, 0x680A0),
        }
    }

    /// 0x38BE6 / 0x675BE: SUBQ.B #1 followed by BMI / BEQ.
    fn form_index(form: u8) -> u32 {
        let d3 = form.wrapping_sub(1);
        if (d3 as i8) < 0 {
            0
        } else if d3 == 0 {
            1
        } else {
            2
        }
    }

    /// 0x52C60 / 0x55756 / 0x38BF4 / 0x675CC / 0x695BE.
    pub fn gate_table(self, form: u8) -> u32 {
        match self {
            Self::Morgan => 0x73B14,
            Self::Morgue => 0x73D54,
            Self::Tyrog | Self::Tyrog2 => 0x74542 + Self::form_index(form) * 0x40,
            Self::Fessenden => 0x74A42,
        }
    }

    /// 0x52CB0 / 0x557AC / 0x38C6A / 0x67642 / 0x69614.
    /// Morgan selects the second table when +$AB is nonnegative.
    pub fn action_table(self, form: u8, ab: u8) -> u32 {
        match self {
            Self::Morgan => {
                if (ab as i8) < 0 {
                    0x73B54
                } else {
                    0x73C54
                }
            }
            Self::Morgue => 0x73D94,
            Self::Tyrog | Self::Tyrog2 => 0x74602 + Self::form_index(form) * 0x100,
            Self::Fessenden => 0x74A82,
        }
    }
}

/// Continuation after a recover entry's shared work. The future class handler
/// enters its own routine in the same frame, not on the next update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossRecover {
    /// Morgan 0x527A8; Tyrog 0x382B0; Tyrog 2 0x66CCC.
    WakeAttack,
    /// Morgan 0x52A70; Tyrog 0x3862A; Tyrog 2 0x668F6.
    OffscreenHop,
    /// Morgue 0x554EE; Fessenden 0x69374. Attack 0x4A is already set.
    Shoulder,
    /// Morgan 0x522A8 (the kite entry).
    Kite,
    /// Morgue 0x55704; Fessenden 0x6956C (the decision).
    Decide,
    /// Tyrog/Tyrog 2 state 0x4C is ready.
    Wait,
}

impl Game {
    /// 0x32032 (BT22): reuse the common target selector, including its >=3-attacker
    /// fallback. No RNG draw. The fewest-attacker path breaks ties toward the later player.
    pub fn boss_pick_target(&self, e: &mut Enemy) {
        e.m.sw(TARGET, self.enemy_pick_target(&e.b));
    }

    /// 0x51928 / 0x54144 / 0x36CF4 / 0x6803A: phase countdown and flip.
    /// Morgue/Fessenden callers handle their off-screen escape before entering the flip.
    /// On that escape they must not call this helper (or consume its RNG draw).
    pub fn boss_phase_tick(&mut self, e: &mut Enemy, ai: BossAi) -> bool {
        let timer = e.m.w(0xA6).wrapping_sub(1);
        e.m.sw(0xA6, timer);
        if timer != 0 {
            return false;
        }
        let r = (self.rand() & 0x1E) as u32;
        let (aggressive, defensive) = ai.phase_tables();
        let phase = u8::from(e.m.b(0xA0) == 0);
        let table = if phase != 0 { aggressive } else { defensive };
        e.m.sw(0xA6, self.assets.enemy.t.u16(table, r));
        e.m.sb(0xA0, phase);
        self.boss_pick_target(e);
        if matches!(ai, BossAi::Morgue | BossAi::Fessenden) {
            self.lib_slot6_new(e); // 0x32C5E
            self.boss_slot4_keep(e); // 0x32D0C, distinct from the 3-slot library
        }
        true
    }

    /// 0x32D0C / 0x32D16: keep +$BD, reroll zero with rand & 3; use
    /// table 0x32D7E even when the preceding six-slot choice left 4..6.
    pub(in crate::game) fn boss_slot4_keep(&mut self, e: &mut Enemy) {
        let mut slot = e.m.b(0xBD);
        while slot == 0 {
            slot = (self.rand() & 3) as u8;
            e.m.sb(0xBD, slot);
        }
        let dx = self.assets.enemy.t.i16(0x32D7E, slot as u32 * 2);
        let t = self.lib_target(e);
        let (x, z) = (t.xi(), t.zi());
        e.m.sw(DEST_X, x.wrapping_add(dx) as u16);
        e.m.sw(DEST_Z, z as u16);
        self.lib_slot_mirror(e, dx);
    }

    /// 0x52C54 / 0x5574C / 0x38BDA: gate table byte by aggression and
    /// +$A5. ADD.B changes only the low byte of the shifted word.
    pub fn boss_gate_limit(&self, e: &Enemy, ai: BossAi) -> u8 {
        let row = self.rank.aggression.wrapping_shl(3);
        let off = (row & 0xFF00) | (row as u8).wrapping_add(e.m.b(0xA5)) as u16;
        self.assets
            .enemy
            .t
            .u8(ai.gate_table((e.b.var26 >> 8) as u8), off as u32)
    }

    /// 0x52C6C / 0x55762 / 0x38C14: one RNG draw, signed word BGE.
    /// Call only from the aggressive decision path after its target check.
    /// Tyrog latches +$A4, rather than choosing an action in this call.
    pub fn boss_aggression_gate(&mut self, e: &mut Enemy, ai: BossAi) -> bool {
        let limit = self.boss_gate_limit(e, ai) as u16;
        let pass = (self.rand() & 31) < limit;
        if matches!(ai, BossAi::Tyrog | BossAi::Tyrog2) {
            if pass && e.m.b(0xA4) == 0 {
                e.m.sb(0xA4, 1);
            }
        }
        pass
    }

    /// 0x52C9A / 0x55796 / 0x38C56: one RNG draw after the class has
    /// recalculated +$A5. This returns the table's action id, not a state byte.
    /// Class handlers dispatch it, including Morgue's +$B9 veto after the draw.
    pub fn boss_action_pick(&mut self, e: &Enemy, ai: BossAi) -> u8 {
        let row = ((self.rand() & 31) as u32) << 3;
        self.assets.enemy.t.u8(
            ai.action_table((e.b.var26 >> 8) as u8, e.m.b(0xAB)),
            row + e.m.b(0xA5) as u32,
        )
    }

    /// Common prologue of recover entries 0x51ACA, 0x543AC, 0x36EAE,
    /// 0x658D4 and 0x682A2 (dispatch table 0x2B56C).
    /// The class must continue with its own wake-up counterattack / off-screen
    /// branch before the shared walk setup. Resetting +$A1 here would be too early.
    pub fn boss_recover_entry(&mut self, e: &mut Enemy) {
        e.b.set_state(0x0200_0000);
        self.enemy_anim(&mut e.b, 0);
        self.enemy_set_speed(&mut e.b, 5);
        self.boss_pick_target(e);
        self.lib_face_target(e);
    }

    /// Recover entries of table 0x2B56C: 0x51ACA, 0x543AC, 0x36E7C,
    /// 0x658D4 and 0x682A2. Returns the class-specific continuation.
    pub fn boss_recover(&mut self, e: &mut Enemy, ai: BossAi) -> BossRecover {
        if matches!(ai, BossAi::Tyrog | BossAi::Tyrog2) {
            e.b.palette_override = 0x9F;
            match e.b.var26 >> 8 {
                1 => {
                    e.b.palette_override = 0x9E;
                    e.m.sb(ATTACKING, 0xFE);
                }
                2 => {
                    e.b.palette_override = 0x9D;
                    e.m.sb(ATTACKING, 0xFE);
                }
                _ => {}
            }
        }
        self.boss_recover_entry(e);
        let heavy = e.b.reaction == 4;
        let close = |g: &Game, e: &Enemy, dx: i16| {
            let t = g.lib_target(e);
            (t.xi().wrapping_add(dx).wrapping_sub(e.b.xi()) as u16) <= (dx as u16).wrapping_mul(2)
                && (t.zi().wrapping_add(8).wrapping_sub(e.b.zi()) as u16) <= 0x10
        };
        if matches!(ai, BossAi::Morgue | BossAi::Fessenden) {
            let wake = e.m.b(0xAE) != 0;
            if wake {
                e.m.sb(0xAE, 0);
            }
            // The random call precedes the reaction check, even for a light hit.
            if wake && close(self, e, 0x60) && (self.rand() & 15) < 15 && heavy {
                self.enemy_begin_attack(&mut e.b, 0x4A);
                return BossRecover::Shoulder;
            }
        } else {
            let wake = ai != BossAi::Morgan || e.m.b(0xC0) != 0;
            if ai == BossAi::Morgan && wake {
                e.m.sb(0xC0, 0);
            }
            if wake && close(self, e, 0x50) && heavy {
                return BossRecover::WakeAttack;
            }
        }
        let x = e.b.xi().wrapping_sub(self.cam.s2.cam_x as i16);
        if !(0..0x180).contains(&x) && (self.rand() & 15) < 10 && heavy {
            if matches!(ai, BossAi::Morgue | BossAi::Fessenden) {
                if (self.rand() & 15) < 10 {
                    // `move.w $10(a0),$10(a6)` (0x68384): the word only, the fraction stays.
                    let tz = self.lib_target(e).zi();
                    e.b.z = (e.b.z & 0xFFFF) | ((tz as i32) << 16);
                }
                self.enemy_begin_attack(&mut e.b, 0x4A);
                return BossRecover::Shoulder;
            }
            return BossRecover::OffscreenHop;
        }
        if matches!(ai, BossAi::Morgue | BossAi::Fessenden) {
            self.spawn.g.b502 = 1;
            e.m.sb(ATTACKING, if ai == BossAi::Morgue { 0xFF } else { 0xFE });
            e.m.sw(0xAC, 0x14);
            e.m.sb(0xA1, 0);
            BossRecover::Decide
        } else {
            self.boss_recover_walk(e, ai);
            if ai == BossAi::Morgan {
                BossRecover::Kite
            } else {
                BossRecover::Wait
            }
        }
    }

    /// 0x51B74 / 0x36F4A: normal recover tail, entered only after the
    /// class-specific counterattack branches. Morgan continues at 0x522A8;
    /// Tyrog sets state 0x4C. The other classes have different tails.
    pub fn boss_recover_walk(&mut self, e: &mut Enemy, ai: BossAi) {
        assert!(matches!(
            ai,
            BossAi::Morgan | BossAi::Tyrog | BossAi::Tyrog2
        ));
        if ai == BossAi::Morgan {
            e.m.sb(ATTACKING, 0xFE);
        }
        self.lib_slot6_new(e);
        self.lib_aim_at_target(e); // 0x32A70 reloads target before 0x1032
        e.m.sw(0xAC, 0x14);
        e.m.sb(0xA1, 0);
        if ai != BossAi::Morgan {
            e.b.act = 0x4C;
        }
    }

    /// 0x325E6 and 0x6148: independently timed score and leader hooks.
    /// Morgan and Tyrog's intermediate forms do not call the score hook.
    pub fn boss_score_hook(&mut self, e: &Enemy) {
        self.enemy_award_kill_score(&e.b);
    }

    /// 0x6148: leader flags use +$02 (killer), including the car-mode rule.
    pub fn boss_leader_hook(&mut self, e: &Enemy) {
        self.enemy_boss_leader(e.b.who);
    }

    /// 0x52D7A / 0x38CDA: intermediate form hit hooks, without 0xACEE.
    /// Morgan and Tyrog forms 1/2 slow the game and kill helpers, not the fight.
    pub fn boss_intermediate_hit(&mut self, ai: BossAi) {
        assert!(matches!(
            ai,
            BossAi::Morgan | BossAi::Tyrog | BossAi::Fessenden
        ));
        self.flow.speed = 3;
        if ai != BossAi::Fessenden {
            self.spawn.g.kill_all = 2;
        }
    }

    /// 0xACEE: final fight defeat, including the Episode 8 flags and killer handle.
    /// Kept separate from score/leader hooks, which the dying routine calls later.
    pub fn boss_end_hook(&mut self, h: Handle) {
        self.flow_boss_defeated(h);
    }

    /// 0x38D44 / 0x55822 / 0x63538: the first dying step may fade music,
    /// award score and/or mark the leader independently. Speed writes stay
    /// in the class handler, at their original position between these calls.
    /// The caller specifies which hooks its original entry actually calls.
    pub fn boss_dying_hooks(&mut self, e: &Enemy, fade: bool, score: bool, leader: bool) {
        if fade {
            self.hook_a5ea();
        }
        if score {
            self.boss_score_hook(e);
        }
        if leader {
            self.boss_leader_hook(e);
        }
    }

    /// 0x52DF4 / 0x38D60 / 0x696D2: SUBQ.W followed by BCS, with
    /// the old zero left in RAM on expiry. A loaded N waits N+1 calls.
    pub fn boss_end_wait(&self, e: &mut Enemy) -> bool {
        let t = e.m.w(T80);
        if t == 0 {
            return true;
        }
        e.m.sw(T80, t.wrapping_sub(1));
        false
    }

    /// 0x696AC / 0x696F6: caption hook. Class timeline supplies portrait,
    /// string id and delay from its instructions; no boss-specific script here.
    pub fn boss_caption_hook(&mut self, e: &mut Enemy, portrait: u16, text: i16, delay: u16) {
        self.text_portrait(0x90A29C, portrait, text);
        e.m.sw(T80, delay);
        e.b.sub = e.b.sub.wrapping_add(2);
    }

    /// 0x52E70 / 0x6976A: the walking part of the death timeline uses
    /// SUBQ.W on RAM, so its expired timer becomes 0xFFFF (unlike end_wait).
    pub fn boss_end_walk(&mut self, e: &mut Enemy) -> bool {
        let old = e.m.w(T80);
        e.m.sw(T80, old.wrapping_sub(1));
        if old == 0 {
            return true;
        }
        self.enemy_step(&mut e.b);
        self.enemy_move_dir(&mut e.b);
        false
    }

    /// 0x63754: final-form end marker. The hidden body remains allocated so
    /// G.$4E0 and the ending script can keep reading +$A6.
    pub fn boss_ending_body_hook(&mut self, e: &mut Enemy) {
        if self.w.frame & 3 == 0 {
            self.cam_shake(3);
        }
        e.m.sb(0xA6, 1);
        e.b.visible = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        static A: std::sync::OnceLock<AssetPack> = std::sync::OnceLock::new();
        Game::new(A.get_or_init(|| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dino.zip");
            dino_extract::extract(&path).expect("boss checks require the user's dino.zip")
        }))
    }

    #[test]
    fn phase_wraps_and_draws_only_on_zero() {
        let mut g = game();
        let mut e = Enemy::default();
        e.m.sw(0xA6, 0);
        let rng = g.w.rng;
        assert!(!g.boss_phase_tick(&mut e, BossAi::Morgan));
        assert_eq!(e.m.w(0xA6), 0xFFFF);
        assert_eq!(g.w.rng, rng);
        for ai in [BossAi::Morgan, BossAi::Tyrog, BossAi::Tyrog2] {
            for phase in [0, 1] {
                e.m.sw(0xA6, 1);
                e.m.sb(0xA0, phase);
                let seed = g.w.rng;
                let r = g.rand();
                g.w.rng = seed;
                let (a, d) = ai.phase_tables();
                let expected = g
                    .assets
                    .enemy
                    .t
                    .u16(if phase == 0 { a } else { d }, (r & 0x1E) as u32);
                assert!(g.boss_phase_tick(&mut e, ai));
                assert_eq!(
                    (e.m.b(0xA0), e.m.w(0xA6), g.w.rng),
                    (phase ^ 1, expected, r)
                );
            }
        }
    }

    #[test]
    fn gates_and_actions_use_every_aggression_form_row_and_column() {
        let mut g = game();
        let mut e = Enemy::default();
        for ai in [
            BossAi::Morgan,
            BossAi::Morgue,
            BossAi::Tyrog,
            BossAi::Tyrog2,
            BossAi::Fessenden,
        ] {
            for form in 0..3 {
                e.b.var26 = (form as u16) << 8;
                for aggression in 0..8 {
                    g.rank.aggression = aggression;
                    // Level must not affect these gates.
                    g.w.level = 31 - aggression;
                    for distance in 0..8 {
                        e.m.sb(0xA5, distance);
                        let limit = g
                            .assets
                            .enemy
                            .t
                            .u8(ai.gate_table(form), aggression as u32 * 8 + distance as u32);
                        assert_eq!(g.boss_gate_limit(&e, ai), limit);
                        for seed in 0..128 {
                            g.w.rng = seed;
                            let r = g.rand();
                            g.w.rng = seed;
                            e.m.sb(0xA4, 0);
                            let pass = g.boss_aggression_gate(&mut e, ai);
                            assert_eq!(pass, (r & 31) < limit as u16);
                            assert_eq!(g.w.rng, r, "one draw even when the gate fails");
                            if matches!(ai, BossAi::Tyrog | BossAi::Tyrog2) {
                                assert_eq!(e.m.b(0xA4), u8::from(pass));
                            }
                            for ab in [0, 0x80] {
                                e.m.sb(0xAB, ab);
                                g.w.rng = seed;
                                let expected = g.assets.enemy.t.u8(
                                    ai.action_table(form, ab),
                                    (r as u32 & 31) * 8 + distance as u32,
                                );
                                assert_eq!(g.boss_action_pick(&e, ai), expected);
                                assert_eq!(g.w.rng, r);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn target_ties_go_to_later_present_player_without_randomness() {
        let mut g = game();
        let mut e = Enemy::default();
        for p in &mut g.players {
            p.b.live = 1;
        }
        let rng = g.w.rng;
        g.boss_pick_target(&mut e);
        assert_eq!(e.m.w(TARGET), Game::player_handle(2));
        g.spawn.g.census_prev[2][0] = 1;
        g.boss_pick_target(&mut e);
        assert_eq!(e.m.w(TARGET), Game::player_handle(1));
        g.players[1].b.live = 0;
        g.boss_pick_target(&mut e);
        assert_eq!(e.m.w(TARGET), Game::player_handle(0));
        assert_eq!(g.w.rng, rng);
    }

    #[test]
    fn recover_counterattack_keeps_the_original_draw_order() {
        let mut g = game();
        g.players[0].b.live = 1;
        let mut e = Enemy::default();
        e.b.anim_table = 0x34892; // existing common enemy animation table
        e.b.reaction = 4;
        let seed = g.w.rng;
        assert_eq!(
            g.boss_recover(&mut e, BossAi::Tyrog),
            BossRecover::WakeAttack
        );
        assert_eq!(g.w.rng, seed, "near Tyrog wake attack draws nothing");
        assert_eq!(e.b.state(), 0x0200_0000);
        assert_eq!(e.b.palette_override, 0x9F); // immediate at 0x36E7C
        e.m.sb(0xAE, 1);
        let r = g.rand();
        g.w.rng = seed;
        if r & 15 < 15 {
            assert_eq!(
                g.boss_recover(&mut e, BossAi::Morgue),
                BossRecover::Shoulder
            );
            assert_eq!(g.w.rng, r);
            assert_eq!((e.m.b(0xAE), e.b.atk), (0, 0x4A));
        }
        // Far offscreen: even a light reaction consumes the hop roll, then
        // the normal tail consumes the slot rolls (no wake roll for Tyrog).
        e.b.set_xi(-1000);
        e.b.reaction = 0;
        let seed = g.w.rng;
        g.rand();
        let mut expected = e.clone();
        g.lib_slot6_new(&mut expected);
        let rng = g.w.rng;
        g.w.rng = seed;
        assert_eq!(g.boss_recover(&mut e, BossAi::Tyrog), BossRecover::Wait);
        assert_eq!(g.w.rng, rng);
        assert_eq!((e.b.act, e.m.w(0xAC), e.m.b(0xA1)), (0x4C, 20, 0));
    }

    #[test]
    fn score_hook_uses_the_rom_name_row_and_killer() {
        let mut g = game();
        let mut e = Enemy::default();
        for id in [0x16, 0x19, 0x1B, 0x22, 0x25] {
            e.b.id = id;
            e.b.who = 1;
            for p in &mut g.players {
                p.score = 0;
            }
            let offset = g.assets.enemy.t.u8(0x325F8, id as u32);
            let expected = g.assets.enemy.t.u32(0x10292, offset as u32);
            g.boss_score_hook(&e);
            assert_eq!(g.players[1].score, expected);
            assert_eq!((g.players[0].score, g.players[2].score), (0, 0));
        }
    }

    #[test]
    fn death_hooks_keep_timers_flags_and_hidden_body_distinct() {
        let mut g = game();
        let mut e = Enemy::default();
        e.m.sw(T80, 1);
        assert!(!g.boss_end_wait(&mut e));
        assert!(g.boss_end_wait(&mut e));
        assert_eq!(e.m.w(T80), 0);
        assert!(g.boss_end_walk(&mut e));
        assert_eq!(e.m.w(T80), 0xFFFF);
        e.m.sw(0xA6, 0x1234);
        e.b.visible = 1;
        e.b.mode = 4;
        g.boss_ending_body_hook(&mut e);
        assert_eq!((e.m.w(0xA6), e.b.visible, e.b.mode), (0x0134, 0, 4));
        g.boss_intermediate_hit(BossAi::Morgan);
        assert_eq!((g.flow.speed, g.spawn.g.kill_all, g.flow.sub), (3, 2, 0));
        g.w.stage = 7;
        g.boss_end_hook(0xDC34);
        assert_eq!(
            (
                g.flow.speed,
                g.spawn.g.kill_all,
                g.flow.sub,
                g.flow.b4d7,
                g.flow.b4db
            ),
            (4, 1, 2, 1, 1)
        );
        e.b.who = 1;
        g.boss_leader_hook(&e);
        assert_eq!(
            g.players.iter().map(|p| p.bc0).collect::<Vec<_>>(),
            vec![0, 1, 0]
        );
        g.spawn.g.car_mode = 1;
        g.boss_leader_hook(&e);
        assert!(g.players.iter().all(|p| p.bc0 == 1));
    }
}
