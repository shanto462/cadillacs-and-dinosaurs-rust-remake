-- Phase 1 task 4a2 (class 14): a BLADE and a RAZOR walk in (entrance 0) from the right and run into containers that stand in their way
-- (the terrain obstacle bit 0x80 starts state 0x34), a third waits (entrance 4). Jack's HP is raised to 0x190.
local DINO = ...
DINO.spawn_item(0x14, 0x0000, 0x4D0, 0x128, 0, 0)  -- container in the path of the first
DINO.spawn_item(0x14, 0x0004, 0x4A0, 0x13C, 0, 3)  -- another one, drops food
local e = DINO.spawn_enemy
e(0x38, 0x0000, 0x520, 0x128, 0, 0, 2)  -- Blade, walks in
e(0x38, 0x0200, 0x530, 0x13C, 0, 0, 3)  -- Razor, walks in
e(0x38, 0x0004, 0x3A0, 0x120, 0, 1, 0)  -- Blade, waits (left)
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
