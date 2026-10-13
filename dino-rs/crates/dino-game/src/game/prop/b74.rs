//! Pool B class 74 (`+$20 = 0x128`, handler 0x89B48): the master of the Episode 1 opening scene (M2-11). The stage script
//! (spawn list 1, camera X 0x1C0) creates it; it makes the cast of the scene and then only counts, and the cast reads its
//! step byte `+$05` to know what to do (pool A classes 54, 55 and 56, `enemy/scenery.rs`: each one remembers the master in
//! its long `+$A4`).
//!
//! | mode | sub | what |
//! | --- | --- | --- |
//! | 0 | | 0x89B5C: mode 2, `+$00` = 2, and the frame of sub 0 at once |
//! | 2 | 0 | 0x89B88: sub 2 |
//! | 2 | 2 | 0x89B90: the first cast (below), `+$A0` = 0x168 frames, sub 4 |
//! | 2 | 4 | 0x89D90: count `+$A0` down, at 0 sub 6 (360 frames after the cast was made) |
//! | 2 | 6 | 0x89CAE: sub 8, `+$A0` = 0x5A, the second cast (three thugs that walk in from the right) |
//! | 2 | 8 | 0x89D90: count `+$A0` down, at 0 sub 0xA |
//! | 2 | 0xA | 0x89D78: sub 0xC, `+$A0` = 0x1E |
//! | 2 | 0xC | 0x89D90: count `+$A0` down, at 0 sub 0xE |
//! | 2 | 0xE | 0x89D86: sub 0x10 |
//! | 2 | 0x10 | 0x89D8E: nothing, for ever (the cast frees itself; the object stays until the area ends) |
//! | 4, 6 | | 0x89DA0: free (0x4938) |
//!
//! The first cast (0x89B90, pool A slots from the allocator 0x483C, a full pool skips the member): the bystander class 54 at
//! (0x4C0, 0, 0x130), the flying member class 56 at (0x4E0, 0x60, 0x131) and the three thugs class 55 (variants 0, 1, 2) at
//! (0x490, 0, 0x130), (0x4A0, 0, 0x14E), (0x4A0, 0, 0x120). The second cast (0x89CAE): the thugs 3, 4, 5 facing right (`+$24` = 1)
//! at (0x370, 0, 0x130), (0x360, 0, 0x14E), (0x370, 0, 0x120). The class 56 member gets neither a variant nor a direction
//! (the original writes only its position and the master).

use super::Prop;
use crate::game::{Game, Handle};

/// Offsets of this class.
mod f {
    /// The counter of the sub steps 4, 8 and 0xC.
    pub const A0: usize = 0xA0;
}

/// One member of a cast: class, variant (None: the original does not write `+$26` and `+$22`), facing, X, height, depth.
struct Member {
    class: u16,
    var: Option<u16>,
    facing: bool,
    x: i16,
    y: i16,
    z: i16,
}

/// 0x89B90: the first cast, in the order the original makes it (the order decides which pool A slot each one gets).
const FIRST: [Member; 5] = [
    Member { class: 54, var: Some(0), facing: false, x: 0x4C0, y: 0, z: 0x130 },
    Member { class: 56, var: None, facing: false, x: 0x4E0, y: 0x60, z: 0x131 },
    Member { class: 55, var: Some(0), facing: false, x: 0x490, y: 0, z: 0x130 },
    Member { class: 55, var: Some(1), facing: false, x: 0x4A0, y: 0, z: 0x14E },
    Member { class: 55, var: Some(2), facing: false, x: 0x4A0, y: 0, z: 0x120 },
];

/// 0x89CAE: the second cast.
const SECOND: [Member; 3] = [
    Member { class: 55, var: Some(3), facing: true, x: 0x370, y: 0, z: 0x130 },
    Member { class: 55, var: Some(4), facing: true, x: 0x360, y: 0, z: 0x14E },
    Member { class: 55, var: Some(5), facing: true, x: 0x370, y: 0, z: 0x120 },
];

impl Game {
    /// 0x89B48 pool B class 74 (jump table 0x89B54: mode 0 -> 0x89B5C, 2 -> 0x89B68, 4 and 6 -> 0x89DA0).
    pub fn prop_class74(&mut self, p: &mut Prop, i: usize) {
        match p.b.mode {
            0 => {
                p.b.mode = 2;
                p.b.live = 2;
                self.prop_b74_frame(p, i); // falls into 0x89B68 (sub 0)
            }
            2 => self.prop_b74_frame(p, i),
            _ => self.prop_free(p, i), // 0x89DA0: jmp 0x4938
        }
    }

    /// 0x89B68: the routine of `+$05` (table 0x89B76).
    fn prop_b74_frame(&mut self, p: &mut Prop, i: usize) {
        match p.b.sub {
            0 => p.b.sub = 2, // 0x89B88
            2 => {
                // 0x89B90: the first cast, then `+$A0` = 0xF0 + 0x78
                let me = self.props.handle(i);
                self.prop_b74_cast(me, &FIRST);
                p.m.sw(f::A0, 0xF0u16.wrapping_add(0x78));
                p.b.sub += 2;
            }
            0xA => {
                // 0x89D78
                p.b.sub += 2;
                p.m.sw(f::A0, 0x1E);
            }
            0xE => p.b.sub += 2, // 0x89D86
            0x10 => {}           // 0x89D8E
            6 => {
                // 0x89CAE: sub 8, `+$A0` = 0x5A, the second cast
                p.b.sub += 2;
                p.m.sw(f::A0, 0x5A);
                let me = self.props.handle(i);
                self.prop_b74_cast(me, &SECOND);
            }
            _ => {
                // 4, 8 and 0xC (0x89D90): count down, at 0 the next sub
                let t = p.m.w(f::A0).wrapping_sub(1);
                p.m.sw(f::A0, t);
                if t == 0 {
                    p.b.sub += 2;
                }
            }
        }
    }

    /// The pool A allocator 0x483C and the writes of 0x89B90 / 0x89CAE for each member: `+$00` = 1, class, variant and `+$22` = 0
    /// (not for class 56), `+$24` (the second cast), X, height and depth as words, the master as a long in `+$A4`
    /// (`move.l a6,$a4(a0)`: the sign extended address). A full pool (carry set) skips the member.
    fn prop_b74_cast(&mut self, master: Handle, cast: &[Member]) {
        for m in cast {
            let Some(h) = self.spawn_alloc(0) else { continue };
            let b = self.body_mut(h);
            b.live = 1;
            b.k20 = 4 * m.class;
            if let Some(v) = m.var {
                b.var26 = v;
                b.dir = 0;
            }
            if m.facing {
                b.facing = 1;
            }
            b.set_xi_keep(m.x);
            b.set_yi_keep(m.y);
            b.set_zi_keep(m.z);
            let i = self.enemy_index(h);
            self.enemies.slots[i].m.sl(0xA4, 0xFFFF_0000 | master as u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dino_assets::AssetPack;

    fn game() -> Game {
        Game::new(Box::leak(Box::new(AssetPack::default())))
    }

    /// Runs the master for `frames` updates from its creation and returns the game and the master's slot.
    fn run(frames: usize) -> (Game, usize) {
        let mut g = game();
        let h = g.spawn_prop(74, 0, None).unwrap();
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
        for _ in 0..frames {
            let mut p = std::mem::take(&mut g.props.slots[i]);
            g.prop_class74(&mut p, i);
            g.props.slots[i] = p;
        }
        (g, i)
    }

    fn cast(g: &Game) -> Vec<(u16, u16, i16, i16, i16)> {
        g.enemies.slots.iter().filter(|e| e.b.live != 0).map(|e| (e.b.k20 / 4, e.b.var26, e.b.xi(), e.b.yi(), e.b.zi())).collect()
    }

    /// The steps of 0x89B48: sub 0 in the first frame (the object starts in the sub 0 code at once, which only sets sub 2), the first
    /// cast one frame later, sub 6 after 0x168 more updates, the second cast in the update after that.
    #[test]
    fn the_master_makes_the_cast_in_two_groups() {
        let (g, i) = run(1);
        let m = &g.props.slots[i].b;
        assert_eq!((m.mode, m.sub, m.live), (2, 2, 2));
        assert!(cast(&g).is_empty());
        let (g, i) = run(2);
        assert_eq!(g.props.slots[i].b.sub, 4);
        let mut c = cast(&g);
        c.sort();
        assert_eq!(
            c,
            vec![(54, 0, 0x4C0, 0, 0x130), (55, 0, 0x490, 0, 0x130), (55, 1, 0x4A0, 0, 0x14E), (55, 2, 0x4A0, 0, 0x120), (56, 0, 0x4E0, 0x60, 0x131)]
        );
        // Every member remembers the master in +$A4 (the sign extended address).
        let master = g.props.handle(i);
        assert!(g.enemies.slots.iter().filter(|e| e.b.live != 0).all(|e| e.m.l(0xA4) == 0xFFFF_0000 | master as u32));
        // +$A0 = 0x168 counts down in sub 4: sub 6 arrives after 0x168 more updates, and the next update makes the second group.
        let (g, i) = run(2 + 0x168);
        assert_eq!(g.props.slots[i].b.sub, 6);
        assert_eq!(cast(&g).len(), 5);
        let (g, i) = run(2 + 0x168 + 1);
        assert_eq!((g.props.slots[i].b.sub, g.props.slots[i].m.w(f::A0)), (8, 0x5A));
        let mut c = cast(&g);
        c.sort();
        assert_eq!(c.len(), 8);
        assert!(c.contains(&(55, 3, 0x370, 0, 0x130)) && c.contains(&(55, 4, 0x360, 0, 0x14E)) && c.contains(&(55, 5, 0x370, 0, 0x120)));
        assert!(g.enemies.slots.iter().filter(|e| e.b.live != 0 && e.b.var26 >= 3).all(|e| e.b.facing == 1), "the second group faces right");
    }

    /// Sub 8 counts 0x5A frames to sub 0xA; 0xA starts the 0x1E count; 0xC ends it; 0xE goes to the last sub, which never ends.
    #[test]
    fn the_master_waits_and_then_stays() {
        let base = 2 + 0x168 + 1;
        let subs = |n: usize| run(base + n).0.props.slots.iter().find(|p| p.b.live != 0).map(|p| p.b.sub).unwrap();
        assert_eq!(subs(0x59), 8);
        assert_eq!(subs(0x5A), 0xA);
        assert_eq!(subs(0x5B), 0xC);
        assert_eq!(subs(0x5B + 0x1E - 1), 0xC);
        assert_eq!(subs(0x5B + 0x1E), 0xE);
        assert_eq!(subs(0x5B + 0x1E + 1), 0x10);
        assert_eq!(subs(5000), 0x10);
    }

    /// A full pool A skips the members (the carry of 0x483C) and the master goes on.
    #[test]
    fn a_full_pool_makes_fewer_members() {
        let mut g = game();
        while g.enemies.alloc().is_some() {}
        let h = g.spawn_prop(74, 0, None).unwrap();
        let i = (0..24).find(|&k| g.props.handle(k) == h).unwrap();
        for _ in 0..3 {
            let mut p = std::mem::take(&mut g.props.slots[i]);
            g.prop_class74(&mut p, i);
            g.props.slots[i] = p;
        }
        assert_eq!(g.props.slots[i].b.sub, 4);
    }
}
