# Phase 1: stage 1 complete

**Goal:** stage 1 (internal stage 0, the flooded city) plays exactly like the original from the
first frame to the stage clear screen, for 1 to 3 players.

## Classes used by stage 1 scripts (from `re/specs/enemy/stage_scripts.txt`)

- Pool A: 2 (Ferris / Gneiss / Driver, done), 10 (Hammer T., Blk Elmer, Wrench T.),
  14 (Blade, Razor), 18 (Vice T. boss, Walther), 20 (Punk, Thug), 49 / 50 (background birds).
- Pool B: 12, 14, 15 (wave controller, done), 20, 31, 40, 66, 69, 73, 74.
- Pool E: 5 (containers and the items they drop).
- Found by the phase 0 comparison, not in the scripts: pool A 54 and 55 (opening scene, see task 4).

## Tasks (one agent each, in order)

How to check a task: `re/harness/README.md`. Every task ends with a step run (`MODE=step`) of the
scenarios `stage1_jack`, `stage1_late`, `explore` and `stage1_fight` (commands in the README) in
which the classes of the task no longer appear as `[NOT PORTED]`, and the new `[ported]` lines show
no difference. Add the class to `PORTED_CLASSES` in `dino-game/src/game/trace.rs` when it is ported.

1. **Terrain collision (all actors).** Found by every comparison: the port has only the floor part
   of 0x12CB4. Port the rest, with the original's tables, for players (entry 0x12CAC, tail 0x1361E),
   enemies, items and props: wall / street edge pushes of x and z and the `+$50` wall bits
   (0x12FE4 floor lookup, 0x1309C wall lookup in the scroll2 tile attributes and its shape handlers
   from 0x13138, tables 0x13B0C by stage and area and 0x1370C), the vertical scene variant 0x136E4,
   the floor only variant 0x12D4E, and the other-object obstacle test 0x12E9A (sets enemy `+$94`
   and wall bit 0x80). Seen: Jack at the right wall (camera lock x 0x63D, `wall` 4), enemies stopped
   at the back line (z 0x120 / 0x14F), an enemy attacking an obstacle (combo 7, 0x33F0C).
   Phase 0 evidence: stage1_jack frame 313 (`player.wall`), explore 335, stage1_fight 186, 441, 473, 780.
2. **Items (enemy area).** Pool E class 5 and the item objects: food (heal), treasure (points),
   weapons with ammo, drop tables (0x24B36, 0xB6D12), pickup rules, the item list the player
   reads. Spec: `re/specs/enemy/SPEC.md` section 13. Handlers from the table 0x233F0: class 0
   0x24D44 (weapons), class 5 0x28C76 (containers: hit by Jack and by thugs, hit sparks, the break
   sounds 0xF5 0x6E, debris effect class 0x0B 0x934F6; G.$67BE changes at explore frame 109),
   class 9 0x2A028 (treasure). A sound 0x34 repeats every 51 frames from frame 967 of stage1_fight;
   its source is not identified yet (probably an item). Pool E update 0x233C6 (also the free
   list G.$65FE / $65F4). Phase 0 evidence: explore / stage1_fight frames 109, 789, 791.
3. **Weapons and pickup (player area).** `find_pickup` 0x5AA2, pickup state, held weapon states
   (0x201/0x202 melee, 0x203 charge, 0x206/0x207 guns, 0x209 recoil, 0x20A heavy object), ammo,
   dropping on hit. Spec: `re/specs/player/SPEC.md` section 4.9.
4. **Remaining stage 1 enemies (enemy area).** Pool A classes (handlers from table 0x2346C):
   10 0x3B828, 14 0x4020E, 20 0x457B0, 49 0x56A70 (three off-screen bird controllers, present at the
   first frame), 50 0x56C80 (birds spawned at run time, one `rand()` each), and the opening scene
   objects that exist at the first controllable frame and are gone by frame ~60: **54 (0x36) 0x576D2 and
   55 (0x37) 0x5785C** (their death calls the shared 0x2AF86: kills `+$10B`, rank 0x182A2, `rand()`,
   and the voice 0x9796). Pool B classes (table 0x2362E): 12 0x78B52, 14 0x7B0BE, 20 0x7BB68 (the idle
   "GO" hint: scroll1 rows 46-49, bank 1 palette 10, sound 0x3E, timer G.$4FE, 600 frames),
   31 0x81046 (draws a sprite list kept in pool D slot 15 at 0xE574 and rewrites it every frame),
   40 0x836A4, 66 0x86D76 (scroll1 text banners, rows 57-58), 69 0x88A84, 73 0x89ABA, 74 0x89B48.
   Decode their roles first.
5. **Vice T. and Walther (enemy area).** Class 18 boss logic, summons, rage, the bullet (class 59),
   the boss HP bar and the boss-defeated flow. Spec: section 12.
6. **In-game prompts of the join task (flow area).** The INSERT COIN prompts of absent players blink
   in scroll1 during play (rows 36-37, columns 17-30 and 33-46, 48 frames on and 80 off): task 0x6C4A,
   redraw 0x7562 (`flow_join_task` and `flow_join_prompts` are stubs). The rest of the join and
   continue logic is phase 2, task 4; port at least the in-game prompts here, or the exactness
   pass cannot match.
7. **Enemy library leftovers (enemy area).** In ported code, known from the enemy handoff and not
   yet seen in a trace: effects 0x606E (water splash), 0x92E5E (dust), 0x94338 (dizzy stars),
   0x93D48 (burn death), 0x2AD98 (flames), the second drop call of the burn death (0x2ACD2 /
   0x2ACE6), 0x1D95E / 0x1DAC4 (scripted walk to the next area), the guard handlers 0x10C10 and
   0x10B50, the multi-part boss link 0x1128E. Check them with scenarios that use them.
8. **Stage 1 flow (flow area).** Area changes (mansion, street), stage clear score tally,
   continue into stage 2 setup (stub the stage 2 content).
9. **Exactness pass.** Phase 0 comparison over a full stage 1 run (scripted inputs, 1 and 3
   players): `run` mode, no `follow`, from the first controllable frame. Fix differences. Extend the
   harness first where it is blind (scroll registers, pan and volume of sound requests, see the
   phase 0 status).

## Done when

The phase 0 comparison of a full stage 1 run shows no unexplained difference, and the boss fight
and stage clear screen match the original.

## Status

Pending. Handoff notes: `re/specs/enemy/HANDOFF.md`, `re/specs/player/HANDOFF.md`, `re/specs/flow/HANDOFF.md`.
Tasks 1 to 3, 6 and 7 come from the phase 0 comparison (2026-10-12).
