//! `--autoplay` (M3-05): a bot for review runs from power on. It puts in a coin, presses start and attacks to pick Jack on the
//! select screen, then holds right and attacks in pulses; kill all runs every `KILL_EVERY` frames so the waves end. While the
//! game is not running (attract, game over) the coin, start and pick presses repeat every `CYCLE` frames.
//! The heroes keep their HP and the clock is topped up (god mode and F4 of `debug.rs`), unless `DINO_AUTO_MORTAL=1`.
//! It prints one line per change of the game mode to stderr: the frame, the episode, the area and the mode. A game that makes no
//! progress for `STUCK` ticks (the camera, the hero's X and the mode stand still) prints a `stuck` line with the state (camera,
//! hero, enemies, clock); `DINO_AUTO_DIAG=1` prints that line every `DIAG_EVERY` ticks of a game, and `DINO_AUTO_RANGE=A-B` every
//! tick from A to B (with a second line of the flow state: mode, fade, task flags).
//! A hero who makes no progress walks up and down by turns (`STALL`): the original's walls and ledges need a hero at the right depth.
//! `DINO_AUTO_NOKILL=1` sends no kill all press (a check that the waves end by the game's own means). `DINO_AUTO_CENSUS=1` prints the census of the
//! pool A classes the game makes, one line per episode at its end and one at the ending (see `Census`).

use crate::debug::Debug;
use crate::game::Game;
use crate::input::{Frame, ATTACK, DOWN, JUMP, LEFT, RIGHT, UP};

/// Frames between kill-all presses in a game.
const KILL_EVERY: u32 = 600;
/// Frames of the attract cycle (coin at 10, start at 100, attack at 200).
const CYCLE: u32 = 300;

/// Ticks without a new best X before the bot walks the hero up and down, and the pixels that count as progress.
const STALL: u32 = 60;
const PROGRESS: u16 = 4;
/// Ticks without progress after which `report` says `stuck`.
const STUCK: u32 = 3000;
/// Stall retreat (no camera lock): after `STALL` ticks without progress the hero walks left in steps of `STEP` pixels. At each
/// stop he sweeps the depth (Z) up, then down, for `SWEEP_LEN` ticks, so a ramp or a lower lane behind him is found. After
/// `MAX_STOPS` stops, or when the walk is blocked for `WALK_MAX` ticks, he walks right for `REST` ticks before the next retreat.
const STEP: u16 = 0x30;
/// Ticks without progress before the first retreat (the walk up and down of `STALL` comes first).
const RETREAT_AFTER: u32 = 2400;
const WALK_MAX: u32 = 240;
const SWEEP_LEN: u32 = 360;
const MAX_STOPS: u32 = 14;
const REST: u32 = 600;
/// Ticks between the state lines of `DINO_AUTO_DIAG`.
const DIAG_EVERY: u32 = 500;

/// One retreat: the goal X of the next stop, the stops made, when the walk began and when the depth sweep began.
#[derive(Clone, Copy)]
struct Back {
    goal: u16,
    stops: u32,
    walk_from: u32,
    sweep: Option<u32>,
}

/// The census of the pool A objects the bot makes (`DINO_AUTO_CENSUS=1`): per episode, the count of objects made per class. An object
/// is made when a slot becomes live, or live with another class (`k20 / 4`). A line is printed for each episode when the game leaves it
/// (and for the last one when the attract loop returns).
#[derive(Default)]
pub struct Census {
    /// The episode (stage) being counted, and the made count per class of it.
    stage: Option<u8>,
    made: std::collections::BTreeMap<u16, u32>,
    /// Per enemy slot: (live, class) at the last tick.
    prev: Vec<(bool, u16)>,
}

impl Census {
    fn tick(&mut self, g: &Game) {
        if self.prev.len() != g.enemies.slots.len() {
            self.prev = vec![(false, 0); g.enemies.slots.len()];
        }
        for (i, e) in g.enemies.slots.iter().enumerate() {
            let now = (e.b.live != 0, e.b.k20 / 4);
            let before = self.prev[i];
            if now.0 && (!before.0 || before.1 != now.1) {
                *self.made.entry(now.1).or_insert(0) += 1;
            }
            self.prev[i] = now;
        }
    }

    /// One line for the episode counted so far (stage `stage`), then the counts start again.
    fn line(&mut self) {
        if let Some(stage) = self.stage.take() {
            let parts: Vec<String> = self
                .made
                .iter()
                .map(|(c, n)| format!("{c}:{n}"))
                .collect();
            eprintln!("census: episode {} made {} classes: {}", stage + 1, self.made.len(), parts.join(" "));
        }
        self.made.clear();
    }
}

#[derive(Default)]
pub struct Bot {
    /// The census (`DINO_AUTO_CENSUS=1`).
    census: Census,
    /// The stall retreat in progress, and the tick until which the hero walks right after one (`REST`).
    back: Option<Back>,
    rest_until: u32,
    /// The last (episode, area, mode) reported.
    last: Option<(u8, u8, u8)>,
    /// The last state that counted as progress (camera X and Y, hero X, mode, sub) and the tick it was first seen.
    progress: Option<([u16; 3], u32)>,
    /// The tick of the last `stuck` line.
    stuck_said: u32,
    /// The tick the ending (mode 0x1C) began and the tick the game was back in the attract loop after it.
    ending: Option<u32>,
    finished: Option<u32>,
    /// The best X of the first hero in this area and the tick he got there.
    best_x: u16,
    best_tick: u32,
    area: (u8, u8),
}

impl Bot {
    /// The input of this tick. Kill all goes through `debug`.
    pub fn frame(&mut self, g: &mut Game, tick: u32, debug: &mut Debug) -> Frame {
        let mut f = Frame::default();
        if g.flow.in_game == 0 {
            let t = tick % CYCLE;
            match t {
                10..=13 => f.coins = 1,
                100..=103 => f.system = 1,
                200..=203 => f.players[0] = ATTACK,
                _ => {}
            }
            return f;
        }
        // The bot is a flow test, not a player: the heroes keep their HP (`DINO_AUTO_MORTAL=1` lets them die, to run the continue and
        // game over paths) and the clock is topped up to 5:00 before it runs out.
        if std::env::var_os("DINO_AUTO_MORTAL").is_none() {
            debug.god_on();
            if g.rank.time_min == 0 && g.rank.time_sec < 0x20 {
                g.rank_set_time(0x0500);
            }
        }
        // Jump pulses get Jack over the walls and the pits of the stages.
        // A swing takes about ten frames: attacking every twelfth frame keeps a hero from ever walking, so the bot punches that often only
        // when something is alive to hit (enemies, a locked camera) and otherwise once in a while (a gate, a barrel, a wall).
        // Only an enemy that can still be hit is a fight. The census G.$4F3 also counts objects with no HP: the pieces a broken steam pipe
        // (pool B class 40) puffs for ever in Episode 8 area 1 (D5-errata "Episode 8 area 1: steam pipe hold") and a dying or idle body.
        // Punching for those kept the hero from walking there; the original does not hold the camera for them.
        // An object still in mode 0 is not a fight either: a steam pipe makes a new piece (pool A class 15) every 6 frames, and in its first frame it keeps
        // the HP word of the slot's last object while the free cleared `+$7D` (D5-errata "Episode 8 area 1: the bot's punch loop"). In step with the
        // 12 frame swing that kept Jack jabbing in place for good.
        let fight = g.cam.s2.lock78 != 0 || g.enemies.slots.iter().any(|e| e.b.live != 0 && e.b.mode != 0 && e.b.hp != 0 && (e.m.b(0x7D) as i8) >= 0);
        let swing = if fight { tick % 12 < 4 } else { tick % 60 < 4 };
        // A stall without a camera lock sends the hero left to retreat (see `STEP`); otherwise he walks right.
        let stalled = tick - self.best_tick;
        if stalled < RETREAT_AFTER {
            self.back = None;
        }
        let retreat = if stalled >= RETREAT_AFTER && g.cam.s2.lock78 == 0 && tick >= self.rest_until { self.retreat(g.players[0].b.xi() as u16, tick) } else { None };
        let (run, depth) = match retreat {
            Some(LEFT) => (LEFT, 0),
            Some(d) => (0, d),
            None => (RIGHT, 0),
        };
        f.players[0] = run | depth | if swing { ATTACK } else { 0 } | if tick % 180 < 8 { JUMP } else { 0 };
        // Progress is a new best X (the areas run left to right). A hero who has made none for `STALL` ticks walks up and down by
        // turns, a longer way each time (60, 120, 180, 240 ticks), to find the way round a wall or a ledge.
        let x = g.players[0].b.xi() as u16;
        if self.area != (g.w.stage, g.cam.s2.area) || x >= self.best_x.wrapping_add(PROGRESS) && x < 0x8000 || g.w.scripted != 0 {
            self.area = (g.w.stage, g.cam.s2.area);
            self.best_x = x;
            self.best_tick = tick;
        }
        if retreat.is_none() && stalled >= STALL {
            let mut t = stalled - STALL;
            let mut leg = 0u32;
            while t >= (60 * (leg + 1)).min(240) {
                t -= (60 * (leg + 1)).min(240);
                leg += 1;
            }
            f.players[0] |= if leg.is_multiple_of(2) { UP } else { DOWN };
        }
        // `DINO_AUTO_NOKILL=1`: no kill all presses (the waves are cleared by the bot's own punches and walks; god mode stays).
        if tick.is_multiple_of(KILL_EVERY) && std::env::var_os("DINO_AUTO_NOKILL").is_none() {
            debug.kill_all();
        }
        f
    }

    /// The input of a stall retreat this tick (LEFT while walking, UP and DOWN while sweeping), or `None` when the hero walks right.
    fn retreat(&mut self, x: u16, tick: u32) -> Option<u8> {
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
        Some(LEFT)
    }

    /// The game was played from the first stage through the ending and is back in the attract loop (a headless run without `--frames` stops there).
    pub fn done(&self) -> bool {
        self.finished.is_some()
    }

    /// After a tick: one line when the game mode, the episode or the area changes.
    pub fn report(&mut self, g: &Game, tick: u32) {
        if std::env::var_os("DINO_AUTO_CENSUS").is_some() {
            if g.flow.in_game != 0 {
                if self.census.stage.is_some_and(|s| s != g.w.stage) {
                    self.census.line();
                }
                self.census.stage = Some(g.w.stage);
                self.census.tick(g);
            } else if self.census.stage.is_some() {
                self.census.line();
            }
        }
        if g.flow.in_game != 0 && g.flow.mode == 0x1C && self.ending.is_none() {
            self.ending = Some(tick);
        }
        if g.flow.in_game == 0 && self.ending.is_some() && self.finished.is_none() {
            self.finished = Some(tick);
            eprintln!("autoplay: the ending began at frame {}; back in the attract loop at frame {tick}", self.ending.unwrap());
        }
        if g.flow.in_game != 0 && g.flow.mode == 0xC {
            let now = [g.cam.s2.cam_x, g.cam.s2.cam_y, g.players[0].b.xi() as u16];
            match self.progress {
                Some((p, since)) if p == now => {
                    if tick - since >= STUCK && tick - self.stuck_said >= STUCK {
                        self.stuck_said = tick;
                        eprintln!("autoplay: stuck since frame {since}: {}", Self::state_line(g));
                    }
                }
                _ => self.progress = Some((now, tick)),
            }
            if std::env::var_os("DINO_AUTO_DIAG").is_some() && (tick.is_multiple_of(DIAG_EVERY) || Self::in_range(tick)) {
                eprintln!("autoplay: diag {tick}: {}", Self::state_line(g));
                eprintln!("autoplay: flow {tick}: {}", Self::flow_line(g));
            }
        }
        let now = (g.w.stage + 1, g.cam.s2.area, g.flow.mode);
        if self.last != Some(now) {
            self.last = Some(now);
            eprintln!("autoplay: frame {tick} episode {} area {} mode {:#04x} sub {}", now.0, now.1, now.2, g.flow.sub);
        }
    }

    /// `DINO_AUTO_RANGE=A-B` holds the tick.
    fn in_range(tick: u32) -> bool {
        std::env::var("DINO_AUTO_RANGE")
            .ok()
            .and_then(|v| {
                let (a, b) = v.split_once('-')?;
                Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))
            })
            .is_some_and(|(a, b)| tick >= a && tick <= b)
    }

    /// The flow state in one line: the frame counter, the lag, the fade mask, the game mode, the stage flags, the intro task.
    fn flow_line(g: &Game) -> String {
        format!(
            "G0 {:x} lag {} fade {:x} mode {:x}.{:x} input {:x} scripted {:x} b4d5 {} b4e2 {} b4e3 {} intro {} {}.{} cam {:x}.{:x} | script id {} bd0 {:x} act {:x} timer {:x} ptr {:x} df {:x} taken {:?} b503 {} chosen {:x}",
            g.w.frame,
            g.flow.lag,
            g.fade.mask,
            g.flow.mode,
            g.flow.sub,
            g.w.input[0],
            g.w.scripted,
            g.spawn.g.b4d5,
            g.flow.b4e2,
            g.flow.b4e3,
            g.flow.intro.live,
            g.flow.intro.state,
            g.flow.intro.sub,
            g.cam.s2.mode,
            g.cam.s2.sub,
            g.spawn.g.b4de,
            g.players[0].bd0,
            g.players[0].sc.act,
            g.players[0].sc.timer,
            g.players[0].sc.ptr,
            g.flow.script.b4df,
            g.flow.script.taken,
            g.flow.script.b503,
            g.flow.chosen
        )
    }

    /// The state of the game in one line: the camera, the first hero, the live enemies (class, X, Z, mode), the clock.
    fn state_line(g: &Game) -> String {
        let live: Vec<String> = g.enemies.slots.iter().filter(|e| e.b.live != 0).map(|e| format!("{}@{:x}:{:x}:{:x}:m{:x}.{:x}.{:x}.{:x}h{:x}a{:x}t{:x}", e.b.k20 / 4, e.b.xi(), e.b.zi(), e.b.yi(), e.b.mode, e.b.sub, e.b.act, e.b.step, e.b.hp, e.b.airborne, e.m.b(0x7D))).collect();
        let props: Vec<String> = g.props.slots.iter().filter(|p| p.b.live != 0).map(|p| format!("{}:m{:x}.{:x}", p.b.k20 / 4, p.b.mode, p.b.sub)).collect();
        let p = &g.players[0];
        format!(
            "episode {} area {} cam x {:x} y {:x} mode {:x}.{:x} lock {} go {} scripted {:x} | p0 x {:x} z {:x} y {:x} hp {} live {} st {:x} wall {:x} vx {:x} | alive {} wave {} enemies {:?} props {:?} time {:x}:{:02x}",
            g.w.stage + 1,
            g.cam.s2.area,
            g.cam.s2.cam_x,
            g.cam.s2.cam_y,
            g.cam.s2.mode,
            g.cam.s2.sub,
            g.cam.s2.lock78,
            g.cam.s2.go,
            g.w.scripted,
            p.b.xi(),
            p.b.zi(),
            p.b.yi(),
            p.b.hp,
            p.b.live,
            p.b.state(),
            p.b.wall,
            p.b.vx,
            g.spawn.g.alive,
            g.spawn.g.wave_timer,
            live,
            props,
            g.rank.time_min,
            g.rank.time_sec
        )
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

    /// M3-29: the bot plays from power on (coin, start, Jack) through the eight episodes to the ending (mode 0x1C) and back to the attract
    /// loop, the clock topped up and the heroes unhurt (god mode, as `--autoplay`). Every episode and every area of it is entered in order.
    #[test]
    fn the_bot_plays_from_power_on_through_all_eight_episodes_to_the_ending() {
        let Some(a) = rom() else { return eprintln!("test skipped: dino.zip is absent") };
        let mut g = Game::new(a);
        g.flow_boot();
        let (mut debug, mut bot) = (Debug::new(false), Bot::default());
        let mut seen: Vec<(u8, u8)> = Vec::new();
        for tick in 0..600_000 {
            let f = bot.frame(&mut g, tick, &mut debug);
            g.w.input = f.players;
            g.w.system = f.system | 0x08;
            g.w.coins = f.coins;
            debug.before_tick(&mut g);
            g.tick();
            debug.after_tick(&mut g);
            bot.report(&g, tick);
            if g.flow.in_game != 0 && g.flow.mode == 0xC && seen.last() != Some(&(g.w.stage, g.cam.s2.area)) {
                seen.push((g.w.stage, g.cam.s2.area));
            }
            if bot.done() {
                break;
            }
        }
        assert!(bot.done(), "the bot did not reach the ending and the attract loop (areas seen: {seen:?})");
        let want: Vec<(u8, u8)> = vec![
            (0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (2, 0), (3, 0), (3, 1), (3, 2), (4, 0), (4, 1),
            (5, 0), (5, 1), (5, 2), (6, 0), (6, 1), (6, 2), (6, 3), (7, 0), (7, 1), (7, 2), (7, 3),
        ];
        assert_eq!(seen, want, "the stages and areas in the order of the game");
    }
}
