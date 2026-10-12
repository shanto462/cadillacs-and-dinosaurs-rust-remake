local DINO = ...
local space = DINO.space
space:write_u16(0xFF85F0, 24)
space:write_u16(0xFF8602, 1)
DINO.spawn_enemy(72, 0, 1150, 296)
