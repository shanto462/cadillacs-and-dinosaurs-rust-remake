local DINO = ...
local space = DINO.space
space:write_u16(0xFFB27C, 1000); space:write_u16(0xFFB284, 296)
space:write_u16(0xFFB3FC, 1034); space:write_u16(0xFFB404, 296)
DINO.spawn_item(0, 20, 1000, 296)
for k = 0, 3 do
    DINO.spawn_enemy(8, 0, 1250 + 8 * k, 296)
end
