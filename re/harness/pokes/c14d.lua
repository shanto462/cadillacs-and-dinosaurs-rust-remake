-- Phase 1 task 4a2 (class 14): BLADE and RAZOR that stand and wait OFF SCREEN (camera X is 0x381: the screen is X 0x369..0x501), one
-- on the right, two on the left. After 0x1E0 / 0x4B0 frames, or when Jack comes near, they wake, are off screen and free, and run
-- back (states 0x36 and 0x38). Jack's HP is raised to 0x190. Jack is at x 0x41E, z 0x128 on the first frame.
local DINO = ...
local e = DINO.spawn_enemy
e(0x38, 0x0004, 0x530, 0x128, 0, 0, 0)  -- Blade, off screen right
e(0x38, 0x0204, 0x350, 0x130, 0, 1, 0)  -- Razor, off screen left
e(0x38, 0x0002, 0x345, 0x120, 0, 1, 0)  -- Blade, off screen left (entrance 2: 0x4B0 frames)
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
