//! Boss services (owner: enemy port, M3-02): the boss reinforcement controller, pool B class 22 (0x7BEF6), made by
//! the boss init 0x7BEDC. Formats and rules: D3c section 1.7 (`plan/game/design/D3c-bosses-1.md`), BS27.
//!
//! The controller keeps its boss in `+$AA`. It runs two lists of the stage: the timed list (every record gets one
//! try: spawned when the alive count is below 4 / 5 / 6 by player count, skipped otherwise) and the threshold groups
//! (taken in order while `G.$502 > 0`: a group waits until the alive count is at or below its thresholds). The
//! spawns use the stage spawner: 0xFDAE (absolute X) and 0xFE6A for Hogg's stage 2 (X relative to the camera).

use super::enemy::{off as enemy_off, Enemy};
use super::prop::Prop;
use super::{slot_of, Game, Handle, Slot};
use dino_assets::boss::BossLists;

/// Controller fields, at the offsets of the original object. The port keeps list indexes where the original keeps
/// ROM pointers (`+$A0` the timed record; `+$A4` the threshold group, record index at `+$A6`); `trace.rs` converts
/// them (`Prop::trace_visit`, class 22), like the wave controller (class 15).
mod off {
    pub const TIMER: usize = 0x80; // word: delay of the group's next record (0x7C036, 0x7C084)
    pub const TIMED: usize = 0xA0; // word: index of the timed record (+$A0 pointer)
    pub const GROUP: usize = 0xA4; // word: threshold group index
    pub const REC: usize = 0xA6; // word: record index in the group
    pub const TIMED_T: usize = 0xA8; // word: timed list timer (negative = end)
    pub const BOSS: usize = 0xAA; // long: the boss object
    pub const DONE: usize = 0xAE; // byte: the threshold list has ended
}

impl Game {
    /// 0x7BEDC: a boss makes its reinforcement controller (class 22, `+$AA` = the boss). `None` when pool B is full
    /// (carry set, nothing made). The boss inits call it (`placeholder.rs` for Vice).
    pub fn boss_spawn_controller(&mut self, boss: Handle) -> Option<Handle> {
        let h = self.spawn_prop(22, 0, None)?;
        if let Some(Slot::Prop(i)) = slot_of(h) {
            self.props.slots[i].m.sl(off::BOSS, 0x00FF_0000 | boss as u32); // move.l a6,$AA(a0)
        }
        Some(h)
    }

    /// A stage boss takes the boss hit path. Until the real boss classes are ported only the placeholder bosses do
    /// (`placeholder.rs`, class 18 variant 0, Vice); each ported boss calls `boss_hit_entry` from its own hit code.
    fn boss_is_stage_boss(b: &super::body::Body) -> bool {
        super::placeholder::spec_of(b.k20 / 4).is_some_and(|s| super::placeholder::boss_of(s, b.var26))
    }

    /// The hit code of a boss variant 0 (Butcher's 0x38CC6 is the model: 0x3262A, then 0x32740 and 0xACEE). The hit pass
    /// (`enemy_hit_by_player`) calls it after the damage is taken. 0x3262A: the damage (HP before the hit minus HP now,
    /// `+$6E` - `+$6C`) goes to the attacker `+$02` (`+$10C`, "best player"), to all three heroes in car mode, and HP
    /// left sets `+$6E` = HP; a negative `+$02` skips all of it. Then at HP 0 or below: the boss is put back inside the
    /// camera (0x32740), and while `+$6E` is set the defeat runs once (0xACEE, then `+$6E` = 0). The music fade 0xA5EA
    /// comes from the first step of the boss's death state (0x38D44), not from here.
    pub(super) fn boss_hit_entry(&mut self, ei: usize) {
        if !Game::boss_is_stage_boss(&self.enemies.slots[ei].b) {
            return;
        }
        let h = self.enemies.handle(ei);
        let mut e = std::mem::take(&mut self.enemies.slots[ei]);
        self.boss_hit(&mut e, h);
        self.enemies.slots[ei] = e;
    }

    /// `boss_hit_entry` on an object the caller holds (an update in progress, see `placeholder.rs`).
    pub(super) fn boss_hit(&mut self, e: &mut Enemy, h: Handle) {
        let (hp, before, who) = (e.b.hp, e.b.hp_prev, e.b.who);
        if (who as i8) >= 0 {
            let dmg = before.wrapping_sub(hp);
            if self.spawn.g.car_mode != 0 {
                for p in self.players.iter_mut() {
                    p.w10c = p.w10c.wrapping_add(dmg);
                }
            } else {
                let p = &mut self.players[who as usize % 3];
                p.w10c = p.w10c.wrapping_add(dmg);
            }
            if hp != 0 {
                e.b.hp_prev = hp;
            }
        }
        // 0x38CCC: cmpi.w #0,$6C ; ble (signed)
        if (hp as i16) > 0 {
            return;
        }
        self.boss_clamp_camera(&mut e.b); // 0x32740
        if e.b.hp_prev != 0 {
            self.flow_boss_defeated(h); // 0xACEE
            e.b.hp_prev = 0; // clr.w $6E
        }
    }

    /// 0x32740: the boss is put back inside the camera, X at least the camera X and at most the camera X + 0x180
    /// (signed word compares, the low bound first).
    fn boss_clamp_camera(&self, b: &mut super::body::Body) {
        let cam = self.cam.s2.cam_x as i16;
        let mut x = b.xi();
        if cam > x {
            x = cam;
        }
        let hi = cam.wrapping_add(0x180);
        if hi < x {
            x = hi;
        }
        b.x = (b.x & 0xFFFF) | ((x as i32) << 16); // the fraction stays
    }

    /// 0x7BF76 .. 0x7C0E2: the controller's mode dispatch. Mode 0 is the init (0x7BF0A), mode 2 the run, mode 4 the
    /// end (0x7C0CE, nothing sets it), mode 6 the free (0x4938).
    pub(super) fn boss_controller(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => self.boss_ctrl_init(p),
            2 => {
                if self.boss_is_defeated(p.m.l(off::BOSS) as Handle) {
                    return self.boss_ctrl_end(p);
                }
                self.boss_timed(p);
                self.boss_threshold(p);
            }
            4 => self.boss_ctrl_end(p),
            _ => self.prop_free(p, i),
        }
    }

    /// 0x7BF0A: the lists of the stage (G.$4D9), timers cleared.
    fn boss_ctrl_init(&mut self, p: &mut Prop) {
        p.m.sw(off::TIMED, 0);
        p.m.sw(off::GROUP, 0);
        p.m.sw(off::REC, 0);
        p.m.sw(off::TIMED_T, 0);
        p.m.sb(off::DONE, 0);
        p.m.sw(off::TIMER, 0);
        p.b.mode = 2;
    }

    /// 0x7C0CE: the boss is dead (the end of its controller): mode 6, G.$7A9 clear, the area end script.
    fn boss_ctrl_end(&mut self, p: &mut Prop) {
        p.b.mode = 6;
        self.cam.s2.go = 0;
        self.script_area_end(); // 0x1D95E
    }

    /// The boss is in its death state: `+$7D` = 0xFF and `+$6C` (HP) = 0 (0x7BF76). A boss that is not a pool A
    /// object reads as alive.
    fn boss_is_defeated(&self, boss: Handle) -> bool {
        match slot_of(boss) {
            Some(Slot::Enemy(i)) => {
                let e = &self.enemies.slots[i];
                e.m.b(enemy_off::ATTACKING) == 0xFF && e.b.hp == 0
            }
            _ => false,
        }
    }

    /// The lists of the stage (the init took them by G.$4D9, which does not change during a stage).
    fn boss_lists(&self, _p: &Prop) -> &'static BossLists {
        &self.assets.boss.stages[self.w.stage as usize]
    }

    /// 0x7BF94: the timed list. Each record is tried once, when its delay has run out: spawned while the alive count
    /// is below the limit of the player count (table 0x7BFEE), skipped otherwise. The next record's delay is loaded
    /// at once, and a zero delay runs in the same frame.
    fn boss_timed(&mut self, p: &mut Prop) {
        if self.spawn.g.car_mode != 0 {
            return;
        }
        let lists = self.boss_lists(p);
        loop {
            let t = p.m.w(off::TIMED_T) as i16;
            if t < 0 {
                return;
            }
            if t != 0 {
                p.m.sw(off::TIMED_T, (t - 1) as u16);
                return;
            }
            // 0x7BFAA: the record at the list position.
            let idx = p.m.w(off::TIMED) as usize;
            let r = lists.timed[idx];
            if (r.trig as i16) < 0 {
                return;
            }
            p.m.sw(off::TIMED_T, r.trig);
            let limit = self.assets.boss.alive_limit[self.w.players_minus_one as usize % 3];
            if (limit as i8) > (self.spawn.g.alive as i8) {
                // 0xFDAE, or 0xFE6A for Hogg (stage 2).
                self.spawn_record(&r, self.w.stage == 2);
            }
            // 0x7BFE0: the next record (the end record stops the loop above).
            let idx = idx + 1;
            p.m.sw(off::TIMED, idx as u16);
            p.m.sw(off::TIMED_T, lists.timed[idx].trig);
        }
    }

    /// 0x7BFF2: the threshold groups. Nothing runs in car mode, after the list has ended, or while `G.$502` is not
    /// above zero (the boss sets it to ask for the next group).
    fn boss_threshold(&mut self, p: &mut Prop) {
        if self.spawn.g.car_mode != 0 || p.m.b(off::DONE) != 0 || (self.spawn.g.b502 as i8) <= 0 {
            return;
        }
        // 0x29ADE, an anti-tamper check of the original, always passes on a genuine ROM and is not ported.
        match p.b.sub {
            0 => self.boss_group_wait(p),
            2 => self.boss_group_spawn(p),
            _ => {} // the original's jump table only has 0 and 2
        }
    }

    /// 0x7C036: wait for the timer, then take the group when the alive count is at or below its threshold for the
    /// player count. The group past the last one holds the final thresholds, and taking them ends the list.
    fn boss_group_wait(&mut self, p: &mut Prop) {
        let t = p.m.w(off::TIMER);
        if t != 0 {
            p.m.sw(off::TIMER, t - 1);
            return;
        }
        let lists = self.boss_lists(p);
        let g = p.m.w(off::GROUP) as usize;
        let thr = match lists.groups.groups.get(g) {
            Some(grp) => grp.thr,
            None => lists.groups.final_thr,
        };
        if self.spawn.g.alive as u16 > thr[self.w.players_minus_one as usize % 3] {
            return;
        }
        // 0x7C076: the final thresholds end the list.
        let Some(grp) = lists.groups.groups.get(g) else {
            self.spawn.g.b502 = 0xFF;
            p.m.sb(off::DONE, 1);
            return;
        };
        p.m.sw(off::REC, 0);
        p.m.sw(off::TIMER, grp.spawns[0].trig);
        p.b.sub = 2;
    }

    /// 0x7C084 .. 0x7C0CC: spawn the group's records, each after its delay. A zero delay spawns in the same frame.
    /// After the last record (the group's end word) the next group waits 3 frames and `G.$502` is cleared.
    fn boss_group_spawn(&mut self, p: &mut Prop) {
        let lists = self.boss_lists(p);
        let g = p.m.w(off::GROUP) as usize;
        let grp = &lists.groups.groups[g];
        loop {
            let t = p.m.w(off::TIMER);
            p.m.sw(off::TIMER, t.wrapping_sub(1));
            if t != 0 {
                return;
            }
            // 0x7C08E: spawn the record (0xFDAE, or 0xFE6A for Hogg), then move on.
            let r = p.m.w(off::REC) as usize;
            self.spawn_record(&grp.spawns[r], self.w.stage == 2);
            let r = r + 1;
            if r >= grp.spawns.len() {
                // 0x7C0B6: the group end: the next group, 3 frames, G.$502 = 0.
                p.m.sw(off::GROUP, (g + 1) as u16);
                p.m.sw(off::REC, 0);
                p.m.sw(off::TIMER, 3);
                self.spawn.g.b502 = 0;
                p.b.sub = 0;
                return;
            }
            p.m.sw(off::REC, r as u16);
            p.m.sw(off::TIMER, grp.spawns[r].trig);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::boss::BossData;
    use dino_assets::enemy::{SpawnRec, Wave, WaveGroup};
    use dino_assets::AssetPack;

    /// A Ferris record (name id 2, pool A) at the left edge of the screen, delay `trig`.
    fn rec(trig: u16) -> SpawnRec {
        SpawnRec { trig, flags: 0, class: 2 * 4, x: 0x100, z: 0x80, var: 0, wc: 0, b5b: 0, b96: 0 }
    }
    fn end() -> SpawnRec {
        SpawnRec { trig: 0x8000, ..rec(0) }
    }
    /// One stage: the timed list `timed` (ends with the end record) and the threshold groups `groups` (1 player
    /// thresholds 99, then 1), with the end word.
    fn game(timed: Vec<SpawnRec>, groups: Vec<WaveGroup>) -> Game {
        let mut a = AssetPack::default();
        let groups = Wave { addr: 0, groups, final_thr: [0, 0, 0], end: 0x8000 };
        a.boss = BossData { stages: vec![BossLists { timed, timed_addr: 0, groups }], alive_limit: [4, 5, 6] };
        Game::new(Box::leak(Box::new(a)))
    }
    fn group(thr: u16, spawns: Vec<SpawnRec>) -> WaveGroup {
        WaveGroup { thr: [thr, thr, thr], spawns }
    }
    /// Starts the controller of boss handle 0 (not a pool A object, so it never counts as defeated).
    fn controller(g: &mut Game) -> Handle {
        g.boss_spawn_controller(0).unwrap()
    }
    fn prop(g: &Game, h: Handle) -> &Prop {
        match slot_of(h) {
            Some(Slot::Prop(i)) => &g.props.slots[i],
            _ => panic!("controller {h:#x} is not a pool B object"),
        }
    }
    fn run(g: &mut Game, frames: usize) {
        for _ in 0..frames {
            g.pool_b_update();
        }
    }
    fn alive_enemies(g: &Game) -> usize {
        g.enemies.slots.iter().filter(|e| e.b.live != 0).count()
    }

    /// The threshold groups wait for the alive count and for `G.$502`, spawn their records after their delays
    /// (zero delay in the same frame), then the next group waits 3 frames and `G.$502` is cleared.
    #[test]
    fn a_group_waits_for_its_threshold_and_the_list_ends_after_the_last() {
        let mut g = game(vec![end()], vec![group(99, vec![rec(0), rec(3)]), group(1, vec![rec(0)])]);
        let h = controller(&mut g);
        run(&mut g, 5);
        assert_eq!(alive_enemies(&g), 0, "no group without G.$502");

        g.spawn.g.b502 = 1;
        run(&mut g, 1); // mode 0 is the init, the group 0 wait passes on this frame
        run(&mut g, 1); // the zero delay record spawns
        assert_eq!(alive_enemies(&g), 1);
        run(&mut g, 4); // the 3 frame delay of the second record
        assert_eq!(alive_enemies(&g), 2);
        assert_eq!(g.spawn.g.b502, 0, "the group end clears G.$502");
        assert_eq!(prop(&g, h).m.w(off::GROUP), 1);

        // Group 1 needs one alive or fewer: 2 alive waits, then 1 alive releases it.
        g.spawn.g.b502 = 1;
        g.spawn.g.alive = 2;
        run(&mut g, 3);
        assert_eq!(alive_enemies(&g), 2);
        assert_eq!(g.spawn.g.b502, 1, "G.$502 stays set while the group waits");
        g.spawn.g.alive = 1;
        run(&mut g, 2);
        assert_eq!(alive_enemies(&g), 3);
        // The final thresholds (0, 0, 0) end the list: G.$502 = 0xFF and no more groups.
        g.spawn.g.alive = 0;
        g.spawn.g.b502 = 1;
        run(&mut g, 4); // the group wait timer (3 frames) runs down first
        assert_eq!(g.spawn.g.b502, 0xFF);
        assert_eq!(prop(&g, h).m.b(off::DONE), 1);
        assert_eq!(alive_enemies(&g), 3);
    }

    /// A timed record is tried once when its delay has run out: below the limit (4 for one player) it spawns,
    /// at the limit it is skipped, and the list goes on either way.
    #[test]
    fn a_timed_record_is_skipped_at_the_alive_limit() {
        let mut g = game(vec![rec(0), rec(2), end()], vec![]);
        let h = controller(&mut g);
        g.spawn.g.alive = 4;
        run(&mut g, 2); // the init frame, then the first record is tried and skipped
        assert_eq!(alive_enemies(&g), 0, "the first record is skipped at 4 alive");
        assert_eq!(prop(&g, h).m.w(off::TIMED), 1);
        g.spawn.g.alive = 3;
        run(&mut g, 1); // the 2 frame delay of the second record runs down (the skip frame already counted one)
        assert_eq!(alive_enemies(&g), 0);
        run(&mut g, 1);
        assert_eq!(alive_enemies(&g), 1, "the second record spawns below the limit");
        assert_eq!(prop(&g, h).m.w(off::TIMED), 2);
        assert!((prop(&g, h).m.w(off::TIMED_T) as i16) < 0, "the end record stops the list");
    }

    /// A boss object in pool A slot 0 (no init, the hit pass only reads the fields) at `hp`, hit from 44 by player 1.
    fn hit_boss(g: &mut Game, hp: u16) -> usize {
        let i = g.enemies.alloc().unwrap();
        let e = &mut g.enemies.slots[i];
        e.b.hp = hp;
        e.b.hp_prev = 44;
        e.b.who = 1;
        e.b.k20 = 18 * 4; // a placeholder boss: class 18 variant 0 (Vice)
        i
    }
    /// A game with the user's ROM (`../../../dino.zip`, the defeat reads the text table of the music fade). None when the
    /// ROM is not there: the tests that need it skip.
    fn rom_game() -> Option<Game> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        let a = *A.get_or_init(|| {
            let path = std::path::Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        });
        Some(Game::new(a?))
    }

    /// 0x3262A: the damage goes to the attacker's `+$10C`, or to all three heroes in car mode. A living boss keeps
    /// `+$6E` = HP. A hit on an object that is not a boss does nothing.
    #[test]
    fn a_boss_hit_credits_the_damage_to_the_attacker() {
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        let i = hit_boss(&mut g, 30);
        g.boss_hit_entry(i);
        assert_eq!(g.players[1].w10c, 14);
        assert_eq!((g.players[0].w10c, g.players[2].w10c), (0, 0));
        assert_eq!(g.enemies.slots[i].b.hp_prev, 30);
        assert_eq!(g.flow.sub, 0, "no defeat while HP is left");

        g.spawn.g.car_mode = 1;
        g.enemies.slots[i].b.hp = 20;
        g.enemies.slots[i].b.hp_prev = 30;
        g.boss_hit_entry(i);
        assert_eq!(g.players.iter().map(|p| p.w10c).collect::<Vec<_>>(), vec![10, 24, 10], "car mode: all three");

        let j = g.enemies.alloc().unwrap();
        g.enemies.slots[j].b.hp = 5;
        g.enemies.slots[j].b.hp_prev = 9;
        g.boss_hit_entry(j);
        assert_eq!(g.players[0].w10c, 10 + 0, "not a boss: no credit");
    }

    /// The defeat (0x3262A, 0x32740, 0xACEE, 0xA5EA): HP 0 with `+$6E` set puts the boss inside the camera, starts the
    /// stage clear (G.$4D1 = 2, slow motion, kill-all, the killer) and clears `+$6E`. A second hit at HP 0 does nothing.
    #[test]
    fn the_defeat_clamps_the_boss_into_the_camera_and_starts_the_stage_clear() {
        let Some(mut g) = rom_game() else { return eprintln!("defeat test skipped: dino.zip is absent") };
        let i = hit_boss(&mut g, 30);
        let h = g.enemies.handle(i);
        g.cam.s2.cam_x = 100;
        g.enemies.slots[i].b.set_xi(600);
        g.enemies.slots[i].b.x |= 0x8000; // a fraction, kept by the clamp
        g.enemies.slots[i].b.hp = 0;
        g.boss_hit_entry(i);
        let b = &g.enemies.slots[i].b;
        assert_eq!(b.xi(), 100 + 0x180, "X above the camera X + 0x180 is put back at the limit");
        assert_eq!(b.x & 0xFFFF, 0x8000, "the fraction stays");
        assert_eq!(b.hp_prev, 0, "+$6E cleared");
        assert_eq!((g.flow.sub, g.flow.speed, g.spawn.g.killer, g.spawn.g.kill_all), (2, 4, h, 1));
        assert_eq!(g.spawn.g.b4d5, 1);
        assert_eq!(g.players[1].w10c, 44, "the last hit (44 to 0) is credited too");

        g.flow.sub = 0;
        g.boss_hit_entry(i);
        assert_eq!(g.flow.sub, 0, "dead already: no second defeat");
    }

    /// 0x32740 low bound: a boss left of the camera is put at the camera X.
    #[test]
    fn the_defeat_puts_a_boss_left_of_the_camera_at_the_camera_x() {
        let Some(mut g) = rom_game() else { return eprintln!("defeat test skipped: dino.zip is absent") };
        let i = hit_boss(&mut g, 0);
        g.enemies.slots[i].b.hp_prev = 3;
        g.cam.s2.cam_x = 300;
        g.enemies.slots[i].b.set_xi(20);
        g.boss_hit_entry(i);
        assert_eq!(g.enemies.slots[i].b.xi(), 300);
    }
}
