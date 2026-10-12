-- Phase 1 task 4a (class 10: HAMMER T., BLK ELMER, WRENCH T.): one of each variant with different entrances around Jack.
-- Class word = class * 4 (class 10 = 0x28), variant = type * 0x100 + entrance (type 0 Hammer, 2 Elmer, 4 Wrench;
-- entrance 0 walk in, 2 drop, 4 / 6 jump attack, 8 wait until a player is near). Jack is at x 0x41E, z 0x128 on the first frame.
local DINO = ...
local e = DINO.spawn_enemy
e(0x28, 0x0008, 0x4C0, 0x128, 0, 0, 0)  -- Hammer T., waits to the right of Jack
e(0x28, 0x0200, 0x4F0, 0x138, 0, 0, 0)  -- Blk Elmer, walks in from the right
e(0x28, 0x0402, 0x3A0, 0x120, 0, 0, 0)  -- Wrench T., drops in on the left
