-- Phase 1 task 3 (player weapons): a club (type 2, kind 0x202) left of Jack and, to his right in steps of 0x30, the
-- wooden bat (type 7, kind 0x203), the stick (type 8, 0x202), a heavy weapon (type 10, 0x203, two swings) and a fire
-- bomb (0x201). Jack is at x 0x41E, z 0x128 on the first frame. Class word 0 = weapon, variant = type * 2.
local DINO = ...
local s = DINO.spawn_item
s(0x00, 0x04, 0x400, 0x128)  -- club
s(0x00, 0x0E, 0x440, 0x128)  -- bat
s(0x00, 0x10, 0x470, 0x128)  -- stick
s(0x00, 0x14, 0x4A0, 0x128)  -- heavy weapon (type 10)
s(0x00, 0x1A, 0x4D0, 0x128)  -- fire bomb
