-- Many record.lua recordings in ONE MAME process (re/harness/coverage/REPORT.md, "Method").
--
-- record.sh runs one MAME process per recording. This autoboot script runs a list of them one after the other: the
-- first from power on, every later one from the state saved at the end of the first video frame (as probe.lua does
-- for its segments), each with its own copy of record.lua and its own settings. Each recording is the same file
-- record.sh writes (run.sh compares it with the original by same.py).
--
-- Environment: MR_LIST (the list file), MR_RECORD_LUA (record.lua), MR_OUT (folder for NAME.mame.trc / NAME.state).
-- MR_LIST lines: `NAME FRAMES KEY=VALUE ...` with the record.sh settings of that recording (DINO_BOOT, DINO_INPUT,
-- DINO_SYNC, DINO_POKE, DINO_CLOCK, DINO_POKES (writes from power frame 3 on); paths absolute or relative to the current
-- folder). # starts a comment.
--
-- How a later recording starts: the state of the end of video frame 1 is loaded; the next frame_done is the end of
-- video frame 2, so the new record.lua gets two frame_done calls at once (power frames 1 and 2). That is the same as a
-- fresh process as long as the boot needs no input and has no vblank IRQ in its first two frames (true for dino: the
-- IRQ starts after the RAM clear, about frame 70).

local real_getenv = os.getenv
local overrides = {}
os.getenv = function(k)
  local v = overrides[k]
  if v ~= nil then return v end
  return real_getenv(k)
end

local list_path = assert(real_getenv("MR_LIST"), "multi.lua: MR_LIST is not set")
local record_lua = assert(real_getenv("MR_RECORD_LUA"), "multi.lua: MR_RECORD_LUA is not set")
local out_dir = assert(real_getenv("MR_OUT"), "multi.lua: MR_OUT is not set")

local runs = {}
for line in io.lines(list_path) do
  line = line:gsub("#.*", "")
  local parts = {}
  for p in line:gmatch("%S+") do parts[#parts + 1] = p end
  if #parts >= 2 then
    local r = { name = parts[1], frames = parts[2], env = {} }
    for i = 3, #parts do
      local k, v = parts[i]:match("^([%w_]+)=(.*)$")
      assert(k, "multi.lua: bad setting " .. parts[i])
      r.env[k] = v
    end
    runs[#runs + 1] = r
  end
end

local real_register = emu.register_frame_done
local real_manager = manager
local real_machine = manager.machine
local finished = false        -- the running record.lua called exit()
local machine_proxy = setmetatable({}, { __index = function(_, k)
  if k == "exit" then return function() finished = true end end
  local v = real_machine[k]
  if type(v) == "function" then return function(_, ...) return v(real_machine, ...) end end
  return v
end })
manager = setmetatable({}, { __index = function(_, k)
  if k == "machine" then return machine_proxy end
  return real_manager[k]
end })

-- The frame callbacks of a record.lua copy are kept here and called by the one real callback below, after it, in order.
local current_cbs = {}
emu.register_frame_done = function(cb) current_cbs[#current_cbs + 1] = cb end

local function log(fmt, ...) io.stderr:write("multi.lua: " .. string.format(fmt, ...) .. "\n") end

local index = 0
local function start(i)
  index = i
  local r = runs[i]
  current_cbs = {}
  finished = false
  if DINO_TAPS then for _, t in ipairs(DINO_TAPS) do t:remove() end end
  if DINO_WATCH_TAP then DINO_WATCH_TAP:remove() end
  DINO_TAPS, DINO_WATCH_TAP = nil, nil
  overrides = {
    DINO_FRAMES = r.frames, DINO_TRACE = out_dir .. "/" .. r.name .. ".mame.trc", DINO_STATE = out_dir .. "/" .. r.name .. ".state",
    DINO_BOOT = "", DINO_INPUT = "", DINO_SYNC = "", DINO_POKE = "", DINO_CLOCK = "", DINO_POKES = "", DINO_BATCH_PROBE = "",
  }
  for k, v in pairs(r.env) do overrides[k] = v end
  -- A timed write at power frame 0 .. 2 would land in the saved power-on state (run 1) or be missed (later runs).
  if overrides.DINO_POKES ~= "" then
    for line in io.lines(overrides.DINO_POKES) do
      local f = tonumber((line:gsub("#.*", "")):match("^%s*(%d+)%s"))
      assert(f == nil or f > 2, "multi.lua: a DINO_POKES write before power frame 3 in " .. r.name)
    end
  end
  log("run %d of %d: %s", i, #runs, r.name)
  if COV_SEGMENT then COV_SEGMENT(r.name) end   -- cov.lua: the hits of this recording go to their own file
  dofile(record_lua)
end

local saved, loading, prev_time = false, false, nil
local function machine_time() return real_machine.time:as_double() end
local function call_copy() for _, cb in ipairs(current_cbs) do cb() end end

local function frame()
  if not saved then                 -- end of video frame 1: keep it for the later runs
    real_machine:save("power")
    saved = true
    prev_time = machine_time()
    call_copy()
    return
  end
  local tm = machine_time()
  if loading then
    if tm < prev_time then          -- the power-on state is back: this frame_done is the end of video frame 2
      loading = false
      prev_time = tm
      start(index + 1)
      call_copy()                   -- power frame 1 of the new copy
      call_copy()                   -- power frame 2
    else
      prev_time = tm
    end
    return
  end
  prev_time = tm
  call_copy()
  if finished then
    finished = false
    current_cbs = {}
    if COV_SEGMENT then COV_SEGMENT("_between") end   -- the frames until the next recording starts belong to none
    if index < #runs then
      real_machine:load("power")
      loading = true
    else
      log("all %d runs done", #runs)
      real_machine:exit()
    end
  end
end
MR_SUB = real_register(frame)

if #runs == 0 then log("no runs in %s", list_path); real_machine:exit() else start(1) end
