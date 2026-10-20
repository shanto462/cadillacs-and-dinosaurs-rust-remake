//! Pool B class 31 (`+$20 = 0x7C`, handler 0x81046): the stage clock. The stage script makes one at every area start
//! (list 1, trigger 0). It shows the time `m:ss` (G.$4E8, BCD) in a raw sprite list that lives in a pool D slot
//! (D3f section 3.2, D4b section 1.4, D4e section 4.2).
//!
//! | mode | what |
//! | --- | --- |
//! | 0 | 0x8105A: outside a game (G.$4CC = 0) the object frees itself. Else it takes a pool D slot (0x48A8; none left: frees itself), sets its screen position X 0xE8, Y 0xB8, depth 0, `+$A0` = the clock word, `+$A2` = 0, copies the 18 longs of the template 0x81194 into the slot, puts the list pointer (slot + 0xA) in its first long, loads the slot as its raw record (0x16C6: `+$28` = the slot, `+$2C..+$2F` = the record bytes), visible, draw flags 0xD0 (raw list, screen space, no shadow), mode 2, and writes the digits (0x8114E). No picture in this frame. |
//! | 2 | 0x810BA: above 3:00 (G.$4E8 > 0x0300, signed) nothing. Else when the clock word differs from `+$A0` (the time that was shown last): if it went up, the window `+$A2` is cleared; the word is kept; with the window open the second tick beeps (code 0x34, d1 0x10, d2 0: no pan); with it closed the window is armed: 300 frames when the seconds are 00 or 30, else 800 frames when the word is 0x15 or less, else 0 (so the clock appears for 5 seconds at each half minute and stays in the last 15 seconds); the digits are written. Then, unless a scripted scene runs (G.$4DD), while the window is open it counts down, the palette is red (`+$23` = 0x84) at 1:00 or less (word <= 0x0100, unsigned) and white (0) above, and the object goes into the draw list (0x49C0). |
//! | 4, 6 | 0x81188: the slot is given back (0x498C) and the object is freed (0x4938). |
//!
//! The slot list is five sprites (the minute, the two second digits and the two dots of the colon), 16 x 16, palette 6; the
//! digit tiles come from the table 0x811CA. The red palette is the palette override of the object, which the raw list
//! drawing (0x147FE) applies to every sprite.

use super::Prop;
use crate::game::draw::slot_frame;
use crate::game::{slot_of, Game, Slot};

/// Offsets of this class.
mod f {
    /// word: the pool D slot (its RAM address).
    pub const SLOT: usize = 0x98;
    /// word: the clock word that was shown last.
    pub const TIME: usize = 0xA0;
    /// word: the frames the clock stays on the screen.
    pub const WINDOW: usize = 0xA2;
}

/// The tables of this class (extracted exactly: `TABLES` of `dino-extract/src/enemy.rs`).
mod rom {
    /// 18 longs copied into the slot.
    pub const TEMPLATE: u32 = 0x81194;
    /// Tile code of the digits 0 to 9.
    pub const DIGITS: u32 = 0x811CA;
}

/// The sound 0x34 (the second tick beep) is queued with d1 = 0x10 (no pan) and d2 = 0 (0x810E2..0x810E8).
const BEEP: (u16, u32, u32) = (0x34, 0x10, 0);

impl Game {
    /// G.$4E8: the stage clock as one BCD word (minutes, seconds).
    fn prop_clock_word(&self) -> u16 {
        (self.rank.time_min as u16) << 8 | self.rank.time_sec as u16
    }

    /// 0x81046 pool B class 31 (jump table 0x81052: mode 0 -> 0x8105A, 2 -> 0x810BA, 4 and 6 -> 0x81188).
    pub fn prop_clock(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => self.prop_clock_init(p, i),
            2 => self.prop_clock_run(p, i),
            _ => self.prop_clock_end(p, i),
        }
    }

    /// 0x81188: give the slot back and free the object.
    fn prop_clock_end(&mut self, p: &mut Prop, i: usize) {
        self.draw_special_free(&mut p.m); // jsr 0x498C
        self.prop_free(p, i); // jmp 0x4938
    }

    /// 0x8105A.
    fn prop_clock_init(&mut self, p: &mut Prop, i: usize) {
        if self.flow.in_game == 0 {
            return self.prop_clock_end(p, i); // tst.b $4cc(a5) ; beq 0x81188
        }
        let Some(slot) = self.draw_special_alloc(&mut p.m) else {
            return self.prop_clock_end(p, i); // jsr 0x48A8 ; bcs 0x81188
        };
        p.b.set_xi(0xE8);
        p.b.set_yi(0xB8);
        p.b.set_zi(0);
        p.m.sw(f::TIME, self.prop_clock_word());
        p.m.sw(f::WINDOW, 0);
        // 0x81088: the template, 18 longs; then the pointer to the list at +$0A (a0 is the slot address as `movea.w` made it,
        // 0xFFFFxxxx, and `addi.w` only moves its low word)
        let handle = self.specials.handle(slot);
        let t = &self.assets.enemy.t;
        let sp = &mut self.specials.slots[slot];
        for k in 0..18u32 {
            sp.m.sl(4 * k as usize, t.u32(rom::TEMPLATE, 4 * k));
        }
        sp.m.sl(0, 0xFFFF_0000 | handle.wrapping_add(0xA) as u32);
        sp.b.live = sp.m.b(0);
        // 0x16C6: the slot is the draw record of the object
        let (a, b, c, d) = sp.record_params();
        p.b.set_frame(slot_frame(slot));
        (p.b.param_a, p.b.param_b, p.b.kind, p.b.timer) = (a, b, c, d);
        p.b.visible = 1;
        p.b.draw_flags = 0xD0;
        p.b.mode += 2;
        self.prop_clock_digits(p); // bra 0x8114E
    }

    /// 0x810BA.
    fn prop_clock_run(&mut self, p: &mut Prop, i: usize) {
        let t = self.prop_clock_word();
        if t as i16 > 0x300 {
            return; // cmpi.w #$300 ; bgt 0x8114C
        }
        let shown = p.m.w(f::TIME);
        if t != shown {
            if t as i16 > shown as i16 {
                p.m.sw(f::WINDOW, 0); // the time went up (a top up): the window closes
            }
            p.m.sw(f::TIME, t);
            if p.m.w(f::WINDOW) != 0 {
                let (code, d1, d2) = BEEP;
                self.snd_request(code, d1, d2); // 0x810E2
            } else {
                // 0x810F2: the half minute marks 5 seconds, the last 15 seconds 800 frames
                let window = if t & 0xFF == 0 || t & 0xFF == 0x30 {
                    0x12C
                } else if t <= 0x15 {
                    0x320
                } else {
                    0
                };
                p.m.sw(f::WINDOW, window);
            }
            self.prop_clock_digits(p); // 0x8111A: bsr 0x8114E
        }
        if self.w.scripted != 0 {
            return; // 0x8111E: tst.b $4dd(a5) ; bne
        }
        let w = p.m.w(f::WINDOW);
        if w == 0 {
            return;
        }
        p.m.sw(f::WINDOW, w - 1);
        p.b.palette_override = if t <= 0x100 { 0x84 } else { 0 }; // 0x81134: red at 1:00 and less (unsigned bhi)
        self.prop_draw_list(p, i); // jmp 0x49C0
    }

    /// 0x8114E: the digits into the slot, the tile codes at +$14, +$1C and +$24 (the codes of the sprites 0 to 2).
    fn prop_clock_digits(&mut self, p: &Prop) {
        let Some(Slot::Special(k)) = slot_of(p.m.w(f::SLOT)) else { return };
        let t = self.prop_clock_word();
        let tile = |d: u16| self.assets.enemy.t.u16(rom::DIGITS, 2 * (d & 0xF) as u32);
        let (m, s10, s1) = (tile(t >> 8), tile(t >> 4), tile(t));
        let sp = &mut self.specials.slots[k];
        sp.m.sw(0x14, m);
        sp.m.sw(0x1C, s10);
        sp.m.sw(0x24, s1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::draw::is_slot_frame;
    use dino_assets::AssetPack;

    /// The data of the class, made up for the test (the real template and tile table are read from the ROM at run time): a
    /// 72 byte template whose list has 5 sprites (the digits at the sprite 0 to 2, `attr` 6), and the digit tiles 0x100 + d.
    fn game() -> Game {
        let mut tpl = vec![0u8; 0x48];
        tpl[7] = 0xD0; // the record parameters (+$04..+$07): the last byte has bit 7 set
        tpl[0xA..0xC].copy_from_slice(&5u16.to_be_bytes()); // count
        tpl[0xC..0xE].copy_from_slice(&5u16.to_be_bytes()); // sprites
        for (k, dx) in [0u16, 0x18, 0x28, 0x10, 0x10].into_iter().enumerate() {
            let e = 0xE + 8 * k;
            tpl[e..e + 2].copy_from_slice(&dx.to_be_bytes());
            tpl[e + 4..e + 6].copy_from_slice(&6u16.to_be_bytes());
        }
        let digits: Vec<u8> = (0..10u16).flat_map(|d| (0x100 + d).to_be_bytes()).collect();
        tpl[0x36..].copy_from_slice(&digits[..0x12]); // the template's last words are the first digits (as in the ROM), the copy takes them along
        let mut a = AssetPack::default();
        a.enemy.t.blobs = vec![(rom::TEMPLATE, tpl), (rom::DIGITS, digits)];
        let mut g = Game::new(Box::leak(Box::new(a)));
        g.flow.in_game = 1;
        g
    }

    fn clock(g: &mut Game) -> usize {
        let i = g.props.alloc().unwrap();
        let b = &mut g.props.slots[i].b;
        b.live = 1;
        b.k20 = 31 * 4;
        i
    }

    fn set_time(g: &mut Game, word: u16) {
        g.rank.time_min = (word >> 8) as u8;
        g.rank.time_sec = word as u8;
    }

    fn tick(g: &mut Game) {
        g.draw_list.clear();
        g.pool_b_update();
    }

    fn drawn(g: &Game) -> bool {
        !g.draw_list.is_empty()
    }

    /// 0x8105A: the first update takes the pool D slot 15 (the first request of a fresh pool), fills it from the template,
    /// writes the digits and the list pointer, and draws nothing in that frame.
    #[test]
    fn init_takes_slot_15_and_writes_the_digits() {
        let mut g = game();
        set_time(&mut g, 0x0123);
        let i = clock(&mut g);
        tick(&mut g);
        let p = &g.props.slots[i];
        assert_eq!((p.b.mode, p.b.visible, p.b.draw_flags, p.b.xi(), p.b.yi(), p.b.zi()), (2, 1, 0xD0, 0xE8, 0xB8, 0));
        assert_eq!((p.m.w(f::SLOT), p.m.w(f::TIME), p.m.w(f::WINDOW)), (0xE574, 0x0123, 0));
        assert_eq!((p.b.param_a, p.b.param_b, p.b.kind, p.b.timer), (0, 0, 0, 0x50), "the record bytes, bit 7 dropped");
        assert!(is_slot_frame(p.b.frame));
        assert!(!drawn(&g));
        let sp = &g.specials.slots[15];
        assert_eq!(sp.m.l(0), 0xFFFF_E57E, "the pointer to the list in the slot");
        assert_eq!(sp.b.live, 0xFF);
        assert_eq!((sp.m.w(0x14), sp.m.w(0x1C), sp.m.w(0x24)), (0x101, 0x102, 0x103));
        let list = sp.sprites(g.specials.handle(15));
        assert_eq!(list.iter().map(|s| (s.dx, s.attr)).collect::<Vec<_>>(), [(0, 6), (0x18, 6), (0x28, 6), (0x10, 6), (0x10, 6)]);
        assert_eq!(g.specials.free_count(), 15);
    }

    /// With the ROM data (skipped without `dino.zip`): the clock takes the real template and tile table, so the extracted tables
    /// cover every byte the class reads, and the slot holds a list of five sprites with the digit tiles of the table.
    #[test]
    fn it_reads_the_extracted_tables() {
        let path = std::path::Path::new("../../../dino.zip");
        if !path.exists() {
            return;
        }
        let assets: &'static AssetPack = Box::leak(Box::new(dino_extract::extract(path).expect("the ROM set loads")));
        let mut g = Game::new(assets);
        g.flow.in_game = 1;
        set_time(&mut g, 0x0259);
        let i = clock(&mut g);
        tick(&mut g);
        let t = &assets.enemy.t;
        let sp = &g.specials.slots[15];
        let list = sp.sprites(g.specials.handle(15));
        assert_eq!(list.len(), 5, "the minute, two second digits and the two dots of the colon");
        assert_eq!((sp.m.w(0x14), sp.m.w(0x1C), sp.m.w(0x24)), (t.u16(rom::DIGITS, 4), t.u16(rom::DIGITS, 10), t.u16(rom::DIGITS, 18)));
        assert_eq!(list[..3].iter().map(|s| s.code).collect::<Vec<_>>(), [sp.m.w(0x14), sp.m.w(0x1C), sp.m.w(0x24)]);
        assert_eq!(g.props.slots[i].b.timer, 0x50);
        // a second of the clock changes the last digit only
        set_time(&mut g, 0x0258);
        g.props.slots[i].m.sw(f::WINDOW, 0x12C);
        tick(&mut g);
        let sp = &g.specials.slots[15];
        assert_eq!((sp.m.w(0x14), sp.m.w(0x1C), sp.m.w(0x24)), (t.u16(rom::DIGITS, 4), t.u16(rom::DIGITS, 10), t.u16(rom::DIGITS, 16)));
    }

    /// Outside a game the clock frees itself in its first update and takes no slot.
    #[test]
    fn it_frees_itself_outside_a_game() {
        let mut g = game();
        g.flow.in_game = 0;
        let i = clock(&mut g);
        tick(&mut g);
        assert_eq!((g.props.slots[i].b.live, g.specials.free_count()), (0, 16));
    }

    /// No pool D slot left: it frees itself too (the original clears +$98 and takes the free path).
    #[test]
    fn it_frees_itself_without_a_slot() {
        let mut g = game();
        while g.specials.alloc().is_some() {}
        let i = clock(&mut g);
        tick(&mut g);
        assert_eq!((g.props.slots[i].b.live, g.props.slots[i].m.w(f::SLOT)), (0, 0));
    }

    /// The display window of D4b 1.4: hidden above 3:00 (not even the digits change), 5 seconds from each :00 and :30 mark,
    /// steady from 0:15, red from 1:00.
    #[test]
    fn the_clock_shows_in_windows() {
        let mut g = game();
        set_time(&mut g, 0x0301);
        let i = clock(&mut g);
        tick(&mut g);
        tick(&mut g);
        assert!(!drawn(&g), "3:01 is above 3:00");
        assert_eq!(g.props.slots[i].m.w(f::TIME), 0x0301, "nothing is looked at above 3:00");
        set_time(&mut g, 0x0300);
        tick(&mut g);
        assert!(drawn(&g), "3:00 is a mark: the window opens and the clock is drawn at once");
        assert_eq!(g.props.slots[i].m.w(f::WINDOW), 0x12C - 1, "300 frames, one counted already");
        for _ in 0..299 {
            tick(&mut g);
            assert!(drawn(&g));
        }
        tick(&mut g);
        assert!(!drawn(&g), "after 300 pictures the window is closed");
        set_time(&mut g, 0x0259); // a tick that is not a mark: no window
        tick(&mut g);
        assert!(!drawn(&g));
        set_time(&mut g, 0x0230);
        tick(&mut g);
        assert!(drawn(&g), "2:30");
        assert_eq!(g.props.slots[i].b.palette_override, 0, "white above 1:00");
        set_time(&mut g, 0x0100);
        tick(&mut g);
        assert!(drawn(&g), "the window of 2:30 is still open");
        assert_eq!(g.props.slots[i].b.palette_override, 0x84, "1:00 is red");

        let mut g = game();
        set_time(&mut g, 0x0116);
        let j = clock(&mut g);
        tick(&mut g);
        set_time(&mut g, 0x0115);
        tick(&mut g);
        assert_eq!((drawn(&g), g.props.slots[j].m.w(f::WINDOW)), (false, 0), "1:15 is no mark");
        set_time(&mut g, 0x0015);
        tick(&mut g);
        assert!(drawn(&g));
        assert_eq!(g.props.slots[j].m.w(f::WINDOW), 0x320 - 1, "the last 15 seconds: 800 frames");
        assert_eq!(g.props.slots[j].b.palette_override, 0x84);
        set_time(&mut g, 0x0000);
        tick(&mut g);
        assert_eq!(g.props.slots[j].m.w(f::WINDOW), 0x320 - 2, "0:00 inside the open window: it only beeps");
        // a clock that reaches 0:00 with the window closed gets the 300 frames (the seconds test comes first)
        let mut g = game();
        set_time(&mut g, 0x0001);
        let k = clock(&mut g);
        tick(&mut g);
        set_time(&mut g, 0x0000);
        tick(&mut g);
        assert_eq!(g.props.slots[k].m.w(f::WINDOW), 0x12C - 1);
    }

    /// The beep (D4e 4.2): a clock change while the window is open queues 0x34 with d1 0x10 and d2 0; the change that
    /// opens the window does not beep; a clock that goes up closes the window (the top up of a respawn) so that change
    /// does not beep either; nothing beeps while the clock is above 3:00.
    #[test]
    fn the_beep_follows_the_window() {
        let mut g = game();
        set_time(&mut g, 0x0016);
        clock(&mut g);
        tick(&mut g);
        set_time(&mut g, 0x0015);
        tick(&mut g);
        assert!(g.snd_take_requests().is_empty(), "the change that opens the window");
        set_time(&mut g, 0x0014);
        tick(&mut g);
        assert_eq!(g.snd_take_requests(), [crate::game::snd::SndRequest { code: 0x34, d1: 0x10, d2: 0 }]);
        assert_eq!(g.w.sounds, [0x34]);
        tick(&mut g);
        assert!(g.snd_take_requests().is_empty(), "no change, no beep");
        set_time(&mut g, 0x0330);
        tick(&mut g);
        assert!(g.snd_take_requests().is_empty(), "above 3:00: nothing");
        set_time(&mut g, 0x0300);
        tick(&mut g);
        assert!(g.snd_take_requests().is_empty(), "above the word shown last (0:14): the window is closed and armed again, no beep");
        set_time(&mut g, 0x0259);
        tick(&mut g);
        assert_eq!(g.snd_take_requests().len(), 1, "the 3:00 window is still open: the next second beeps");
        set_time(&mut g, 0x0300);
        tick(&mut g);
        assert!(g.snd_take_requests().is_empty(), "up: the window closes (and opens again at the mark), no beep");
    }

    /// The scripted scenes (G.$4DD) freeze the window and the picture.
    #[test]
    fn a_scripted_scene_hides_it() {
        let mut g = game();
        set_time(&mut g, 0x0100);
        let i = clock(&mut g);
        tick(&mut g);
        set_time(&mut g, 0x0030); // a mark: the window opens
        tick(&mut g);
        let w = g.props.slots[i].m.w(f::WINDOW);
        assert_ne!(w, 0);
        g.w.scripted = 1;
        tick(&mut g);
        assert!(!drawn(&g));
        assert_eq!(g.props.slots[i].m.w(f::WINDOW), w);
    }

    /// The end: modes 4 and 6 give the slot back (it stays as it was, `+$00` included) and free the object.
    #[test]
    fn the_end_gives_the_slot_back() {
        let mut g = game();
        let i = clock(&mut g);
        tick(&mut g);
        assert_eq!(g.specials.free_count(), 15);
        g.props.slots[i].b.mode = 4;
        tick(&mut g);
        assert_eq!((g.props.slots[i].b.live, g.props.slots[i].m.w(f::SLOT), g.specials.free_count()), (0, 0, 16));
        assert_eq!(g.specials.slots[15].m.b(0), 0xFF, "the stale bytes stay");
        assert_eq!(g.specials.slots[15].b.live, 0xFF);
    }
}
