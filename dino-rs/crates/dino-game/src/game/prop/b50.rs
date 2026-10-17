//! Boss dialogue box, pool B 50. Text and portrait tiles remain runtime ROM assets.
use super::Prop;
use crate::game::vram::PaletteRam;
use crate::game::{Game, Handle};

impl Game {
    /// 0x844B4 / 0x844C8 / 0x84562 / 0x84F4A: dialogue dispatch and fades.
    pub fn prop_b50(&mut self, p: &mut Prop, h: Handle, i: usize) {
        if !matches!(p.b.var26, 0 | 1 | 2 | 4) {
            // Other conversations, including ending variant 7, have separate task owners.
            if !self.trace_hold(h, &p.b) {
                self.prop_free(p, i);
            }
            return;
        }
        match p.b.mode {
            0 => {
                p.m.sw(0xA6, 0);
                p.m.sw(0xAA, 0);
                p.m.sw(0xB0, 0);
                // 0x84504: signed X comparison, live byte exactly 1, first player wins ties.
                let (mut hero, mut x) = (0, 0i16);
                for (n, player) in self.players.iter().enumerate() {
                    if player.b.live == 1 && (n == 0 || player.b.xi() > x) {
                        hero = player.b.id;
                        x = player.b.xi();
                    }
                }
                p.m.sw(0xB0, (hero as u16).wrapping_mul(2));
                self.fade.mask = 0x0C;
                self.fade.delay = 1;
                self.start_fade_task(false);
                p.b.mode = p.b.mode.wrapping_add(2);
            }
            2 => self.prop_b50_dialogue(p),
            4 => {
                self.fade.mask = 0x0C;
                self.fade.delay = 1;
                self.start_fade_task(true);
                p.b.mode = p.b.mode.wrapping_add(2);
            }
            _ => self.prop_free(p, i), // 0x84F6C
        }
    }

    /// 0x84562, 0xA5CEC..0xA5E1C: per-hero dialogue state tables.
    fn prop_b50_dialogue(&mut self, p: &mut Prop) {
        let dispatch = self
            .assets
            .frontend
            .text
            .u32(0x84584, p.b.var26 as u32 * 16 + p.m.w(0xB0) as u32 * 2);
        // Native ports of the wrapper routines. Each table address is its LEA operand;
        // the variant/hero pointer table at 0x84584 is read from the ROM assets.
        let table = match dispatch {
            0xA5CEC => 0xA5E42,
            0xA5CFE => 0xA5E5A,
            0xA5D10 => 0xA5E72,
            0xA5D22 => 0xA5E8A,
            0xA5D34 => 0xA5EA2,
            0xA5D46 => 0xA5EBE,
            0xA5D58 => 0xA5EDA,
            0xA5D6A => 0xA5EF6,
            0xA5D7C => 0xA5F12,
            0xA5D8E => 0xA5F2A,
            0xA5DA0 => 0xA5F42,
            0xA5DB2 => 0xA5F5A,
            0xA5E0C => 0xA6036,
            _ => panic!("unported B50 dispatcher {dispatch:06X}"),
        };
        let routine = self.assets.frontend.text.u32(table, 2 * p.b.sub as u32);
        // 0xA6072..0xA6370, 0xA655A: immediate operands of the print routines.
        let line = match routine {
            0xA6072 => Some((0, 0, None)),
            0xA608E => Some((2, 2, None)),
            0xA60AA => Some((1, 1, None)),
            0xA60C6 => Some((3, 3, None)),
            0xA60E2 => Some((5, 6, Some(1))),
            0xA6106 => Some((5, 8, Some(1))),
            0xA612A => Some((5, 7, Some(1))),
            0xA614E => Some((5, 9, Some(1))),
            0xA6172 => Some((0, 0xA, None)),
            0xA618E => Some((2, 0xC, None)),
            0xA61AA => Some((1, 0xB, None)),
            0xA61C6 => Some((3, 0xD, None)),
            0xA61E2 => Some((6, 0xE, Some(2))),
            0xA6206 => Some((6, 0x10, Some(2))),
            0xA622A => Some((6, 0xF, Some(2))),
            0xA624E => Some((6, 0x11, Some(2))),
            0xA6272 => Some((0, 0x12, None)),
            0xA628E => Some((2, 0x14, None)),
            0xA62AA => Some((1, 0x13, None)),
            0xA62C6 => Some((3, 0x15, None)),
            0xA62E2 => Some((9, 0x16, Some(5))),
            0xA6306 => Some((9, 0x18, Some(5))),
            0xA632A => Some((9, 0x17, Some(5))),
            0xA634E => Some((9, 0x19, Some(5))),
            0xA655A => Some((4, 4, Some(0))),
            _ => None,
        };
        if let Some((portrait, text, palette)) = line {
            if let Some(row) = palette {
                self.prop_b50_palette(row);
            }
            p.m.sw(0xA2, self.assets.frontend.text.u16(0x8461A, portrait * 2));
            self.text_portrait(0x90A29C, portrait as u16, text);
            p.m.sw(0xA4, if routine == 0xA655A { 150 } else { 90 });
            p.b.sub = p.b.sub.wrapping_add(2);
            return;
        }
        match routine {
            // 0xA65C6 / 0xA65D6 / 0xA65E6: clear the upper / lower 45 by 13 box.
            0xA65C6 | 0xA65D6 => {
                let base = if routine == 0xA65C6 {
                    0x90A29C
                } else {
                    0x90A2C8
                };
                for col in 0..45 {
                    for row in 0..13 {
                        self.hud_put(base + col * 0x80 + row * 4, 0x20, 0);
                    }
                }
                p.b.sub = p.b.sub.wrapping_add(2);
            }
            0xA6612 => p.b.mode = p.b.mode.wrapping_add(2),
            // 0xA6628..0xA6768: wrappers selecting the animated face for 0x84910.
            _ => {
                let face = match routine {
                    0xA6628 | 0xA6638 => 0,
                    0xA6658 | 0xA6668 => 2,
                    0xA6688 | 0xA6698 => 1,
                    0xA66B8 | 0xA66C8 => 3,
                    0xA6708 => 4,
                    0xA6718 => 5,
                    0xA6728 => 6,
                    0xA6768 => 9,
                    _ => panic!("unported B50 routine {routine:06X}"),
                };
                self.prop_b50_face(p, face);
                // 0x8491A..0x84974: attack alone on a newly changed input skips the line.
                for n in 0..3 {
                    if self.players[n].b.live == 1
                        && self.w.input[n] == 0x10
                        && self.w.input[n] != self.flow.front.prev_input[n]
                    {
                        p.m.sw(0xA4, 1);
                    }
                }
                let left = p.m.w(0xA4).wrapping_sub(1);
                p.m.sw(0xA4, left);
                if left as i16 <= 0 {
                    p.b.sub = p.b.sub.wrapping_add(2);
                }
            }
        }
    }

    /// 0x84E2E: replace the portrait palette RGB, preserving each brightness nibble.
    fn prop_b50_palette(&mut self, row: u32) {
        for n in 0..16 {
            let k = PaletteRam::SCROLL1 + 0x1C0 + n;
            self.pal.words[k] = (self.pal.words[k] & 0xF000)
                | self
                    .assets
                    .frontend
                    .text
                    .u16(0x84E8A, row * 32 + n as u32 * 2);
        }
        self.pal.dirty = true;
    }

    /// 0x84996 / 0x849C2 and 0x849F4..0x84CF6: portrait mouth and eye animation.
    fn prop_b50_face(&mut self, p: &mut Prop, face: u16) {
        let timer = p.m.w(0xA4);
        let mouth = timer & 7 == 0;
        if mouth {
            let mut phase = p.m.w(0xA6).wrapping_add(1) & 3;
            if phase == 3 {
                phase = 0;
            }
            p.m.sw(0xA6, phase);
        }
        let eyes = face != 3 && matches!(timer & 31, 0 | 29);
        if eyes {
            p.m.sw(
                0xAA,
                if p.m.w(0xAA).wrapping_add(1) == 2 {
                    0
                } else {
                    p.m.w(0xAA).wrapping_add(1)
                },
            );
        }
        // LEA operands and moveq dimensions in each face routine. Tables are extracted at runtime.
        let (mt, mo, mw, mh, et, eo, ew, eh) = match face {
            0 => (0x84A3E, 0x118, 5, 3, 0x84A4A, 0x90, 6, 1),
            1 => (0x84AD6, 0x114, 4, 3, 0x84AE6, 0x10C, 5, 2),
            2 => (0x84B40, 0x118, 5, 3, 0x84B4C, 0x110, 4, 1),
            3 => (0x84B78, 0x118, 6, 3, 0, 0, 0, 0),
            4 => (0x84BD2, 0x118, 4, 3, 0x84BDE, 0x110, 2, 1),
            5 => (0x84C34, 0x118, 5, 3, 0x84C40, 0x10C, 5, 2),
            6 => (0x84C96, 0x98, 6, 3, 0x84CA2, 0x110, 5, 1),
            9 => (0x84CF8, 0x94, 5, 3, 0x84D04, 0x8C, 4, 2),
            _ => unreachable!(),
        };
        if mouth {
            if face != 3 {
                p.m.sl(0xAC, 0x90A29C);
            }
            self.prop_b50_face_block(p, mt, p.m.w(0xA6), 0x90A29C + mo, mw, mh);
        }
        if eyes && et != 0 {
            self.prop_b50_face_block(p, et, p.m.w(0xAA), p.m.l(0xAC) + eo, ew, eh);
        }
        // 0x84A9E: Hannah resets both patches on timer 1 before the input skip test.
        if face == 1 && timer == 1 {
            self.prop_b50_face_block(p, mt, 3, p.m.l(0xAC) + mo, mw, mh);
            self.prop_b50_face_block(p, et, 2, p.m.l(0xAC) + eo, ew, eh);
        }
    }

    /// 0x84DD0: row-major portrait patch; tile 0xA05 adds the horizontal flip bit to +$A2.
    fn prop_b50_face_block(
        &mut self,
        p: &Prop,
        table: u32,
        phase: u16,
        at: u32,
        cols: u32,
        rows: u32,
    ) {
        let t = &self.assets.frontend.text;
        let mut source = t.u32(table, phase as u32 * 4);
        for row in 0..rows {
            for col in 0..cols {
                let code = t.u16(source, 0);
                source += 2;
                let attr =
                    p.m.w(0xA2)
                        .wrapping_add(if code == 0xA05 { 0x40 } else { 0 });
                self.hud_put(at + row * 4 + col * 0x80, code, attr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::TraceReader;
    use dino_assets::AssetPack;
    use std::path::Path;

    #[test]
    fn b50_selects_the_rightmost_active_hero_with_signed_x_and_stable_ties() {
        let mut g = Game::new(Box::leak(Box::new(AssetPack::default())));
        for (n, (live, x)) in [(1, -10), (2, 300), (1, -11)].into_iter().enumerate() {
            g.players[n].b.live = live;
            g.players[n].b.id = n as u8;
            g.players[n].b.set_xi(x);
        }
        let mut p = Prop::default();
        g.prop_b50(&mut p, 0, 0);
        assert_eq!(p.m.w(0xB0), 0);
        p.b.mode = 0;
        g.players[2].b.set_xi(-9);
        g.prop_b50(&mut p, 0, 0);
        assert_eq!(p.m.w(0xB0), 4);
        p.b.mode = 0;
        g.players[2].b.set_xi(-10);
        g.prop_b50(&mut p, 0, 0);
        assert_eq!(p.m.w(0xB0), 0);
    }

    fn step_go(g: &mut Game, i: usize) {
        if g.props.slots[i].b.live != 0 && g.props.slots[i].b.k20 == 20 * 4 {
            let mut p = std::mem::take(&mut g.props.slots[i]);
            g.prop_go_hint(&mut p, i);
            g.props.slots[i] = p;
        }
    }

    /// Local recordings are optional, never embedded in the test. Compare each service invocation
    /// independently so the still-placeholder boss code cannot mask a service error.
    #[test]
    fn b50_b76_services_match_mame() {
        let rom = Path::new("../../../dino.zip");
        let directory = std::env::var("DINO_INTRO_TRACES")
            .unwrap_or_else(|_| "../../../re/out/m503-traces".into());
        if !rom.exists() || !Path::new(&directory).exists() {
            return;
        }
        let assets = Box::leak(Box::new(dino_extract::extract(rom).unwrap()));
        let mut total = 0;
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if !path.to_string_lossy().ends_with(".mame.trc") {
                continue;
            }
            let mut rd = TraceReader::open(path.to_str().unwrap()).unwrap();
            let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
            let mut g = Game::new(assets);
            let mut checked = [0; 2];
            while let Some(frame) = rd.next() {
                let ram = rd.region("ram").unwrap().data.clone();
                let gfx = rd.region("gfx").unwrap().data.clone();
                if let Some((old_ram, old_gfx)) = &previous {
                    g.trace_load(old_ram, old_gfx);
                    // The scripted-control task can create a GO prop before pool B runs.
                    for j in 0..24 {
                        let a = 0xB6F4 + j * 0xC0;
                        if old_ram[a] == 0
                            && ram[a] != 0
                            && u16::from_be_bytes([ram[a + 0x20], ram[a + 0x21]]) == 20 * 4
                        {
                            let go = &mut g.props.slots[j];
                            go.b.live = 1;
                            go.b.k20 = 20 * 4;
                            go.b.mode = 0;
                            go.b.var26 = u16::from_be_bytes([ram[a + 0x26], ram[a + 0x27]]);
                        }
                    }
                    g.w.input.copy_from_slice(&ram[0x8016..0x8019]);
                    // The IRQ copies old current inputs to previous before the new sample.
                    g.flow
                        .front
                        .prev_input
                        .copy_from_slice(&old_ram[0x8016..0x8019]);
                    for i in 0..24 {
                        let a = 0xB6F4 + i * 0xC0;
                        let class = g.props.slots[i].b.k20 / 4;
                        if old_ram[a] == 0 || !matches!(class, 50 | 76) {
                            continue;
                        }
                        if class == 50 && !matches!(g.props.slots[i].b.var26, 0 | 1 | 2 | 4) {
                            continue;
                        }
                        // B20 writes a GO patch inside the dialogue rectangle. Keep its pool order
                        // so a simultaneous GO erase is not misattributed to the dialogue printer.
                        for j in 0..i {
                            step_go(&mut g, j);
                        }
                        let h = g.props.handle(i);
                        let mut p = std::mem::take(&mut g.props.slots[i]);
                        if class == 50 {
                            g.prop_b50(&mut p, h, i);
                        } else {
                            g.prop_b76(&mut p, i);
                        }
                        for j in i + 1..24 {
                            step_go(&mut g, j);
                        }
                        let what = format!("{} frame {frame}, B{class} slot {i}", path.display());
                        assert_eq!(p.b.live, ram[a], "{what} live");
                        if p.b.live == 0 {
                            continue;
                        }
                        assert_eq!(
                            (p.b.mode, p.b.sub),
                            (ram[a + 4], ram[a + 5]),
                            "{what} state"
                        );
                        let fields: &[usize] = if class == 50 {
                            &[0xA2, 0xA4, 0xA6, 0xAA, 0xB0]
                        } else {
                            &[0x82, 0x84, 0x86]
                        };
                        for &offset in fields {
                            assert_eq!(
                                p.m.w(offset),
                                u16::from_be_bytes([ram[a + offset], ram[a + offset + 1]]),
                                "{what} +{offset:X}"
                            );
                        }
                        if class == 50 {
                            assert_eq!(
                                p.m.l(0xAC),
                                u32::from_be_bytes(ram[a + 0xAC..a + 0xB0].try_into().unwrap()),
                                "{what} origin"
                            );
                            for col in 0..45 {
                                for row in 0..13 {
                                    let at = 0xA29C + col * 0x80 + row * 4;
                                    let cell = g.vram.scroll1[(at - 0x8000) / 4];
                                    assert_eq!(
                                        (cell.code, cell.attr),
                                        (
                                            u16::from_be_bytes([gfx[at], gfx[at + 1]]),
                                            u16::from_be_bytes([gfx[at + 2], gfx[at + 3]])
                                        ),
                                        "{what} text cell {at:X}"
                                    );
                                }
                            }
                            for n in 0..16 {
                                let k = PaletteRam::SCROLL1 + 0x1C0 + n;
                                let at = 0x14000 + k * 2;
                                assert_eq!(
                                    g.pal.words[k],
                                    u16::from_be_bytes([gfx[at], gfx[at + 1]]),
                                    "{what} portrait palette {n}"
                                );
                            }
                        } else {
                            // 0x2042E runs after pool B. In normal camera states its far-layer
                            // task recomputes the scroll words that B76 has just written.
                            g.background_update();
                            assert_eq!(p.m.b(0x88), ram[a + 0x88], "{what} flash flag");
                            for (offset, value) in [
                                (0x8028, g.w.layer_ctrl),
                                (0x802E, g.w.scroll3_base),
                                (0x87E4, g.cam.s3.cam_x),
                                (0x87E8, g.cam.s3.cam_y),
                            ] {
                                assert_eq!(
                                    value,
                                    u16::from_be_bytes([ram[offset], ram[offset + 1]]),
                                    "{what} global {offset:X}"
                                );
                            }
                            for (n, cell) in g.vram.intro3.iter().enumerate() {
                                let at = 0x20000 + n * 4;
                                assert_eq!(
                                    (cell.code, cell.attr),
                                    (
                                        u16::from_be_bytes([gfx[at], gfx[at + 1]]),
                                        u16::from_be_bytes([gfx[at + 2], gfx[at + 3]])
                                    ),
                                    "{what} flash map {at:X}"
                                );
                            }
                        }
                        checked[(class == 76) as usize] += 1;
                    }
                }
                previous = Some((ram, gfx));
            }
            eprintln!(
                "{}: B50 {} frames, B76 {} frames equal",
                path.display(),
                checked[0],
                checked[1]
            );
            total += checked[0] + checked[1];
        }
        assert!(total > 0, "intro recordings must exercise a service");
    }
}
