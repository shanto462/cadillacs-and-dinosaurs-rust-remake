//! dino-game: native Rust port of the game logic, rendered on the GPU.
//!
//! Usage: `cargo run -p dino-game -- [--rom path/to/dino.zip] [--stage N] [--chars 0,2]
//!         [--input SCRIPT | --input-file PATH] [--viewer] [--view char,set,anim]
//!         [--screenshot out.png --frames N] [--load-state FILE] [--trace FILE --frames N]`
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
    stage: u8,
    chars: Vec<u8>,
    script: Option<input::Script>,
    trace: Option<String>,
    load_state: Option<String>,
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
        stage: value("--stage").and_then(|v| v.parse().ok()).unwrap_or(0),
        chars: value("--chars").map(|v| v.split(',').filter_map(|c| c.parse().ok()).collect()).unwrap_or(vec![0]),
        script: match value("--input-file") {
            Some(p) => Some(input::Script::load(&p).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            })),
            None => value("--input").map(|v| input::Script::parse(&v)),
        },
        trace: value("--trace"),
        load_state: value("--load-state"),
    }
}

fn load_assets() -> &'static dino_assets::AssetPack {
    match Rom::locate().and_then(|p| dino_extract::extract(&p)) {
        Ok(a) => Box::leak(Box::new(a)),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn main() {
    let args = args();
    if let Some(trace) = &args.trace {
        // Headless harness run: no window.
        let frames = std::env::args().any(|a| a == "--frames").then_some(args.frames).unwrap_or(600);
        let r = harness::run(load_assets(), trace, args.load_state.as_deref(), args.stage, &args.chars, args.script.as_ref(), frames);
        if let Err(e) = r {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    macroquad::Window::from_config(window_conf(), windowed(args));
}

async fn windowed(args: Args) {
    let assets = load_assets();
    let mut gpu = Gpu::new(assets);

    let mut viewer = args.viewer.then(|| viewer::Viewer::new(assets, args.view));
    let mut inputs = input::Inputs::new();
    let mut game = game::Game::new(assets);
    match &args.load_state {
        Some(p) => match harness::load_state(&mut game, p) {
            Ok(warnings) => warnings.iter().for_each(|w| eprintln!("load-state: {w}")),
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        None => game.start_stage(args.stage, &args.chars),
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
