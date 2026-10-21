//! dino-game: native Rust port of the game logic, rendered on the GPU.
//!
//! Usage: `cargo run -p dino-game -- [--rom path/to/dino.zip] [--stage N] [--chars 0,2]
//!         [--config FILE] [--print-config]
//!         [--input SCRIPT | --input-file PATH] [--viewer] [--view char,set,anim]
//!         [--screenshot out.png --frames N] [--screenshots F1,F2 --shot-dir DIR]
//!         [--headless | --windowed] [--fullscreen] [--mute] [--load-state FILE] [--trace FILE --frames N]
//!         [--step-from MAME.trc] [--follow-rng MAME.trc] [--lagsync MAME.trc]
//!         [--poke FRAME:ADDR:SIZE:VALUE]... [--poke-file FILE]... [--poke-clock power|game]
//!         [--episode 1..8 | --stage 0..7] [--area 0..3] [--camx X] [--autoplay] [--debug-keys]
//!         [--jump NAME] [--list-jumps] [--test]`
//! Without `--stage`, `--episode` and `--chars` the game starts from power on (the boot screens and the attract
//! mode); `--stage N` / `--episode N` / `--chars` start directly in a stage (development). `--area` and `--camx` (hex or
//! decimal) start that area with the camera at X. `--autoplay` is the bot of `autoplay.rs`: from power on through the eight episodes
//! to the ending and the attract loop (mode changes on stderr; `--headless --autoplay` without `--frames` stops there, about 250,000
//! ticks in a second; the env vars `DINO_AUTO_MORTAL`, `DINO_AUTO_DIAG` and `DINO_AUTO_RANGE` are described in `autoplay.rs`); `--debug-keys` turns on the debug keys of `debug.rs` in windowed runs.
//! `--config FILE` reads the settings file (`key = value` lines: difficulty, coin mode, free play ...; without the
//! option `dino.cfg` next to the ROM is read, see `game/flow/config.rs`), `--print-config` prints the factory settings file.
//! `--mute` (windowed runs) opens no audio device: the game runs silent, the same as `sound = off` in `dino.cfg`.
//! `--fullscreen` starts the game in fullscreen mode (F11 or Alt+Enter / Option+Enter toggles fullscreen).
//! `--jump NAME` starts at a named point (an episode start or an area start; `--list-jumps` prints them):
//! the same start as `--episode E --area A --camx X`, and it takes `--chars`. It cannot be combined with those flags.
//! `--test` is a windowed run with the debug keys on and the test menu open (`menu.rs`); `--test --headless --screenshots F --shot-dir DIR`
//! draws the menu in the picture. With the debug keys on, F1 opens and closes the menu at any time (without them F1 shows the hit boxes).
//! The menu: Up / Down choose, Enter jumps, Left / Right change the hero and the players, Esc or F1 closes it; the game is paused while it is open.
//! `--poke` / `--poke-file` write RAM values into the running game at a frame (the lines of MAME's
//! `DINO_POKES`: `frame hexaddr size value`, power frames unless `--poke-clock game`; see
//! `harness.rs`, `PokeSet`). An address the port has no field for is an error.
//! See `input.rs` for keys and the input script format, `harness.rs` and `re/harness/README.md`
//! for `--load-state` (start from a MAME snapshot) and `--trace` (headless run for the diff). `--keep-dip` (with `--trace`): a loaded
//! snapshot keeps the settings of `--config` (G.$777A..C of the port) instead of the bytes of the snapshot.

mod audio;
mod autoplay;
mod planner;
mod debug;
mod font5x7;
mod game;
mod harness;
mod headless;
mod input;
mod jumps;
mod menu;
mod present;
mod render;
mod viewer;

use dino_extract::rom::Rom;
use macroquad::prelude::*;
use render::Gpu;

/// CPS1 master clock and raster dimensions (MAME `mame -listxml dino`, `re/specs/sound/SPEC.md` section 2).
/// Pixel clock: 8 MHz. Htotal: 512 dots. Vtotal: 262 lines.
/// Refresh rate = 8,000,000 / (512 * 262) = 59.63740458015267 Hz (~59.64 Hz).
/// Frame duration = (512 * 262) / 8,000,000 s = 0.016768 s = 16.768 ms = 16,768,000 ns.
pub const CPS1_PIXCLOCK: f64 = 8_000_000.0;
pub const CPS1_HTOTAL: f64 = 512.0;
pub const CPS1_VTOTAL: f64 = 262.0;
pub const VIDEO_HZ: f64 = CPS1_PIXCLOCK / (CPS1_HTOTAL * CPS1_VTOTAL);
pub const FRAME_TIME_SECS: f64 = (CPS1_HTOTAL * CPS1_VTOTAL) / CPS1_PIXCLOCK;
pub const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos(16_768_000);

/// The length of an autoplay run that has no `--frames`: far more than one game from power on to the ending (about 240,000 ticks).
const AUTOPLAY_CAP: u32 = 1_000_000;

fn window_conf(fullscreen: bool) -> Conf {
    Conf {
        window_title: "dino-rs".to_string(),
        window_width: 384 * 3,
        window_height: 224 * 3,
        high_dpi: false,
        fullscreen,
        window_resizable: true,
        ..Default::default()
    }
}

struct Args {
    shots: Vec<(u32, String)>,
    headless: bool,
    frames: u32,
    /// `--frames` was given: the run is exactly that long.
    frames_given: bool,
    view: Option<(usize, usize, u16)>,
    viewer: bool,
    /// `--stage`, `--episode`, `--area`, `--camx` or `--chars` start directly in a stage; without them the game boots (power on).
    direct: bool,
    stage: u8,
    area: u8,
    camx: u16,
    /// `--area` above 0: the area still to reach and the camera X to jump to there (see `tick_game`).
    goto: Option<(u8, u16)>,
    chars: Vec<u8>,
    autoplay: Option<autoplay::Bot>,
    /// `--autoplay smart`: the planning bot of `planner.rs`.
    smart: Option<planner::Smart>,
    debug: debug::Debug,
    config: Option<String>,
    script: Option<input::Script>,
    trace: Option<String>,
    load_state: Option<String>,
    step_from: Option<String>,
    follow_rng: Option<String>,
    /// `--lagsync MAME.trc`: a free run that resyncs to MAME's state after every lag record (`harness::run`).
    lagsync: Option<String>,
    pokes: harness::PokeSet,
    /// `--mute`: no audio device (windowed runs only).
    mute: bool,
    /// `--fullscreen`: start in fullscreen mode.
    fullscreen: bool,
    /// `--test`: the debug keys are on and the test menu is open at start.
    test: bool,
}

fn args() -> Args {
    let a: Vec<String> = std::env::args().collect();
    let value = |flag: &str| a.iter().position(|x| x == flag).and_then(|i| a.get(i + 1)).cloned();
    let view = value("--view").and_then(|v| {
        let n: Vec<u16> = v.split(',').filter_map(|x| x.parse().ok()).collect();
        (n.len() == 3).then(|| (n[0] as usize, n[1] as usize, n[2]))
    });
    let frames = value("--frames").map(|v| v.parse::<u32>().unwrap_or_else(|_| fail("invalid --frames"))).unwrap_or(2);
    let mut shots = Vec::new();
    if let Some(list) = value("--screenshots") {
        let dir = value("--shot-dir").unwrap_or_else(|| fail("--screenshots requires --shot-dir"));
        for f in list.split(',') {
            let f = f.parse::<u32>().ok().filter(|&f| f > 0).unwrap_or_else(|| fail("--screenshots needs positive frame numbers"));
            shots.push((f, std::path::Path::new(&dir).join(format!("f{f:05}.png")).to_string_lossy().into_owned()));
        }
    }
    if let Some(path) = value("--screenshot") {
        if frames == 0 { fail("--screenshot needs --frames greater than zero"); }
        shots.push((frames, path));
    }
    shots.sort_by_key(|s| s.0);
    let num = |v: String| -> Option<u32> {
        match v.strip_prefix("0x") {
            Some(h) => u32::from_str_radix(h, 16).ok(),
            None => v.parse().ok(),
        }
    };
    let jump = value("--jump").map(|n| {
        jumps::find(&n).unwrap_or_else(|| fail(&format!("unknown jump point `{n}`; the points are: {}", jumps::names())))
    });
    if jump.is_some() && ["--episode", "--stage", "--area", "--camx"].iter().any(|f| value(f).is_some()) {
        fail("--jump cannot be combined with --episode, --stage, --area or --camx");
    }
    let episode = value("--episode").map(|v| v.parse::<u8>().ok().filter(|e| (1..=8).contains(e)).unwrap_or_else(|| fail("--episode must be 1 to 8")));
    if episode.is_some() && value("--stage").is_some() {
        fail("--episode and --stage cannot be combined");
    }
    let stage = episode.map(|e| e - 1).or_else(|| value("--stage").and_then(|v| v.parse().ok())).unwrap_or(0);
    let area = value("--area").map(|v| v.parse::<u8>().ok().filter(|a| *a < 4).unwrap_or_else(|| fail("--area must be 0 to 3")));
    if area.is_some_and(|a| a > 0) && stage == 2 {
        fail("--area above 0 is not meaningful in Episode 3 (its camera has one area): use --camx");
    }
    let camx = value("--camx").map(|v| num(v).filter(|x| *x <= 0xFFFF).unwrap_or_else(|| fail("--camx must be 0 to 0xFFFF")));
    // `--jump NAME` is the same start as the flags it stands for (the jump list, `jumps.rs`).
    let (stage, area, camx) = match jump {
        Some(j) => (j.episode - 1, Some(j.area), Some(j.camx as u32)),
        None => (stage, area, camx),
    };
    let direct = jump.is_some() || ["--stage", "--chars", "--episode", "--area", "--camx"].iter().any(|f| value(f).is_some());
    let test = a.iter().any(|x| x == "--test");
    let viewer = view.is_some() || a.iter().any(|x| x == "--viewer");
    let smart = a.windows(2).any(|w| w[0] == "--autoplay" && w[1] == "smart");
    let explicit_headless = a.iter().any(|x| x == "--headless");
    let windowed = a.iter().any(|x| x == "--windowed");
    if explicit_headless && (viewer || windowed) {
        fail("--headless cannot be combined with --viewer, --view or --windowed");
    }
    let headless = explicit_headless || (!shots.is_empty() && !windowed && !viewer);
    let frames = shots.iter().map(|s| s.0).max().unwrap_or(frames);
    // `--headless --autoplay` without `--frames` plays until the ending and the return to the attract loop (at most `AUTOPLAY_CAP` ticks)
    let frames = if a.iter().any(|x| x == "--autoplay") && value("--frames").is_none() && shots.is_empty() { AUTOPLAY_CAP } else { frames };
    Args {
        shots,
        headless,
        frames,
        frames_given: value("--frames").is_some(),
        viewer,
        view,
        direct,
        stage,
        area: area.unwrap_or(0),
        camx: camx.unwrap_or(0) as u16,
        goto: area.filter(|&a| a > 0).map(|a| (a, camx.unwrap_or(0) as u16)),
        autoplay: (a.iter().any(|x| x == "--autoplay") && !smart).then(autoplay::Bot::default),
        smart: smart.then(|| {
            let chars: Vec<u8> = value("--chars").map(|v| v.split(',').filter_map(|c| c.parse().ok()).collect()).unwrap_or(vec![0]);
            planner::Smart::new(&chars, value("--smart-continues").and_then(|v| v.parse().ok()).unwrap_or(3))
        }),
        debug: debug::Debug::new(a.iter().any(|x| x == "--debug-keys") || test),
        test,
        chars: value("--chars").map(|v| v.split(',').filter_map(|c| c.parse().ok()).collect()).unwrap_or(vec![0]),
        config: value("--config"),
        script: match value("--input-file") {
            Some(p) => Some(input::Script::load(&p).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            })),
            None => value("--input").map(|v| input::Script::parse(&v)),
        },
        trace: value("--trace"),
        load_state: value("--load-state"),
        step_from: value("--step-from"),
        follow_rng: value("--follow-rng"),
        lagsync: value("--lagsync"),
        pokes: pokes(&a),
        mute: a.iter().any(|x| x == "--mute"),
        fullscreen: a.iter().any(|x| x == "--fullscreen"),
    }
}

fn fail(message: &str) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}

/// `--poke F:ADDR:SIZE:VAL` (repeatable), `--poke-file FILE` (repeatable), `--poke-clock power|game`
/// (the clock of `--poke` and of the files that have no `#!clock` line; default `power`).
fn pokes(a: &[String]) -> harness::PokeSet {
    let fail = |e: String| -> ! {
        eprintln!("error: {e}");
        std::process::exit(1);
    };
    let clock = match a.iter().position(|x| x == "--poke-clock").and_then(|i| a.get(i + 1)).map(String::as_str) {
        None | Some("power") => harness::PokeClock::Power,
        Some("game") => harness::PokeClock::Game,
        Some(other) => fail(format!("--poke-clock must be power or game, not `{other}`")),
    };
    let mut set = harness::PokeSet::default();
    for (i, x) in a.iter().enumerate() {
        let Some(v) = a.get(i + 1) else { continue };
        let r = match x.as_str() {
            "--poke" => set.add_spec(v, clock),
            "--poke-file" => set.add_file(v, clock),
            _ => Ok(()),
        };
        r.unwrap_or_else(|e| fail(e));
    }
    set
}

/// Extracts the assets. With `--load-state`, the frame records the snapshot's objects point at
/// are decoded too (`harness::scan_frames`), so the game never needs to read ROM bytes itself.
fn load_assets(load_state: Option<&str>, step_from: Option<&str>) -> &'static dino_assets::AssetPack {
    let extra = step_from
        .map(|p| harness::scan_frames(p, true))
        .or_else(|| load_state.map(|p| harness::scan_frames(p, false)))
        .and_then(|r| r.ok())
        .unwrap_or_default();
    match Rom::locate().and_then(|p| dino_extract::extract_with(&p, &extra)) {
        Ok(a) => Box::leak(Box::new(a)),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

/// The settings file (`--config`, `$DINO_CONFIG`, `dino.cfg` next to the ROM; none = factory values), see `game/flow/config.rs`.
fn load_config(path: Option<&str>) -> game::flow::Config {
    game::flow::Config::locate(path).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    })
}

/// The usage text: the `//!` lines at the top of this file, so the help cannot drift from the code.
fn usage() -> String {
    include_str!("main.rs")
        .lines()
        .filter_map(|l| l.strip_prefix("//!"))
        .map(|l| l.strip_prefix(' ').unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!("{}", usage());
        return;
    }
    if std::env::args().any(|a| a == "--list-jumps") {
        print!("{}", jumps::list_text());
        return;
    }
    if std::env::args().any(|a| a == "--print-config") {
        print!("{}", game::flow::Config::default().to_text());
        return;
    }
    let args = args();
    let config = load_config(args.config.as_deref());
    if let Some(trace) = &args.trace {
        // Headless harness run: no window.
        let frames = std::env::args().any(|a| a == "--frames").then_some(args.frames).unwrap_or(600);
        let assets = load_assets(args.load_state.as_deref(), args.step_from.as_deref());
        let (load, step) = (args.load_state.as_deref(), args.step_from.as_deref());
        let follow = args.follow_rng.as_deref();
        let lagsync = args.lagsync.as_deref();
        let start = harness::Start { boot: !args.direct, stage: args.stage, area: args.area, camx: args.camx, chars: &args.chars };
        game::trace::set_keep_dip(std::env::args().any(|a| a == "--keep-dip"));
        let r = harness::run(assets, trace, load, step, follow, lagsync, start, config, args.script.as_ref(), args.pokes, frames);
        if let Err(e) = r {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    for (_, path) in &args.shots {
        if let Some(parent) = std::path::Path::new(path).parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| fail(&format!("{}: {e}", parent.display())));
        }
    }
    if args.headless {
        run_headless(args, config);
        return;
    }
    macroquad::Window::from_config(window_conf(args.fullscreen), windowed(args, config));
}

/// The start of a direct run (`--jump`, `--episode`, `--stage`, `--area`, `--camx`, `--chars`) and of a menu jump: a new game in
/// the stage with the camera X and the heroes, from the settings of `config`. Returns the game and the area still to reach
/// (`tick_game` walks there); a stage start always begins in area 0.
fn enter_target(assets: &'static dino_assets::AssetPack, config: &game::flow::Config, stage: u8, area: u8, camx: u16, chars: &[u8]) -> (game::Game, Option<(u8, u16)>) {
    let mut game = game::Game::new(assets);
    game.flow_apply_config(config.clone());
    game.start_stage_at(stage, 0, if area > 0 { 0 } else { camx }, chars);
    (game, (area > 0).then_some((area, camx)))
}

fn start_game(assets: &'static dino_assets::AssetPack, args: &mut Args, config: game::flow::Config) -> game::Game {
    let mut game = if args.direct && args.load_state.is_none() {
        enter_target(assets, &config, args.stage, args.area, args.camx, &args.chars).0
    } else {
        let mut game = game::Game::new(assets);
        game.flow_apply_config(config);
        match &args.load_state {
            Some(p) => match harness::load_state(&mut game, p) {
                Ok(warnings) => warnings.iter().for_each(|w| eprintln!("load-state: {w}")),
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            },
            None => game.flow_boot(), // power on: the boot screens, then the attract mode
        }
        game
    };
    // Pokes (`--poke`): the same clocks as a headless run, where tick n is trace record n.

    let poke_fail = |e: String| -> ! {
        eprintln!("error: {e}");
        std::process::exit(1);
    };
    args.pokes.check(&mut game).unwrap_or_else(|e| poke_fail(e));
    args.pokes.apply(&mut game, -1, true).unwrap_or_else(|e| poke_fail(e));
    game
}

/// What the combo toggle (Q) reads of player 1's hero, frame by frame (`input::Hero`).
#[derive(Default)]
struct HeroWatch {
    /// The enemy slots (pool A) the hero's last landed hit struck (in hit stop with the hero as their attacker `+$68`).
    struck: Vec<usize>,
}

impl HeroWatch {
    /// Before the frame's tick.
    fn read(&mut self, game: &game::Game) -> input::Hero {
        let Some(p) = game.players.first() else { return input::Hero::default() };
        let me = game::Game::player_handle(0);
        let enemy = |i: usize| game.enemies.slots.get(i).map(|e| &e.b).filter(|b| b.live != 0 && b.attacker == me);
        // a hit lands: the hero and its victims are in hit stop together (a dying enemy keeps its hit stop, so the list is taken only then)
        if p.b.hitstop != 0 {
            let now: Vec<_> = (0..game.enemies.slots.len()).filter(|&i| enemy(i).is_some_and(|b| b.hitstop != 0)).collect();
            if !now.is_empty() {
                self.struck = now;
            }
        } else if p.combo == 0 {
            self.struck.clear();
        }
        // in play (mode 0xC), not in a scripted scene, alive (`+$00` 1, or 2 in some attack frames) on the floor in the normal state
        // (mode 2 sub 0: not hurt, grabbing or grabbed) and without a weapon (a weapon swing keeps another value in the chain count `+$B2`,
        // the run swing of player_weapon.rs)
        let normal = game.flow.mode == 0xC
            && game.w.scripted == 0
            && p.b.live != 0
            && p.b.airborne == 0
            && p.b.mode == 2
            && p.b.sub == 0
            && p.item_type == 0;
        let state = p.b.state();
        let stand_or_walk = matches!(state, 0x0200_0000 | 0x0200_0002);
        // the hit confirm 0x19464 counts the hits that land in `+$B2` and a miss sets it back to 0 (0x19416, 0x19552); with 2 the next
        // press is the third hit (0x194D4 -> 0x1956C). The chain is open in the hit confirm (0x02000006) and, standing or walking, while
        // the chain timer `+$AE` runs (idle 0x1916E, walk 0x192DE). The second hit (0x0200_0008) with its hit stop set lands in this
        // tick (0x1953E): the hit confirm takes a press of the same tick into `+$AD` already
        let chain = (p.combo == 2 && (state == 0x0200_0006 || (stand_or_walk && p.combo_t != 0)))
            || (p.combo == 1 && state == 0x0200_0008 && p.b.hitstop != 0);
        // an enemy the chain struck is still alive (when the second hit killed it, the third press stays a plain hit)
        let alive = self.struck.iter().any(|&i| enemy(i).is_some_and(|b| b.hp != 0));
        input::Hero {
            third_next: normal && chain && alive,
            // standing or walking and free to walk, as only the walk state (0x192DE) reads the command flag of 0x6AC8: the idle state keeps
            // the hero from walking while `+$B3` is set or the chain timer is 0x24 or more, and a buffered press `+$AD` continues the chain
            ready: normal && stand_or_walk && p.lock == 0 && p.combo_t < 0x24 && p.atk_buf == 0,
        }
    }
}

fn tick_game(game: &mut game::Game, args: &mut Args, tick: u32, mut f: input::Frame, audio: &mut Option<audio::Audio>) {
    // `--area`: the game's own end of area (camera mode 1 sub 4, the debug key F6) once the hero has control, area after
    // area like the probe `jump`, then `--camx`. Episode 3 has one area (`--area` above 0 is refused at the start).
    if let Some((area, camx)) = args.goto {
        if game.flow.mode == 0xC && game.w.scripted == 0 && game.cam.s2.mode == 0 {
            if game.cam.s2.area < area {
                debug::next_area(game);
            } else {
                if camx != 0 {
                    game.flow_jump_camera(camx);
                }
                args.goto = None;
            }
        }
    }
    if let Some(bot) = &mut args.autoplay {
        f = bot.frame(game, tick, &mut args.debug);
    }
    if let Some(bot) = &mut args.smart {
        f = bot.frame(game, tick);
    }
    if let Some(script) = &args.script {
        let s = script.at(tick);
        for p in 0..3 { f.players[p] |= s.players[p]; }
        f.system |= s.system;
        f.coins |= s.coins;
    }
    game.w.input = f.players;
    game.w.system = f.system | 0x08; // 0x738 quirk, see harness.rs
    game.w.coins = f.coins;
    args.pokes.apply(game, tick as i32, false).unwrap_or_else(|e| fail(&e));
    args.debug.before_tick(game);
    game.tick();
    if let Some(a) = audio {
        a.frame(game); // reads the requests of this frame; the game does not see the sound side
    }
    args.debug.after_tick(game);
    if let Some(bot) = &mut args.autoplay {
        bot.report(game, tick);
    }
    if let Some(bot) = &mut args.smart {
        bot.report(game, tick);
    }
    args.pokes.apply(game, tick as i32, true).unwrap_or_else(|e| fail(&e));
}

fn run_headless(mut args: Args, config: game::flow::Config) {
    let assets = load_assets(args.load_state.as_deref(), None);
    let mut game = start_game(assets, &mut args, config);
    let mut cpu = headless::Cpu::new(assets);
    // `--test`: the menu is open, so the game does not run (headless runs have no keys, the picture shows the menu over the start)
    let mut menu = menu::Menu::new(args.chars[0], args.chars.len() as u8, args.test);
    for tick in 0..args.frames {
        if !menu.open {
            tick_game(&mut game, &mut args, tick, input::Frame::default(), &mut None);
        }
        if (args.autoplay.as_ref().is_some_and(|b| b.done()) || args.smart.as_ref().is_some_and(|b| b.done())) && args.shots.is_empty() && !args.frames_given {
            break;
        }
        if args.shots.iter().any(|s| s.0 == tick + 1) {
            cpu.draw(&mut game);
            if menu.open {
                cpu.menu(&mut menu, args.debug.is_god());
            }
            for (_, path) in args.shots.iter().filter(|s| s.0 == tick + 1) {
                cpu.save(path).unwrap_or_else(|e| fail(&e));
            }
        }
    }
    if let Some(bot) = &mut args.smart {
        bot.summary(args.frames.saturating_sub(1));
    }
}

async fn windowed(mut args: Args, config: game::flow::Config) {
    let assets = load_assets(args.load_state.as_deref(), None);
    let mut gpu = Gpu::new(assets);
    // The audio device opens only here, in a windowed run (never headless, harness or tests).
    let mut audio = audio::Audio::start(&assets.sound, config.qsound, args.mute || !config.sound);

    let mut viewer = args.viewer.then(|| viewer::Viewer::new(assets, args.view));
    let mut inputs = input::Inputs::new();
    let mut hero_watch = HeroWatch::default();
    let mut game = start_game(assets, &mut args, config.clone());
    // The test menu (`menu.rs`): open at start with `--test`, F1 opens and closes it with the debug keys on. No menu in viewer runs.
    let mut menu = menu::Menu::new(args.chars[0], args.chars.len() as u8, args.test);
    let mut tick_no = 0u32;
    if viewer.is_some() {
        gpu.upload_palettes(&viewer::Viewer::palettes(assets).words);
    }

    let mut is_fullscreen = args.fullscreen;
    let mut user_paused = false;
    let mut frame = 0u32;
    let screenshot_run = !args.shots.is_empty();
    let mut next_frame_time = std::time::Instant::now();
    loop {
        // Fullscreen toggle: F11 or Alt+Enter / Option+Enter
        let alt_enter = (is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt))
            && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter));
        let f11 = is_key_pressed(KeyCode::F11);
        if alt_enter || f11 {
            is_fullscreen = !is_fullscreen;
            macroquad::window::set_fullscreen(is_fullscreen);
        }

        let menu_ok = viewer.is_none() && args.debug.enabled;
        if menu_ok && is_key_pressed(KeyCode::F1) {
            menu.toggle();
        }
        if menu.open && menu_ok {
            // Menu keys: the game is paused, the picture stays
            if is_key_pressed(KeyCode::Escape) {
                menu.open = false;
            }
            if is_key_pressed(KeyCode::Up) {
                menu.up();
            }
            if is_key_pressed(KeyCode::Down) {
                menu.down();
            }
            if is_key_pressed(KeyCode::Left) {
                menu.side(-1);
            }
            if is_key_pressed(KeyCode::Right) {
                menu.side(1);
            }
            if (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter)) && !alt_enter {
                match menu.enter() {
                    menu::Enter::Jump(j) => {
                        // the same start as the command line (`enter_target`), in place: a new game object, the sound goes on
                        let (next, goto) = enter_target(assets, &config, j.episode - 1, j.area, j.camx, &menu.chars());
                        game = next;
                        args.goto = goto;
                        menu.open = false;
                    }
                    menu::Enter::God => args.debug.toggle_god(&game),
                    menu::Enter::Nothing => {}
                }
            }
        }
        // P: pause and go on (a feature of the port, the arcade has no pause). The picture stays, the sound stops.
        if viewer.is_none() && !(menu.open && menu_ok) && is_key_pressed(KeyCode::P) {
            user_paused = !user_paused;
        }
        let paused = (menu.open && menu_ok) || user_paused;
        if menu.open && menu_ok {
            // the game stands still; the sound keeps running at the frame rate (no game tick, no requests)
            if let Some(a) = &mut audio {
                a.frame(&mut game);
            }
        }

        // Screenshots fast-forward ticks up to the target shot frame so export is instantaneous.
        // In normal play, we keep exactly one fixed logic step per paced video frame.
        let ticks = if screenshot_run {
            args.shots.iter().find(|s| s.0 > frame).map_or(1, |s| s.0 - frame)
        } else if paused {
            0
        } else {
            1
        };

        for _ in 0..ticks {
            match &mut viewer {
                Some(v) => v.update(assets),
                None => {
                    let f = if screenshot_run { input::Frame::default() } else { inputs.poll(hero_watch.read(&game)) };
                    tick_game(&mut game, &mut args, tick_no, f, &mut audio);
                    tick_no += 1;
                }
            }
        }
        // F1 is the hit box overlay when the debug keys are off (with them on, F1 is the menu)
        if is_key_pressed(KeyCode::F1) && !args.debug.enabled {
            game.sprites.debug_boxes = !game.sprites.debug_boxes; // hit box overlay
        }
        if viewer.is_none() {
            args.debug.keys(&mut game);
        }
        match &mut viewer {
            Some(v) => v.draw(assets, &mut gpu),
            None => present::draw(&mut game, &mut gpu),
        }
        gpu.present();
        if let Some(v) = &viewer {
            draw_text(&v.label, 8.0, 20.0, 20.0, WHITE);
        }
        if menu.open && menu_ok {
            menu::draw_window(&mut menu, args.debug.is_god());
        }
        if user_paused {
            let (w, h) = (screen_width(), screen_height());
            let size = (h / 10.0).max(24.0);
            let dim = measure_text("PAUSED", None, size as u16, 1.0);
            draw_text("PAUSED", (w - dim.width) / 2.0, h / 2.0, size, WHITE);
        }
        if inputs.p1.combo.on {
            draw_text("COMBO", 8.0, screen_height() - 8.0, (screen_height() / 30.0).max(14.0), YELLOW);
        }

        frame += if screenshot_run { ticks } else { 1 };
        for (_, path) in args.shots.iter().filter(|s| s.0 == frame) {
            get_screen_data().export_png(path);
        }
        if (screenshot_run || args.frames_given) && frame >= args.frames {
            break;
        }

        next_frame().await;

        // Pacing at the original frame rate (~59.64 Hz, 16.768 ms per frame):
        if !screenshot_run {
            next_frame_time += FRAME_DURATION;
            let now = std::time::Instant::now();
            if now < next_frame_time {
                let remaining = next_frame_time - now;
                if remaining > std::time::Duration::from_millis(1) {
                    std::thread::sleep(remaining - std::time::Duration::from_millis(1));
                }
                while std::time::Instant::now() < next_frame_time {
                    std::hint::spin_loop();
                }
            } else if now > next_frame_time + FRAME_DURATION * 4 {
                next_frame_time = now;
            }
        }
    }
}
