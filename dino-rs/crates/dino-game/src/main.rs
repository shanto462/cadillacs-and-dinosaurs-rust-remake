//! dino-game: native Rust port of the game logic, rendered on the GPU.
//!
//! Usage: `cargo run -p dino-game -- [--rom path/to/dino.zip] [--stage N] [--chars 0,2]
//!         [--input SCRIPT] [--viewer] [--view char,set,anim] [--screenshot out.png --frames N]`
//! See `input.rs` for keys and the input script format.

mod game;
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
        script: value("--input").map(|v| input::Script::parse(&v)),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = args();
    let assets = match Rom::locate().and_then(|p| dino_extract::extract(&p)) {
        Ok(a) => &*Box::leak(Box::new(a)),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    let mut gpu = Gpu::new(assets);

    let mut viewer = args.viewer.then(|| viewer::Viewer::new(assets, args.view));
    let mut game = game::Game::new(assets);
    game.start_stage(args.stage, &args.chars);
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
                    let mut pads = input::keyboard();
                    if let Some(script) = &args.script {
                        pads[0] |= script.at(tick_no);
                    }
                    game.w.input = pads;
                    game.tick();
                    tick_no += 1;
                }
            }
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
