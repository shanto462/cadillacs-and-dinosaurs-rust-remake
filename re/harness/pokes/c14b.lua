-- Phase 1 task 4a2 (class 14): two BLADE and two RAZOR that stand and wait (entrances 2 and 4) round Jack: they wake up when Jack
-- comes near (or after 0x4B0 / 0x1E0 frames) and fight for the whole run. Jack's HP is raised to 0x190. Jack is at x 0x41E, z 0x128.
local DINO = ...
local e = DINO.spawn_enemy
e(0x38, 0x0002, 0x440, 0x128, 0, 0, 0)  -- Blade, waits (entrance 2)
e(0x38, 0x0202, 0x3F0, 0x138, 0, 1, 0)  -- Razor, waits (entrance 2)
e(0x38, 0x0004, 0x480, 0x120, 0, 0, 0)  -- Blade, waits (entrance 4)
e(0x38, 0x0204, 0x3C0, 0x130, 0, 1, 0)  -- Razor, waits (entrance 4)
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
