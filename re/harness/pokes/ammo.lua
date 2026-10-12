-- Phase 1 task 2 (items): a pistol (6 shots) and two ammo boxes (class word 8) left of Jack (x 0x420, z 0x128), and
-- a second ammo box further left. The first box refills the pistol, the second is worth points (the gun is full).
local DINO = ...
local s = DINO.spawn_item
s(0x00, 0x02, 0x400, 0x128)  -- pistol
s(0x08, 0x00, 0x3E0, 0x128)  -- ammo box
s(0x08, 0x00, 0x3C0, 0x128)  -- ammo box
