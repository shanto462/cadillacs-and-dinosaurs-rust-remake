-- Phase 1 task 3 (player weapons): an Uzi (type 3, 48 shots) left of Jack (x 0x400) and an M-16 (type 4, 48 shots) at x 0x4A0.
local DINO = ...
local s = DINO.spawn_item
s(0x00, 0x06, 0x400, 0x128)  -- Uzi
s(0x00, 0x08, 0x4A0, 0x128)  -- M-16
