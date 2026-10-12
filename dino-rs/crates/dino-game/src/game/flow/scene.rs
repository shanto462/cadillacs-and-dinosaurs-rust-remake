//! Front end scene runner: one cooperative task of the original, written as a state machine.
//!
//! The original runs its scenes (attract task 0x97C20, story slides 0x988C0, opening movie
//! 0x9BF50, profile cards 0x9A7E6, boot screens 0x9DCF6 ...) as tasks: straight code that calls
//! the sleep routine 0x95A (or its helpers 0xA76 = 1, 0xA7A = 2, 0xA7E = 30, 0xA82 = 60,
//! 0xA86 = 120, 0xA8C = 180 frames) and goes on when the task wakes up. A [`Scene`] holds the
//! state of one such task: a state byte (the script position), a second state byte for nested
//! steps, a free counter for frame loops, and the sleep counter of the task kernel. A script is
//! one `match scene.state { ... }` that is entered once per frame while the scene is awake:
//!
//! ```ignore
//! fn flow_my_scene(&mut self) {
//!     if !self.flow.my.tick() { return; }            // asleep (or finished): nothing this frame
//!     match self.flow.my.state {
//!         0 => { /* draw the first picture */ self.flow.my.wait(30, 2); }      // sleep 30, then state 2
//!         2 => { /* clear it */ self.flow.my.wait(60, 4); }
//!         _ => self.flow.my.kill(),
//!     }
//! }
//! ```
//!
//! Sleep timing is the one of the kernel (`re/specs/CONTEXT.md`): a task that calls `sleep(n)` in
//! frame F runs again in frame F + n. The vertical blank interrupt ticks the sleep counters before
//! the tasks run, so `tick` is called once per frame at the place where the original runs the task
//! (slot order, see `Game::flow_frame`).

// The services are used by the scenes of the next tasks (T1.2 to T1.8); until they are, parts of
// them are unused.
#![allow(dead_code)]

/// State of one task (see the module comment).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Scene {
    /// The script position. The original keeps it in a RAM byte or word of its own (the attract
    /// state byte G.-$7030 is one of them).
    pub state: u16,
    /// A nested step (the original's second byte, for example G.$4D1 next to the mode G.$4D0).
    pub sub: u16,
    /// A free counter for frame loops (the title counter G.-$701C, the timeline counters).
    pub count: u16,
    /// Kernel sleep counter (+1 of the task slot). 0 = ready to run.
    sleep: u16,
    /// Task state 2: suspended until `wake` (sleep(0) does this).
    suspended: bool,
    live: bool,
}

impl Scene {
    /// A new task in state `state`, ready to run in the next `tick`.
    pub fn start(state: u16) -> Scene {
        Scene { state, live: true, ..Default::default() }
    }

    /// The task exists (it has not ended and was not killed).
    pub fn alive(&self) -> bool {
        self.live
    }

    /// Frames left in the current sleep (0 = awake). For tests and for loops that poll a sleeper.
    pub fn sleeping(&self) -> u16 {
        self.sleep
    }

    /// One frame of the task kernel. Returns true when the script must run now.
    /// The vertical blank decrements the counter of a sleeping task and makes it ready at 0.
    pub fn tick(&mut self) -> bool {
        if !self.live || self.suspended {
            return false;
        }
        if self.sleep > 0 {
            self.sleep -= 1;
            if self.sleep > 0 {
                return false;
            }
        }
        true
    }

    /// `d0 = frames; jsr 0x95A`: sleep for `frames` frames. 0 suspends the task (kernel state 2:
    /// the sleep call writes state 0x200 when the low byte of d0 is 0), see [`Scene::wake`].
    /// Like the kernel, only the low byte counts.
    pub fn sleep(&mut self, frames: u16) {
        let n = frames & 0xFF;
        if n == 0 {
            self.suspended = true;
        } else {
            self.sleep = n;
        }
    }

    /// `jsr 0xA76` and friends followed by the next script step: sleep, then continue in `next`.
    pub fn wait(&mut self, frames: u16, next: u16) {
        self.sleep(frames);
        self.state = next;
    }

    /// Kernel call 0x9FA: wake a suspended task (it runs in the next `tick`) with a sleep counter.
    pub fn wake(&mut self, counter: u16) {
        self.suspended = false;
        self.sleep = counter;
    }

    /// The skip-wait task 0x9E38C (slot 0xF0), which runs after all other tasks: while scripted
    /// control is on and a present player holds attack or jump it takes one more frame off every
    /// sleeping task (a task whose counter reaches 0 becomes ready and runs next frame).
    pub fn speed_up(&mut self) {
        if self.live && !self.suspended && self.sleep > 0 {
            self.sleep -= 1;
        }
    }

    /// Kernel call 0x92E (kill) or the end of the script (trap 1).
    pub fn kill(&mut self) {
        self.live = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs a scene for `frames` frames with a tiny script and records the frame of each state:
    /// the sleep counts of the original (a task that sleeps n frames runs again n frames later).
    fn run(frames: u32, script: impl Fn(&mut Scene, u32)) -> Vec<(u32, u16)> {
        let mut s = Scene::start(0);
        let mut seen = Vec::new();
        for f in 0..frames {
            if s.tick() {
                seen.push((f, s.state));
                script(&mut s, f);
            }
        }
        seen
    }

    #[test]
    fn sleep_n_runs_again_n_frames_later() {
        let seen = run(300, |s, _| match s.state {
            0 => s.wait(30, 2),
            2 => s.wait(60, 4),
            4 => s.wait(1, 6),
            6 => s.wait(2, 8),
            _ => s.kill(),
        });
        assert_eq!(seen, vec![(0, 0), (30, 2), (90, 4), (91, 6), (93, 8)]);
    }

    #[test]
    fn a_killed_scene_never_runs() {
        let mut s = Scene::start(0);
        assert!(s.tick());
        s.kill();
        assert!(!s.alive());
        assert!(!(0..10).any(|_| s.tick()));
    }

    #[test]
    fn sleep_zero_suspends_until_woken() {
        let mut s = Scene::start(0);
        assert!(s.tick());
        s.sleep(0);
        assert!(!(0..100).any(|_| s.tick()));
        s.wake(3); // 0x9FA: runs 3 frames later
        assert_eq!((0..5).map(|_| s.tick()).collect::<Vec<_>>(), vec![false, false, true, true, true]);
    }

    /// The sleep call keeps only the low byte of the frame count, like the kernel.
    #[test]
    fn sleep_uses_the_low_byte_like_the_kernel() {
        let mut s = Scene::start(0);
        s.tick();
        s.sleep(0x103); // low byte 3
        assert_eq!((0..4).map(|_| s.tick()).collect::<Vec<_>>(), vec![false, false, true, true]);
    }

    /// The skip-wait task takes one more frame off a sleeper each frame, so a long sleep passes
    /// at double speed while a button is held: counter 10 goes 9, 8 (skip), 7, 6, 5, 4, 3, 2, 1, 0
    /// in 5 frames and the task runs in the 6th.
    #[test]
    fn speed_up_halves_a_sleep() {
        let mut s = Scene::start(0);
        s.tick();
        s.sleep(10);
        let mut frames = 0;
        loop {
            frames += 1;
            if s.tick() {
                break;
            }
            s.speed_up();
        }
        assert_eq!(frames, 6);
    }

    #[test]
    fn frame_loop_with_the_free_counter() {
        // "for 600 frames: sleep 1, count += 1" as the title screen does.
        let mut s = Scene::start(0);
        let mut ran = 0;
        for _ in 0..1000 {
            if !s.tick() {
                continue;
            }
            ran += 1;
            s.count += 1;
            if s.count == 600 {
                s.kill();
            } else {
                s.sleep(1);
            }
        }
        assert_eq!((ran, s.alive()), (600, false));
    }
}
