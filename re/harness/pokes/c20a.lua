-- Phase 1 task 4a (class 20: PUNK and THUG): five thugs with the five entrances. Class word = class * 4 (class 20 = 0x50),
-- variant = type * 0x100 + entrance (type 0 Punk, 2 Thug; entrance 0 / 2 stand and wait, 4 walk in, 6 dash in, 8 fade in,
-- 0xA drop). Jack's HP is raised to 0x190. Jack is at x 0x41E, z 0x128 on the first frame.
local DINO = ...
local e = DINO.spawn_enemy
e(0x50, 0x0004, 0x4E0, 0x128, 0, 0, 0)  -- Punk walks in from the right
e(0x50, 0x0206, 0x3A0, 0x138, 0, 0, 0)  -- Thug dashes in from the left
e(0x50, 0x0008, 0x4A0, 0x120, 0, 0, 0)  -- Punk fades in
e(0x50, 0x020A, 0x470, 0x130, 0, 1, 0)  -- Thug drops in
e(0x50, 0x0002, 0x4D0, 0x128, 0, 0, 0)  -- Punk waits (sits)
e(0x50, 0x0200, 0x3C0, 0x128, 0, 1, 0)  -- Thug waits
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
