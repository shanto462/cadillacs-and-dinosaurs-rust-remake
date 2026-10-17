-- QSound register trace (M4-01). Run with sound.sh. Does not change probe.lua.
--
-- One MAME process boots until the 68000 sound mailbox is live, then records every code from
-- that idle driver. The next code is not loaded from a saved state: with -sound none the state
-- does not restore the Z80, so the previous sound would keep playing. Instead the script queues
-- 0xFF00 (stop everything, helper 0xA5D2) and waits until the QSound writes go quiet, then
-- queues the next request. Each request is one ring entry, the way enqueue 0xA63C does
-- (16 bytes at 0xFF803A + write index 0xFF8034: code word, d1 long, d2 long).
-- The game's own vblank flush (0x610) delivers the entry. While a code is recorded the script
-- holds G.$777C bit 5 (demo sound) so the attract filter does not drop it, and it drops any
-- later ring entry so the trace is that one request. Between codes the bit is cleared, so
-- ordinary attract codes are dropped while 0xFF00 still passes.
--
-- A write tap on the Z80 QSound ports logs commits and bank writes:
--   0xD000 data high, 0xD001 data low, 0xD002 register (the write that commits), 0xD003 bank.
-- The driver helper 0x0A13 writes low, high, then the register. Value is (high << 8) | low.
--
-- Environment (sound.sh sets these):
--   SQ_REQ     file of `sound CODE [d1] [d2]` lines
--   SQ_OUT     output directory (traces CODE.qs.trc)
--   SQ_FRAMES  video frames to record after the ring write (default 240)
--   SQ_STATUS  status file path
--   SQ_SOUND   the -sound value this process was started with (written into the status)

local audio_dev = manager.machine.devices[":audiocpu"]
local main_dev = manager.machine.devices[":maincpu"]
if audio_dev == nil or main_dev == nil then
  io.stderr:write("sound_tap.lua: need :audiocpu and :maincpu\n")
  manager.machine:exit()
  return
end
local audio = audio_dev.spaces["program"]
local main = main_dev.spaces["program"]

local function env(n, d)
  local v = os.getenv(n)
  if v == nil or v == "" then return d end
  return v
end

local function read_file(path)
  local fh = io.open(path, "rb")
  if not fh then
    io.stderr:write("sound_tap.lua: cannot open " .. tostring(path) .. "\n")
    return ""
  end
  local s = fh:read("a")
  fh:close()
  return s or ""
end

local function num(s)
  if s == nil then return nil end
  s = s:gsub("^%$", "0x")
  return tonumber(s)
end

-- Ring and mailbox (SPEC section 3). Shared RAM: 68000 0xF18000+2*n+1 = Z80 0xC000+n.
local RING, WI, RI = 0xFF803A, 0xFF8034, 0xFF8036
local MAIL = 0xF1801F       -- Z80 C00F, 0xFF means the mailbox is free
local CFFD = 0xF19FFB       -- Z80 CFFD, 0x88 means the 68000 has enabled commands
local DEMO = 0xFFF77C       -- G.$777C bit 5: demo sound (attract filter at 0x610)

local OUT = env("SQ_OUT", ".")
local FRAMES = num(env("SQ_FRAMES", "240")) or 240
if FRAMES < 1 then FRAMES = 1 end
local STATUS = env("SQ_STATUS", OUT .. "/qs-status.txt")
local SOUND_MODE = env("SQ_SOUND", "?")

local cmds = {}
for line in read_file(env("SQ_REQ", "")):gmatch("[^\n]+") do
  line = line:gsub("#.*", ""):gsub("^%s+", ""):gsub("%s+$", "")
  if line ~= "" then
    local tk = {}
    for w in line:gmatch("%S+") do tk[#tk + 1] = w end
    if tk[1] and tk[1]:lower() == "sound" then table.remove(tk, 1) end
    local code = num(tk[1] or "")
    -- Optional words: `name=X` names this trace; `at=N` (on a line after a request) queues
    -- this request N frames into the trace of the request before it (M4-06: a fade over music).
    local name, at = nil, nil
    for i = 4, #tk do
      name = name or tk[i]:match("^name=(.+)$")
      at = at or tonumber(tk[i]:match("^at=(%d+)$") or "")
    end
    if code ~= nil and at ~= nil and #cmds > 0 then
      cmds[#cmds].extra = {
        at = at,
        code = code & 0xFFFF,
        d1 = (num(tk[2] or "0") or 0) & 0xFFFFFFFF,
        d2 = (num(tk[3] or "0") or 0) & 0xFFFFFFFF,
      }
    elseif code ~= nil then
      cmds[#cmds + 1] = {
        code = code & 0xFFFF,
        d1 = (num(tk[2] or "0") or 0) & 0xFFFFFFFF,
        d2 = (num(tk[3] or "0") or 0) & 0xFFFFFFFF,
        name = name,
      }
    end
  end
end

local function code_file(c)
  -- Four hex digits. d1 and d2 are added only when the request is not 0, 0,
  -- so two pan variants of one code do not overwrite each other.
  if c.name then return OUT .. "/" .. c.name .. ".qs.trc", c.name end
  local name = string.format("%04x", c.code)
  if c.d1 ~= 0 or c.d2 ~= 0 then
    name = string.format("%s_%08x_%08x", name, c.d1, c.d2)
  end
  return OUT .. "/" .. name .. ".qs.trc", name
end

-- Tap state. The tap object must stay referenced or Lua collects it and MAME removes it.
-- A state save drops taps, so they are installed again before each code.
-- The write-index guard blocks later enqueues (0xA63C updates 0xFF8034). The attract
-- loop queues its own codes in the same vblank that flushes, which is too late for a
-- frame-end clamp to drop them before they are sent.
local tap = nil
local wi_tap = nil
local guard = false
local latch_h, latch_l = 0, 0
local logging = false
local trace = nil
local rec_f = 0
local data_lines = 0
local boot_commits = 0
local first_off = nil
local sample = {}
-- Declared up here so the port tap and the frame hook share them.
local phase = "boot" -- boot, record, silence, dead
local silence_writes = 0

local function port_of(offset)
  if offset >= 0xD000 and offset <= 0xD003 then return offset - 0xD000 end
  if offset >= 0 and offset <= 3 then return offset end
  return nil
end

local function on_write(offset, data)
  local p = port_of(offset)
  if p == nil then return end
  if first_off == nil then first_off = offset end
  data = data & 0xFF
  -- Any port write during the stop counts. The frame hook waits until these stop.
  if phase == "silence" then silence_writes = silence_writes + 1 end
  if p == 0 then
    latch_h = data
  elseif p == 1 then
    latch_l = data
  elseif p == 2 then
    local value = (latch_h << 8) | latch_l
    if logging and trace then
      trace:write(string.format("f %d r %02x v %04x\n", rec_f, data, value))
      data_lines = data_lines + 1
    elseif phase ~= "silence" then
      boot_commits = boot_commits + 1
      if #sample < 8 then
        sample[#sample + 1] = string.format("r %02x v %04x", data, value)
      end
    end
  elseif p == 3 then
    if logging and trace then
      trace:write(string.format("f %d b %02x\n", rec_f, data))
      data_lines = data_lines + 1
    end
  end
end

local function install_tap()
  if tap ~= nil then pcall(function() tap:remove() end) end
  tap = audio:install_write_tap(0xD000, 0xD003, "sq_qsound", function(offset, data, mask)
    on_write(offset, data)
    return data
  end)
  if wi_tap ~= nil then pcall(function() wi_tap:remove() end) end
  wi_tap = main:install_write_tap(WI, WI + 1, "sq_wi", function(offset, data, mask)
    if not guard then return data end
    local ri = main:read_u16(RI)
    mask = mask or 0xFFFF
    return (data & ~mask) | (ri & mask)
  end)
end
install_tap()

local frame = 0
local ready_frame = 0
local empty_run = 0
local irq_total = 0
local prev_irq = nil
local cmd_i = 0
local flushed = 0
local our_wi = 0
local sent = false
local results = {}
local failed = nil

local function now()
  return manager.machine.time:as_double()
end

local function update_irq()
  local irq = audio:read_u8(0xF004)
  if prev_irq ~= nil then irq_total = irq_total + ((irq - prev_irq) & 0xFF) end
  prev_irq = irq
end

local function write_status()
  local fh = io.open(STATUS, "w")
  if not fh then return end
  fh:write(string.format("sound_mode %s\n", SOUND_MODE))
  fh:write(string.format("ready_frame %d\n", ready_frame))
  fh:write(string.format("z80_irq %d\n", irq_total))
  fh:write(string.format("qsound_writes %d\n", boot_commits))
  fh:write(string.format("first_tap %s\n", first_off == nil and "-" or tostring(first_off)))
  fh:write(string.format("codes %d\n", #cmds))
  if #sample > 0 then fh:write("boot_sample " .. table.concat(sample, " | ") .. "\n") end
  for _, row in ipairs(results) do fh:write(row .. "\n") end
  if failed then fh:write("error " .. failed .. "\n") end
  fh:close()
end

local function finish(err)
  if phase == "dead" then return end
  phase = "dead"
  failed = err
  if trace then trace:close(); trace = nil end
  logging = false
  write_status()
  if err then io.stderr:write("sound_tap.lua: " .. err .. "\n") end
  manager.machine:exit()
end

local function hold_demo()
  local v = main:read_u8(DEMO)
  if (v & 0x20) == 0 then main:write_u8(DEMO, v | 0x20) end
end

-- Attract (0x610) drops a normal code when this bit is clear. 0xFF00 still passes, and so does
-- the coin code. Clearing the bit between recordings stops the title music reaching the chip.
local function release_demo()
  local v = main:read_u8(DEMO)
  if (v & 0x20) ~= 0 then main:write_u8(DEMO, v & 0xDF) end
end

-- After our entry is flushed, force the write index back to the read index so a later
-- attract request is not delivered. Until it is flushed, keep the write index on our entry
-- so a request the game queues behind it is dropped.
local function clamp_ring()
  local ri = main:read_u16(RI) % 0x400
  local wi = main:read_u16(WI) % 0x400
  if not sent then
    if ri == our_wi then
      sent = true
      flushed = 1
      guard = true
    else
      if wi ~= our_wi then main:write_u16(WI, our_wi) end
      return
    end
  end
  -- Drop anything queued behind our entry. The guard keeps it that way.
  if wi ~= ri then main:write_u16(WI, ri) end
end

-- One ring entry at the current read index, then advance the write index (0xA63C).
local function enqueue(code, d1, d2)
  local ri = main:read_u16(RI) % 0x400
  main:write_u16(WI, ri)
  local addr = RING + ri
  main:write_u16(addr, code)
  main:write_u32(addr + 2, d1)
  main:write_u32(addr + 6, d2)
  main:write_u16(addr + 10, 0)
  main:write_u16(addr + 12, 0)
  main:write_u16(addr + 14, 0)
  our_wi = (ri + 0x10) % 0x400
  main:write_u16(WI, our_wi)
  sent = false
  flushed = 0
  return addr
end

local function inject(c)
  guard = false
  install_tap()
  enqueue(c.code, c.d1, c.d2)
  hold_demo()
  latch_h, latch_l = 0, 0
  local path, name = code_file(c)
  trace = io.open(path, "w")
  if not trace then finish("cannot write " .. path); return end
  trace:write(string.format("# sound code=%04x d1=%08x d2=%08x frames=%d\n", c.code, c.d1, c.d2, FRAMES))
  logging = true
  rec_f = 0
  data_lines = 0
  cmd_i = cmd_i + 1
  io.stderr:write(string.format("sound_tap: record %s\n", name))
end

local function close_code()
  if trace then
    trace:flush()
    trace:close()
    trace = nil
  end
  logging = false
  local _, name = code_file(cmds[cmd_i])
  results[#results + 1] = string.format(
    "code %s lines %d flushed %d", name, data_lines, flushed)
end

local phase_frame = 0
local silence_wait = 0
local quiet = 0

local function boot_ready()
  if phase_frame < 120 then return false end
  if main:read_u8(CFFD) ~= 0x88 then return false end
  if main:read_u8(MAIL) ~= 0xFF then return false end
  if main:read_u16(WI) ~= main:read_u16(RI) then return false end
  return true
end

local function begin_silence()
  phase = "silence"
  silence_wait = 0
  silence_writes = 0
  quiet = 0
  guard = false
  release_demo()
  enqueue(0xFF00, 0, 0)
end

local function frame_body()
  if phase == "dead" then return end
  frame = frame + 1
  update_irq()

  if phase == "boot" then
    phase_frame = phase_frame + 1
    if boot_ready() then
      empty_run = empty_run + 1
    else
      empty_run = 0
    end
    -- DSP init writes the ports well before the first flushed request. If the Z80 is
    -- ticking and nothing has hit 0xD002, -sound none is not keeping the QSound device.
    if phase_frame >= 90 and irq_total > 20 and boot_commits == 0 then
      finish("no QSound port writes")
      return
    end
    if phase_frame >= 90 and irq_total == 0 then
      finish("Z80 interrupt counter did not move")
      return
    end
    if phase_frame > 4000 then
      finish("sound mailbox never became ready")
      return
    end
    if empty_run >= 8 and boot_commits > 0 then
      if #cmds == 0 then finish("no sound commands"); return end
      ready_frame = frame
      phase = "record"
      inject(cmds[1])
      return
    end
    return
  end

  if phase == "silence" then
    silence_wait = silence_wait + 1
    clamp_ring()
    if silence_writes > 0 then
      quiet = 0
      silence_writes = 0
    elseif sent then
      quiet = quiet + 1
    end
    if silence_wait > 180 then
      finish("stop-all did not go quiet")
      return
    end
    if quiet >= 8 then
      io.stderr:write(string.format("sound_tap: quiet after %d\n", silence_wait))
      phase = "record"
      inject(cmds[cmd_i + 1])
    end
    return
  end

  if phase == "record" then
    clamp_ring()
    hold_demo()
    rec_f = rec_f + 1
    if trace then trace:flush() end
    local cur = cmds[cmd_i]
    if cur.extra and rec_f == cur.extra.at then
      local x = cur.extra
      if trace then trace:write(string.format("# extra code=%04x d1=%08x d2=%08x at=%d\n", x.code, x.d1, x.d2, x.at)) end
      guard = false
      enqueue(x.code, x.d1, x.d2)
      cur.extra = nil
    end
    if rec_f >= FRAMES then
      close_code()
      if cmd_i >= #cmds then
        phase = "dead"
        write_status()
        manager.machine:exit()
        return
      end
      begin_silence()
    end
  end
end

emu.register_frame_done(function()
  local ok, err = pcall(frame_body)
  if not ok then finish("lua " .. tostring(err)) end
end)
