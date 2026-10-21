//! `--ally 2|3|2,3`: the computer plays player 2 and / or player 3 next to the human player 1.
//!
//! An ally is a front end like `autoplay.rs` and `planner.rs`: the game logic is not changed, the ally only presses the buttons of its
//! own slot (`input::Frame`), like a player at the cabinet. The trace of a run without `--ally` is the same as before.
//!
//! # Joining (only through inputs)
//! * Nothing happens before player 1 has started a game (`flow.in_game`): the ally waits in the attract mode.
//! * Then, in the select screen or in the level (the original lets a player join at any time with a credit), the ally puts a coin into
//!   its own coin slot (a 4 frame pulse; the system task converts it, `system.rs`) until it has the credits of a start (1, or 2 with
//!   "2 coins start"; none in free play) and pushes its start button (a 3 frame pulse, repeated until the join task takes it).
//!   With the common credit pool (the chute modes 1 and 2 of the settings) the credits are shared, so the ally first notes the credits
//!   that are there (player 1's) and starts only when its own coin has added what a start costs on top of them: it never spends the
//!   credit of player 1. With the separate chutes of the factory settings each player has his own credits, there is nothing to share.
//! * In the select screen it moves the cursor with right and picks with attack. Its hero is the one of `--ally-chars` (in the order of
//!   the allies) when that one is free (when another cursor stands on it, the ally steps over the held places for `SELECT_WAIT` ticks, so
//!   two allies that want each other's heroes swap), else the first hero (Jack, Mustapha, Hannah, Mess) that no other player has. An ally with a named
//!   hero joins at once; one without joins after player 1 has picked, so its cursor never stands on a hero player 1 wants (a joined player's
//!   cursor holds its position, as in the original). In the level it does the same on the quick hero strip (right moves to the next hero
//!   that is free, attack picks).
//! * After its death it continues by itself while the countdown runs (a coin if it has no credit, then start), at most
//!   `--ally-continues N` times (default: without limit).
//!
//! # Playing
//! The smart bot of `planner.rs` plays the slot: a beam search over copies of the game with the macro actions of `K` frames, the same
//! `Policy` and the same score, for its own player index. Two things differ from the bot of player 1 (`planner::Team`): the ally stays
//! near player 1 (the x of the walk counts for nothing; being inside `planner::FOLLOW` pixels of player 1's hero does), and it takes no
//! food when player 1 needs it (player 1's HP under `planner::NEEDY_HP` and not above the ally's: every HP it would gain costs it points).
//!
//! # Speed
//! A decision is a few hundred simulated plans, far more than a video frame has time for. The decision for the next macro is therefore
//! made in the background while the current macro is played: at the start of a macro the game is copied, the copy plays the macro (with
//! the input player 1 holds now), and the planner decides on the state at its end; the macro is ready when the next one has to start
//! (`Ally::play`). The result does not depend on the speed of the machine or the number of threads (the thread is always waited for at the
//! end of the macro), so a run is the same every time. `DINO_ALLY_H`, `DINO_ALLY_D`, `DINO_ALLY_B` and `DINO_ALLY_THREADS` change the
//! planner settings (the defaults are smaller than those of the bot of player 1: `Ally::params`).

use crate::game::Game;
use crate::input::{Frame, ATTACK, COIN3, RIGHT, START3};
use crate::planner::{self, Macro, Params, Planner, K};
use std::thread::JoinHandle;

/// Hero id to the cursor position of the select screen (both ways: the table is its own inverse).
const POS: [u8; 4] = [0, 2, 1, 3];
const HEROES: [&str; 4] = ["Jack", "Mustapha", "Hannah", "Mess"];

/// Frames of a coin pulse and the frames to the next one (the debounce of the coin chute, `system.rs`, and the conversion into credits).
const COIN_HIGH: u32 = 4;
const COIN_GAP: u32 = 28;
/// Frames of a start pulse and the frames to the next one.
const START_HIGH: u32 = 3;
const START_GAP: u32 = 10;
/// Ticks an ally waits in the select screen for a named hero that another cursor stands on.
const SELECT_WAIT: u32 = 150;
/// Frames without control after which the plan in the background is dropped (a hit or a scene is short, a long lock is a new situation).
const AWAY_MAX: u32 = 3 * K as u32;

/// What the ally presses in one frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Out {
    pub pad: u8,
    pub start: bool,
    pub coin: bool,
}

/// What the background job gives back: the planner, the macro for the next `K` frames and the time the plan took.
type Job = JoinHandle<(Planner, Macro, u128)>;

/// Numbers of the speed report.
#[derive(Clone, Copy, Debug, Default)]
pub struct Timing {
    /// Decisions, the sum and the longest time of a plan (in the background thread), the sum and the longest wait of the game for a plan
    /// that was not ready, the sum and the longest time of the copy of the game at the start of a macro (all in nanoseconds).
    pub decisions: u64,
    pub plan_sum: u128,
    pub plan_max: u128,
    pub wait_sum: u128,
    pub wait_max: u128,
    pub clone_sum: u128,
    pub clone_max: u128,
}

pub struct Ally {
    /// The player slot (1 or 2).
    pub slot: usize,
    /// The hero of `--ally-chars` for this ally.
    hero: Option<u8>,
    /// Continues the ally may use, and the ones it used.
    continues: Option<u32>,
    used: u32,
    t: u32,
    coin_at: Option<u32>,
    start_at: Option<u32>,
    /// The credits of the common pool the ally waits for (see the header).
    target: Option<u8>,
    /// The cursor position of the select screen the ally goes to, and the ticks it waited for a named hero.
    pick: Option<u8>,
    waited: u32,
    prev_join: u8,
    planner: Option<Planner>,
    pending: Option<Job>,
    cur: Option<(Macro, usize)>,
    away: u32,
    joined_at: Option<u32>,
    pub timing: Timing,
}

impl Ally {
    pub fn new(slot: usize, hero: Option<u8>, continues: Option<u32>, params: Params) -> Ally {
        let mut planner = Planner::new(params);
        planner.me = slot;
        planner.team = true;
        Ally {
            slot,
            hero,
            continues,
            used: 0,
            t: 0,
            coin_at: None,
            start_at: None,
            target: None,
            pick: None,
            waited: 0,
            prev_join: 0,
            planner: Some(planner),
            pending: None,
            cur: None,
            away: 0,
            joined_at: None,
            timing: Timing::default(),
        }
    }

    /// The planner settings of an ally: smaller than those of the bot of player 1 (a horizon of 64 frames, 2 macro steps, a beam of 3), on
    /// at most 4 threads (the game and the picture need the rest).
    pub fn params() -> Params {
        let var = |n: &str, d: usize| std::env::var(n).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
        let mut p = Params::from_env(0);
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        p.horizon = var("DINO_ALLY_H", 64);
        p.depth = var("DINO_ALLY_D", 2);
        p.beam = var("DINO_ALLY_B", 3);
        p.threads = var("DINO_ALLY_THREADS", (cores / 2).clamp(1, 4));
        p
    }

    /// The ally forgets what it was doing (the game is over or has not started).
    fn reset(&mut self) {
        self.coin_at = None;
        self.start_at = None;
        self.target = None;
        self.pick = None;
        self.waited = 0;
        self.cur = None;
        self.drop_plan();
        self.joined_at = None;
    }

    /// Drops the plan in the background (waits for the thread; the planner comes back).
    fn drop_plan(&mut self) {
        if let Some(job) = self.pending.take() {
            let (planner, _, _) = job.join().expect("the ally planner failed");
            self.planner = Some(planner);
        }
    }

    /// The press of one frame. `p1` is what player 1 holds.
    fn act(&mut self, g: &Game, p1: u8) -> Out {
        self.t += 1;
        let i = self.slot;
        // nothing before player 1 has started a game; and the two player layout (chute mode 2) has no third player
        if g.flow.in_game == 0 || (i == 2 && g.flow.b4cf != 0) {
            self.reset();
            self.prev_join = 0;
            return Out::default();
        }
        let join = g.flow.join[i].state;
        let live = g.players[i].b.live != 0;
        let mode = g.flow.mode;
        // a continue is used when the countdown ends with a start: the player is on the hero strip again (state 4)
        if self.prev_join == 8 && join == 4 {
            self.used += 1;
            eprintln!("ally: player {} continues (number {}, tick {})", i + 1, self.used, self.t);
        }
        self.prev_join = join;
        if join != 6 {
            self.joined_at = None;
        }
        // (in the select screen the state is 6 as soon as the hero is picked, the level shows the hero it got)
        if live && join == 6 && mode == 0xC && self.joined_at.is_none() {
            self.joined_at = Some(self.t);
            eprintln!("ally: player {} is in the game as {} (tick {})", i + 1, HEROES[(g.players[i].b.id & 3) as usize], self.t);
        }
        if !matches!(join, 0 | 2 | 8) {
            self.target = None;
        }
        let in_select = matches!(mode, 0 | 4);
        let mut out = Out::default();
        if join == 8 && !live {
            if self.continues.is_none_or(|n| self.used < n) {
                self.buy(g, 1, &mut out);
            }
        } else if in_select && join == 4 && g.flow.front.select.rec[i].state == 2 {
            out.pad = self.select(g);
        } else if join == 4 && !live && mode == 0xC {
            out.pad = self.strip(g);
        } else if matches!(join, 0 | 2) && !live && matches!(mode, 0 | 4 | 0xC) {
            // an ally with no named hero joins the select screen after player 1 has picked (its cursor would stand on a hero player 1 may want)
            let waits = in_select && self.hero.is_none() && g.flow.front.select.rec[0].state != 4;
            if !waits {
                let s = g.flow.front.sys;
                self.buy(g, if s.two_coin_start { 2 } else { 1 }, &mut out);
            }
        } else if live {
            out.pad = self.play(g, p1);
        }
        out
    }

    /// The coin and the start button for a start or a continue that costs `need` credits (0 in free play).
    fn buy(&mut self, g: &Game, need: u8, out: &mut Out) {
        let s = g.flow.front.sys;
        let credits = if s.common { s.credits } else { s.slots[self.slot].credits };
        // the common pool: the credits there now are not the ally's
        let target = if s.free_play { 0 } else if s.common { *self.target.get_or_insert((credits + need).min(9)) } else { need };
        if credits >= target {
            out.start = pulse(&mut self.start_at, self.t, START_HIGH, START_GAP);
        } else {
            out.coin = pulse(&mut self.coin_at, self.t, COIN_HIGH, COIN_GAP);
        }
    }

    /// The hero select screen: the cursor to the hero, then attack.
    fn select(&mut self, g: &Game) -> u8 {
        let i = self.slot;
        let sel = &g.flow.front.select;
        let rec = sel.rec[i];
        if rec.pos > 3 {
            return 0;
        }
        let taken = |pos: u8| (0..3).any(|k| k != i && sel.rec[k].state != 0 && sel.rec[k].pos == pos);
        let picked = |pos: u8| (0..3).any(|k| k != i && sel.rec[k].state == 4 && sel.rec[k].pos == pos);
        let keep = self.pick.filter(|&p| !taken(p));
        let pos = match keep {
            Some(p) => p,
            None => {
                let named = self.hero.map(|h| POS[(h & 3) as usize]);
                // a named hero whose position another cursor stands on (not picked yet): keep moving right, which steps over the held
                // positions, so a cursor that waits for ours can take our place (two allies that want each other's heroes swap this way);
                // after `SELECT_WAIT` ticks it is not going to be free
                if named.is_some_and(|p| taken(p) && !picked(p)) {
                    self.waited += 1;
                    if self.waited < SELECT_WAIT {
                        let tap = self.t % 4 < 2;
                        return if tap { RIGHT } else { 0 };
                    }
                }
                // the named hero when it is free, else the first free one (Jack, Mustapha, Hannah, Mess)
                let p = named.filter(|&p| !taken(p)).or_else(|| (0..4).map(|id| POS[id]).find(|&p| !taken(p))).unwrap_or(rec.pos);
                self.pick = Some(p);
                p
            }
        };
        let tap = self.t % 4 < 2;
        if rec.pos != pos {
            if tap { RIGHT } else { 0 }
        } else if tap {
            ATTACK
        } else {
            0
        }
    }

    /// The quick hero strip of a join in the level: right to the next free hero, attack to pick.
    fn strip(&mut self, g: &Game) -> u8 {
        let i = self.slot;
        let id = g.players[i].b.id;
        let held = |h: u8| (0..3).any(|k| k != i && g.flow.started & (1 << k) != 0 && matches!(g.flow.join[k].state, 4 | 6) && g.players[k].b.id == h);
        let want = self.hero.filter(|&h| !held(h & 3)).map(|h| h & 3).or_else(|| (0..4).find(|&h| !held(h))).unwrap_or(id);
        let tap = self.t % 4 < 2;
        if id != want {
            if tap { RIGHT } else { 0 }
        } else if tap {
            ATTACK
        } else {
            0
        }
    }

    /// The level: the planner decides in the background (see the header).
    fn play(&mut self, g: &Game, p1: u8) -> u8 {
        let me = self.slot;
        if !planner::controllable(g, me) {
            // a hit, a scene, a lock: the macro waits; a long stop drops the plan
            self.away += 1;
            if self.away > AWAY_MAX {
                self.cur = None;
                self.drop_plan();
            }
            return 0;
        }
        self.away = 0;
        if self.cur.is_none_or(|(_, k)| k >= K) {
            let t = std::time::Instant::now();
            let (planner, m) = match self.pending.take() {
                Some(job) => {
                    let (planner, m, ns) = job.join().expect("the ally planner failed");
                    let wait = t.elapsed().as_nanos();
                    self.timing.decisions += 1;
                    self.timing.plan_sum += ns;
                    self.timing.plan_max = self.timing.plan_max.max(ns);
                    self.timing.wait_sum += wait;
                    self.timing.wait_max = self.timing.wait_max.max(wait);
                    (planner, m)
                }
                None => (self.planner.take().expect("the ally planner is away"), Macro::idle()),
            };
            self.cur = Some((m, 0));
            let mut planner = planner;
            planner.others = [p1, 0, 0];
            let t = std::time::Instant::now();
            let copy = g.clone();
            let ns = t.elapsed().as_nanos();
            self.timing.clone_sum += ns;
            self.timing.clone_max = self.timing.clone_max.max(ns);
            self.pending = Some(std::thread::spawn(move || {
                let t = std::time::Instant::now();
                let mut c = copy;
                for k in 0..K {
                    planner::step(&mut c, planner.me, m.input(k), planner.others);
                }
                let next = if planner::controllable(&c, planner.me) { planner.decide(&c) } else { Macro::idle() };
                (planner, next, t.elapsed().as_nanos())
            }));
        }
        let (m, k) = self.cur.expect("a macro is on");
        self.cur = Some((m, k + 1));
        m.input(k)
    }
}

/// A pulse of `high` frames every `gap` frames (`at`: the frame the last one began). Returns the line level of this frame.
fn pulse(at: &mut Option<u32>, t: u32, high: u32, gap: u32) -> bool {
    match *at {
        Some(a) if t - a < high => true,
        Some(a) if t - a < gap => false,
        _ => {
            *at = Some(t);
            true
        }
    }
}

/// The allies of a run (none without `--ally`).
#[derive(Default)]
pub struct Allies {
    list: Vec<Ally>,
    /// The `--ally-chars` heroes (by the order of the allies of the command line, kept for the ones added later), the continue limit.
    chars: Vec<u8>,
    continues: Option<u32>,
}

impl Allies {
    /// `slots`: the players (index 1 or 2) the computer plays, `chars`: their heroes in the same order.
    pub fn new(slots: &[usize], chars: &[u8], continues: Option<u32>) -> Allies {
        Allies::with_params(slots, chars, continues, Ally::params())
    }

    pub fn with_params(slots: &[usize], chars: &[u8], continues: Option<u32>, params: Params) -> Allies {
        let list = slots.iter().enumerate().map(|(k, &s)| Ally::new(s, chars.get(k).copied(), continues, params)).collect();
        Allies { list, chars: chars.to_vec(), continues }
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[cfg(test)]
    pub fn has(&self, slot: usize) -> bool {
        self.list.iter().any(|a| a.slot == slot)
    }

    /// The window key: the computer plays the slot, or the slot is the human's again.
    pub fn toggle(&mut self, slot: usize) {
        if let Some(k) = self.list.iter().position(|a| a.slot == slot) {
            self.list.remove(k);
            eprintln!("ally: player {} is yours again", slot + 1);
        } else {
            let hero = self.chars.get(self.list.len()).copied();
            self.list.push(Ally::new(slot, hero, self.continues, Ally::params()));
            eprintln!("ally: the computer plays player {}", slot + 1);
        }
    }

    /// Puts the allies' presses into the frame `f` of the tick (after the keys, the pads and the input script: an ally slot is the
    /// ally's alone, it ignores its pad and its keys).
    pub fn apply(&mut self, g: &Game, f: &mut Frame) {
        let p1 = f.players[0];
        for a in &mut self.list {
            let i = a.slot;
            f.players[i] = 0;
            f.system &= !(1 << i);
            f.coins &= !(1 << i);
            let out = a.act(g, p1);
            f.players[i] = out.pad;
            if out.start {
                f.system |= 1 << i;
            }
            if out.coin {
                f.coins |= 1 << i;
            }
            // the third slot's start and coin are also bits of its port (`input.rs`)
            if i == 2 {
                if out.coin {
                    f.players[2] |= COIN3;
                }
                if out.start {
                    f.players[2] |= START3;
                }
            }
        }
    }

    /// The lines at the end of a run: the time the plans took.
    pub fn summary(&mut self) {
        for a in &mut self.list {
            a.drop_plan();
            let t = a.timing;
            if t.decisions == 0 {
                continue;
            }
            let ms = |ns: u128| ns as f64 / 1e6;
            eprintln!(
                "ally: player {} summary: {} decisions | plan in the background {:.2} ms average, {:.2} ms longest | wait of the game {:.3} ms average, {:.2} ms longest | copy of the game {:.3} ms average, {:.3} ms longest",
                a.slot + 1,
                t.decisions,
                ms(t.plan_sum) / t.decisions as f64,
                ms(t.plan_max),
                ms(t.wait_sum) / t.decisions as f64,
                ms(t.wait_max),
                ms(t.clone_sum) / t.decisions as f64,
                ms(t.clone_max),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Script;
    use dino_assets::AssetPack;

    fn rom() -> Option<&'static AssetPack> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../dino.zip");
            let path = if path.exists() { path } else { std::path::Path::new("../../../dino.zip") };
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    /// Player 1: a coin, start, the first hero (Jack: the cursor starts there), then right in Episode 1 (power-on frames, like `boot_jack.txt`).
    const P1_WALKS: &str = "600-607:1C, 700-705:1S, 1100-1105:1A, 1900-9000:1R";
    /// Player 1 stands still after the pick.
    const P1_STANDS: &str = "600-607:1C, 700-705:1S, 1100-1105:1A";

    /// A run from power on: the script of player 1 and the allies, the frame as `tick_game` of main.rs plays it.
    struct Run {
        g: Game,
        allies: Allies,
        script: Script,
        tick: u32,
        /// The HP the enemies lost to each player (their attacker `+$68` when the HP went down), and the kills.
        dealt: [i64; 3],
        kills: [u32; 3],
    }

    impl Run {
        fn new(a: &'static AssetPack, config: crate::game::flow::Config, script: &str, allies: Allies) -> Run {
            let mut g = Game::new(a);
            g.flow_apply_config(config);
            g.flow_boot();
            Run { g, allies, script: Script::parse(script), tick: 0, dealt: [0; 3], kills: [0; 3] }
        }

        fn step(&mut self) {
            let mut f = Frame::default();
            let s = self.script.at(self.tick);
            for p in 0..3 {
                f.players[p] |= s.players[p];
            }
            f.system |= s.system;
            f.coins |= s.coins;
            self.allies.apply(&self.g, &mut f);
            self.g.w.input = f.players;
            self.g.w.system = f.system | 0x08;
            self.g.w.coins = f.coins;
            let before: Vec<(u16, u8)> = self.g.enemies.slots.iter().map(|e| (e.b.hp, e.b.live)).collect();
            self.g.tick();
            for (e, (hp, live)) in self.g.enemies.slots.iter().zip(before) {
                if live == 0 || e.b.hp >= hp {
                    continue;
                }
                for p in 0..3 {
                    if e.b.attacker == Game::player_handle(p) {
                        self.dealt[p] += (hp - e.b.hp) as i64;
                        if e.b.hp == 0 {
                            self.kills[p] += 1;
                        }
                    }
                }
            }
            self.tick += 1;
        }

        fn run(&mut self, frames: u32) {
            for _ in 0..frames {
                self.step();
            }
        }

        /// Runs until `done` is true, at most `max` ticks; returns the tick it became true.
        fn until(&mut self, max: u32, done: impl Fn(&Game) -> bool) -> Option<u32> {
            while self.tick < max {
                self.step();
                if done(&self.g) {
                    return Some(self.tick);
                }
            }
            None
        }
    }

    fn small() -> Params {
        Params { horizon: 32, depth: 2, beam: 2, threads: 2, ..Params::from_env(0) }
    }

    /// The player is in the game (the select screen is over and the level runs).
    fn playing(g: &Game, i: usize) -> bool {
        g.players[i].b.live != 0 && g.flow.join[i].state == 6 && g.flow.mode == 0xC
    }

    #[test]
    fn the_ally_joins_picks_a_hero_and_hurts_enemies() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut r = Run::new(a, Default::default(), P1_WALKS, Allies::with_params(&[1], &[], None, small()));
        let joined = r.until(3000, |g| playing(g, 1)).expect("player 2 joins");
        eprintln!("joined at tick {joined}, hero {}", r.g.players[1].b.id);
        // player 1 picks at 1100: the ally (no named hero) is in the game within 3 seconds after that
        assert!(joined < 1100 + 5 * 60, "joined at {joined}");
        assert_ne!(r.g.players[1].b.id, r.g.players[0].b.id, "the ally takes another hero than player 1");
        r.run(4000);
        eprintln!("dealt {:?} kills {:?} ticks {}", r.dealt, r.kills, r.tick);
        assert!(r.dealt[1] >= 60 && r.kills[1] >= 1, "the ally hurt no one: {:?} {:?}", r.dealt, r.kills);
    }

    #[test]
    fn two_allies_both_join_with_the_heroes_of_ally_chars() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        // player 2 takes Mess (3), player 3 Hannah (2): named heroes need not wait for player 1's pick
        let mut r = Run::new(a, Default::default(), P1_STANDS, Allies::with_params(&[1, 2], &[3, 2], None, small()));
        // in the select screen both have joined within 4 seconds of player 1's start (tick 700) ...
        let joined = r.until(3000, |g| g.flow.join[1].state == 6 && g.flow.join[2].state == 6).expect("both join");
        eprintln!("both joined at tick {joined}");
        assert!(joined < 700 + 4 * 60, "joined at {joined}");
        // ... and the level starts with the heroes they named
        r.until(3000, |g| playing(g, 1) && playing(g, 2)).expect("both are in the level");
        assert_eq!((r.g.players[0].b.id, r.g.players[1].b.id, r.g.players[2].b.id), (0, 3, 2));
        // the heroes are in the game, in the level
        r.run(900);
        assert!(r.g.flow.in_game != 0 && (0..3).all(|i| r.g.players[i].b.live != 0), "mode {:x}", r.g.flow.mode);
    }

    /// Player 2 wants the hero the cursor of player 3 stands on and the other way round (the cursors start on the first free positions):
    /// they step over each other and swap.
    #[test]
    fn allies_that_want_each_others_places_swap() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut r = Run::new(a, Default::default(), P1_STANDS, Allies::with_params(&[1, 2], &[1, 2], None, small()));
        let t = r.until(3000, |g| playing(g, 1) && playing(g, 2)).expect("both are in the level");
        assert_eq!((r.g.players[0].b.id, r.g.players[1].b.id, r.g.players[2].b.id), (0, 1, 2), "level at {t}");
    }

    #[test]
    fn the_default_heroes_are_the_first_ones_player_1_did_not_pick() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        // player 1 picks Mess (cursor: Jack, left wraps to Mess); the allies take the first two free heroes, Jack and Mustapha
        let mut r = Run::new(a, Default::default(), "600-607:1C, 700-705:1S, 800-803:1L, 1100-1105:1A", Allies::with_params(&[1, 2], &[], None, small()));
        r.until(3000, |g| playing(g, 1) && playing(g, 2)).expect("both join");
        assert_eq!((r.g.players[0].b.id, r.g.players[1].b.id, r.g.players[2].b.id), (3, 0, 1));
    }

    #[test]
    fn an_ally_joins_in_the_level_through_the_hero_strip() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut r = Run::new(a, Default::default(), P1_WALKS, Allies::default());
        r.until(3000, |g| g.flow.mode == 0xC && g.w.scripted == 0).expect("the level starts");
        r.run(120);
        assert_eq!(r.g.flow.join[1].state, 0, "player 2 is absent");
        r.allies.toggle(1);
        let t0 = r.tick;
        let joined = r.until(t0 + 1200, |g| playing(g, 1)).expect("player 2 joins in the level");
        eprintln!("joined in the level {} ticks after the key", joined - t0);
        assert_eq!(r.g.flow.front.sys.slots[1].credits, 0, "its own coin paid");
        assert_ne!(r.g.players[1].b.id, 0);
        // the key again: the slot is the human's
        r.allies.toggle(1);
        assert!(!r.allies.has(1));
    }

    #[test]
    fn the_result_does_not_depend_on_the_number_of_threads() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let digest = |threads: usize| {
            let params = Params { threads, ..small() };
            let mut r = Run::new(a, Default::default(), P1_WALKS, Allies::with_params(&[1], &[], None, params));
            r.run(3200);
            let g = &r.g;
            (g.w.rng, g.players[1].b.xi(), g.players[1].b.hp, g.players[1].score, g.cam.s2.cam_x, r.dealt, r.kills)
        };
        assert_eq!(digest(1), digest(3));
    }

    #[test]
    fn without_ally_the_frame_is_the_players_alone() {
        let mut none = Allies::default();
        assert!(none.is_empty());
        let g = Game::new(match rom() {
            Some(a) => a,
            None => return eprintln!("test skipped: dino.zip is absent"),
        });
        let f0 = Frame { players: [0x13, 0x21, 0xC5], system: 0x87, coins: 5 };
        let mut f = f0;
        none.apply(&g, &mut f);
        assert_eq!((f.players, f.system, f.coins), (f0.players, f0.system, f0.coins));
    }

    /// A run without `--ally` goes through the same code as before: the same frames and the same game as a run that never calls `apply`.
    #[test]
    fn a_run_without_ally_is_the_same_game() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut with = Run::new(a, Default::default(), P1_WALKS, Allies::default());
        let mut plain = Run::new(a, Default::default(), P1_WALKS, Allies::default());
        for t in 0..2600 {
            with.step();
            // the game of main.rs without `Allies::apply`
            let s = plain.script.at(t);
            plain.g.w.input = s.players;
            plain.g.w.system = s.system | 0x08;
            plain.g.w.coins = s.coins;
            plain.g.tick();
            plain.tick += 1;
            let (x, y) = (&with.g, &plain.g);
            assert_eq!((x.w.rng, x.flow.mode, x.cam.s2.cam_x, x.players[0].b.xi(), x.players[0].b.hp, x.players[1].b.live), (y.w.rng, y.flow.mode, y.cam.s2.cam_x, y.players[0].b.xi(), y.players[0].b.hp, y.players[1].b.live), "tick {t}");
        }
        assert_eq!(with.g.flow.mode, 0xC);
    }

    #[test]
    fn an_ally_slot_ignores_its_keys_and_pad_but_player_1_does_not() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let g = Game::new(a);
        let mut allies = Allies::with_params(&[1, 2], &[], None, small());
        // attract mode: the allies press nothing; what the human presses on their slots is dropped, player 1's stays
        let mut f = Frame { players: [0x13, 0x21, 0xC5], system: 0x07, coins: 7 };
        allies.apply(&g, &mut f);
        assert_eq!((f.players, f.system, f.coins), ([0x13, 0, 0], 0x01, 1));
    }

    /// The ally dies with no life left (its HP and lives are set to 0 by the test: the game does the rest) and continues by itself.
    fn kill_and_continue(config: crate::game::flow::Config, limit: Option<u32>) -> Option<(Run, u32)> {
        let a = rom()?;
        let mut r = Run::new(a, config, P1_STANDS, Allies::with_params(&[1], &[1], limit, small()));
        r.until(3000, |g| playing(g, 1)).expect("the ally joins");
        r.run(300);
        r.g.players[1].lives = 0;
        r.g.players[1].b.hp = 0;
        let died = r.until(r.tick + 1200, |g| g.flow.join[1].state == 8).expect("the ally dies and the continue countdown runs");
        let back = r.until(died + 1200, |g| playing(g, 1) && g.players[1].b.hp > 50).map(|t| t - died);
        eprintln!("died at {died}, back after {back:?} ticks, credits left {}", r.g.flow.front.sys.slots[1].credits);
        back.map(|b| (r, b))
    }

    #[test]
    fn a_dead_ally_continues_by_itself() {
        if rom().is_none() {
            return eprintln!("test skipped: dino.zip is absent");
        }
        let (r, back) = kill_and_continue(Default::default(), None).expect("the ally continues");
        assert!(back < 12 * 60, "back after {back}");
        assert_eq!(r.allies.list[0].used, 1);
        assert_eq!(r.g.flow.front.sys.slots[1].credits, 0, "the continue took its own credit");
    }

    #[test]
    fn ally_continues_zero_leaves_the_ally_dead() {
        if rom().is_none() {
            return eprintln!("test skipped: dino.zip is absent");
        }
        assert!(kill_and_continue(Default::default(), Some(0)).is_none(), "no continue was allowed");
    }

    /// The common credit pool (chute mode 1): the credit player 1 has in the pool is not spent by the ally.
    #[test]
    fn the_common_pool_credit_of_player_1_stays() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let config = crate::game::flow::Config { chute_mode: 1, ..Default::default() };
        // player 1 puts in two coins and starts: one credit is left in the pool
        let script = "600-607:1C, 640-647:1C, 700-705:1S, 1100-1105:1A";
        let mut r = Run::new(a, config, script, Allies::with_params(&[1], &[2], None, small()));
        r.until(3000, |g| playing(g, 1)).expect("the ally joins");
        r.run(30);
        let s = r.g.flow.front.sys;
        assert!(s.common);
        assert_eq!((s.credits, s.common_coins), (1, 0), "player 1's spare credit is still there");
    }

    /// Free play: no coin at all.
    #[test]
    fn free_play_needs_no_coin() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let config = crate::game::flow::Config { free_play: true, ..Default::default() };
        let mut r = Run::new(a, config, "700-705:1S, 1100-1105:1A", Allies::with_params(&[1, 2], &[1, 2], None, small()));
        r.until(3000, |g| playing(g, 1) && playing(g, 2)).expect("both join");
        assert!(r.g.flow.front.sys.slots.iter().all(|c| c.credits == 0 && c.coins == 0));
    }
}
