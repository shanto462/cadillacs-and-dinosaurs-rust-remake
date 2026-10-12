-- Phase 1 task 2 (items): four guns on the floor left of Jack (x 0x420, z 0x128) at the start of stage 1,
-- far from the first wave. Class words are class * 4 (0 = weapon). Variant = type * 2.
local DINO = ...
local s = DINO.spawn_item
s(0x00, 0x02, 0x400, 0x128)  -- pistol (6 shots)
s(0x00, 0x00, 0x3E0, 0x128)  -- rifle (6 shots)
s(0x00, 0x06, 0x3C0, 0x128)  -- Uzi (48 shots, automatic)
s(0x00, 0x08, 0x3A0, 0x128)  -- M-16 (48 shots, automatic)
