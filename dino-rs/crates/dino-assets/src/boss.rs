//! Boss services data (owner: enemy port, M3-02). Formats: `plan/game/design/D3c-bosses-1.md` section 1.7.

use crate::enemy::{SpawnRec, Wave};

/// The reinforcement controller lists of one stage (pool B class 22, 0x7BEF6).
#[derive(Clone, Debug, Default)]
pub struct BossLists {
    /// Timed list (table 0x7BF36): records with a delay. The last record has a negative delay (the end).
    pub timed: Vec<SpawnRec>,
    /// ROM address of the timed list (16 bytes per record): the original's `+$A0` points into it.
    pub timed_addr: u32,
    /// Threshold groups (table 0x7BF56): each group waits for its thresholds, the final thresholds end the list.
    pub groups: Wave,
}

#[derive(Clone, Debug, Default)]
pub struct BossData {
    /// Lists by stage index (G.$4D9).
    pub stages: Vec<BossLists>,
    /// Timed list limit of the alive count by players - 1 (table 0x7BFEE): spawn while the count is below it.
    pub alive_limit: [u8; 3],
}
