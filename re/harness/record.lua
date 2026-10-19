-- MAME reference recorder for "dino" (Cadillacs and Dinosaurs). See README.md in this folder.
--
-- MAME passes no arguments to an autoboot script, so the settings come from environment
-- variables (record.sh sets them):
--   DINO_BOOT    boot input script, frames counted in vblanks from power on (coin, start, select)
--   DINO_INPUT   test input script, frames counted in game frames from the sync point (k = 0, 1, ...)
--   DINO_FRAMES  number of test frames to record (default 600)
--   DINO_TRACE   output trace file (format: README.md, reader: trace.py)
--   DINO_STATE   output snapshot file (all RAM at the sync point, for the port's --load-state)
--   DINO_SYNC    "control" (default): first frame where player 1 can be controlled in a stage;
--                "control+N": N game frames after that (the frame counter G.$0000 counts them);
--                "vblank:N": sync at vblank N
--   DINO_CLOCK   "game" (default): completed game frames at the IRQ tap;
--                "video": every video frame, including boot and slowdown, at frame_done.
--                With video, vblank:0 and vblank:1 both start at the first video frame.
--   DINO_MAX_VBLANKS  give up after this many vblanks without reaching the sync point (default 30000)
--   DINO_SNAP_EVERY   also save a PNG screenshot every N test frames (0 = never)
--   DINO_POKES   timed RAM writes from power on: frame hexaddr size value (PR_POKES format).
--   DINO_SYNC=power uses video frames and input frames from power on.
--   DINO_POKE    optional Lua file run once at the sync point, before the snapshot is taken: it edits
--                RAM (the snapshot and the whole recording start from the edited state, the port loads
--                it from the .state). It receives one argument, the table DINO (`local DINO = ...`)
--                with DINO.space, DINO.spawn_item(...) and DINO.spawn_enemy(...).
--
-- Timing. The vblank IRQ (0x542) sets the frame flag -$7EAE(a5) = 0xFF0152 at 0x5C8, before it
-- flushes the sound queue, writes the scroll registers, reads the inputs (0x738) and ticks the
-- task timers. A write tap on that flag therefore sees the RAM exactly as the game tasks left it at
-- the end of the previous game frame. That is where every record is taken. Inputs are injected
-- with read taps on the input ports, so the IRQ that follows reads the script value of the next
-- game frame: record k is the state after game frame k, which used script line k.
--
-- Lag: if a game task (slots 0..14 of the task table at 0xFF0052) is still running at the vblank,
-- the game frame did not finish (slowdown). That vblank is not recorded and the frame index does
-- not advance; the next record carries flag bit 0 and the number of lag vblanks.
-- DINO_CLOCK=video instead samples at frame_done, after the IRQ and frame's CPU work. It keeps
-- every video frame, even when a task is busy. The first completed video frame is the sync snapshot;
-- input line k then runs video frame k + 2 when syncing at vblank:1. This matches the probe clock.

local cpu = manager.machine.devices[":maincpu"]
local space = cpu.spaces["program"]
local shares = manager.machine.memory.shares

local function env(name, default)
  local v = os.getenv(name)
  if v == nil or v == "" then return default end
  return v
end

local cfg = {
  boot = env("DINO_BOOT", nil),
  input = env("DINO_INPUT", nil),
  frames = tonumber(env("DINO_FRAMES", "600")),
  trace = env("DINO_TRACE", nil),
  state = env("DINO_STATE", nil),
  sync = env("DINO_SYNC", "control"),
  clock = env("DINO_CLOCK", "game"),
  max_vblanks = tonumber(env("DINO_MAX_VBLANKS", "30000")),
  snap_every = tonumber(env("DINO_SNAP_EVERY", "0")),
  poke = env("DINO_POKE", nil),
  pokes = env("DINO_POKES", nil),
  batch_probe = env("DINO_BATCH_PROBE", nil),
}
if cfg.sync == "power" then cfg.clock = "video" end
assert(cfg.clock == "game" or cfg.clock == "video", "bad DINO_CLOCK " .. cfg.clock)
assert(cfg.clock ~= "video" or (cfg.sync == "power" or cfg.sync:match("^vblank:%d+$")),
  "DINO_CLOCK=video requires DINO_SYNC=vblank:N")

local function log(fmt, ...)
  io.stderr:write("record.lua: " .. string.format(fmt, ...) .. "\n")
end

-- ---------------------------------------------------------------------------------------------
-- Trace writer
-- ---------------------------------------------------------------------------------------------

local BLOCK = 64
local REGIONS = {
  { name = "ram", base = 0xFF0000, size = 0x10000 },  -- work RAM: globals, tasks, all objects
  { name = "gfx", base = 0x900000, size = 0x30000 },  -- sprite lists, tilemaps, palettes
  { name = "cpsa", base = 0x800100, size = 0x40 },    -- CPS-A registers (scroll, bases)
  { name = "cpsb", base = 0x800140, size = 0x40 },    -- CPS-B registers (layer control)
}

local function share_bytes(tag, size)
  local sh = shares[tag]
  if sh == nil then return string.rep("\0", size) end
  local parts = {}
  for o = 0, size - 2, 2 do parts[#parts + 1] = string.pack(">I2", sh:read_u16(o)) end
  return table.concat(parts)
end

local function snapshot()
  return {
    space:read_range(0xFF0000, 0xFFFFFF, 8),
    space:read_range(0x900000, 0x92FFFF, 8),
    share_bytes(":cps_a_regs", 0x40),
    share_bytes(":cps_b_regs", 0x40),
  }
end

local function json_regions()
  local items = {}
  for _, r in ipairs(REGIONS) do
    items[#items + 1] = string.format('{"name":"%s","base":%d,"size":%d}', r.name, r.base, r.size)
  end
  return "[" .. table.concat(items, ",") .. "]"
end

local function json_str(s)
  return '"' .. tostring(s):gsub('\\', '\\\\'):gsub('"', '\\"') .. '"'
end

local function write_header(fh, extra)
  local json = string.format('{"source":"mame","format":1,"block":%d,"regions":%s%s}', BLOCK, json_regions(), extra or "")
  fh:write("DINOTRC1", string.pack("<I4", #json), json)
end

-- Blocks of `cur_s` that differ from `prev_s` (all blocks when prev_s is nil).
local function changed_blocks(prev_s, cur_s)
  local out = {}
  local n = #cur_s // BLOCK
  local CHUNK = 4096
  for c = 0, #cur_s - 1, CHUNK do
    local a = cur_s:sub(c + 1, c + CHUNK)
    if prev_s == nil or a ~= prev_s:sub(c + 1, c + CHUNK) then
      for o = 0, #a - 1, BLOCK do
        local blk = a:sub(o + 1, o + BLOCK)
        if prev_s == nil or blk ~= prev_s:sub(c + o + 1, c + o + BLOCK) then
          out[#out + 1] = string.pack("<I4", (c + o) // BLOCK) .. blk
        end
      end
    end
  end
  return out
end

local function write_record(fh, frame, flags, vblank, lagv, prev, snap)
  fh:write(string.pack("<i4I4I4I4", frame, flags, vblank, lagv))
  for i = 1, #REGIONS do
    local blocks = changed_blocks(prev and prev[i] or nil, snap[i])
    fh:write(string.pack("<I4", #blocks))
    if #blocks > 0 then fh:write(table.concat(blocks)) end
  end
  fh:write(string.pack("<I4", 0)) -- no extra JSON on the MAME side
end

-- Library mode: probe.lua loads this file with DINO_LIB_ONLY set to share the trace writer above
-- (re/harness/probe/probe.lua, commands `snapshot` and `trace N`). Nothing else runs then.
if DINO_LIB_ONLY then
  return { snapshot = snapshot, write_header = write_header, write_record = write_record, json_str = json_str,
    regions = REGIONS, block = BLOCK }
end

-- ---------------------------------------------------------------------------------------------
-- Input scripts. Same format as the port's --input (dino-game/src/input.rs):
--   "start-end:KEYS" segments separated by commas or new lines; '#' starts a comment.
--   KEYS: U D L R (stick), A (attack), J (jump), S (start), C (coin). The digits 1, 2, 3 select
--   the player for the letters that follow (default player 1), e.g. "0-9:R2L3AS".
-- ---------------------------------------------------------------------------------------------

local BITS = { R = 1, L = 2, D = 4, U = 8, A = 0x10, J = 0x20 }

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
          if c == "1" or c == "2" or c == "3" then
            pl = tonumber(c)
          elseif BITS[c] then
            f.p[pl] = f.p[pl] | BITS[c]
          elseif c == "S" then
            f.start = f.start | (1 << (pl - 1))
          elseif c == "C" then
            f.coin = f.coin | (1 << (pl - 1))
          end
        end
        segs[#segs + 1] = { tonumber(a), tonumber(b), f }
      end
    end
  end
  return segs
end

local function read_file(path)
  local fh = assert(io.open(path, "rb"), "cannot open " .. tostring(path))
  local s = fh:read("a")
  fh:close()
  return s
end

-- PR_POKES format: decimal frame, hexadecimal address, size 1/2/4,
-- decimal or 0x-prefixed value. Validate the entire file before starting.
local function parse_pokes(path)
  local out = {}
  if not path then return out end
  local n = 0
  for line in (read_file(path) .. "\n"):gmatch("([^\n]*)\n") do
    n = n + 1
    line = line:gsub("#.*", ""):match("^%s*(.-)%s*$")
    if line ~= "" then
      local f, a, z, v = line:match("^(%d+)%s+([%x]+)%s+([124])%s+(%S+)$")
      local frame, addr, size, value = tonumber(f), a and tonumber(a, 16), tonumber(z), tonumber(v)
      assert(frame and math.type(frame) == "integer" and frame <= 0x7FFFFFFF
        and addr and size and value and math.type(value) == "integer"
        and addr >= 0xFF0000 and addr + size <= 0x1000000
        and (size == 1 or addr % 2 == 0)
        and value >= 0 and value < 2 ^ (8 * size),
        string.format("bad DINO_POKES line %s:%d: %s", path, n, line))
      out[#out + 1] = { frame, addr, size, value, n }
    end
  end
  table.sort(out, function(a, b) return a[1] < b[1] or (a[1] == b[1] and a[5] < b[5]) end)
  return out
end
local valid_pokes, timed_pokes = pcall(parse_pokes, cfg.pokes)
if not valid_pokes then
  log("%s", timed_pokes)
  manager.machine:exit()
  return
end
local next_poke = 1
local function apply_pokes(frame)
  while timed_pokes[next_poke] and timed_pokes[next_poke][1] <= frame do
    local p = timed_pokes[next_poke]
    if p[3] == 1 then space:write_u8(p[2], p[4])
    elseif p[3] == 2 then space:write_u16(p[2], p[4])
    else space:write_u32(p[2], p[4]) end
    log("poke at power frame %d: %X size %d = %X", frame, p[2], p[3], p[4])
    next_poke = next_poke + 1
  end
end

local function script_at(segs, frame)
  local out = { p = { 0, 0, 0 }, start = 0, coin = 0 }
  if segs == nil then return out end
  for _, s in ipairs(segs) do
    if frame >= s[1] and frame <= s[2] then
      local f = s[3]
      for i = 1, 3 do out.p[i] = out.p[i] | f.p[i] end
      out.start = out.start | f.start
      out.coin = out.coin | f.coin
    end
  end
  return out
end

-- Pokes (DINO_POKE): helpers for scenarios that need objects the first waves do not give.
DINO = { space = space }

-- Spawn a pool E object like the stage script does (0xFDAE with pool selector 3): pops the free
-- list ($65F4 count, $65FE pointer, 0x4818), then writes +$00 = 1, +$20 class word (class * 4),
-- +$08 X, +$10 Z, +$26 variant, +$5C and +$96. The object starts at mode 0 and initialises itself.
function DINO.spawn_item(class_word, var, x, z, w5c, drop_row)
  local cnt = space:read_u16(0xFFE5F4)
  assert(cnt > 0, "pool E is full")
  local ptr = space:read_u16(0xFFE5FE)
  local slot = space:read_u16(0xFF0000 + ptr)
  space:write_u16(0xFFE5FE, (ptr + 2) & 0xFFFF)
  space:write_u16(0xFFE5F4, cnt - 1)
  local a = 0xFF0000 + slot
  space:write_u8(a, 1)
  space:write_u16(a + 0x20, class_word)
  space:write_u16(a + 0x08, x)
  space:write_u16(a + 0x10, z)
  space:write_u16(a + 0x26, var)
  space:write_u16(a + 0x5C, w5c or 0)
  space:write_u8(a + 0x96, drop_row or 0)
  return a
end

-- Spawn a pool A object (an enemy) the same way (0xFDAE with pool selector 0: free list $65F8 count,
-- $6602 pointer, 0x483C): +$00 = 1, +$20 class word (class * 4), +$08 X, +$10 Z, +$26 variant, +$5C,
-- +$5B (entrance facing) and +$96 (drop row). The enemy starts at mode 0 and runs its entrance.
function DINO.spawn_enemy(class_word, var, x, z, w5c, b5b, drop_row)
  local cnt = space:read_u16(0xFFE5F8)
  assert(cnt > 0, "pool A is full")
  local ptr = space:read_u16(0xFFE602)
  local slot = space:read_u16(0xFF0000 + ptr)
  space:write_u16(0xFFE602, (ptr + 2) & 0xFFFF)
  space:write_u16(0xFFE5F8, cnt - 1)
  local a = 0xFF0000 + slot
  space:write_u8(a, 1)
  space:write_u16(a + 0x20, class_word)
  space:write_u16(a + 0x08, x)
  space:write_u16(a + 0x10, z)
  space:write_u16(a + 0x26, var)
  space:write_u16(a + 0x5C, w5c or 0)
  space:write_u8(a + 0x5B, b5b or 0)
  space:write_u8(a + 0x96, drop_row or 0)
  return a
end

local boot_segs = cfg.boot and parse_script(read_file(cfg.boot)) or nil
local test_segs = cfg.input and parse_script(read_file(cfg.input)) or nil

-- Current input (cooked, active high), applied by the read taps below.
local cur = script_at(nil, 0)

-- Raw port values (active low), as MAME's cps1 driver maps them:
--   0x800000 word: high byte = IN1 player 2, low byte = IN1 player 1
--   0x800018 word: high byte = IN0 (coin1 0x01, coin2 0x02, start1 0x10, start2 0x20)
--   0xF1C000 word: low byte = IN2 player 3 (+ coin3 0x40, start3 0x80)
local function raw_in1()
  return (~((cur.p[2] << 8) | cur.p[1])) & 0xFFFF
end
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

-- Flow batches keep the probe's runtime jump/hold/object helpers. The trace
-- includes the entire boot and jump, not only the post-jump segment. Only these
-- explicit extension points are added to the loaded copy of probe.lua.
if cfg.batch_probe then
  local fh, old, count, last_frame, batch_count, power_snapshot
  batch_count = 0
  function DINO_BATCH_BEGIN(name)
    assert(name:match("^[%w_-]+$"), "bad batch segment name")
    assert(not fh, "unfinished batch trace")
    fh = assert(io.open(env("DINO_TRACE_DIR", ".") .. "/flow_" .. name .. ".mame.trc", "wb"))
    write_header(fh, ',"kind":"trace","clock":"video","sync":"power","probe_segment":' .. json_str(name))
    old, count, last_frame = nil, 0, 0
    batch_count = batch_count + 1
    if power_snapshot then
      write_record(fh, -1, 0, 1, 0, nil, power_snapshot)
      old, count, last_frame = power_snapshot, 1, 1
    end
    next_poke = 1
    apply_pokes(0)
  end
  function DINO_BATCH_FRAME(frame)
    apply_pokes(frame)
    local snap = snapshot()
    if frame == 1 then power_snapshot = snap end
    write_record(fh, count - 1, 0, frame, 0, old, snap)
    old, count, last_frame = snap, count + 1, frame
  end
  function DINO_BATCH_END(msg)
    if fh then fh:close(); fh = nil end
    assert(not msg:match("FAILED") and not msg:match("LUA ERROR"), msg)
    log("batch segment recorded %d video records through power frame %d: %s", count, last_frame, msg)
    log("batch completed segments: %d", batch_count)
  end
  local source = read_file(cfg.batch_probe)
  local function hook(anchor, replacement)
    local a, b = source:find(anchor, 1, true)
    assert(a, "probe extension point missing: " .. anchor)
    assert(not source:find(anchor, b + 1, true), "ambiguous probe extension point")
    source = source:sub(1, a - 1) .. replacement .. source:sub(b + 1)
  end
  hook('  log("SEGMENT", "%s frames=%d", seg.name, maxT)',
    '  DINO_BATCH_BEGIN(seg.name)\n  log("SEGMENT", "%s frames=%d", seg.name, maxT)')
  hook('  F = F + 1', '  F = F + 1\n  DINO_BATCH_FRAME(F)')
  hook('local function end_segment(msg)',
    'local function end_segment(msg)\n  DINO_BATCH_END(msg)')
  hook('  if not ok then\n    io.stderr:write("probe.lua error: "',
    '  if not ok then\n    manager.machine:exit()\n    io.stderr:write("probe.lua error: "')
  assert(load(source, "@" .. cfg.batch_probe, "t", _ENV))()
  return
end

-- ---------------------------------------------------------------------------------------------
-- Game state probes
-- ---------------------------------------------------------------------------------------------

local P1 = 0xFFB274

-- A game task (slots 0..14) is still running or waiting to run: the game frame is not finished.
-- Slot 15 (0xF0, task 0x9E38C) is a background task that always yields; it is ignored.
local function lagging()
  for t = 0, 14 do
    local s = space:read_u8(0xFF0052 + t * 16)
    if s == 4 or s == 8 or s == 0xC then return true end
  end
  return false
end

-- Player 1 can be controlled: in game (G.$4CC), game mode 0x0C (stage), no scripted control
-- (G.$4DD), player 1 present, state 0x02000000 (idle, alive), no input lock (+$11A).
local function p1_controllable()
  return space:read_u8(0xFF84CC) ~= 0
    and space:read_u8(0xFF84D0) == 0x0C
    and space:read_u8(0xFF84DD) == 0
    and space:read_u8(P1) ~= 0
    and space:read_u32(P1 + 4) == 0x02000000
    and space:read_u8(P1 + 0x11A) == 0
end

local control_frame = nil  -- G.$0000 when player 1 first could be controlled ("control+N")

local function sync_now(vblank)
  if cfg.sync == "power" then return true end
  if cfg.sync == "control" then return p1_controllable() end
  local n = cfg.sync:match("^control%+(%d+)$")
  if n then
    if control_frame == nil and p1_controllable() then control_frame = space:read_u16(0xFF8000) end
    return control_frame ~= nil and ((space:read_u16(0xFF8000) - control_frame) & 0xFFFF) >= tonumber(n)
  end
  n = cfg.sync:match("^vblank:(%d+)$")
  if n then return vblank >= tonumber(n) end
  error("bad DINO_SYNC " .. cfg.sync)
end

-- ---------------------------------------------------------------------------------------------
-- Main state machine (runs in the frame flag write tap)
-- ---------------------------------------------------------------------------------------------

local phase = "boot"
local vblank = 0
local power_frame = 0 -- video clock, independent of game-frame IRQ taps
local k = 0          -- test frame index
local lag = 0        -- lag vblanks since the last record
local trace_fh = nil
local prev = nil

local function finish(msg)
  if trace_fh then trace_fh:close(); trace_fh = nil end
  log("%s", msg)
  phase = "done"
  manager.machine:exit()
end

local function write_state(snap)
  local fh = assert(io.open(cfg.state, "wb"))
  write_header(fh, string.format(',"kind":"state","vblank":%d', vblank))
  write_record(fh, -1, 0, vblank, 0, nil, snap)
  fh:close()
end

local function on_vblank()
  vblank = vblank + 1
  if phase == "boot" then
    if vblank > cfg.max_vblanks then
      finish(string.format("no sync point after %d vblanks", vblank))
      return
    end
    if (cfg.clock == "video" or not lagging()) and sync_now(vblank) then
      if cfg.poke then assert(loadfile(cfg.poke))(DINO) end
      local snap = snapshot()
      if cfg.state then write_state(snap) end
      if cfg.trace then
        trace_fh = assert(io.open(cfg.trace, "wb"))
        write_header(trace_fh, string.format(',"kind":"trace","clock":%s,"sync_vblank":%d,"frames":%d,"input":%s', json_str(cfg.clock), vblank, cfg.frames, json_str(cfg.input or "")))
        write_record(trace_fh, -1, 0, vblank, 0, nil, snap)
      end
      prev = snap
      log("sync at vblank %d (frame counter G.$0000 = %d)", vblank, space:read_u16(0xFF8000))
      if not cfg.trace or cfg.frames <= 0 then
        finish("state written")
        return
      end
      phase = "test"
      k = 0
      cur = script_at(test_segs, cfg.sync == "power" and vblank or k)
      return
    end
    cur = script_at(boot_segs, vblank)
  elseif phase == "test" then
    if cfg.clock == "game" and lagging() then
      lag = lag + 1
      return
    end
    local snap = snapshot()
    write_record(trace_fh, k, (lag > 0) and 1 or 0, vblank, lag, prev, snap)
    if cfg.snap_every > 0 and k % cfg.snap_every == 0 then manager.machine.video:snapshot() end
    prev = snap
    lag = 0
    k = k + 1
    if k >= cfg.frames then
      finish(string.format("recorded %d frames (sync + %d vblanks)", k, vblank))
      return
    end
    cur = script_at(test_segs, cfg.sync == "power" and vblank or k)
  end
end


-- DINO_WATCH_WRITE=ADDR (hex, RAM address, e.g. FF84FC): log every CPU write to that byte or word (frame, PC, A6, value).
-- The tap covers the even word of ADDR; each line still shows the real write address.
-- The lines go to the MAME log (record.log / mame.log) as "WATCH ...". Used to find the routine that writes a variable.
-- "frame" is MAME's screen frame (counted from power on); "tf" is the trace record that shows the write (-1 before the
-- sync point). Compare trace frames with tf, never with frame.
local function install_watch(space, cpu)
  local w = os.getenv("DINO_WATCH_WRITE")
  if w == nil or w == "" then return nil end
  local a = tonumber(w, 16) & ~1   -- taps need an even start (the word); the log shows the write's address
  return space:install_write_tap(a, a + 1, "dino_watch", function(offset, data, mask)
    local scr = manager.machine.screens[":screen"]
    local tf = DINO_TF or ((phase == "test") and k or -1)
    print(string.format("WATCH frame %d tf %d addr %06X data %04X mask %04X pc %06X a6 %08X",
      scr and scr:frame_number() or -1, tf, offset, data, mask, cpu.state["PC"].value, cpu.state["A6"].value))
    return data
  end)
end

local installed = false
local function install()
  -- Keep the tap objects in globals: a collected tap is removed.
  DINO_WATCH_TAP = install_watch(space, cpu)
  DINO_TAPS = {
    space:install_write_tap(0xFF0152, 0xFF0153, "dino_vblank", function(offset, data, mask)
      -- st.b -$7EAE(a5) at 0x5C8 writes 0xFF to the high byte; the main loop clears it with 0.
      if cfg.clock == "game" and phase ~= "done" and offset == 0xFF0152 and (mask & 0xFF00) ~= 0 and (data & 0xFF00) ~= 0 then
        on_vblank()
      end
    end),
    space:install_read_tap(0x800000, 0x800001, "dino_in1", function(offset, data, mask)
      return (data & ~mask) | (raw_in1() & mask)
    end),
    space:install_read_tap(0x800018, 0x800019, "dino_in0", function(offset, data, mask)
      return (data & ~mask) | ((raw_in0() | 0xFF) & mask)
    end),
    space:install_read_tap(0xF1C000, 0xF1C001, "dino_in2", function(offset, data, mask)
      return (data & ~mask) | ((raw_in2() | 0xFF00) & mask)
    end),
  }
  installed = true
end

-- Taps installed before the machine starts are lost at reset, so install them on the first frame.
-- The boot script must not need input in the first frame.
emu.register_frame_done(function()
  power_frame = power_frame + 1
  if not installed then
    install()
    apply_pokes(0)
  end
  if phase ~= "done" then apply_pokes(power_frame) end
  if cfg.clock == "video" and phase ~= "done" then on_vblank() end
end)
