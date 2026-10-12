//! dino-game: native Rust port of the game logic, rendered on the GPU.
//!
//! Usage: `cargo run -p dino-game -- [--rom path/to/dino.zip] [--stage N] [--chars 0,2]
//!         [--config FILE] [--print-config]
//!         [--input SCRIPT | --input-file PATH] [--viewer] [--view char,set,anim]
//!         [--screenshot out.png --frames N] [--load-state FILE] [--trace FILE --frames N]
//!         [--step-from MAME.trc] [--follow-rng MAME.trc]`
//! Without `--stage` and `--chars` the game starts from power on (the boot screens and the attract
//! mode); `--stage N` / `--chars` start directly in a stage (development).
//! `--config FILE` reads the settings (difficulty, coin mode, free play ...; see `game/flow/config.rs`),
//! `--print-config` prints the factory settings file.
//! See `input.rs` for keys and the input script format, `harness.rs` and `re/harness/README.md`
//! for `--load-state` (start from a MAME snapshot) and `--trace` (headless run for the diff).

mod game;
mod harness;
mod input;
mod present;
mod render;
mod viewer;

use dino_extract::rom::Rom;
use macroquad::prelude::*;
use render::Gpu;

/// The original runs one game tick per video frame (about 59.6 Hz). We run a fixed 60 Hz tick
/// so every rule that counts frames behaves exactly as in the original.
const TICK: f64 = 1.0 / 60.0;

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
    screenshot: Option<String>,
    frames: u32,
    view: Option<(usize, usize, u16)>,
    viewer: bool,
    /// `--stage` and `--chars` start directly in a stage; without both the game boots (power on).
    direct: bool,
    stage: u8,
    chars: Vec<u8>,
    config: Option<String>,
    script: Option<input::Script>,
    trace: Option<String>,
    load_state: Option<String>,
    step_from: Option<String>,
    follow_rng: Option<String>,
}

fn args() -> Args {
    let a: Vec<String> = std::env::args().collect();
    let value = |flag: &str| a.iter().position(|x| x == flag).and_then(|i| a.get(i + 1)).cloned();
    let view = value("--view").and_then(|v| {
        let n: Vec<u16> = v.split(',').filter_map(|x| x.parse().ok()).collect();
        (n.len() == 3).then(|| (n[0] as usize, n[1] as usize, n[2]))
    });
    Args {
        screenshot: value("--screenshot"),
        frames: value("--frames").and_then(|v| v.parse().ok()).unwrap_or(2),
        viewer: view.is_some() || a.iter().any(|x| x == "--viewer"),
        view,
        direct: value("--stage").is_some() || value("--chars").is_some(),
        stage: value("--stage").and_then(|v| v.parse().ok()).unwrap_or(0),
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
    }
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

/// The settings file (`--config`, `$DINO_CONFIG`, `./dino-config.json`; none = factory values).
fn load_config(path: Option<&str>) -> game::flow::Config {
    game::flow::Config::locate(path).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    })
}

fn main() {
    if std::env::args().any(|a| a == "--print-config") {
        print!("{}", game::flow::Config::default().to_json());
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
        let r = harness::run(assets, trace, load, step, follow, start, config, args.script.as_ref(), frames);
        if let Err(e) = r {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    macroquad::Window::from_config(window_conf(), windowed(args, config));
}

async fn windowed(args: Args, config: game::flow::Config) {
    let assets = load_assets(args.load_state.as_deref(), None);
    let mut gpu = Gpu::new(assets);

    let mut viewer = args.viewer.then(|| viewer::Viewer::new(assets, args.view));
    let mut inputs = input::Inputs::new();
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
        None if args.direct => game.start_stage(args.stage, &args.chars),
        None => game.flow_boot(), // power on: the boot screens, then the attract mode
    }
    let mut tick_no = 0u32;
    if viewer.is_some() {
        gpu.upload_palettes(&viewer::Viewer::palettes(assets).words);
    }

    let mut acc = 0.0;
    let mut frame = 0u32;
    loop {
        acc += get_frame_time() as f64;
        // Screenshots run exactly one tick per rendered frame, so they are reproducible.
        let mut ticks = if args.screenshot.is_some() { 1 } else { 0 };
        while args.screenshot.is_none() && acc >= TICK && ticks < 4 {
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
                    let mut f = inputs.poll();
                    if let Some(script) = &args.script {
                        let s = script.at(tick_no);
                        for p in 0..3 {
                            f.players[p] |= s.players[p];
                        }
                        f.system |= s.system;
                        f.coins |= s.coins;
                    }
                    game.w.input = f.players;
                    game.w.system = f.system | 0x08; // 0x738 quirk, see harness.rs
                    game.w.coins = f.coins;
                    game.tick();
                    tick_no += 1;
                }
            }
        }
        if is_key_pressed(KeyCode::F1) {
            game.sprites.debug_boxes = !game.sprites.debug_boxes; // hit box overlay
        }
        match &mut viewer {
            Some(v) => v.draw(assets, &mut gpu),
            None => present::draw(&mut game, &mut gpu),
        }
        gpu.present();
        if let Some(v) = &viewer {
            draw_text(&v.label, 8.0, 20.0, 20.0, WHITE);
        }

        frame += 1;
        if let Some(path) = &args.screenshot
            && frame >= args.frames
        {
            get_screen_data().export_png(path);
            break;
        }
        next_frame().await;
    }
}
