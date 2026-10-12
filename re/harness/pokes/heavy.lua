-- Phase 1 task 3 (player weapons): containers (class word 0x14, kind 0x20A) in Jack's row: a barrel (variant 2) left of
-- Jack (x 0x400), then variants 6 and 8 (picked up without pushing the stick towards them) to the right (only 6 is used).
local DINO = ...
local s = DINO.spawn_item
s(0x14, 0x02, 0x400, 0x128, 0, 0x00)
s(0x14, 0x06, 0x4A0, 0x128, 0, 0x00)
s(0x14, 0x08, 0x4E0, 0x128, 0, 0x00)
