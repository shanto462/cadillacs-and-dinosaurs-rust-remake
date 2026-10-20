local DINO = ...
local space = DINO.space
space:write_u16(0xFFB27C, 1000); space:write_u16(0xFFB284, 296)
space:write_u16(0xFFB3FC, 1010); space:write_u16(0xFFB404, 296)
DINO.spawn_enemy(8, 0, 1150, 296)
