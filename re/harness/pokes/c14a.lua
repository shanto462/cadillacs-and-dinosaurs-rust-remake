-- Phase 1 task 4a2 (class 14): three BLADE / RAZOR (three pool A slots each) that walk in (entrance 0) and wait (entrances 2 and 4)
-- round Jack. Jack's HP is raised to 0x190. Jack is at x 0x41E, z 0x128 on the first frame.
local DINO = ...
local e = DINO.spawn_enemy
e(0x38, 0x0000, 0x500, 0x128, 0, 0, 0)  -- Blade, walks in from the right
e(0x38, 0x0202, 0x3C0, 0x138, 0, 1, 0)  -- Razor, waits (left)
e(0x38, 0x0004, 0x4A0, 0x120, 0, 0, 0)  -- Blade, waits
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
