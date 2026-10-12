# Player port handoff (dino-rs)

Files (player port owns them):
- `dino-rs/crates/dino-game/src/game/player.rs`: Player struct, frame loop, whole state machine.
- `dino-rs/crates/dino-game/src/game/player_combat.rs`: collision passes, damage, hit effects on players.
- `dino-rs/crates/dino-assets/src/player.rs` + `dino-extract/src/player.rs`: all player/combat tables
  (`AssetPack::player`).

Character ids: 0 Jack, 1 Mustapha, 2 Hannah, 3 Mess (SPEC.md has a correction banner).

## Ported (with addresses)

- Join / start: `player_join` = 0x4D46/0x4D8E/0x4DD6 + 0x4E0A (start position table 0x4E56); spawn
  mode 0 (0x18A82 / 0x18A94, vehicle entrance state 0x432 set up), respawn 0x18B38..0x18BD4 (360 f
  invulnerability, drop-in state 0x42E).
- Frame: `players_update` 0x18834, `held_players_update` 0x188DA, `player_frame` 0x18952 (edge detect,
  input lock, dispatch, collision list 0x6718, draw list 0x49CA).
- Alive 0x18CD4 (all timers), command FSM 0x6AC8 (down, up + attack), terrain collision 0x12CAC /
  0x12D4E (floor, walls, obstacles, ceiling: `game/terrain.rs`, phase 1 task 1), the drop-in placement
  0x62D6 (`player_drop_place`), screen clamp 0x4FCE, power factor 0x11738.
- Ground: idle 0x19122/0x1916E (double-tap run), walk 0x192B8/0x192DE, jab/punch2/punch3/finisher +
  hit confirm + combo window (0x193CE..0x1965A), combo throw 0x195B2, special 0x1968E/0x196F2 and
  Mustapha's air special 0x1AAE6, Jack dash attack 0x19720, airborne lunge 0x1ABD4, recovery 0x19812,
  command moves for all 4 characters 0x1B6EC..0x1BBA0 (+ states 0x24/0x26/0x28).
- Run 0x19FC6/0x1A012. Air: jump 0x1A5D0/0x1A64A, air control 0x1BE3C, jump attacks 0x1A704 (low
  0x1AA54, dive 0x1A9A4 + landing 0x1BD56), wall jump 0x1A744 (chars 1, 2), landing 0x1A81A, fall
  0x1A848, running jump 0x1A906, running-jump attacks of all 4 chars (0x1AD06, 0x1B496, 0x1B568, 0x1B5D4),
  drop 0x1B67C.
- Hurt 0x1C02C: light 0/2/0x1A, knockdown 4/6/0x1C, thrown 8, burn 0xA/0xC (flames via effects port),
  pushback 0xE..0x18, wall hit 0x20, stagger 0x22, getup 0x1CD9A, dizzy (0x1D0FA roll, 0x1CE02),
  mash 0x6012, lie 0x1D17E, special buffer 0x63E6, recover 0x1C0DA (20 f invulnerability).
- Grab 0x1D652: hold, knees (0x1D746), throw (0x1D850, offsets 0x2CB32), release 0x1D72A.
- Held by enemy 0x1D2BE: mash, escape attack 0x1D3F8 (Mustapha reverses the hold), stumble 0x1D48E,
  follow 0x1D4EA (table 0x2CBE0 kept as a byte blob).
- Death 0x1BF42, lives / continue 0x1BFD0.
- Combat (player_combat.rs): `pass_10664` (player vs player), `pass_107f8` (probe 0x108FA, grab
  0x1084A/0x108AE, attack 0x10962/0x109C2), `player_damage` 0x2B904, `enemy_damage` 0x6422,
  `player_grab_hit` 0x12526, special HP cost 0x10B08, score add 0xFFC8 (BCD table 0x10292),
  `player_hit_by_enemy(p, e)` = 0x11682 (+ 0x117D8 variant), weapon drop 0x1177A. Unit tests for the
  damage formulas and BCD add are in player_combat.rs.

## Stubbed (TODO markers in the code)

- Items / weapons: `player_find_pickup` (0x5AA2) always finds nothing; pickup 0x19832 and weapon
  states 0x14..0x22 and run state 0x202 fall back to a jab; weapon idle/walk anims (0x5BDC/0x5CA6),
  overlay layer 0x1542, heavy object 0x20A jumps (0x1B038).
- Enemy fields not exposed yet: `player_enemy_w84` (+$84) returns 0, `player_enemy_b87` (+$87, guard
  0x10B50) returns 0, `player_g4f3` (G.$4F3) returns 3 (player vs player off), 0x6606 downed claim,
  0x1128E multi-part boss link.
- Scripted control G.$4DD input (0x1DB5E), vehicle wait G.$505 (state 0x432 waits forever), states
  0x18/0x1A/0x2A/0x2C and air 0x18..0x26/0x34, sound codes (the voice routines 0x95CE..0x9C0C are ported in `snd.rs`; pan and volume are not recorded),
  respawn sparkle 0x958E4, dust 0x92E5E, water ripple 0x60E2, DIP lives (fixed 2).

## Next steps (in order)

1. Screenshots for jump (`J`), run (double tap `R`), jab chain into an enemy, special (`AJ`);
   compare frame by frame against MAME where possible.
2. Wire the enemy port: replace `player_enemy_w84` / `player_enemy_b87`, call `player_hit_by_enemy`
   from its pass 0x115B0, test grabs and hurt reactions against real enemies.
3. (done in phase 1 task 1: `Game::terrain`, `terrain.rs`.)
4. Items: port find_pickup / pickup / weapon states once pool E items expose their type (+$A0).
5. Scripted control 0x1DB0C/0x1DB5E and the vehicle entrance (G.$505) with the flow port.

## Known issues / not yet verified

- Verified at wrap-up: `cargo build` green, `cargo test -p dino-game player` 5/5 pass, screenshot
  `re/out/shots/p_walk.png` (Jack placed at the stage 1 start, walks right, jab frame shown, HUD bar).
  Not yet checked: jumps, runs, the chain against enemies, hurt reactions (no enemy hits wired yet).
- `player_g4f3` default (3) disables player-vs-player hits; the real G.$4F3 value is unknown.
- First frame of a stage: the floor pointer emulation may differ from the original by one frame.
