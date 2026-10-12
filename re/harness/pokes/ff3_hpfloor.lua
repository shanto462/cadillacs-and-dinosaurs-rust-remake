local DINO = ...
local space = DINO.space
space:write_u16(0xFFB27C, 1000); space:write_u16(0xFFB284, 296)
space:write_u16(0xFFB3FC, 1034); space:write_u16(0xFFB404, 296)
space:write_u16(0xFFB57C, 1100); space:write_u16(0xFFB584, 296)
-- P2 Hannah HP poked to 3
space:write_u16(0xFFB460, 3)
space:write_u16(0xFFB462, 3)
