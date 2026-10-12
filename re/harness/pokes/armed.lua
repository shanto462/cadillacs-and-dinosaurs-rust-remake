-- Phase 1 task 3 (player weapons): the wooden bat (type 7, kind 0x203) at x 0x550 and a shotgun (type 5, kind 0x208) at
-- x 0x572, both in Jack's row (z 0x128), on his way to the first wave. Jack takes the bat, fights with it and loses it
-- when he is hit. (Weapons more than 0x200 right of the camera free themselves at once, so they cannot lie further right.)
local DINO = ...
local s = DINO.spawn_item
s(0x00, 0x0E, 0x550, 0x128)  -- bat
s(0x00, 0x0A, 0x572, 0x128)  -- shotgun
