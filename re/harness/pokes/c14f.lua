-- Phase 1 task 4a2 (class 14): four BLADE / RAZOR (twelve pool A slots) with drop rows: entrances 0, 4, 0xA and 0xC, they fight Jack one after
-- the other. Jack's HP is raised to 0x190. Jack is at x 0x41E, z 0x128 on the first frame.
local DINO = ...
local e = DINO.spawn_enemy
e(0x38, 0x0000, 0x4F0, 0x128, 0, 0, 1)  -- Blade, walks in
e(0x38, 0x0204, 0x3A0, 0x130, 0, 1, 3)  -- Razor, waits
e(0x38, 0x000A, 0x4C0, 0x138, 0, 0, 5)  -- Blade, walks in with two knives
e(0x38, 0x020C, 0x3C0, 0x120, 0, 1, 9)  -- Razor, walks in with two knives
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
