-- Phase 1 task 4a2 (class 14): BLADE and RAZOR with two knives each (entrances 0xA and 0xC) in front of Jack, who punches all the time
-- (script: A every 8 frames): the thrown blades (class 41) fly into his fist (the hit blade falls and becomes a knife item), the
-- guard blocks, knives drop. Jack's HP is raised to 0x190. Jack is at x 0x41E, z 0x128 on the first frame.
local DINO = ...
local e = DINO.spawn_enemy
e(0x38, 0x000A, 0x4D0, 0x128, 0, 0, 0)  -- Blade, walks in with two knives
e(0x38, 0x020C, 0x4F0, 0x128, 0, 0, 0)  -- Razor, walks in with two knives, level with Jack
e(0x38, 0x020A, 0x500, 0x130, 0, 0, 0)  -- Razor, walks in with two knives
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
