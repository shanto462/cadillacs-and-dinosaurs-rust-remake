-- Phase 1 task 4a2 (class 14): five BLADE / RAZOR wait (entrance 4, 0x1E0 frames) OFF SCREEN on both sides (camera X is 0x381: the screen is
-- X 0x369..0x501). When they wake they are off screen and free: state 0x36 (run in, or run away in 3 of 16 cases), then 0x38 (chase) when
-- they are still off screen after 100 frames. Jack stands still for 1100 frames; his HP is raised to 0x190.
local DINO = ...
local e = DINO.spawn_enemy
e(0x38, 0x0004, 0x540, 0x128, 0, 0, 0)  -- Blade, off screen right
e(0x38, 0x0204, 0x548, 0x140, 0, 0, 0)  -- Razor, off screen right
e(0x38, 0x0004, 0x34A, 0x120, 0, 1, 0)  -- Blade, off screen left
e(0x38, 0x0204, 0x340, 0x138, 0, 1, 0)  -- Razor, off screen left
e(0x38, 0x0004, 0x550, 0x118, 0, 0, 0)  -- Blade, off screen right
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
