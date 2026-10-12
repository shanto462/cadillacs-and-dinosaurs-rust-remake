-- Phase 1 task 2 (items): food, treasure and an ammo box on the floor left of Jack at the start of stage 1,
-- and Jack at 48 of 100 HP so that the first food heals and a later one is worth points. Class words are
-- class * 4: 0x10 food (+$26 = 1..18), 0x24 treasure (1..16), 8 ammo box.
local DINO = ...
local s = DINO.spawn_item
local sp = DINO.space
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6A, 0x30)  -- HP bar, HP, HP before the last hit
sp:write_u16(p1 + 0x6C, 0x30)
sp:write_u16(p1 + 0x6E, 0x30)
s(0x10, 0x08, 0x400, 0x128)  -- SALAD: heals 32 (and 5000 points at full HP)
s(0x10, 0x01, 0x3E0, 0x128)  -- BARBECUE: heals 100
s(0x10, 0x05, 0x3C0, 0x128)  -- BURGER: Jack is at full HP by now: 5000 points
s(0x24, 0x0D, 0x3A0, 0x128)  -- DIME: 1000 points
