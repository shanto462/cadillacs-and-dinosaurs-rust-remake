//! Jump points for the test menu and `--jump NAME`: a named place in the game (an episode or area start) that starts the game there.
//! The boss scene points (Vice T., Butcher, the car, Slice, Morgan, the lift, the Slisaurs, Fessenden) were removed on 2026-10-21 (user:
//! not needed; the Butcher and Slice points never brought the boss). A point is the same start as `--episode E --area A --camx X` (see `main.rs`).
//!
//! Area starts have camera X 0. Each point is checked with a headless screenshot (`re/out/jumps/NAME/`).

/// One jump point. `episode` is 1..8, `area` 0..3 (Episode 3 has one area: its points use `camx` only).
pub struct Jump {
    pub name: &'static str,
    pub title: &'static str,
    pub episode: u8,
    pub area: u8,
    pub camx: u16,
    pub note: &'static str,
}

pub const JUMPS: &[Jump] = &[
    Jump { name: "ep1", title: "Episode 1 start: flooded city", episode: 1, area: 0, camx: 0, note: "first frame of control" },
    Jump { name: "ep1a1", title: "Episode 1 area 1: the building", episode: 1, area: 1, camx: 0, note: "hall with the armour statues" },
    Jump { name: "ep1a2", title: "Episode 1 area 2: 47th street", episode: 1, area: 2, camx: 0, note: "street with the ruined wall" },
    Jump { name: "ep2", title: "Episode 2 start: swamp forest", episode: 2, area: 0, camx: 0, note: "forest edge, the Cadillac at the left" },
    Jump { name: "ep2a1", title: "Episode 2 area 1: forest", episode: 2, area: 1, camx: 0, note: "forest sub-area" },
    Jump { name: "ep2a2", title: "Episode 2 area 2: Butcher yard", episode: 2, area: 2, camx: 0, note: "dusk, tyre pile and crates" },
    Jump { name: "ep3", title: "Episode 3 start: hell road", episode: 3, area: 0, camx: 0, note: "open desert, the only area" },
    Jump { name: "ep4", title: "Episode 4 start: Jack's garage", episode: 4, area: 0, camx: 0, note: "rubble and sea at night" },
    Jump { name: "ep4a1", title: "Episode 4 area 1: inside the garage", episode: 4, area: 1, camx: 0, note: "tyres and pipes" },
    Jump { name: "ep4a2", title: "Episode 4 area 2: boiler hall", episode: 4, area: 2, camx: 0, note: "boiler hall of the garage" },
    Jump { name: "ep5", title: "Episode 5 start: village of flame", episode: 5, area: 0, camx: 0, note: "savanna with a burning group" },
    Jump { name: "ep5a1", title: "Episode 5 area 1: rock cliff", episode: 5, area: 1, camx: 0, note: "rock cliff, sub-area start" },
    Jump { name: "ep6", title: "Episode 6 start: jungle and mine", episode: 6, area: 0, camx: 0, note: "wooden mine wall" },
    Jump { name: "ep6a1", title: "Episode 6 area 1: jungle swamp", episode: 6, area: 1, camx: 0, note: "jungle swamp at night" },
    Jump { name: "ep6a2", title: "Episode 6 area 2: giant head", episode: 6, area: 2, camx: 0, note: "swamp with the giant dinosaur head" },
    Jump { name: "ep7", title: "Episode 7 start: the vault", episode: 7, area: 0, camx: 0, note: "roof of the vault, orange sky" },
    Jump { name: "ep7a1", title: "Episode 7 area 1: elevator", episode: 7, area: 1, camx: 0, note: "wooden elevator cage" },
    Jump { name: "ep7a2", title: "Episode 7 area 2: stone room", episode: 7, area: 2, camx: 0, note: "stone room with a fence" },
    Jump { name: "ep8", title: "Episode 8 start: the under world", episode: 8, area: 0, camx: 0, note: "lab with the specimen tubes" },
    Jump { name: "ep8a1", title: "Episode 8 area 1", episode: 8, area: 1, camx: 0, note: "area 1 start" },
    Jump { name: "ep8a2", title: "Episode 8 area 2", episode: 8, area: 2, camx: 0, note: "area 2 start" },
    Jump { name: "ep8a3", title: "Episode 8 area 3", episode: 8, area: 3, camx: 0, note: "area 3 start" },
];

/// The point with this name (lower case, as listed by `--list-jumps`).
pub fn find(name: &str) -> Option<&'static Jump> {
    JUMPS.iter().find(|j| j.name == name)
}

/// The names of all points, separated by spaces (the error text of an unknown name).
pub fn names() -> String {
    JUMPS.iter().map(|j| j.name).collect::<Vec<_>>().join(" ")
}

/// One line per point: name, title, episode, area and camera X, note (for `--list-jumps`).
pub fn list_text() -> String {
    let mut out = String::new();
    for j in JUMPS {
        out.push_str(&format!("{:<10} {:<34} {} {} 0x{:04X}  {}\n", j.name, j.title, j.episode, j.area, j.camx, j.note));
    }
    out
}
