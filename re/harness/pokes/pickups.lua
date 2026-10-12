-- Phase 1 task 3 (player weapons): an ammo box (class word 8) at x 0x400, an Uzi left of it (0x3C0), a pistol (0x440) and a
-- rifle (0x4A0) to the right. Jack takes the box with no gun in the hand (+$10F = 1), the Uzi (refilled at once), then the
-- pistol and the rifle (each one drops the weapon in his hand).
local DINO = ...
local s = DINO.spawn_item
s(0x08, 0x00, 0x400, 0x128)  -- ammo box
s(0x00, 0x06, 0x3C0, 0x128)  -- Uzi
s(0x00, 0x02, 0x440, 0x128)  -- pistol
s(0x00, 0x00, 0x4A0, 0x128)  -- rifle
