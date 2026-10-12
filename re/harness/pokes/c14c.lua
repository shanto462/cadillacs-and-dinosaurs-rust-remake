-- Phase 1 task 4a2 (class 14): the jump in entrances 6, 8, 0xA, 0xC of BLADE and RAZOR (two knives in the hands), round Jack.
-- Jack's HP is raised to 0x190. Jack is at x 0x41E, z 0x128 on the first frame.
local DINO = ...
local e = DINO.spawn_enemy
e(0x38, 0x0006, 0x470, 0x128, 0, 0, 0)  -- Blade jumps in from above
e(0x38, 0x0208, 0x3E0, 0x138, 0, 1, 0)  -- Razor jumps in beside Jack
e(0x38, 0x000A, 0x4A0, 0x120, 0, 0, 0)  -- Blade walks in with two knives
e(0x38, 0x020C, 0x3C0, 0x130, 0, 1, 0)  -- Razor walks in with two knives, level with Jack
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
