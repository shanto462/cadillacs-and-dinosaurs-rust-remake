-- Phase 1 task 3b (enemy grab): five thugs (class 2: Ferris, Gneiss, Driver) around Jack, standing in the entrance "wait"
-- (low byte of the variant 4: they wake up when a player is near). Jack is at x 0x41E, z 0x128 on the first frame.
-- Class word = class * 4, variant = type * 0x100 + entrance (type 0 Ferris, 2 Gneiss, 4 Driver).
local DINO = ...
local e = DINO.spawn_enemy
e(0x08, 0x0004, 0x470, 0x128, 0, 0, 0)  -- Ferris, right of Jack
e(0x08, 0x0204, 0x4A0, 0x130, 0, 0, 0)  -- Gneiss
e(0x08, 0x0404, 0x4D0, 0x120, 0, 0, 0)  -- Driver
e(0x08, 0x0004, 0x3C0, 0x128, 0, 0, 0)  -- Ferris, left of Jack
e(0x08, 0x0204, 0x3A0, 0x138, 0, 0, 0)  -- Gneiss, left of Jack
