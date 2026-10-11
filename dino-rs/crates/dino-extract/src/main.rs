//! `dino-extract [--rom path] [anim <table_hex> <id>]`: extracts the assets and prints a summary,
//! or dumps one animation's frame records.

use dino_assets::FrameDraw;
use dino_extract::rom::Rom;

fn main() {
    let path = Rom::locate().unwrap_or_else(|e| fail(&e));
    let pack = dino_extract::extract(&path).unwrap_or_else(|e| fail(&e));
    let args: Vec<String> = std::env::args().collect();
    let s = &pack.sprites;

    if let Some(i) = args.iter().position(|a| a == "anim") {
        let table = u32::from_str_radix(args.get(i + 1).map(|s| s.trim_start_matches("0x")).unwrap_or("9edfe"), 16).unwrap();
        let id: u16 = args.get(i + 2).and_then(|v| v.parse().ok()).unwrap_or(0);
        let mut f = s.anim(table, id);
        for _ in 0..64 {
            let fr = s.frame(f);
            let draw = match &fr.draw {
                FrameDraw::Comp { main, overlay } => format!("comp {main:#06x} overlay {overlay:?}"),
                FrameDraw::Compound(parts) => format!("compound {} parts", parts.len()),
            };
            println!(
                "{:#07x} {draw} a={:#04x} b={:#04x} type={:#04x} dur={:3} body={:#04x} atk={:#04x}{}",
                fr.rom_addr,
                fr.param_a,
                fr.param_b,
                fr.kind,
                fr.duration,
                fr.body_box,
                fr.attack_box,
                if fr.last { " LAST" } else { "" }
            );
            if fr.last {
                println!("  -> link to {:#07x}", s.frame(fr.next).rom_addr);
                break;
            }
            f = fr.next;
        }
        return;
    }

    println!("tiles16:         {}", pack.tile16_count());
    println!("compositions:    {}", s.compositions.len());
    println!("frames:          {}", s.frames.len());
    println!("anim tables:     {}", s.anim_tables.len());
    println!("hitbox words:    {}", s.hitboxes.words.len());
    println!("obj palettes:    {} bank entries, {} stage/area sets", pack.obj_palette_bank.len(), pack.obj_palette_sets.len());
    let broken = s.frames.iter().filter(|f| f.next == u32::MAX).count();
    println!("unresolved links: {broken}");
    for (i, st) in pack.stages.stages.iter().enumerate() {
        let areas: Vec<String> = st
            .areas
            .iter()
            .map(|a| format!("{}x{}/{}x{} sh{}", a.scroll2.w, a.scroll2.h, a.scroll3.w, a.scroll3.h, a.scroll3_shift))
            .collect();
        println!("stage {i}: {} screen words, {} blocks, {} camera events, areas {areas:?}", st.scroll2_screen_words.len(), st.scroll2_blocks.len(), st.camera_events.len());
    }
    for (set, chars) in s.player_sets.iter().enumerate() {
        let n: Vec<usize> = chars.iter().map(|t| s.table(*t).anims.iter().flatten().count()).collect();
        println!("player set {set}: valid anims per character {n:?}");
    }
    let jack = s.player_box_tables[0];
    println!("Jack box table {jack:#x}: body 0x1a = {:?}", s.hitboxes.single(jack, 0x1a));
    println!("Jack attack k0 = {:?}", s.player_attacks[0][0]);
}

fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1)
}
