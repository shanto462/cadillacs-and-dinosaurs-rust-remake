-- Screenshot probe for "dino" under MAME (used by re/tools/dump/dump_frontend.py). Contains no ROM data.
-- Inputs and RAM pokes by frame (frames = vblanks from power on), screenshots by frame list.
-- ONE MAME process runs every scenario one after the other (soft reset between them), so MAME is started only once.
-- Env: SH_MANIFEST file, one scenario per line: name|input file|pokes file|snaps|last frame
--        input file lines "a-b:KEYS" (R L D U move, A attack, J jump, S start, C coin, digit = player)
--        pokes file lines "frame hexaddr size value" (size 1, 2 or 4); frames count from the reset of the scenario
--        snaps "every:N:from:to" or "f1,f2,..."
--      SH_LOG log file (scenario, frame of every screenshot, mode changes)
local cpu = manager.machine.devices[":maincpu"]
local space = cpu.spaces["program"]
local function env(n, d) local v = os.getenv(n); if v == nil or v == "" then return d end return v end
local BITS = { R = 1, L = 2, D = 4, U = 8, A = 0x10, J = 0x20 }
local function read_file(path) local fh = io.open(path, "rb"); if not fh then return "" end local s = fh:read("a"); fh:close(); return s end
local function parse_script(text)
  local segs = {}
  text = text:gsub("#[^\n]*", "")
  for part in text:gmatch("[^,\n]+") do
    part = part:gsub("^%s+", ""):gsub("%s+$", "")
    if part ~= "" then
      local range, keys = part:match("^([^:]*):?(.*)$")
      local a, b = range:match("^%s*(%d+)%s*%-%s*(%d+)%s*$")
      if not a then a = range:match("^%s*(%d+)%s*$"); b = a end
      if a then
        local f = { p = { 0, 0, 0 }, start = 0, coin = 0 }
        local pl = 1
        for c in keys:upper():gmatch(".") do
          if c == "1" or c == "2" or c == "3" then pl = tonumber(c)
          elseif BITS[c] then f.p[pl] = f.p[pl] | BITS[c]
          elseif c == "S" then f.start = f.start | (1 << (pl - 1))
          elseif c == "C" then f.coin = f.coin | (1 << (pl - 1)) end
        end
        segs[#segs + 1] = { tonumber(a), tonumber(b), f }
      end
    end
  end
  return segs
end

local manifest = {}
for line in io.lines(env("SH_MANIFEST", "manifest.txt")) do
  local name, inp, pk, sn, last = line:match("^([^|]*)|([^|]*)|([^|]*)|([^|]*)|(%d+)")
  if name then manifest[#manifest + 1] = { name = name, input = inp, pokes = pk, snaps = sn, last = tonumber(last) } end
end
local logf = io.open(env("SH_LOG", "shots.log"), "w")
local segs, pokes, snaps, every, maxf = {}, {}, {}, nil, 0
local cur = { p = { 0, 0, 0 }, start = 0, coin = 0 }
local sc_index, frame = 0, 0
local lastsig = ""
local function script_at(f)
  local out = { p = { 0, 0, 0 }, start = 0, coin = 0 }
  for _, s in ipairs(segs) do
    if f >= s[1] and f <= s[2] then
      local g = s[3]
      for i = 1, 3 do out.p[i] = out.p[i] | g.p[i] end
      out.start = out.start | g.start; out.coin = out.coin | g.coin
    end
  end
  return out
end
local function load_scenario(i)
  local sc = manifest[i]
  segs = parse_script(read_file(sc.input))
  pokes = {}
  for line in string.gmatch(read_file(sc.pokes), "[^\n]+") do
    local f, a, s, v = line:match("^%s*(%d+)%s+(%x+)%s+(%d+)%s+(%S+)")
    if f then pokes[#pokes + 1] = { tonumber(f), tonumber(a, 16), tonumber(s), tonumber(v) } end
  end
  snaps, every = {}, nil
  local e1, e2, e3 = sc.snaps:match("^every:(%d+):(%d+):(%d+)")
  if e1 then every = { tonumber(e1), tonumber(e2), tonumber(e3) } else for s in sc.snaps:gmatch("%d+") do snaps[tonumber(s)] = true end end
  maxf = sc.last
  frame = 0
  lastsig = ""
  cur = { p = { 0, 0, 0 }, start = 0, coin = 0 }
  logf:write("SCENARIO " .. sc.name .. "\n")
end
local function raw_in1() return (~((cur.p[2] << 8) | cur.p[1])) & 0xFFFF end
local function raw_in0()
  local b = 0
  if cur.coin & 1 ~= 0 then b = b | 0x01 end
  if cur.coin & 2 ~= 0 then b = b | 0x02 end
  if cur.start & 1 ~= 0 then b = b | 0x10 end
  if cur.start & 2 ~= 0 then b = b | 0x20 end
  return (~(b << 8)) & 0xFF00
end
local function raw_in2()
  local b = cur.p[3]
  if cur.coin & 4 ~= 0 then b = b | 0x40 end
  if cur.start & 4 ~= 0 then b = b | 0x80 end
  return (~b) & 0xFF
end
local installed = false
local function install()
  SH_TAPS = {
    space:install_read_tap(0x800000, 0x800001, "sh_in1", function(o, d, m) return (d & ~m) | (raw_in1() & m) end),
    space:install_read_tap(0x800018, 0x800019, "sh_in0", function(o, d, m) return (d & ~m) | ((raw_in0() | 0xFF) & m) end),
    space:install_read_tap(0xF1C000, 0xF1C001, "sh_in2", function(o, d, m) return (d & ~m) | ((raw_in2() | 0xFF00) & m) end) }
  installed = true
end
local waiting_reset = 0
emu.register_frame_done(function()
  if not installed then install() end
  if sc_index == 0 then sc_index = 1; load_scenario(1); return end
  if waiting_reset > 0 then      -- a few frames after the reset, then the next scenario starts counting from frame 0
    waiting_reset = waiting_reset - 1
    if waiting_reset == 0 then sc_index = sc_index + 1; load_scenario(sc_index) end
    return
  end
  frame = frame + 1
  cur = script_at(frame)
  for _, p in ipairs(pokes) do
    if p[1] == frame then
      if p[3] == 1 then space:write_u8(p[2], p[4]) elseif p[3] == 2 then space:write_u16(p[2], p[4]) else space:write_u32(p[2], p[4]) end
    end
  end
  local sig = string.format("mode=%02x sub=%02x stage=%d area=%d ingame=%02x", space:read_u8(0xFF84D0), space:read_u8(0xFF84D1),
    space:read_u8(0xFF84D9), space:read_u8(0xFF8786), space:read_u8(0xFF84CC))
  if sig ~= lastsig then logf:write(string.format("f=%d %s\n", frame, sig)); lastsig = sig end
  local want = snaps[frame] or (every and frame >= every[2] and frame <= every[3] and (frame - every[2]) % every[1] == 0)
  if want then manager.machine.video:snapshot(); logf:write(string.format("f=%d SNAP\n", frame)) end
  logf:flush()
  if frame >= maxf then
    if sc_index >= #manifest then logf:write("END\n"); logf:close(); manager.machine:exit(); return end
    manager.machine:soft_reset()
    waiting_reset = 2
  end
end)
