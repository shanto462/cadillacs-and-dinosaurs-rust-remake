-- Phase 1 task 2 (items): containers (class word 0x14) left of Jack (x 0x420, z 0x128), in two rows of
-- depth. Variants (+$26): 2 (barrel) in Jack's row, 0 (crate) 0x20 nearer the camera. The drop rows
-- (+$96, table 0xB64F2): 0x0C = food 8 (salad), 0x2A = a bomb or a treasure.
local DINO = ...
local s = DINO.spawn_item
s(0x14, 0x02, 0x3C0, 0x128, 0, 0x0C)
s(0x14, 0x00, 0x3C0, 0x108, 0, 0x2A)
