//! Pool C class 19 (+$20 = 0x4C), handler 0x94462: the stars that circle the head of a dizzy hero or enemy. Made by 0x94338 (`fx_spawn_dizzy`) with the
//! owner in +$58, the dizzy time (d0) in +$5A and the owner's state long +$04..+$07 in +$5C..+$5F. The stars stand where they were made (the owner's
//! position plus a height by its class, the table 0x94398) and end when the owner's state changes. The animation 0 (a time above 0xB4) or 1 loops; a hero who
//! mashes (a new button press this frame, +$A8) makes it run faster (the timer loses 3 a frame while it is above 3).

use super::*;

/// 0x944DC: the animation table of the stars.
const STAR_ANIMS: u32 = 0x944DC;
/// 0x94398: the height above the owner by its class index (a word each).
const STAR_HEIGHTS: u32 = 0x94398;

impl Game {
    /// 0x94462: mode 0 (init, falls into the run) and 2; 4 and 6 free the slot (0x944D6).
    pub(super) fn fx_dizzy(&mut self, i: usize) {
        let fx = self.fx_data();
        match self.effects.slots[i].b.mode {
            0 => {
                let e = &mut self.effects.slots[i];
                e.b.anim_table = STAR_ANIMS;
                let anim = if e.owner_class > 0xB4 { 0 } else { 1 }; // cmpi.w #$B4, $5A(a6) ; bhi
                fx_start(&mut e.b, fx, anim);
                e.b.draw_flags = 0x30;
                e.b.mode = e.b.mode.wrapping_add(2);
            }
            2 => {}
            _ => return self.fx_free(i),
        }
        let e = &self.effects.slots[i];
        let owner = e.owner58;
        let saved = u32::from_be_bytes([(e.w5c >> 8) as u8, e.w5c as u8, e.b5e, e.b5f]);
        if self.body(owner).state() != saved {
            return self.fx_free(i); // bne 0x944D6
        }
        // tst.b $82(a0): only a pool tag 0 owner (a hero) ; tst.b $A8(a0): a new press
        let mashing = match slot_of(owner) {
            Some(Slot::Player(p)) => self.players[p].pressed != 0,
            _ => false,
        };
        let b = &mut self.effects.slots[i].b;
        if mashing && b.timer > 3 {
            b.timer -= 3; // subq.b #3, $2F(a6)
        }
        fx_tick(b, fx); // jsr 0x15F4
        self.draw_list_add_check(self.effects.handle(i)); // jmp 0x49CA
    }

    /// 0x94338: the stars over the object `owner` (a6) with the dizzy time `time` (d0), at its position plus the height of its class. Nothing when pool C is full.
    /// `o` is the body of `owner` (a caller that works on a copy of its object passes the copy: the stars read the state it has now).
    pub fn fx_spawn_dizzy(&mut self, owner: Handle, o: &Body, time: u16) {
        let Some(j) = self.effects.alloc() else { return };
        let (x, y, z, state, class) = (o.xi(), o.yi(), o.zi(), o.state(), o.k20);
        let height = self.assets.enemy.t.u16(STAR_HEIGHTS, (class >> 1) as u32); // lsr.w #1, d0 ; move.w (pc, d0.w), d0
        let e = &mut self.effects.slots[j];
        e.b.live = 1;
        e.b.k20 = FX_DIZZY;
        set_word(&mut e.b.x, x);
        set_word(&mut e.b.y, y);
        set_word(&mut e.b.z, z);
        e.owner58 = owner;
        let [a, b, c, d] = state.to_be_bytes();
        e.w5c = u16::from_be_bytes([a, b]);
        e.b5e = c;
        e.b5f = d;
        e.owner_class = time;
        // tst.b $82(a0): a pool C slot has no pool tag (0), so the height is always added.
        let y = e.b.yi().wrapping_add(height as i16);
        set_word(&mut e.b.y, y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom() -> Option<&'static dino_assets::AssetPack> {
        static A: std::sync::OnceLock<Option<&'static dino_assets::AssetPack>> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let path = std::path::Path::new("../../../dino.zip");
            path.exists().then(|| &*Box::leak(Box::new(dino_extract::extract(path).expect("ROM loads"))))
        })
    }

    #[test]
    fn the_stars_stand_over_the_head_and_end_when_the_owner_changes_state() {
        let Some(a) = rom() else { return };
        let mut g = Game::new(a);
        let h = Game::player_handle(0);
        g.players[0].b.set_xi(0x200);
        g.players[0].b.set_yi(0);
        g.players[0].b.set_zi(0x50);
        g.players[0].b.set_state(0x0200_0021);
        let b = g.players[0].b.clone();
        g.fx_spawn_dizzy(h, &b, 0x100);
        let i = g.effects.slots.iter().position(|e| e.b.live != 0).unwrap();
        assert_eq!(g.effects.slots[i].b.yi(), 0x48, "the height of class 0 is 0x48");
        assert_eq!(g.effects.slots[i].w5c, 0x0200);
        let mut frames = Vec::new();
        for _ in 0..40 {
            g.pool_c_update();
            frames.push(g.effects.slots[i].b.frame);
            assert_eq!(g.effects.slots[i].b.live, 1);
        }
        assert!(frames.windows(2).any(|w| w[0] != w[1]), "the animation runs");
        g.effects.slots[i].b.timer = 20;
        g.players[0].pressed = 0x10; // a new press: 3 less, then the tick
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.timer, 16, "a mashing hero speeds the stars up");
        g.players[0].pressed = 0;
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.timer, 15);
        g.players[0].b.set_state(0x0200_0000);
        g.pool_c_update();
        assert_eq!(g.effects.slots[i].b.live, 0);
    }
}
