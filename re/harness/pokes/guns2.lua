-- Phase 1 task 2 (items): a shotgun (six pellets per shot), a bazooka (a rocket shell and an explosion) and the
-- wooden bat (type 7) left of Jack (x 0x420, z 0x128).
local DINO = ...
local s = DINO.spawn_item
s(0x00, 0x0A, 0x400, 0x128)  -- shotgun
s(0x00, 0x0C, 0x3E0, 0x128)  -- bazooka
s(0x00, 0x0E, 0x3C0, 0x128)  -- bat
