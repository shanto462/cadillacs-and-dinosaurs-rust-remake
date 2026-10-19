//! Jump points for the test menu and `--jump NAME`: a named place in the game (an episode or area start, or the spot just before a
//! boss fight) that starts the game there. A point is the same start as `--episode E --area A --camx X` (see `main.rs`).
//!
//! The camera X values come from the MAME snapshots of the boss scenarios (`re/out/traces/*.state`: episode, area and camera X
//! at the sync point, which is at or just before the fight) and from the probe jumps of `re/harness/scripts`. Area starts have
//! camera X 0. Each point is checked with a headless screenshot (`re/out/jumps/NAME/`).

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
    Jump { name: "vice", title: "Episode 1: Vice T.", episode: 1, area: 2, camx: 0xA0, note: "just before the Vice T. fight (BS-V1 snapshot)" },
    Jump { name: "ep2", title: "Episode 2 start: swamp forest", episode: 2, area: 0, camx: 0, note: "forest edge, the Cadillac at the left" },
    Jump { name: "ep2a1", title: "Episode 2 area 1: forest", episode: 2, area: 1, camx: 0, note: "forest sub-area" },
    Jump { name: "ep2a2", title: "Episode 2 area 2: Butcher yard", episode: 2, area: 2, camx: 0, note: "dusk, tyre pile and crates" },
    Jump { name: "butcher", title: "Episode 2: Butcher", episode: 2, area: 2, camx: 0x1A0, note: "just before the Butcher fight (BS-B1 snapshot)" },
    Jump { name: "ep3", title: "Episode 3 start: hell road", episode: 3, area: 0, camx: 0, note: "open desert, the only area" },
    Jump { name: "car", title: "Episode 3: Cadillac drive", episode: 3, area: 0, camx: 0x0140, note: "just before the car arrives (ep3 natural recording: car alive at camera X 0x1A1)" },
    Jump { name: "hogg", title: "Episode 3: Hogg", episode: 3, area: 0, camx: 0x2D00, note: "just before the Hogg fight on foot (BS-H1 snapshot)" },
    Jump { name: "ep4", title: "Episode 4 start: Jack's garage", episode: 4, area: 0, camx: 0, note: "rubble and sea at night" },
    Jump { name: "ep4a1", title: "Episode 4 area 1: inside the garage", episode: 4, area: 1, camx: 0, note: "tyres and pipes" },
    Jump { name: "ep4a2", title: "Episode 4 area 2: boiler hall", episode: 4, area: 2, camx: 0, note: "boiler hall of the garage" },
    Jump { name: "slice", title: "Episode 4: Slice", episode: 4, area: 2, camx: 0x3D8, note: "just before the Slice fight (BS-S1 snapshot)" },
    Jump { name: "ep5", title: "Episode 5 start: village of flame", episode: 5, area: 0, camx: 0, note: "savanna with a burning group" },
    Jump { name: "ep5a1", title: "Episode 5 area 1: rock cliff", episode: 5, area: 1, camx: 0, note: "rock cliff, sub-area start" },
    Jump { name: "morgan", title: "Episode 5: Morgan", episode: 5, area: 1, camx: 0x1120, note: "just before the Morgan fight (BT-A1 snapshot)" },
    Jump { name: "ep6", title: "Episode 6 start: jungle and mine", episode: 6, area: 0, camx: 0, note: "wooden mine wall" },
    Jump { name: "ep6a1", title: "Episode 6 area 1: jungle swamp", episode: 6, area: 1, camx: 0, note: "jungle swamp at night" },
    Jump { name: "ep6a2", title: "Episode 6 area 2: giant head", episode: 6, area: 2, camx: 0, note: "swamp with the giant dinosaur head" },
    Jump { name: "tyrog", title: "Episode 6: Tyrog", episode: 6, area: 2, camx: 0x0EF0, note: "the Tyrog boss intro (E6A2 snapshot)" },
    Jump { name: "ep7", title: "Episode 7 start: the vault", episode: 7, area: 0, camx: 0, note: "roof of the vault, orange sky" },
    Jump { name: "ep7a1", title: "Episode 7 area 1: elevator", episode: 7, area: 1, camx: 0, note: "wooden elevator cage" },
    Jump { name: "lift", title: "Episode 7: the lift", episode: 7, area: 1, camx: 0, note: "the vertical shaft of the lift" },
    Jump { name: "ep7a2", title: "Episode 7 area 2: stone room", episode: 7, area: 2, camx: 0, note: "stone room with a fence" },
    Jump { name: "slisaur", title: "Episode 7: Slisaurs", episode: 7, area: 3, camx: 0x240, note: "the Slisaur fight (BT-SL snapshot)" },
    Jump { name: "ep8", title: "Episode 8 start: the under world", episode: 8, area: 0, camx: 0, note: "lab with the specimen tubes" },
    Jump { name: "ep8a1", title: "Episode 8 area 1", episode: 8, area: 1, camx: 0, note: "area 1 start" },
    Jump { name: "ep8a2", title: "Episode 8 area 2", episode: 8, area: 2, camx: 0, note: "area 2 start" },
    Jump { name: "ep8a3", title: "Episode 8 area 3", episode: 8, area: 3, camx: 0, note: "area 3 start" },
    Jump { name: "fessenden", title: "Episode 8: Fessenden", episode: 8, area: 3, camx: 0x0F30, note: "the Fessenden scene starts (ep8 natural recording: scripted from camera X 0xF36)" },
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
