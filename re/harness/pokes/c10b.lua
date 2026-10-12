-- Phase 1 task 4a (class 10): five thugs of class 10 near Jack, all waiting (entrance 8): two Blk Elmer, two Wrench T., one Hammer T.
-- Jack's HP is raised to 0x190 (the HP bar stays 100) so that the fight lasts. Jack is at x 0x41E, z 0x128 on the first frame.
local DINO = ...
local e = DINO.spawn_enemy
e(0x28, 0x0208, 0x470, 0x128, 0, 0, 0)  -- Blk Elmer, right of Jack
e(0x28, 0x0408, 0x4A0, 0x138, 0, 0, 0)  -- Wrench T.
e(0x28, 0x0008, 0x3C0, 0x128, 0, 0, 0)  -- Hammer T., left of Jack
e(0x28, 0x0208, 0x3A0, 0x138, 0, 0, 0)  -- Blk Elmer, left
e(0x28, 0x0408, 0x4D0, 0x120, 0, 0, 0)  -- Wrench T.
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
