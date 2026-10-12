local DINO = ...
local space = DINO.space
space:write_u16(0xFFB27C, 1000); space:write_u16(0xFFB284, 296)
space:write_u16(0xFFB3FC, 1004); space:write_u16(0xFFB404, 296)
space:write_u16(0xFFB57C, 1008); space:write_u16(0xFFB584, 296)
DINO.spawn_item(0, 2, 1002, 296)
