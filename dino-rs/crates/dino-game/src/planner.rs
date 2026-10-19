//! `--autoplay smart`: a bot that plays the game like a player, with a planner (a beam search over simulated futures).
//!
//! The bot is a front end like `autoplay.rs`: the game logic is not changed, and the bot only presses inputs (`input::Frame`). No god
//! mode, no kill all, no clock top up, no debug keys; the one thing it has that a player has not is a copy of the game: `Game` is `Clone`,
//! so every plan can be played ahead on a copy of the real game with the same random number state (the game is deterministic).
//!
//! # What it does
//! * Before the game: coin, start, the hero select (the cursor goes to the hero of `--chars`, then attack picks it), and after a death
//!   the continue (a coin and start, at most `--smart-continues N` times, default 3; with 0 the run ends at the game over).
//! * In a scripted scene or a screen with no control (the stage intro, the tally, the story cards) it holds attack, which skips the
//!   waits of the scripted scenes (`skip_wait_rounds`) like a player who mashes a button.
//! * In the game it decides every `K` = 8 frames. The candidates are macro actions of `K` frames: the 8 directions and none, each with
//!   nothing, attack, a two hit combo, jump, attack + jump (the special), jump attack; and a run (double tap) for the walks. Each candidate
//!   is played on a clone of the game for `K` frames and then, for the rest of the horizon `H`, with a simple default policy
//!   (`Policy`: walk to the nearest enemy and hit it, pick up food when hurt and a weapon when unarmed, walk right otherwise, sweep up and
//!   down when a wall or a ledge stops the walk). The end state is scored (`value`). A beam search keeps the best `B` partial plans of
//!   `D` macro steps; the first step of the best plan is played, then the game is planned again from the real state. The rest of the
//!   best plan is kept as a candidate of the next decision (the plan is valid while nothing surprising happens).
//! * The candidates of a level are simulated on all cores (`std::thread::scope`); the results are collected in the order of the
//!   candidates, so a run is the same every time (no random numbers of its own, no clock).
//! * `value` scores an end state against the start state: a large penalty for the hero's HP lost and a larger one for a lost life,
//!   a reward for the damage dealt to enemies and bosses (the sum of their HP drop) and for kills, for progress (X, the area and the
//!   episode), for HP gained (food) and for the weapon in hand, and a small penalty for the stage clock running low.
//! * A hero who makes no progress for a long time (the camera not locked) gets the sweep and retreat of the old bot (`Unstick`).
//!
//! # Output
//! One line on stderr for every episode and area change, the boss fights, the deaths and the continues; at the end a summary:
//! episodes cleared, lives and continues used, score, frames, HP lost per episode and the boss fight times.
//!
//! The numbers (`K`, `H`, `D`, `B`, the weights) are the ones of `Params`; the environment variables `DINO_SMART_H`, `DINO_SMART_D`,
//! `DINO_SMART_B` and `DINO_SMART_THREADS` change them for a test run.

use crate::game::Game;
use crate::input::{Frame, ATTACK, DOWN, JUMP, LEFT, RIGHT, UP};

/// Frames of one macro action (one decision).
pub const K: usize = 8;

/// The planner settings and the weights of the score (see `value`).
#[derive(Clone, Copy, Debug)]
pub struct Params {
    /// The horizon in frames (a plan and the default policy after it).
    pub horizon: usize,
    /// Macro steps of a plan.
    pub depth: usize,
    /// Partial plans kept per level.
    pub beam: usize,
    /// Threads for the simulations.
    pub threads: usize,
    /// Continues the bot may use.
    pub continues: u32,
    /// Points for a lost life.
    pub life: i64,
    /// Points for one HP of damage to an enemy, and for a kill.
    pub damage: i64,
    pub kill: i64,
    /// Points for one pixel of progress, one point of score, a weapon taken.
    pub progress: i64,
    pub score: i64,
    pub armed: i64,
    /// Points for one unit of distance closed to the nearest enemy (X plus three times the depth).
    pub near: i64,
    /// What one HP of the hero is worth at full HP (`hp_full`) and at no HP (`hp_empty`), in points; between the two it is linear.
    pub hp_full: i64,
    pub hp_empty: i64,
}

impl Params {
    pub fn from_env(continues: u32) -> Params {
        let var = |n: &str, d: i64| std::env::var(n).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as i64;
        Params {
            horizon: var("DINO_SMART_H", 96) as usize,
            depth: var("DINO_SMART_D", 3) as usize,
            beam: var("DINO_SMART_B", 4) as usize,
            threads: var("DINO_SMART_THREADS", cores.min(8)) as usize,
            continues,
            life: var("DINO_SMART_LIFE", 20_000),
            damage: var("DINO_SMART_DAMAGE", 30),
            kill: var("DINO_SMART_KILL", 60),
            progress: var("DINO_SMART_PROGRESS", 6),
            score: var("DINO_SMART_SCORE", 1),
            armed: var("DINO_SMART_ARMED", 40),
            near: var("DINO_SMART_NEAR", 3),
            hp_full: var("DINO_SMART_HP_FULL", 20),
            hp_empty: var("DINO_SMART_HP_EMPTY", 120),
        }
    }
}

/// What `hp` points of the hero are worth, in hundredths of a point: the worth of one HP goes from `hp_full` at 100 HP to `hp_empty` at 0
/// (a hurt hero is more careful).
fn hp_worth(prm: &Params, hp: i64) -> i64 {
    let h = hp.clamp(0, 100);
    // the integral of w(h) = hp_empty + (hp_full - hp_empty) * h / 100 from 0 to h
    100 * (prm.hp_empty * h) + (prm.hp_full - prm.hp_empty) * h * h / 2
}
/// The progress of one area step (an area or an episode is entered): more than any walk inside an area.
const AREA_STEP: i64 = 20_000;
const STAGE_STEP: i64 = 100_000;

// ---------------------------------------------------------------------------------------------------------------------------------
// Macro actions

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Act {
    Idle,
    Attack,
    Combo,
    Jump,
    Special,
    JumpAttack,
    Run,
}

/// K frames of input: a direction byte and what the buttons do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Macro {
    dir: u8,
    act: Act,
}

const DIRS: [u8; 9] = [0, RIGHT, LEFT, UP, DOWN, RIGHT | UP, RIGHT | DOWN, LEFT | UP, LEFT | DOWN];

impl Macro {
    /// The input byte of frame `i` (0 to K - 1) of the macro.
    fn input(self, i: usize) -> u8 {
        let d = self.dir;
        match self.act {
            Act::Idle => d,
            Act::Attack => d | if i < 3 { ATTACK } else { 0 },
            Act::Combo => d | if i < 2 || (4..6).contains(&i) { ATTACK } else { 0 },
            Act::Jump => d | if i < 3 { JUMP } else { 0 },
            Act::Special => d | if i < 2 { ATTACK | JUMP } else { 0 },
            Act::JumpAttack => d | if i < 3 { JUMP } else { 0 } | if (4..7).contains(&i) { ATTACK } else { 0 },
            // a double tap of the direction starts the run: press, release, press and hold
            Act::Run => {
                if i == 1 {
                    0
                } else {
                    d
                }
            }
        }
    }
}

/// One frame of the game with the input of player 1 (what `tick_game` of main.rs does).
fn step(g: &mut Game, f: u8) {
    g.w.input = [f, 0, 0];
    g.w.system = 0x08;
    g.w.coins = 0;
    g.tick();
}

// ---------------------------------------------------------------------------------------------------------------------------------
// Reading the game

/// An enemy that can still be hit: alive (mode 1 to 3; mode 0 is the init and 4 and up the dying, as the census 0x3242E counts) with
/// HP. +$7D negative means "not counted" in the census: the steam pieces (class 15, HP 1) have it, and so have some bosses (HP 550).
fn hostile(e: &crate::game::enemy::Enemy) -> bool {
    e.b.live != 0 && (1..4).contains(&e.b.mode) && e.b.hp != 0 && ((e.m.b(0x7D) as i8) >= 0 || e.b.hp > 1)
}

/// The distance to the nearest enemy that can be hit.
fn nearest(g: &Game) -> Option<i64> {
    let p = &g.players[0].b;
    g.enemies.slots.iter().filter(|e| hostile(e)).map(|e| (e.b.xi() as i64 - p.xi() as i64).abs() + 3 * (e.b.zi() as i64 - p.zi() as i64).abs()).min()
}

/// An enemy of the start state that is gone or dying.
fn gone(e: &crate::game::enemy::Enemy) -> bool {
    e.b.live == 0 || e.b.hp == 0 || e.b.mode >= 4
}

/// The BCD score as a number.
fn bcd(v: u32) -> i64 {
    (0..8).fold(0, |acc, i| acc + ((v >> (4 * i)) & 15) as i64 * 10i64.pow(i))
}

/// The hero can be controlled: the game runs the level, no scripted scene, the hero is alive and not locked.
fn controllable(g: &Game) -> bool {
    g.flow.in_game != 0 && g.flow.mode == 0xC && g.w.scripted == 0 && g.players[0].b.live != 0 && g.players[0].b.hp != 0 && g.players[0].input_lock == 0
}

/// A number that grows when the hero goes right, through the areas and the episodes.
fn progress(g: &Game) -> i64 {
    let x = (g.players[0].b.xi() as i64).clamp(-512, 0x3FFF);
    g.w.stage as i64 * STAGE_STEP + g.cam.s2.area as i64 * AREA_STEP + x
}

/// The state a plan is scored against.
#[derive(Clone)]
struct Snap {
    hp: i64,
    lives: i64,
    score: i64,
    progress: i64,
    armed: bool,
    enemy_hp: Vec<u16>,
    stage: u8,
    /// The distance to the nearest enemy that can be hit (X plus three times the depth), if there is one.
    near: Option<i64>,
    prm: Params,
}

impl Snap {
    fn of(g: &Game, prm: Params) -> Snap {
        let p = &g.players[0];
        Snap {
            hp: p.b.hp as i64,
            lives: p.lives as i64,
            score: bcd(p.score),
            progress: progress(g),
            armed: p.item != 0,
            enemy_hp: g.enemies.slots.iter().map(|e| if hostile(e) { e.b.hp } else { 0 }).collect(),
            stage: g.w.stage,
            near: nearest(g),
            prm,
        }
    }
}

/// The score of an end state (higher is better).
fn value(s: &Snap, g: &Game) -> i64 {
    let prm = &s.prm;
    let p = &g.players[0];
    let hp = p.b.hp as i64;
    let mut v = 0;
    v += (hp_worth(prm, hp) - hp_worth(prm, s.hp)) / 100;
    if p.lives as i64 != s.lives && g.w.stage == s.stage || p.b.live == 0 || p.b.hp == 0 {
        v -= prm.life;
    }
    let (mut damage, mut kills) = (0i64, 0i64);
    for (e, &hp0) in g.enemies.slots.iter().zip(&s.enemy_hp) {
        if hp0 == 0 {
            continue;
        }
        if gone(e) {
            damage += hp0 as i64;
            kills += 1;
        } else if e.b.hp < hp0 {
            damage += (hp0 - e.b.hp) as i64;
        }
    }
    v += damage * prm.damage + kills * prm.kill;
    v += (progress(g) - s.progress) * prm.progress;
    if let (Some(d0), Some(d1)) = (s.near, nearest(g)) {
        v += (d0 - d1).clamp(-400, 400) * prm.near;
    }
    v += (bcd(p.score) - s.score) * prm.score;
    if p.item != 0 && !s.armed {
        v += prm.armed;
    }
    // the stage clock: the last 20 seconds
    if g.rank.time_min == 0 && g.rank.time_sec < 0x20 {
        v -= 30;
    }
    v
}

// ---------------------------------------------------------------------------------------------------------------------------------
// The default policy (what the hero does after the first macro of a plan)

/// The state of the policy: the X of the last progress and the frames without it, for the sweep up and down at a wall or a ledge.
#[derive(Clone, Copy, Default)]
struct Policy {
    best_x: i16,
    still: u16,
    t: u16,
}

impl Policy {
    fn new(g: &Game) -> Policy {
        Policy { best_x: g.players[0].b.xi(), still: 0, t: 0 }
    }

    /// The input of the next frame.
    fn input(&mut self, g: &Game) -> u8 {
        self.t = self.t.wrapping_add(1);
        let p = &g.players[0];
        let (px, pz) = (p.b.xi() as i32, p.b.zi() as i32);
        if p.b.xi() > self.best_x {
            self.best_x = p.b.xi();
            self.still = 0;
        } else {
            self.still += 1;
        }
        // the nearest enemy that can be hit
        let mut best: Option<(i32, i32, i32)> = None;
        for e in g.enemies.slots.iter().filter(|e| hostile(e)) {
            let (dx, dz) = (e.b.xi() as i32 - px, e.b.zi() as i32 - pz);
            let d = dx.abs() + 3 * dz.abs();
            if best.is_none_or(|b| d < b.0) {
                best = Some((d, dx, dz));
            }
        }
        match best {
            Some((_, dx, dz)) if dx.abs() < 420 && dz.abs() < 120 => {
                let toward = if dx > 0 { RIGHT } else { LEFT };
                let (adx, adz) = (dx.abs(), dz.abs());
                if adx <= 44 && adz <= 12 {
                    // in reach: face it and hit (a press of attack every 6th frame is a hit and a rest of the combo timer)
                    let f = if self.t % 6 < 3 { ATTACK } else { 0 };
                    return f | if adx > 20 { toward } else { 0 };
                }
                let mut f = 0;
                if adx > 36 {
                    f |= toward;
                }
                if adz > 8 {
                    f |= if dz > 0 { UP } else { DOWN };
                }
                f
            }
            _ => {
                // nothing to hit: go right; at a wall sweep the depth up and down
                if self.still > 12 {
                    let leg = ((self.still - 12) / 24) % 4;
                    return RIGHT | if leg < 2 { UP } else { DOWN };
                }
                RIGHT
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// The planner

/// One partial plan: the game after its macros, the macros and the score of the end of the horizon.
struct Node {
    g: Game,
    path: Vec<Macro>,
    val: i64,
}

/// What one simulation job returns.
struct Done {
    val: i64,
    path: Vec<Macro>,
    g: Option<Game>,
}

/// Plays `n` frames of the policy on `g`.
fn rollout(g: &mut Game, n: usize) {
    let mut pol = Policy::new(g);
    for _ in 0..n {
        if g.players[0].b.live == 0 || g.players[0].b.hp == 0 {
            break;
        }
        let f = pol.input(g);
        step(g, f);
    }
}

fn par_map<T: Sync, R: Send>(items: &[T], threads: usize, f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    if threads <= 1 || items.len() < 2 {
        return items.iter().map(&f).collect();
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let parts: Vec<Vec<(usize, R)>> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..threads.min(items.len()))
            .map(|_| {
                s.spawn(|| {
                    let mut out = Vec::new();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if i >= items.len() {
                            break;
                        }
                        out.push((i, f(&items[i])));
                    }
                    out
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a simulation thread failed")).collect()
    });
    let mut all: Vec<(usize, R)> = parts.into_iter().flatten().collect();
    all.sort_by_key(|x| x.0);
    all.into_iter().map(|x| x.1).collect()
}

/// The candidate macros for a state. `wide` = an enemy is near: every action; else only the walks and the jumps.
fn candidates(g: &Game, wide: bool) -> Vec<Macro> {
    let mut v = Vec::new();
    let acts: &[Act] = if wide { &[Act::Idle, Act::Attack, Act::Combo, Act::Jump, Act::Special, Act::JumpAttack] } else { &[Act::Idle, Act::Jump] };
    for &dir in &DIRS {
        for &act in acts {
            v.push(Macro { dir, act });
        }
    }
    for &dir in &[RIGHT, LEFT, RIGHT | UP, RIGHT | DOWN] {
        v.push(Macro { dir, act: Act::Run });
    }
    if wide {
        for &dir in &[RIGHT, LEFT] {
            v.push(Macro { dir, act: Act::Attack });
        }
    }
    let _ = g;
    v
}

/// The planner: the real game goes in, the macro to play comes out.
pub struct Planner {
    pub params: Params,
    /// The best plan of the last decision (its tail is a candidate of the next one).
    plan: Vec<Macro>,
    /// Simulated frames (for the report).
    pub sim_frames: u64,
    pub decisions: u64,
}

impl Planner {
    pub fn new(params: Params) -> Planner {
        Planner { params, plan: Vec::new(), sim_frames: 0, decisions: 0 }
    }

    /// An enemy near enough that the attack actions matter.
    fn near_enemy(g: &Game) -> bool {
        let p = &g.players[0];
        let (px, pz) = (p.b.xi() as i32, p.b.zi() as i32);
        g.enemies.slots.iter().any(|e| hostile(e) && (e.b.xi() as i32 - px).abs() < 150 && (e.b.zi() as i32 - pz).abs() < 60)
    }

    pub fn decide(&mut self, g: &Game) -> Macro {
        self.decisions += 1;
        let prm = self.params;
        let snap = Snap::of(g, prm);
        let wide = Self::near_enemy(g);
        let mut root_cands = candidates(g, wide);
        if let Some(&m) = self.plan.get(1) {
            if !root_cands.contains(&m) {
                root_cands.push(m);
            }
        }
        let mut level: Vec<Node> = vec![Node { g: g.clone(), path: Vec::new(), val: 0 }];
        let mut best: Option<(i64, Vec<Macro>)> = None;
        for d in 0..prm.depth {
            let last = d + 1 == prm.depth;
            let cands = if d == 0 { root_cands.clone() } else { candidates(g, wide) };
            let jobs: Vec<(usize, Macro)> = (0..level.len()).flat_map(|i| cands.iter().map(move |&m| (i, m))).collect();
            let rest = prm.horizon.saturating_sub(K * (d + 1));
            let results = par_map(&jobs, prm.threads, |&(pi, m)| {
                let parent = &level[pi];
                let mut c = parent.g.clone();
                for i in 0..K {
                    step(&mut c, m.input(i));
                }
                let mut path = parent.path.clone();
                path.push(m);
                if last {
                    rollout(&mut c, rest);
                    Done { val: value(&snap, &c), path, g: None }
                } else {
                    let mut r = c.clone();
                    rollout(&mut r, rest);
                    Done { val: value(&snap, &r), path, g: Some(c) }
                }
            });
            self.sim_frames += jobs.len() as u64 * (K + rest) as u64;
            // the best first: higher value, then the earlier candidate
            let mut order: Vec<usize> = (0..results.len()).collect();
            order.sort_by_key(|&i| (-results[i].val, i));
            if last {
                let b = &results[order[0]];
                best = Some((b.val, b.path.clone()));
            } else {
                let mut results: Vec<Option<Done>> = results.into_iter().map(Some).collect();
                level = order
                    .iter()
                    .take(prm.beam)
                    .map(|&i| {
                        let r = results[i].take().unwrap();
                        Node { g: r.g.unwrap(), path: r.path, val: r.val }
                    })
                    .collect();
                best = Some((level[0].val, level[0].path.clone()));
            }
        }
        let (_, path) = best.unwrap_or((0, vec![Macro { dir: RIGHT, act: Act::Idle }]));
        let first = path[0];
        self.plan = path;
        first
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// The bot

/// The stall retreat of the old bot (see `autoplay.rs`): after a long stall the hero sweeps the depth and walks back.
#[derive(Clone, Copy)]
struct Back {
    goal: i16,
    stops: u32,
    walk_from: u32,
    sweep: Option<u32>,
}

/// The stats of one episode and the whole run.
#[derive(Clone, Default, Debug)]
pub struct Stats {
    pub frames: u32,
    pub hp_lost: [u32; 8],
    pub lives_lost: u32,
    pub continues: u32,
    pub episodes_cleared: u32,
    pub boss_times: Vec<(u8, u32)>,
    pub score: i64,
    pub ending: bool,
    pub game_over: bool,
}

pub struct Smart {
    chars: Vec<u8>,
    pub planner: Planner,
    /// The macro in progress and the frame of it.
    cur: Option<(Macro, usize)>,
    // the escape from a stall
    best_x: i16,
    best_tick: u32,
    area: (u8, u8),
    back: Option<Back>,
    rest_until: u32,
    // the report
    pub stats: Stats,
    last: Option<(u8, u8)>,
    last_hp: i64,
    last_lives: i64,
    boss_from: Option<u32>,
    played: bool,
    in_state8: u32,
    prev_join: u8,
    summary_done: bool,
    clone_ns: u64,
    start: std::time::Instant,
    plan_ns: u128,
    diag_every: u32,
    /// Walk with the default policy when no enemy can be hit (off by default: it lost lives in Episodes 1 and 2; `DINO_SMART_CALM=1` turns it on).
    calm_walk: bool,
    calm_pol: Policy,
}

impl Smart {
    pub fn new(chars: &[u8], continues: u32) -> Smart {
        Smart::with_params(chars, Params::from_env(continues))
    }

    pub fn with_params(chars: &[u8], params: Params) -> Smart {
        Smart {
            chars: chars.to_vec(),
            planner: Planner::new(params),
            cur: None,
            best_x: 0,
            best_tick: 0,
            area: (0, 0),
            back: None,
            rest_until: 0,
            stats: Stats::default(),
            last: None,
            last_hp: 0,
            last_lives: 0,
            boss_from: None,
            played: false,
            in_state8: 0,
            prev_join: 0,
            summary_done: false,
            clone_ns: 0,
            start: std::time::Instant::now(),
            plan_ns: 0,
            calm_walk: std::env::var("DINO_SMART_CALM").map(|v| v == "1").unwrap_or(false),
            calm_pol: Policy::default(),
            diag_every: std::env::var("DINO_SMART_DIAG").ok().and_then(|v| v.parse().ok()).unwrap_or(0),
        }
    }

    /// The game ended: the ending was seen and the attract loop is back, or a game over with no continue left.
    pub fn done(&self) -> bool {
        self.stats.ending || self.stats.game_over
    }

    /// The input of this tick.
    pub fn frame(&mut self, g: &Game, tick: u32) -> Frame {
        let mut f = Frame::default();
        if g.flow.in_game == 0 {
            // the attract loop: a coin, then start
            let t = tick % 300;
            match t {
                10..=13 => f.coins = 1,
                100..=103 => f.system = 1,
                _ => {}
            }
            return f;
        }
        self.played = true;
        // the hero select
        let sel = &g.flow.front.select;
        if matches!(g.flow.mode, 0 | 4) && sel.rec[0].state == 2 && g.players[0].b.live == 0 {
            let want = [0u8, 2, 1, 3].iter().position(|&id| id == self.chars.first().copied().unwrap_or(0)).unwrap_or(0) as u8;
            let pos = sel.rec[0].pos;
            f.players[0] = if pos == 0xFF {
                0
            } else if pos != want {
                if tick % 4 < 2 { RIGHT } else { 0 }
            } else if tick % 4 < 2 {
                ATTACK
            } else {
                0
            };
            return f;
        }
        // the continue: a coin and start while the countdown runs
        if g.flow.join[0].state == 8 && g.players[0].b.live == 0 {
            self.in_state8 += 1;
            if self.stats.continues < self.planner.params.continues {
                match self.in_state8 % 90 {
                    10..=13 => f.coins = 1,
                    40..=43 => f.system = 1,
                    _ => {}
                }
            }
            return f;
        }
        self.in_state8 = 0;
        if !controllable(g) {
            self.cur = None;
            // a scene: attack skips the waits of a scripted scene
            f.players[0] = ATTACK;
            return f;
        }
        let x = g.players[0].b.xi();
        if self.area != (g.w.stage, g.cam.s2.area) || x >= self.best_x.saturating_add(4) {
            self.area = (g.w.stage, g.cam.s2.area);
            self.best_x = x;
            self.best_tick = tick;
        }
        // a fight is not a stall (an enemy that can be hit, or a boss)
        if g.enemies.slots.iter().any(hostile) || g.cam.s2.lock78 != 0 && Planner::near_enemy(g) {
            self.best_tick = tick;
        }
        if let Some(b) = self.unstick(g, tick) {
            self.cur = None;
            f.players[0] = b;
            return f;
        }
        // nothing to fight: walk right with the default policy (no planning, the free frames of the game)
        if self.calm_walk && !g.enemies.slots.iter().any(hostile) {
            self.cur = None;
            let mut pol = self.calm_pol;
            f.players[0] = pol.input(g);
            self.calm_pol = pol;
            return f;
        }
        let (m, i) = match self.cur {
            Some((m, i)) if i < K => (m, i),
            _ => {
                if self.clone_ns == 0 {
                    let t = std::time::Instant::now();
                    let c = g.clone();
                    self.clone_ns = t.elapsed().as_nanos() as u64;
                    drop(c);
                }
                let t = std::time::Instant::now();
                let m = self.planner.decide(g);
                self.plan_ns += t.elapsed().as_nanos();
                (m, 0)
            }
        };
        self.cur = Some((m, i + 1));
        f.players[0] = m.input(i);
        f
    }

    /// The input of a stall escape, or None. After `SWEEP_AFTER` ticks with no new best X and no enemy to fight the hero walks right and
    /// sweeps the depth up and down by turns, a longer way each time (the old bot's walk, for a wall or a ledge that needs another depth),
    /// with a jump now and then and a punch for a gate. After `RETREAT_AFTER` ticks, the camera not locked, he also walks back
    /// and sweeps from there (see `autoplay.rs`).
    fn unstick(&mut self, g: &Game, tick: u32) -> Option<u8> {
        const SWEEP_AFTER: u32 = 90;
        const RETREAT_AFTER: u32 = 2400;
        const STEP: i16 = 0x30;
        const WALK_MAX: u32 = 240;
        const SWEEP_LEN: u32 = 360;
        const MAX_STOPS: u32 = 14;
        const REST: u32 = 600;
        let stalled = tick - self.best_tick;
        if stalled < RETREAT_AFTER {
            self.back = None;
        }
        if stalled < SWEEP_AFTER {
            return None;
        }
        let x = g.players[0].b.xi();
        if stalled >= RETREAT_AFTER && g.cam.s2.lock78 == 0 && tick >= self.rest_until {
            let b = self.back.get_or_insert(Back { goal: x.wrapping_sub(STEP), stops: 0, walk_from: tick, sweep: None });
            if let Some(s) = b.sweep {
                let t = tick - s;
                if t < SWEEP_LEN {
                    return Some(if t < SWEEP_LEN / 3 { UP } else { DOWN });
                }
                b.sweep = None;
                b.stops += 1;
                if b.stops >= MAX_STOPS {
                    self.back = None;
                    self.rest_until = tick + REST;
                    self.best_tick = tick;
                    return None;
                }
                b.goal = b.goal.wrapping_sub(STEP);
                b.walk_from = tick;
                return Some(LEFT);
            }
            if x <= b.goal || tick - b.walk_from > WALK_MAX {
                b.sweep = Some(tick);
                return Some(UP);
            }
            return Some(LEFT);
        }
        let mut t = stalled - SWEEP_AFTER;
        let mut leg = 0u32;
        while t >= (60 * (leg + 1)).min(240) {
            t -= (60 * (leg + 1)).min(240);
            leg += 1;
        }
        Some(RIGHT | if leg % 2 == 0 { UP } else { DOWN } | if tick % 60 < 4 { ATTACK } else { 0 } | if tick % 180 < 8 { JUMP } else { 0 })
    }

    /// After a tick: the lines and the stats.
    pub fn report(&mut self, g: &Game, tick: u32) {
        self.stats.frames = tick + 1;
        if self.diag_every != 0 && tick % self.diag_every == 0 && g.flow.in_game != 0 {
            let p = &g.players[0];
            let en: Vec<String> = g.enemies.slots.iter().filter(|e| hostile(e)).map(|e| format!("{}@{},{}:h{}", e.b.k20 / 4, e.b.xi() as i32 - p.b.xi() as i32, e.b.zi() as i32 - p.b.zi() as i32, e.b.hp)).collect();
            let all: Vec<String> = g.enemies.slots.iter().filter(|e| e.b.live != 0).map(|e| format!("{}@{},{}:m{:x}h{}f{:x}", e.b.k20 / 4, e.b.xi() as i32 - p.b.xi() as i32, e.b.zi() as i32 - p.b.zi() as i32, e.b.mode, e.b.hp, e.m.b(0x7D))).collect();
            let props: Vec<String> = g.props.slots.iter().filter(|e| e.b.live != 0).map(|e| format!("{}@{},{}:m{:x}h{}", e.b.k20 / 4, e.b.xi() as i32 - p.b.xi() as i32, e.b.zi() as i32 - p.b.zi() as i32, e.b.mode, e.b.hp)).collect();
            eprintln!("smart: diag all {tick}: boss {:x} live enemies {:?} props {:?}", g.hud.boss, all, props);
            eprintln!("smart: diag {tick}: scripted {:x} lock_in {:x} ctl {} mode {:x} hp {} x {} z {} y {} state {:x} cam {} lock {} | item {:x} | enemies {:?}", g.w.scripted, p.input_lock, controllable(g), g.flow.mode, p.b.hp, p.b.xi(), p.b.zi(), p.b.yi(), p.b.state(), g.cam.s2.cam_x, g.cam.s2.lock78, p.item, en);
        }
        if g.flow.in_game != 0 {
            let p = &g.players[0];
            let (hp, lives) = (p.b.hp as i64, p.lives as i64);
            let ep = (g.w.stage as usize).min(7);
            if self.last.is_some() {
                if hp < self.last_hp && p.b.live != 0 {
                    self.stats.hp_lost[ep] += (self.last_hp - hp) as u32;
                }
                if lives < self.last_lives && g.w.stage == self.last.unwrap().0 {
                    self.stats.lives_lost += 1;
                    eprintln!("smart: frame {tick} lost a life (lives left {lives}) episode {} area {}", g.w.stage + 1, g.cam.s2.area);
                }
            }
            if p.b.live != 0 {
                self.last_hp = hp;
                self.last_lives = lives;
            }
            self.stats.score = bcd(p.score);
            let now = (g.w.stage, g.cam.s2.area);
            if g.flow.mode == 0xC && self.last != Some(now) {
                if self.last.is_some_and(|l| l.0 != now.0) {
                    self.stats.episodes_cleared += 1;
                }
                eprintln!("smart: frame {tick} episode {} area {} hp {hp} lives {lives} score {} hp lost so far {}", now.0 + 1, now.1, self.stats.score, self.stats.hp_lost.iter().sum::<u32>());
                self.last = Some(now);
            }
            // the boss fights
            let boss = g.hud.boss != 0 && g.body(g.hud.boss).live != 0;
            match (boss, self.boss_from) {
                (true, None) => {
                    self.boss_from = Some(tick);
                    eprintln!("smart: frame {tick} boss fight begins (episode {})", g.w.stage + 1);
                }
                (false, Some(from)) => {
                    self.boss_from = None;
                    self.stats.boss_times.push((g.w.stage + 1, tick - from));
                    eprintln!("smart: frame {tick} boss fight over after {} frames (episode {})", tick - from, g.w.stage + 1);
                }
                _ => {}
            }
            if g.flow.mode == 0x1C && !self.stats.ending {
                self.stats.ending = true;
                self.stats.episodes_cleared += 1;
                eprintln!("smart: frame {tick} the ending");
            }
            // a continue: the countdown (join state 8) ends and the hero is alive again
            let js = g.flow.join[0].state;
            if self.prev_join == 8 && js != 8 && p.b.live != 0 {
                self.stats.continues += 1;
                eprintln!("smart: frame {tick} continue {}", self.stats.continues);
            }
            self.prev_join = js;
        } else if self.played && !self.stats.ending && g.flow.in_game == 0 {
            self.stats.game_over = true;
            eprintln!("smart: frame {tick} game over");
        }
        if self.done() && !self.summary_done {
            self.summary(tick);
        }
    }

    /// The summary line (once).
    pub fn summary(&mut self, tick: u32) {
        if self.summary_done {
            return;
        }
        self.summary_done = true;
        let s = &self.stats;
        let hp: Vec<String> = s.hp_lost.iter().enumerate().filter(|(_, h)| **h > 0).map(|(i, h)| format!("{}:{h}", i + 1)).collect();
        let boss: Vec<String> = s.boss_times.iter().map(|(e, t)| format!("{e}:{t}")).collect();
        let secs = self.start.elapsed().as_secs_f64();
        eprintln!(
            "smart: summary: episodes cleared {} | lives lost {} | continues {} | score {} | frames {} | hp lost per episode [{}] | boss fights (episode:frames) [{}] | ending {} | {:.1} s, {:.0} ticks/s real, {} decisions, {:.0} M simulated frames, clone {} ns",
            s.episodes_cleared,
            s.lives_lost,
            s.continues,
            s.score,
            tick + 1,
            hp.join(" "),
            boss.join(" "),
            s.ending,
            secs,
            (tick + 1) as f64 / secs.max(0.001),
            self.planner.decisions,
            self.planner.sim_frames as f64 / 1e6,
            self.clone_ns
        );
        let _ = self.plan_ns;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn rom() -> Option<&'static AssetPack> {
        static A: std::sync::OnceLock<Option<&'static AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../dino.zip");
            let path = if path.exists() { path } else { std::path::Path::new("../../../dino.zip") };
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads"))))
        })
    }

    fn stage(a: &'static AssetPack, ep: u8) -> Game {
        let mut g = Game::new(a);
        g.flow_apply_config(crate::game::flow::Config::default());
        g.start_stage_at(ep, 0, 0, &[0]);
        g
    }

    /// The settings of a test: a short horizon on few threads.
    fn small(threads: usize, continues: u32) -> Params {
        Params { horizon: 32, depth: 2, beam: 2, threads, continues, ..Params::from_env(continues) }
    }

    /// One frame as `tick_game` plays it, with the bot's input.
    fn play(g: &mut Game, bot: &mut Smart, tick: u32) -> u8 {
        let f = bot.frame(g, tick);
        g.w.input = f.players;
        g.w.system = f.system | 0x08;
        g.w.coins = f.coins;
        g.tick();
        bot.report(g, tick);
        f.players[0]
    }

    /// A copy of the game goes on exactly as the game does: the planner depends on it.
    #[test]
    fn a_clone_plays_the_same_future() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut g = stage(a, 0);
        for t in 0..400 {
            step(&mut g, if t < 300 { ATTACK } else { RIGHT | (t as u8 & ATTACK) });
        }
        let mut c = g.clone();
        for t in 0..300 {
            let f = if t % 40 < 20 { RIGHT } else { ATTACK | LEFT };
            step(&mut g, f);
            step(&mut c, f);
            assert_eq!((g.w.rng, g.players[0].b.xi(), g.players[0].b.hp, g.cam.s2.cam_x), (c.w.rng, c.players[0].b.xi(), c.players[0].b.hp, c.cam.s2.cam_x), "frame {t}");
        }
    }

    /// From power on the bot puts in a coin, starts, picks the hero of `--chars` and plays: the hero is in the level and moves right.
    #[test]
    fn the_bot_starts_the_game_and_picks_the_hero() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        for hero in [0u8, 2, 3] {
            let mut g = Game::new(a);
            g.flow_boot();
            let mut bot = Smart::with_params(&[hero], small(2, 0));
            let mut cam_max = 0;
            for t in 0..2600 {
                play(&mut g, &mut bot, t);
                cam_max = cam_max.max(g.cam.s2.cam_x);
            }
            assert!(g.flow.in_game != 0, "hero {hero}: the game did not start");
            assert_eq!(g.players[0].b.id, hero, "hero {hero}: the select picked another hero");
            assert!(cam_max > 200, "hero {hero}: the bot did not walk (camera {cam_max})");
        }
    }

    /// The same run every time, and on any number of threads: the inputs of 600 frames of Episode 1 are the same.
    #[test]
    fn the_bot_is_deterministic_on_any_number_of_threads() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let run = |threads: usize| {
            let mut g = stage(a, 0);
            let mut bot = Smart::with_params(&[0], small(threads, 0));
            (0..700).map(|t| play(&mut g, &mut bot, t)).collect::<Vec<u8>>()
        };
        let (one, four, again) = (run(1), run(4), run(4));
        assert_eq!(one, four, "one thread and four threads");
        assert_eq!(four, again, "two runs");
        assert!(one.iter().any(|&b| b & (LEFT | RIGHT) != 0), "the bot never moved");
    }

    /// The macro actions are what they say: eight frames, the double tap of the run has a gap, the special presses both buttons.
    #[test]
    fn the_macros_make_the_inputs_they_name() {
        let run = Macro { dir: RIGHT, act: Act::Run };
        assert_eq!((0..K).map(|i| run.input(i)).collect::<Vec<_>>(), [RIGHT, 0, RIGHT, RIGHT, RIGHT, RIGHT, RIGHT, RIGHT]);
        let sp = Macro { dir: 0, act: Act::Special };
        assert_eq!(sp.input(0), ATTACK | JUMP);
        assert_eq!(sp.input(3), 0);
        let ja = Macro { dir: LEFT, act: Act::JumpAttack };
        assert_eq!((ja.input(0), ja.input(3), ja.input(5)), (LEFT | JUMP, LEFT, LEFT | ATTACK));
        assert!(candidates_len(true) > candidates_len(false));
    }

    fn candidates_len(wide: bool) -> usize {
        let Some(a) = rom() else { return if wide { 2 } else { 1 } };
        candidates(&stage(a, 0), wide).len()
    }

    /// The score: a hero who loses HP scores less, damage and progress score more, a lost life costs most.
    #[test]
    fn the_score_orders_the_outcomes() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let g0 = stage(a, 0);
        let prm = small(1, 0);
        let snap = Snap::of(&g0, prm);
        let base = value(&snap, &g0);
        let mut hurt = g0.clone();
        hurt.players[0].b.hp -= 20;
        assert!(value(&snap, &hurt) < base);
        let mut ahead = g0.clone();
        ahead.players[0].b.set_xi(g0.players[0].b.xi() + 50);
        assert!(value(&snap, &ahead) > base);
        let mut dead = g0.clone();
        dead.players[0].b.hp = 0;
        assert!(value(&snap, &dead) < value(&snap, &hurt));
    }

    /// Episode 1 with Jack, no continue: the bot clears it (the camera is in Episode 2 and no life was lost). Slow: about 10 s with all cores.
    #[test]
    #[ignore]
    fn smart_clears_episode_1_with_jack_without_a_continue() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut g = Game::new(a);
        g.flow_boot();
        let mut bot = Smart::with_params(&[0], Params::from_env(0));
        let mut cleared = None;
        for t in 0..40_000 {
            play(&mut g, &mut bot, t);
            if g.flow.in_game != 0 && g.w.stage == 1 && g.flow.mode == 0xC {
                cleared = Some(t);
                break;
            }
        }
        let t = cleared.unwrap_or_else(|| panic!("Episode 1 was not cleared: {:?}", bot.stats));
        assert_eq!(bot.stats.lives_lost, 0, "a life was lost: {:?}", bot.stats);
        assert_eq!(bot.stats.continues, 0);
        eprintln!("Episode 1 cleared at frame {t}: {:?}", bot.stats);
    }

    #[test]
    #[ignore]
    fn probe_directions() {
        let Some(a) = rom() else { return };
        let mut g = stage(a, 0);
        let mut n = 0;
        while !controllable(&g) && n < 3000 {
            step(&mut g, ATTACK);
            n += 1;
        }
        for _ in 0..30 {
            step(&mut g, 0);
        }
        eprintln!("controllable after {n} frames, x {} z {}", g.players[0].b.xi(), g.players[0].b.zi());
        for (name, d) in [("UP", UP), ("DOWN", DOWN), ("RIGHT", RIGHT), ("LEFT", LEFT)] {
            let (x0, z0) = (g.players[0].b.xi(), g.players[0].b.zi());
            for _ in 0..20 {
                step(&mut g, d);
            }
            eprintln!("{name}: dx {} dz {}  facing {}", g.players[0].b.xi() - x0, g.players[0].b.zi() - z0, g.players[0].b.facing);
        }
    }
}
