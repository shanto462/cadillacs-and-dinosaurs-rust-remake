-- Code coverage of the original 68000 program under MAME (re/harness/coverage/REPORT.md, "Method").
--
-- An autoboot script that wraps another one: bin/mame (the shim) starts MAME with `-debug -debugger none` and this
-- file as the autoboot script, and passes the recorder the harness asked for (record.lua, probe.lua, ...) in COV_INNER.
-- This file sets one debugger breakpoint on every address of COV_ADDRS (one hex address at the start of each line:
-- the routine entries and block leaders of blocks.py). A breakpoint that is hit runs its action once,
-- `bpdisable N; g`: it turns itself off and the CPU goes on, so every address costs one stop, and the game runs as it
-- does without the debugger (the debugger only watches; the recordings stay bit for bit the same). At the end, and
-- every COV_EVERY frames (default 3000, so a killed run keeps most of its result), the addresses whose breakpoint is
-- off are written to COV_OUT, one hex address per line.
--
-- Environment: COV_INNER (the wrapped autoboot script), COV_ADDRS, COV_OUT, COV_EVERY. COV_HISTORY=N (diagnostic):
-- the action also prints the last N program counters before the hit (debugger `history`); the console goes to
-- COV_OUT.console at the end.

local function env(name, default)
  local v = os.getenv(name)
  if v == nil or v == "" then return default end
  return v
end

local inner = env("COV_INNER", nil)
local list = assert(env("COV_ADDRS", nil), "cov.lua: COV_ADDRS is not set")
local out = assert(env("COV_OUT", nil), "cov.lua: COV_OUT is not set")
local every = tonumber(env("COV_EVERY", "3000"))
local history = tonumber(env("COV_HISTORY", "0"))

local dbg = manager.machine.debugger
assert(dbg ~= nil, "cov.lua: MAME runs without the debugger (the shim adds -debug -debugger none)")
local cpu = manager.machine.devices[":maincpu"]
local d = cpu.debug

local addrs = {}
for line in io.lines(list) do
  local a = line:match("^%s*(%x+)")
  if a then addrs[#addrs + 1] = tonumber(a, 16) end
end

-- Breakpoint numbers are given in order; the action of each names its own number.
local probe = d:bpset(addrs[1], "0", "")
d:bpclear(probe)
local addr_of = {}
local want = probe + 1
for _, a in ipairs(addrs) do
  local act = string.format("bpdisable #%d; g", want)   -- decimal with #: a bare number is hex, and "a0" or "d0" is a register
  if history > 0 then act = string.format('printf "COVHIT %%x", pc; history 0,%x; ', history) .. act end
  local got = d:bpset(a, "1", act)
  assert(got == want, string.format("cov.lua: breakpoint number %d, expected %d", got, want))
  addr_of[got] = a
  want = want + 1
end

-- Per recording (multi.lua): COV_SEGMENT(NAME) writes the hits so far to the file of the recording before (COV_OUT with
-- .NAME before .hits) and turns every breakpoint on again, so each recording of a multi.lua process gets its own hits.
local segment = nil
local function seg_path(name) return (out:gsub("%.hits$", "")) .. "." .. name .. ".hits" end

local function write_hits(path)
  local hit = {}
  for index, bp in pairs(d:bplist()) do
    if addr_of[index] ~= nil and not bp.enabled then hit[#hit + 1] = addr_of[index] end
  end
  table.sort(hit)
  local fh = io.open(path .. ".tmp", "w")
  if fh == nil then return 0 end
  for _, a in ipairs(hit) do fh:write(string.format("%x\n", a)) end
  fh:close()
  os.rename(path .. ".tmp", path)
  return #hit
end

-- MAME runs an autoboot script in an environment of its own: put the function in the global table the inner script sees.
_G.COV_SEGMENT = function(name)
  if segment then write_hits(seg_path(segment)) end
  segment = name
  dbg:command("bpenable")
end

local function dump(final)
  local n = write_hits(segment and seg_path(segment) or out)
  local hit = { n = n }
  if final and history > 0 then
    local ch = io.open(out .. ".console", "w")
    for _, line in ipairs(dbg.consolelog) do ch:write(line, "\n") end
    ch:close()
  end
  if final then
    io.stderr:write(string.format("cov.lua: %d of %d addresses hit -> %s\n", hit.n, #addrs, segment and seg_path(segment) or out))
  end
end

local frames = 0
COV_SUBS = {
  emu.add_machine_frame_notifier(function()
    frames = frames + 1
    if frames % every == 0 then dump(false) end
  end),
  emu.add_machine_stop_notifier(function() dump(true) end),
}

io.stderr:write(string.format("cov.lua: %d breakpoints, inner %s\n", #addrs, tostring(inner)))
dbg.execution_state = "run"
if inner ~= nil then dofile(inner) end
