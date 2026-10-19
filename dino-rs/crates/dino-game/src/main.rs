//! dino-game: native Rust port of the game logic, rendered on the GPU.
//!
//! Usage: `cargo run -p dino-game -- [--rom path/to/dino.zip] [--stage N] [--chars 0,2]
//!         [--config FILE] [--print-config]
//!         [--input SCRIPT | --input-file PATH] [--viewer] [--view char,set,anim]
//!         [--screenshot out.png --frames N] [--screenshots F1,F2 --shot-dir DIR]
//!         [--headless | --windowed] [--mute] [--load-state FILE] [--trace FILE --frames N]
//!         [--step-from MAME.trc] [--follow-rng MAME.trc]
//!         [--poke FRAME:ADDR:SIZE:VALUE]... [--poke-file FILE]... [--poke-clock power|game]
//!         [--episode 1..8 | --stage 0..7] [--area 0..3] [--camx X] [--autoplay] [--debug-keys] [--list-placeholders]
//!         [--jump NAME] [--list-jumps] [--test]`
//! Without `--stage`, `--episode` and `--chars` the game starts from power on (the boot screens and the attract
//! mode); `--stage N` / `--episode N` / `--chars` start directly in a stage (development). `--area` and `--camx` (hex or
//! decimal) start that area with the camera at X. `--autoplay` is the bot of `autoplay.rs`: from power on through the eight episodes
//! to the ending and the attract loop (mode changes on stderr; `--headless --autoplay` without `--frames` stops there, about 250,000
//! ticks in a second; the env vars `DINO_AUTO_MORTAL`, `DINO_AUTO_DIAG` and `DINO_AUTO_RANGE` are described in `autoplay.rs`); `--debug-keys` turns on the debug keys of `debug.rs` in windowed runs.
//! `--config FILE` reads the settings file (`key = value` lines: difficulty, coin mode, free play ...; without the
//! option `dino.cfg` next to the ROM is read, see `game/flow/config.rs`), `--print-config` prints the factory settings file.
//! `--mute` (windowed runs) opens no audio device: the game runs silent, the same as `sound = off` in `dino.cfg`.
//! `--jump NAME` starts at a named point (episode start, area start or the place before a boss fight; `--list-jumps` prints them):
//! the same start as `--episode E --area A --camx X`, and it takes `--chars`. It cannot be combined with those flags.
//! `--test` is a windowed run with the debug keys on and the test menu open (`menu.rs`); `--test --headless --screenshots F --shot-dir DIR`
//! draws the menu in the picture. With the debug keys on, F1 opens and closes the menu at any time (without them F1 shows the hit boxes).
//! The menu: Up / Down choose, Enter jumps, Left / Right change the hero and the players, Esc or F1 closes it; the game is paused while it is open.
//! `--poke` / `--poke-file` write RAM values into the running game at a frame (the lines of MAME's
//! `DINO_POKES`: `frame hexaddr size value`, power frames unless `--poke-clock game`; see
//! `harness.rs`, `PokeSet`). An address the port has no field for is an error.
//! See `input.rs` for keys and the input script format, `harness.rs` and `re/harness/README.md`
//! for `--load-state` (start from a MAME snapshot) and `--trace` (headless run for the diff).

mod audio;
mod autoplay;
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

/// The original runs one game tick per video frame (about 59.6 Hz). We run a fixed 60 Hz tick
/// so every rule that counts frames behaves exactly as in the original.
const TICK: f64 = 1.0 / 60.0;

/// The length of an autoplay run that has no `--frames`: far more than one game from power on to the ending (about 240,000 ticks).
const AUTOPLAY_CAP: u32 = 1_000_000;

fn window_conf() -> Conf {
    Conf {
        window_title: "dino-rs".to_string(),
        window_width: 384 * 3,
        window_height: 224 * 3,
        high_dpi: false,
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
    debug: debug::Debug,
    config: Option<String>,
    script: Option<input::Script>,
    trace: Option<String>,
    load_state: Option<String>,
    step_from: Option<String>,
    follow_rng: Option<String>,
    pokes: harness::PokeSet,
    /// `--mute`: no audio device (windowed runs only).
    mute: bool,
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
        autoplay: a.iter().any(|x| x == "--autoplay").then(autoplay::Bot::default),
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
        pokes: pokes(&a),
        mute: a.iter().any(|x| x == "--mute"),
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
    if std::env::args().any(|a| a == "--list-placeholders") {
        print!("{}", game::placeholder::list_text());
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
        let start = harness::Start { boot: !args.direct, stage: args.stage, chars: &args.chars };
        let r = harness::run(assets, trace, load, step, follow, start, config, args.script.as_ref(), args.pokes, frames);
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
    macroquad::Window::from_config(window_conf(), windowed(args, config));
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
        if args.autoplay.as_ref().is_some_and(|b| b.done()) && args.shots.is_empty() && !args.frames_given {
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
}

async fn windowed(mut args: Args, config: game::flow::Config) {
    let assets = load_assets(args.load_state.as_deref(), None);
    let mut gpu = Gpu::new(assets);
    // The audio device opens only here, in a windowed run (never headless, harness or tests).
    let mut audio = audio::Audio::start(&assets.sound, config.qsound, args.mute || !config.sound);

    let mut viewer = args.viewer.then(|| viewer::Viewer::new(assets, args.view));
    let mut inputs = input::Inputs::new();
    let mut game = start_game(assets, &mut args, config.clone());
    // The test menu (`menu.rs`): open at start with `--test`, F1 opens and closes it with the debug keys on. No menu in viewer runs.
    let mut menu = menu::Menu::new(args.chars[0], args.chars.len() as u8, args.test);
    let mut tick_no = 0u32;
    if viewer.is_some() {
        gpu.upload_palettes(&viewer::Viewer::palettes(assets).words);
    }

    let mut acc = 0.0;
    let mut frame = 0u32;
    let screenshot_run = !args.shots.is_empty();
    loop {
        acc += get_frame_time() as f64;
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
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
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
        let paused = menu.open && menu_ok;
        if paused {
            // the game stands still; the sound keeps running at 60 Hz (no game tick, no requests)
            acc = acc.min(TICK * 4.0);
            while acc >= TICK {
                acc -= TICK;
                if let Some(a) = &mut audio {
                    a.frame(&mut game);
                }
            }
        }
        // Screenshots run exactly one tick per rendered frame, so they are reproducible.
        let mut ticks = if screenshot_run {
            args.shots.iter().find(|s| s.0 > frame).map_or(1, |s| s.0 - frame)
        } else { 0 };
        while !screenshot_run && !paused && acc >= TICK && ticks < 4 {
            acc -= TICK;
            ticks += 1;
        }
        if ticks == 4 {
            acc = 0.0;
        }
        for _ in 0..ticks {
            match &mut viewer {
                Some(v) => v.update(assets),
                None => {
                    let f = if screenshot_run { input::Frame::default() } else { inputs.poll() };
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

        frame += if screenshot_run { ticks } else { 1 };
        for (_, path) in args.shots.iter().filter(|s| s.0 == frame) {
            get_screen_data().export_png(path);
        }
        if screenshot_run && frame >= args.frames { break; }
        next_frame().await;
    }
}
