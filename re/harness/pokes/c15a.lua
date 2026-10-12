-- Phase 1 task 4a2 (class 15): the flying pieces (pool A class 15, spawned by the unported pool B prop B#40 in the game): rows 0, 2, 4, 6, 8
-- of the table 0x42050 facing both ways, round Jack. Jack's HP is raised to 0x190. Jack is at x 0x41E, z 0x128 on the first frame.
local DINO = ...
local sp = DINO.space
local function piece(row, x, y, z, facing)
  local a = DINO.spawn_enemy(0x3C, row, x, z, 0, 0, 0)
  sp:write_u16(a + 0x0C, y)
  sp:write_u8(a + 0x24, facing)
end
piece(0, 0x440, 0x20, 0x128, 0)
piece(2, 0x450, 0x20, 0x130, 1)
piece(4, 0x460, 0x20, 0x120, 0)
piece(6, 0x430, 0x20, 0x138, 1)
piece(8, 0x470, 0x20, 0x118, 0)
piece(0, 0x3E0, 0x40, 0x128, 1)
piece(4, 0x3F0, 0x40, 0x130, 1)
local p1 = 0xFFB274
sp:write_u16(p1 + 0x6C, 0x190)
sp:write_u16(p1 + 0x6E, 0x190)
