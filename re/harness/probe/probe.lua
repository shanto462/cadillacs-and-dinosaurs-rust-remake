-- Probe for "dino" (Cadillacs and Dinosaurs) under MAME: put the original in any situation and log it.
-- Run it with probe.sh (same folder). Full manual: plan/game/design/capture-recipes.md.
--
-- Environment (probe.sh sets the output paths):
--   PR_INPUT   file with key lines and directives (see below)
--   PR_POKES   a second file in the same format (kept for the old name; both files are merged)
--   PR_LINES   directives given inline, separated by ';' (merged with the files)
--   PR_SNAPS   screenshots: "t1,t2,.." or "every:N:from:to" (frames as in the input file); default for every segment
--   PR_WATCH   watch lines separated by ';', same text as a "watch" directive without the word
--   PR_FRAMES  number of frames to run (default 600); default for every segment
--   PR_OUTDIR  output folder (probe.sh sets it), PR_NAME  name of the run
--
-- One MAME process can run many probes. A line `segment NAME` starts a new segment; every segment is an
-- independent run from power on (the probe saves the power-on state and loads it again), with its own
-- `jump`, `frames N`, `snaps SPEC` and directives. Lines before the first `segment` line belong to every
-- segment. The output of a segment is PR_OUTDIR/NAME/log.txt and PR_OUTDIR/NAME/snap/0000.png ...; a file
-- without `segment` lines writes PR_OUTDIR/log.txt and PR_OUTDIR/snap/.
--
-- Frame numbers. With a `jump` line, t = 0 is the first frame after the jump (you can play), and every
-- frame number in a segment counts from there. Without a jump, frames count from power on.
-- A directive line is `[FRAME] COMMAND args`; a key line is `a-b:KEYS` (see re/harness/README.md;
-- U D L R A J S C, digits 1 2 3 choose the player). Old poke lines `FRAME HEXADDR SIZE VALUE` still work.
-- Commands: jump, poke, hold, release, spawn, hp, lives, score, god, time, killall, rank, level, aggr,
-- watch, unwatch, snap, log, stop, events, peek, objset, frames, snaps, snapshot, trace.
--
-- Scenarios (probe_record.sh, plan task M3-06). `[FRAME] snapshot [control|control+N] [keep]` ends the
-- setup of a segment: the lines before it (jump, poke, spawn, hold, god, key lines, ...) build the scene
-- with frames counted as above; at the first game frame boundary at or after FRAME the whole RAM and video
-- RAM are written in the harness format (PR_TRACE_DIR/NAME.state). `control` waits for the frame where
-- player 1 can be controlled (the DINO_SYNC=control rule of record.lua), `control+N` N game frames later.
-- Holds and `god` stop at the snapshot unless `keep` is given. `trace N` then records N game frames of
-- harness records (NAME.mame.trc, the same file as record.sh writes). The key lines after the snapshot are
-- the input script of the scenario, with frame 0 = the first game frame after the snapshot; they are also
-- written to NAME.input.txt for re/harness/port.sh. Directives after the snapshot run at their game frame
-- (the port does not see them). `frames N` does not apply to such a segment (PR_SNAP_MAXF is the limit
-- for the snapshot). Key lines before the snapshot of a segment WITHOUT `jump` count vblank IRQs from power
-- on (what record.sh boot scripts count, about 70 fewer than video frames at the stage start), so the
-- lines of scripts/boot_jack.txt give the state of `DINO_SYNC=control` bit for bit. Needs PR_RECORD_LUA
-- (record.lua, loaded as a library for its trace writer); PR_ONLY=a,b or * selects segments (* = every
-- segment that has a snapshot line); PR_TRACE_DIR is the output folder (default: the segment folder),
-- PR_TRACE_NAME overrides NAME (one segment only).

local cpu = manager.machine.devices[":maincpu"]
local space = cpu.spaces["program"]

local function env(n, d) local v = os.getenv(n); if v == nil or v == "" then return d end return v end
local function read_file(path)
  if path == nil then return "" end
  local fh = io.open(path, "rb")
  if not fh then io.stderr:write("probe.lua: cannot open " .. path .. "\n"); return "" end
  local s = fh:read("a"); fh:close(); return s
end

local logf = nil
local OUTDIR = env("PR_OUTDIR", ".")
local RUNNAME = env("PR_NAME", "probe")

-- ---------------------------------------------------------------------------------------------
-- Memory helpers (RAM base 0xFF8000 = register a5 of the game; G.$xxxx = 0xFF8000 + xxxx)
-- ---------------------------------------------------------------------------------------------
local function r8(a) return space:read_u8(a) end
local function r16(a) return space:read_u16(a) end
local function r32(a) return space:read_u32(a) end
local function w8(a, v) space:write_u8(a, v & 0xFF) end
local function w16(a, v) space:write_u16(a, v & 0xFFFF) end
local function w32(a, v) space:write_u32(a, v & 0xFFFFFFFF) end
local function s16(v) if v >= 0x8000 then return v - 0x10000 end return v end
local function s8(v) if v >= 0x80 then return v - 0x100 end return v end
local function s32(v) if v >= 0x80000000 then return v - 0x100000000 end return v end

local G = {
  frame = 0xFF8000, ingame = 0xFF84CC, mode = 0xFF84D0, sub = 0xFF84D1, stage = 0xFF84D9,
  scripted = 0xFF84DD, vertical = 0xFF84DC, players = 0xFF84EF, alive = 0xFF84F3, time = 0xFF84E8,
  killall = 0xFF850E, car = 0xFF8518, aggr = 0xFF85EC, level = 0xFF85EE, rank = 0xFF85F0,
  rankfreeze = 0xFF8602, camstate = 0xFF8740, cammode = 0xFF8741, camsub = 0xFF8742, camx = 0xFF8744,
  camy = 0xFF8748, sx2 = 0xFF8782, sy2 = 0xFF8784, evy = 0xFF878A, area = 0xFF8786, nextev = 0xFF8792, evidx = 0xFF87A8, go = 0xFF87A9, lock = 0xFF87B4,
  byy = 0xFF87B5, spawn = 0xFF866C,
}
local PLAYER = { 0xFFB274, 0xFFB3F4, 0xFFB574 }   -- P1, P2, P3 (0x180 bytes each)

-- Object pools: base of slot 0, slot count, slot size, free-list count word and pointer word
-- (allocators 0x483C A, 0x4860 B, 0x4818 E, 0x4884 C, 0x48A8 D; re/specs/enemy/SPEC.md section 1).
local POOLS = {
  A = { base = 0xFFC8F4, n = 24, size = 0xE0, cnt = 0xFFE5F8, ptr = 0xFFE602 },
  B = { base = 0xFFB6F4, n = 24, size = 0xC0, cnt = 0xFFE5F6, ptr = 0xFFE600 },
  E = { base = 0xFF8874, n = 24, size = 0xC0, cnt = 0xFFE5F4, ptr = 0xFFE5FE },
  C = { base = 0xFF9A74, n = 48, size = 0x80, cnt = 0xFFE5FA, ptr = 0xFFE604 },
  D = { base = 0xFFDDF4, n = 16, size = 0x80, cnt = 0xFFE5FC, ptr = 0xFFE606 },
}

-- ---------------------------------------------------------------------------------------------
-- Field names for `watch`: name = { offset, size in bytes, style }. style: s signed, h hex, c class/4.
-- Also usable: +0xNN[.b|.w|.l|.sb|.sw|.sl] (offset from the object), @0xADDR[.type] (absolute).
-- ---------------------------------------------------------------------------------------------
local OF = {
  active = { 0, 1 }, on = { 1, 1 }, who = { 2, 1 }, name = { 3, 1 }, r4 = { 4, 1 }, r5 = { 5, 1 }, ai = { 6, 1 },
  ai2 = { 7, 1 }, st = { 4, 4, "h" }, stw = { 4, 2, "h" },
  xf = { 8, 4, "h" }, x = { 8, 2, "s" }, y = { 0xC, 2, "s" }, z = { 0x10, 2, "s" },
  vx = { 0x14, 2, "s" }, vy = { 0x16, 2, "s" }, vz = { 0x18, 2, "s" },
  ax = { 0x1A, 2, "s" }, ay = { 0x1C, 2, "s" }, az = { 0x1E, 2, "s" },
  cls = { 0x20, 2, "c" }, dir = { 0x22, 1 }, pal = { 0x23, 1, "h" }, face = { 0x24, 1 }, draw = { 0x25, 1, "h" },
  var = { 0x26, 2, "h" }, anim = { 0x28, 4, "h" }, fcls = { 0x2E, 1 }, ftimer = { 0x2F, 1 },
  air = { 0x51, 1 }, floor = { 0x54, 2, "s" }, atk = { 0x58, 1 }, hs = { 0x59, 1 }, b5b = { 0x5B, 1, "h" },
  w5c = { 0x5C, 2, "h" }, serial = { 0x5E, 1 }, hpbar = { 0x6A, 2 }, hp = { 0x6C, 2, "s" }, hpprev = { 0x6E, 2, "s" },
  target = { 0x76, 2, "h" }, poise = { 0x78, 1 }, t80 = { 0x80, 2 }, drop = { 0x96, 1 },
  -- players only
  char = { 3, 1 }, lives = { 0xA3, 1 }, held = { 0xB4, 2, "h" }, inv = { 0xF9, 1 }, rinv = { 0xEC, 2 },
  score = { 0x100, 4, "h" }, input = { 0xA0, 1, "h" },
}
local GF = {   -- game globals, absolute addresses
  time = { G.time, 2, "h" }, camx = { G.camx, 2 }, camy = { G.camy, 2 }, mode = { G.mode, 1, "h" },
  sub = { G.sub, 1, "h" }, stage = { G.stage, 1 }, area = { G.area, 1 }, ingame = { G.ingame, 1 },
  cammode = { G.cammode, 1, "h" }, go = { G.go, 1 }, lock = { G.lock, 1 }, alive = { G.alive, 1 },
  players = { G.players, 1 }, rank = { G.rank, 2 }, level = { G.level, 2 }, aggr = { G.aggr, 2 },
  frame = { G.frame, 2 }, killall = { G.killall, 1 }, car = { G.car, 1 }, scripted = { G.scripted, 1 },
  nextev = { G.nextev, 4, "h" }, evidx = { G.evidx, 1 },
}
local function rd(addr, size, style)
  local v
  if size == 1 then v = r8(addr) elseif size == 2 then v = r16(addr) else v = r32(addr) end
  if style == "s" then
    if size == 1 then v = s8(v) elseif size == 2 then v = s16(v) else v = s32(v) end
  end
  return v
end
local function fmt_val(v, size, style)
  if style == "h" then return string.format("%0" .. (size * 2) .. "X", v) end
  if style == "c" then return tostring(v // 4) end
  return tostring(v)
end
-- Parse one field token into { label, offset_or_addr, size, style, absolute }.
local function parse_field(tok, is_global)
  local nm = tok:lower()
  if is_global and GF[nm] then local f = GF[nm]; return { label = nm, off = f[1], size = f[2], style = f[3], abs = true } end
  if (not is_global) and OF[nm] then local f = OF[nm]; return { label = nm, off = f[1], size = f[2], style = f[3] } end
  local base, ty = nm:match("^([+@]0?x?%x+)%.?(%a*)$")
  if base then
    local isabs = base:sub(1, 1) == "@"
    local n = tonumber((base:sub(2):gsub("^0?x", "")), 16)
    local size, style = 2, nil
    if ty == "b" then size = 1 elseif ty == "w" or ty == "" then size = 2 elseif ty == "l" then size = 4
    elseif ty == "sb" then size, style = 1, "s" elseif ty == "sw" then size, style = 2, "s"
    elseif ty == "sl" then size, style = 4, "s" elseif ty == "hb" then size, style = 1, "h"
    elseif ty == "hw" then size, style = 2, "h" elseif ty == "hl" then size, style = 4, "h" end
    return { label = tok, off = n, size = size, style = style, abs = isabs }
  end
  return nil
end

-- ---------------------------------------------------------------------------------------------
-- Logging
-- ---------------------------------------------------------------------------------------------
local F = 0          -- video frames since power on (counted at the end of each frame)
local T = nil        -- frames since the jump finished (or = F with no jump); nil while jumping
local function tstr() if T == nil then return "-" end return tostring(T) end
local function log(tag, fmt, ...)
  logf:write(string.format("t=%s f=%d gf=%d %s ", tstr(), F, r16(G.frame), tag))
  logf:write(string.format(fmt, ...))
  logf:write("\n")
end

-- ---------------------------------------------------------------------------------------------
-- Key scripts (same format as the harness)
-- ---------------------------------------------------------------------------------------------
local BITS = { R = 1, L = 2, D = 4, U = 8, A = 0x10, J = 0x20 }
local function new_in() return { p = { 0, 0, 0 }, start = 0, coin = 0 } end
local function parse_keys(text, segs)
  for part in text:gmatch("[^,]+") do
    part = part:gsub("^%s+", ""):gsub("%s+$", "")
    if part ~= "" then
      local range, keys = part:match("^([^:]*):?(.*)$")
      local a, b = range:match("^%s*(%d+)%s*%-%s*(%d+)%s*$")
      if not a then a = range:match("^%s*(%d+)%s*$"); b = a end
      if a then
        local f = new_in()
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
end
local function script_at(segs, frame)
  local out = new_in()
  for _, s in ipairs(segs) do
    if frame >= s[1] and frame <= s[2] then
      local f = s[3]
      for i = 1, 3 do out.p[i] = out.p[i] | f.p[i] end
      out.start = out.start | f.start; out.coin = out.coin | f.coin
    end
  end
  return out
end

-- ---------------------------------------------------------------------------------------------
-- Parse the directive files
-- ---------------------------------------------------------------------------------------------
local COMMANDS = { jump = true, poke = true, hold = true, release = true, spawn = true, hp = true, lives = true,
  score = true, god = true, time = true, killall = true, rank = true, level = true, aggr = true, watch = true,
  unwatch = true, snap = true, log = true, stop = true, events = true, peek = true, objset = true, snapshot = true }
local keysegs = {}
local items = {}
local jump_item = nil
local maxT = 600
-- Scenario parts (`snapshot`, `trace N`): lines after the snapshot line go to post_items / postkeys.
local postkeys, postlines, post_items = {}, {}, {}
local snap_cmd, trace_n, after_snap = nil, nil, false
local snaps, every = {}, nil
local function split(s) local t = {}; for w in s:gmatch("%S+") do t[#t + 1] = w end return t end
local function num(s)
  if s == nil then return nil end
  s = s:gsub("^%$", "0x")
  return tonumber(s)
end

local function parse_snaps(sn)
  snaps, every = {}, nil
  local e1, e2, e3 = sn:match("^every:(%d+):?(%d*):?(%d*)")
  if e1 then
    every = { tonumber(e1), (e2 ~= "" and tonumber(e2)) or 0, (e3 ~= "" and tonumber(e3)) or math.huge }
  else
    for v in sn:gmatch("%d+") do snaps[tonumber(v)] = true end
  end
end

local function parse_text(text)
  for line in text:gmatch("[^\n]+") do
    line = line:gsub("^%s+", ""):gsub("%s+$", "")
    if line ~= "" then
      if line:match("^%d+%s*%-%s*%d+%s*:") or line:match("^%d+%s*:") then
        if after_snap then parse_keys(line, postkeys); postlines[#postlines + 1] = line
        else parse_keys(line, keysegs) end
      else
        local tk = split(line)
        local frame, cmdi = 0, 1
        local first = tk[1]:lower()
        if first == "frames" then maxT = num(tk[2]) or maxT; cmdi = nil
        elseif first == "trace" then trace_n = num(tk[2]); cmdi = nil
        elseif first == "snaps" then parse_snaps(tk[2] or ""); cmdi = nil
        elseif tonumber(tk[1]) and tk[2] and COMMANDS[tk[2]:lower()] then frame = tonumber(tk[1]); cmdi = 2
        elseif tonumber(tk[1]) and tk[2] and tk[2]:match("^%x+$") and #tk == 4 then
          -- old poke line: FRAME HEXADDR SIZE VALUE
          items[#items + 1] = { frame = tonumber(tk[1]), cmd = "poke", args = { "0x" .. tk[2], tk[3], tk[4] }, line = line }
          cmdi = nil
        end
        if cmdi then
          local cmd = tk[cmdi] and tk[cmdi]:lower()
          if cmd and COMMANDS[cmd] then
            local args = {}
            for i = cmdi + 1, #tk do args[#args + 1] = tk[i] end
            local it = { frame = frame, cmd = cmd, args = args, line = line }
            if cmd == "jump" then
              if after_snap then io.stderr:write("probe.lua: jump after the snapshot ignored: " .. line .. "\n")
              else jump_item = it end
            elseif cmd == "snapshot" then
              if snap_cmd then io.stderr:write("probe.lua: second snapshot line ignored: " .. line .. "\n")
              else snap_cmd = it; items[#items + 1] = it; after_snap = true end
            elseif after_snap then post_items[#post_items + 1] = it
            else items[#items + 1] = it end
          else
            io.stderr:write("probe.lua: cannot read line: " .. line .. "\n")
          end
        end
      end
    end
  end
end

-- Split the whole input into segments. Lines before the first `segment` line are added to every segment.
local function split_segments(text)
  text = (text:gsub("#[^\n]*", "")):gsub(";", "\n")
  local header, segs, curseg = {}, {}, nil
  for line in text:gmatch("[^\n]+") do
    local nm, rest = line:match("^%s*segment%s+(%S+)%s*(.*)")
    if nm then
      curseg = { name = nm, lines = {}, opts = rest }
      segs[#segs + 1] = curseg
    else
      local dst = curseg and curseg.lines or header
      dst[#dst + 1] = line
    end
  end
  if #segs == 0 then segs[1] = { name = RUNNAME, lines = header, single = true }; header = {} end
  for _, sg in ipairs(segs) do
    local all = {}
    for _, l in ipairs(header) do all[#all + 1] = l end
    for _, l in ipairs(sg.lines) do all[#all + 1] = l end
    sg.text = table.concat(all, "\n")
  end
  return segs
end

local function load_segment_text(sg)
  keysegs, items, jump_item = {}, {}, nil
  postkeys, postlines, post_items, snap_cmd, trace_n, after_snap = {}, {}, {}, nil, nil, false
  maxT = tonumber(env("PR_FRAMES", "600"))
  parse_snaps(env("PR_SNAPS", ""))
  local o = sg.opts or ""
  for k, v in o:gmatch("(%w+)=(%S+)") do
    if k == "frames" then maxT = num(v) or maxT elseif k == "snaps" then parse_snaps(v) end
  end
  parse_text(sg.text)
  after_snap = false
  local pw = env("PR_WATCH", "")
  if pw ~= "" then
    for w in pw:gmatch("[^;]+") do
      if w:match("^%s*%x+:%d+") then      -- old raw form "hexaddr:size,hexaddr:size"
        for a, sz in w:gmatch("(%x+):(%d+)") do
          items[#items + 1] = { frame = 0, cmd = "watch", args = { "30", "G", "@0x" .. a .. (sz == "1" and ".b" or sz == "2" and ".w" or ".l") }, line = "watch " .. w }
        end
      else
        parse_text("watch " .. w)
      end
    end
  end
  for i, it in ipairs(items) do it.seq = i end
  table.sort(items, function(a, b) if a.frame ~= b.frame then return a.frame < b.frame end return a.seq < b.seq end)
  for i, it in ipairs(post_items) do it.seq = i end
  table.sort(post_items, function(a, b) if a.frame ~= b.frame then return a.frame < b.frame end return a.seq < b.seq end)
  if trace_n and not snap_cmd then
    io.stderr:write("probe.lua: `trace` needs a `snapshot` line in segment " .. sg.name .. "; ignored\n")
    trace_n = nil
  end
  if snap_cmd and (trace_n == nil or trace_n < 1) then trace_n = 600 end
end

local snapcount = 0
local seg = nil              -- the running segment { name, text, ... , rel = "NAME/" or "" }
local scr = nil
for _, sc in pairs(manager.machine.screens) do scr = sc end
local function take_snap()
  local name = string.format("%ssnap/%04d.png", seg.rel, snapcount)
  if scr then scr:snapshot(name) else manager.machine.video:snapshot() end
  log("SNAP", "%d", snapcount)
  snapcount = snapcount + 1
end

-- ---------------------------------------------------------------------------------------------
-- Input injection (read taps on the CPS1 input ports, as in re/harness/record.lua)
-- ---------------------------------------------------------------------------------------------
local cur = new_in()
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
local on_irq = function() end   -- scenario recorder (snapshot and trace), set below
local irq_n = 0                  -- vblank IRQ ticks since power on (record.lua's `vblank`; fewer than video frames)
local irq_time = 0
local want_irq_tap = false      -- only a batch with a `snapshot` line installs the IRQ tap (other runs stay as they were)
local function install()
  PR_TAPS = {
    space:install_read_tap(0x800000, 0x800001, "pr_in1", function(o, d, m) return (d & ~m) | (raw_in1() & m) end),
    space:install_read_tap(0x800018, 0x800019, "pr_in0", function(o, d, m) return (d & ~m) | ((raw_in0() | 0xFF) & m) end),
    space:install_read_tap(0xF1C000, 0xF1C001, "pr_in2", function(o, d, m) return (d & ~m) | ((raw_in2() | 0xFF00) & m) end),
  }
  if want_irq_tap then
    -- The vblank IRQ (0x542) writes the frame flag -$7EAE(a5) = 0xFF0152 at 0x5C8: the game frame boundary of
    -- re/harness/record.lua (the same tap). Only `snapshot` / `trace` segments use it.
    PR_TAPS[#PR_TAPS + 1] = space:install_write_tap(0xFF0152, 0xFF0153, "pr_irq", function(offset, data, mask)
      if offset == 0xFF0152 and (mask & 0xFF00) ~= 0 and (data & 0xFF00) ~= 0 then
        local tm = manager.machine.time:as_double()
        if tm < irq_time then irq_n = 0 end       -- the saved power-on state was loaded: a new run from power on
        irq_time = tm
        irq_n = irq_n + 1
        on_irq()
      end
    end)
  end
  installed = true
end

-- ---------------------------------------------------------------------------------------------
-- Object helpers: spawn (allocator copy), players, selectors
-- ---------------------------------------------------------------------------------------------
-- Take a slot from a pool's free list, like the allocators do (stack of 16-bit slot offsets).
local function alloc(pool)
  local P = POOLS[pool]
  local cnt = r16(P.cnt)
  if cnt == 0 then return nil end
  local ptr = r16(P.ptr)
  local slot = r16(0xFF0000 + ptr)
  w16(P.ptr, ptr + 2)
  w16(P.cnt, cnt - 1)
  return 0xFF0000 + slot
end
local function slot_of(pool, addr) return (addr - POOLS[pool].base) // POOLS[pool].size end

local function cam_x() return r16(G.camx) end
local function present_players(sel)
  local out = {}
  for i = 1, 3 do
    if (sel == nil or sel == i) and r8(PLAYER[i]) ~= 0 then out[#out + 1] = i end
  end
  return out
end
local function player_arg(args, from)
  -- "P2", "2" or nothing (= all present players)
  for i = from, #args do
    local p = args[i]:lower():match("^p?([123])$")
    if p then return tonumber(p) end
  end
  return nil
end

local function kv(args, from)
  local t = {}
  for i = from, #args do
    local k, v = args[i]:match("^([%w_]+)=(.*)$")
    if k then t[k:lower()] = v end
  end
  return t
end

local function do_spawn(args)
  local pool = (args[1] or ""):upper()
  if not POOLS[pool] then log("ERR", "spawn: pool must be A, B, C, D or E: %s", table.concat(args, " ")); return end
  local o = kv(args, 3)
  local cls = num(args[2])
  local cw = o.cw and num(o.cw) or (cls and cls * 4)
  if cw == nil then log("ERR", "spawn: class missing"); return end
  local count = o.n and num(o.n) or 1
  local zdef = o.z and num(o.z) or (r8(PLAYER[1]) ~= 0 and r16(PLAYER[1] + 0x10)) or 0x60   -- default: depth of player 1
  local dx = o.dx and num(o.dx) or 0x30
  local xs = o.ax or o.x or "+0x120"
  local xrel = (not o.ax) and (xs:sub(1, 1) == "+" or xs:sub(1, 1) == "-")
  local x0 = num(xs)
  for k = 0, count - 1 do
    local a = alloc(pool)
    if not a then log("ERR", "spawn %s class %s: pool is full", pool, tostring(cls)); return end
    local x = (xrel and (cam_x() + x0) or x0) + k * dx
    w8(a, 1)
    w16(a + 0x20, cw)
    w16(a + 0x08, x)
    w16(a + 0x0C, o.y and num(o.y) or 0)
    w16(a + 0x10, zdef)
    w16(a + 0x26, o.var and num(o.var) or 0)
    w16(a + 0x5C, o.w5c and num(o.w5c) or 0)
    w8(a + 0x5B, o.b5b and num(o.b5b) or 0)
    w8(a + 0x96, (o.drop and num(o.drop)) or (o.b96 and num(o.b96)) or 0)
    log("SPAWN", "%s#%d class=%s var=%X x=%d z=%d addr=%X", pool, slot_of(pool, a), cls and tostring(cls) or ("cw" .. cw),
      o.var and num(o.var) or 0, s16(x & 0xFFFF), zdef, a)
  end
end

-- ---------------------------------------------------------------------------------------------
-- Persistent effects: holds (re-poked every frame) and god mode
-- ---------------------------------------------------------------------------------------------
local holds = {}      -- { addr, size, value, until_t, on_end }
local god = {}        -- player index -> true
local function put(addr, size, v) if size == 1 then w8(addr, v) elseif size == 2 then w16(addr, v) else w32(addr, v) end end
local function apply_holds()
  for i = #holds, 1, -1 do
    local h = holds[i]
    if h.until_t and T and T > h.until_t then
      if h.on_end ~= nil then put(h.addr, h.size, h.on_end) end
      table.remove(holds, i)
    else
      put(h.addr, h.size, h.value)
    end
  end
  for i = 1, 3 do
    if god[i] and r8(PLAYER[i]) ~= 0 then
      w8(PLAYER[i], 2)               -- +$00 = 2: active but invulnerable (hit tests skip it)
      w8(PLAYER[i] + 0xF9, 0x7F)     -- keep the invulnerability timer from ending
      if r16(PLAYER[i] + 0x6C) == 0 then w16(PLAYER[i] + 0x6C, 1) end
    end
  end
end
local function add_hold(addr, size, value, until_t, on_end)
  for i = #holds, 1, -1 do if holds[i].addr == addr then table.remove(holds, i) end end
  holds[#holds + 1] = { addr = addr, size = size, value = value, until_t = until_t, on_end = on_end }
end
local function drop_hold(addr) for i = #holds, 1, -1 do if holds[i].addr == addr then table.remove(holds, i) end end end

-- ---------------------------------------------------------------------------------------------
-- Watch
-- ---------------------------------------------------------------------------------------------
local watches = {}
local function make_watch(args, start_t)
  local interval = num(args[1]) or 1
  local sel = args[2] or "G"
  local w = { interval = interval, sel = sel, fields = {}, start = start_t, raw = table.concat(args, " ") }
  local su = sel:upper()
  local pool, slot, cls = su:match("^([ABCDE])#(%d+)$")
  if pool then w.pool, w.slot = pool, tonumber(slot) else
    pool, cls = su:match("^([ABCDE]):(%d+)$")
    if pool then w.pool, w.cls = pool, tonumber(cls) else
      pool = su:match("^([ABCDE])$")
      if pool then w.pool = pool
      elseif su:match("^P[123]?$") then w.player = tonumber(su:sub(2)) or 0
      elseif su == "G" then w.global = true
      else log("ERR", "watch: bad selector %s", sel); return nil end
    end
  end
  for i = 3, #args do
    local f = parse_field(args[i], w.global)
    if f then w.fields[#w.fields + 1] = f else log("ERR", "watch: unknown field %s", args[i]) end
  end
  if #w.fields == 0 then
    local defaults = w.global and { "mode", "sub", "stage", "area", "time", "camx" } or { "x", "y", "z", "hp", "st" }
    for _, n in ipairs(defaults) do w.fields[#w.fields + 1] = parse_field(n, w.global) end
  end
  return w
end
local function watch_line(label, base, w, with_class)
  local parts = { label }
  if with_class then parts[#parts + 1] = string.format("c=%d", r16(base + 0x20) // 4) end
  for _, f in ipairs(w.fields) do
    local addr = f.abs and f.off or (base + f.off)
    parts[#parts + 1] = f.label .. "=" .. fmt_val(rd(addr, f.size, f.style), f.size, f.style)
  end
  log("W", "%s", table.concat(parts, " "))
end
-- Call fn(label, base) for every live object that a selector (as parsed by make_watch) names.
local function each_object(w, fn)
  if w.global then return end
  if w.player then
    for i = 1, 3 do
      if (w.player == 0 or w.player == i) and r8(PLAYER[i]) ~= 0 then fn("P" .. i, PLAYER[i], false) end
    end
    return
  end
  local P = POOLS[w.pool]
  for sl = 0, P.n - 1 do
    local base = P.base + sl * P.size
    if w.slot ~= nil then
      if sl == w.slot then
        if r8(base) ~= 0 then fn(w.pool .. "#" .. sl, base, true) else log("W", "%s#%d -", w.pool, sl) end
      end
    elseif r8(base) ~= 0 and (w.cls == nil or r16(base + 0x20) // 4 == w.cls) then
      fn(w.pool .. "#" .. sl, base, w.cls == nil)
    end
  end
end

local function sample(w)
  if w.global then
    watch_line("G", 0, w, false); return
  end
  each_object(w, function(label, base, with_class) watch_line(label, base, w, with_class) end)
end

-- ---------------------------------------------------------------------------------------------
-- Cheats
-- ---------------------------------------------------------------------------------------------
-- Camera events of the current episode, read from the program ROM at run time (table 0x21B2A).
local function camera_events()
  local stage = r8(G.stage)
  local base = 0x21B2A + s16(r16(0x21B2A + 2 * stage))
  if stage == 2 then base = 0x22B9E end          -- Episode 3 has its own table (stage 2 controller 0x22398)
  local evs = {}
  for k = 0, 60 do
    local a = base + 16 * k
    local x = r16(a)
    if x == 0xFFFF then break end
    evs[#evs + 1] = { idx = k, addr = a, x = x, y = r16(a + 2), area = r8(a + 0xA), mode = r8(a + 0xB), byy = r8(a + 0xF) }
  end
  return evs
end

local RANK_LEVEL_TABLE = 0x1860C        -- level = W[0x1860C + 2*(rank*3 + players-1)]

-- Scenario state (`snapshot` / `trace N`, section "Scenarios" below). phase: "" setup, "armed" waiting for the
-- game frame boundary, "trace" recording, "done" finished.
local sn = { phase = "", k = 0, lag = 0 }
local arm_snapshot = function() end

local function exec(it)
  local c, a = it.cmd, it.args
  if c == "poke" then
    local addr = num(a[1]); local size = num(a[2]) or 1; local v = num(a[3])
    if addr == nil or v == nil then log("ERR", "poke: %s", it.line); return end
    put(addr, size, v)
    log("X", "poke %X size %d = %X", addr, size, v)
  elseif c == "hold" then
    local addr = num(a[1]); local size = num(a[2]) or 1; local v = num(a[3])
    if addr == nil or v == nil then log("ERR", "hold: %s", it.line); return end
    add_hold(addr, size, v)
    log("X", "hold %X size %d = %X", addr, size, v)
  elseif c == "release" then
    local addr = num(a[1])
    if addr then drop_hold(addr) end
    log("X", "release %X", addr or 0)
  elseif c == "spawn" then
    do_spawn(a)
  elseif c == "hp" then
    local v = num(a[1]) or 100
    local p = player_arg(a, 2)
    for _, i in ipairs(present_players(p)) do
      w16(PLAYER[i] + 0x6A, math.min(v, 100)); w16(PLAYER[i] + 0x6C, v); w16(PLAYER[i] + 0x6E, v)
    end
    log("X", "hp %d", v)
  elseif c == "lives" then
    local v = num(a[1]) or 2
    local p = player_arg(a, 2)
    for _, i in ipairs(present_players(p)) do w8(PLAYER[i] + 0xA3, v) end
    log("X", "lives %d", v)
  elseif c == "score" then
    local v = tonumber(a[1] or "0", 16) or 0       -- decimal digits are stored as BCD digits
    local p = player_arg(a, 2)
    for _, i in ipairs(present_players(p)) do
      w32(PLAYER[i] + 0x100, v)
      w8(0xFF0ECC, 1); w8(0xFF0ECC + i, 1)        -- the HUD "score changed" flags (G.-$7134, G.-$7133 + player)
    end
    log("X", "score %s", a[1] or "0")
  elseif c == "god" then
    local on = (a[1] or "on"):lower() ~= "off"
    local p = player_arg(a, 2)
    for i = 1, 3 do if p == nil or p == i then god[i] = on or nil end end
    if not on then for _, i in ipairs(present_players(p)) do w8(PLAYER[i], 1) end end
    log("X", "god %s", on and "on" or "off")
  elseif c == "time" then
    local v
    local m, s = (a[1] or ""):match("^(%d+):(%d%d)$")
    if m then v = tonumber(m .. s, 16) else v = num(a[1]) end
    if v == nil then log("ERR", "time: %s", it.line); return end
    w16(G.time, v)                       -- BCD m:ss; the seconds tick takes 51 frames
    log("X", "time %04X", v)
  elseif c == "killall" then
    add_hold(G.killall, 1, 2, T + 6, 0)  -- kill-all flag 2 (no score) for 7 frames, like a wave timeout
    log("X", "killall")
  elseif c == "level" then
    local v = num(a[1])
    if (a[1] or "free"):lower() == "free" then drop_hold(G.level); drop_hold(G.rankfreeze); w16(G.rankfreeze, 0); log("X", "level free"); return end
    add_hold(G.rankfreeze, 2, 1); add_hold(G.level, 2, v)
    log("X", "level %d (rank clock frozen)", v)
  elseif c == "rank" then
    if (a[1] or "free"):lower() == "free" then drop_hold(G.rank); drop_hold(G.level); drop_hold(G.rankfreeze); w16(G.rankfreeze, 0); log("X", "rank free"); return end
    local v = num(a[1])
    local pl = r8(G.players)               -- players - 1
    local lv = r16(RANK_LEVEL_TABLE + 2 * (v * 3 + pl))
    add_hold(G.rankfreeze, 2, 1); add_hold(G.rank, 2, v); add_hold(G.level, 2, lv)
    log("X", "rank %d -> level %d (rank clock frozen)", v, lv)
  elseif c == "aggr" then
    if (a[1] or "free"):lower() == "free" then drop_hold(G.aggr); log("X", "aggr free"); return end
    add_hold(G.aggr, 2, num(a[1]))
    log("X", "aggr %d", num(a[1]))
  elseif c == "watch" then
    local w = make_watch(a, it.frame)
    if w then watches[#watches + 1] = w; log("X", "watch %s", w.raw) end
  elseif c == "unwatch" then
    watches = {}
    log("X", "unwatch")
  elseif c == "snap" then
    take_snap()
  elseif c == "log" then
    log("NOTE", "%s", table.concat(a, " "))
  elseif c == "snapshot" then
    arm_snapshot(it)
  elseif c == "stop" then
    if sn.phase == "trace" then log("ERR", "stop: not possible while the trace runs") else maxT = T end
  elseif c == "objset" then
    -- objset SELECTOR FIELD VALUE: write a field of every selected object (A:18 hp 1, E#3 x 400, P2 hp 50)
    local w = make_watch({ "1", a[1] or "", a[2] or "" }, 0)
    local v = num(a[3])
    if w and #w.fields == 1 and v ~= nil and not w.global then
      local n = 0
      each_object(w, function(label, base) put(base + w.fields[1].off, w.fields[1].size, v); n = n + 1 end)
      log("X", "objset %s %s = %d (%d objects)", a[1], a[2], v, n)
    else
      log("ERR", "objset: %s", it.line)
    end
  elseif c == "peek" then
    local addr = num(a[1]) or 0
    local n = num(a[2]) or 8
    local w = {}
    for i = 0, n - 1 do w[#w + 1] = string.format("%04X", r16(addr + 2 * i)) end
    log("PEEK", "%X: %s", addr, table.concat(w, " "))
  elseif c == "events" then
    for _, e in ipairs(camera_events()) do
      log("EVENT", "idx=%d addr=%X x=%X y=%X area=%d mode=%X vert=%d", e.idx, e.addr, e.x, e.y, e.area, e.mode, e.byy)
    end
  end
end

-- ---------------------------------------------------------------------------------------------
-- The jump
-- ---------------------------------------------------------------------------------------------
-- jump EP AREA [CAMX] [char=NAME|N[,..]] [at=start]
--   EP 1..8, AREA = camera area number G.$786 (0-based as in the camera table), CAMX = camera X to start at.
-- Boots from power on (coin, start, select), enters episode EP by the story-panel poke, then moves
-- through areas with the game's own end-of-area camera events, and finally shifts the camera to CAMX.
local CHAR_POS = { jack = 0, hannah = 1, mustapha = 2, mess = 3 }     -- cursor position on the select screen
local CHAR_ID = { jack = 0, mustapha = 1, hannah = 2, mess = 3 }      -- id in +$03 (verified by art)
-- Camera modes. Stages 0 and 3..7 use the table of 0x2092E, stage 2 (Episode 3) has its own controller
-- (0x22398, modes of table 0x2250A: 0 follow, 1 area end with fade, 2 lock, 3 stage end, 4 sub-area,
-- 5 auto scroll, 6 area change without fade, 7 stage end with fade, 8 Cadillac drive).
local ENDMODES = { [1] = true, [8] = true, [9] = true, [0x12] = true }   -- end of area (next area follows)
local STAGEEND = { [6] = true, [0x10] = true, [0x13] = true }            -- end of the episode
local ENDMODES2 = { [1] = true, [4] = true, [6] = true }
local STAGEEND2 = { [3] = true, [7] = true }
local function mode_sets()
  if r8(G.stage) == 2 then return ENDMODES2, STAGEEND2, 2 end
  return ENDMODES, STAGEEND, 4
end
local J = nil
local BOOT = { coin = 600, start = 700, pick = 900 }

-- Cursor moves for the select screen. Player p starts on position p-1 (Jack, Hannah, Mustapha, Mess). A
-- cursor moves one step right or left and jumps over the cursors of the other players (seen in MAME), so
-- the moves needed for several players are searched (breadth first) instead of counted.
local function plan_moves(targets)
  local n = #targets
  local function key(pos) local k = 0; for i = 1, n do k = k * 4 + pos[i] end return k end
  local start = {}
  for i = 1, n do start[i] = i - 1 end
  local queue, seen = { { pos = start, path = {} } }, { [key(start)] = true }
  local qi = 1
  while qi <= #queue do
    local cur_s = queue[qi]; qi = qi + 1
    local done = true
    for i = 1, n do if cur_s.pos[i] ~= targets[i] then done = false end end
    if done then return cur_s.path end
    for i = 1, n do
      for _, dir in ipairs({ 1, -1 }) do
        local np = (cur_s.pos[i] + dir) % 4
        local guard = 0
        while guard < 4 do
          local taken = false
          for j = 1, n do if j ~= i and cur_s.pos[j] == np then taken = true end end
          if not taken then break end
          np = (np + dir) % 4; guard = guard + 1
        end
        if guard < 4 then
          local pos = { table.unpack(cur_s.pos) }
          pos[i] = np
          if not seen[key(pos)] then
            seen[key(pos)] = true
            local path = { table.unpack(cur_s.path) }
            path[#path + 1] = { p = i, dir = dir }
            queue[#queue + 1] = { pos = pos, path = path }
          end
        end
      end
    end
  end
  return {}
end

local function parse_jump(it)
  local a = it.args
  local ep, area = num(a[1]), num(a[2])
  if ep == nil or area == nil or ep < 1 or ep > 8 then
    io.stderr:write("probe.lua: jump EP(1-8) AREA [CAMX]: " .. it.line .. "\n"); return nil
  end
  local x = (a[3] and not a[3]:find("=")) and num(a[3]) or 0
  local o = kv(a, 1)
  local chars = {}
  for name in (o.char or o.chars or "jack"):gmatch("[^,]+") do
    local n = tonumber(name)
    if n then -- an id, convert to cursor position
      local pos = ({ [0] = 0, [1] = 2, [2] = 1, [3] = 3 })[n]
      chars[#chars + 1] = pos or 0
    else chars[#chars + 1] = CHAR_POS[name:lower()] or 0 end
  end
  local seen = {}
  for _, c in ipairs(chars) do
    if seen[c] then io.stderr:write("probe.lua: jump: the players need different characters\n"); return nil end
    seen[c] = true
  end
  return { ep = ep, area = area, x = x, chars = chars, moves = plan_moves(chars), at_start = (o.at == "start"), phase = "boot",
    maxf = tonumber(env("PR_JUMP_MAXF", "9000")) }
end

-- Boot input: player p puts in a coin 40 frames after player p-1, presses start 40 frames after, moves its
-- cursor on the select screen (J.moves) and presses attack.
local function boot_input(f)
  local out = new_in()
  local function seg(a, b, p, key)
    if f >= a and f <= b then
      if key == "C" then out.coin = out.coin | (1 << (p - 1))
      elseif key == "S" then out.start = out.start | (1 << (p - 1))
      else out.p[p] = out.p[p] | BITS[key] end
    end
  end
  for p = 1, #J.chars do
    seg(BOOT.coin + (p - 1) * 40, BOOT.coin + (p - 1) * 40 + 7, p, "C")
    seg(BOOT.start + (p - 1) * 40, BOOT.start + (p - 1) * 40 + 5, p, "S")
  end
  for k, m in ipairs(J.moves) do seg(830 + (k - 1) * 20, 833 + (k - 1) * 20, m.p, m.dir > 0 and "R" or "L") end
  local a0 = math.max(BOOT.pick, 830 + #J.moves * 20 + 20)
  for p = 1, #J.chars do seg(a0 + (p - 1) * 20, a0 + (p - 1) * 20 + 5, p, "A") end
  return out
end

local function controllable()
  return r8(G.ingame) ~= 0 and r8(G.mode) == 0x0C and r8(G.scripted) == 0 and r8(PLAYER[1]) ~= 0
    and r32(PLAYER[1] + 4) == 0x02000000 and r8(PLAYER[1] + 0x11A) == 0
end

-- Move the spawn script past records that were "already passed" (no enemies, props or waves appear for
-- them). thr1 for list 1 (props, 14 byte records), thr2 for list 2 (enemies and waves, 16 byte records).
local function skip_spawns(thr1, thr2)
  if r32(G.spawn + 4) == 0 then return 0 end     -- the spawn script is not initialised yet
  local n = 0
  local p = r32(G.spawn + 8)
  while true do
    local t = r16(p)
    if t > thr1 or t >= 0x9000 then break end
    p = p + 14; n = n + 1
  end
  w32(G.spawn + 8, p)
  p = r32(G.spawn + 12)
  while true do
    local t = r16(p)
    if t > thr2 or t >= 0x9000 then break end
    p = p + 16; n = n + 1
  end
  w32(G.spawn + 12, p)
  return n
end

local function put_players_at(x)
  local k = 0
  for i = 1, 3 do
    if r8(PLAYER[i]) ~= 0 then w16(PLAYER[i] + 8, x + 0x60 + 0x20 * k); k = k + 1 end
  end
end

-- Move the camera one map column (0x20 px) per frame towards X. The scroll maps stream in one column per
-- frame, from the column the camera passed last ($782), so the next step is always "streamed X +- 0x20":
-- a bigger step leaves holes in the background. The players are kept just inside the left of the screen.
local function step_cam(target)
  local sx = r16(G.sx2)
  local nxt
  if target > sx + 0x20 then nxt = sx + 0x20 elseif target < sx - 0x20 then nxt = sx - 0x20 else nxt = target end
  w16(G.camx, nxt); w16(G.camx + 2, 0)
  put_players_at(nxt)
  J.cx = nxt
  return nxt == target
end

-- Diagonal camera modes (2 and 3) follow the players down or up a slope and end when the camera reaches
-- the event's Y. The players are teleported, so bring the camera to that Y one row (0x20) per frame and
-- end the mode ourselves.
local function fix_diag()
  local m = r8(G.cammode)
  if r8(G.stage) ~= 2 and (m == 2 or m == 3) then
    local ty, sy = r16(G.evy), r16(G.sy2)
    local nxt = ty
    if ty > sy + 0x20 then nxt = sy + 0x20 elseif ty < sy - 0x20 then nxt = sy - 0x20 end
    w16(G.camy, nxt); w16(G.camy + 2, 0)
    J.diag_fixed = true
    if nxt == ty then w8(G.cammode, 0) end
  end
end

-- After a diagonal camera mode was cut short the players can stand far above the new camera (the screen
-- row of a player is camY + 0xEF - (floor + z)): put them back on screen, near the bottom.
local function fix_depth()
  for i = 1, 3 do
    if r8(PLAYER[i]) ~= 0 then
      local scr_y = (r16(G.camy) + 0xEF - (r16(PLAYER[i] + 0x54) + r16(PLAYER[i] + 0x10))) & 0xFFFF
      if scr_y >= 0x8000 or scr_y < 100 then
        local z = (r16(PLAYER[i] + 0x10) - (190 - s16(scr_y))) & 0xFFFF
        w16(PLAYER[i] + 0x10, z)
      end
    end
  end
end

local jump_failed = nil
local function jump_fail(msg)
  log("J", "FAILED %s", msg)
  jump_failed = msg
  J = nil
end

local function jump_done()
  if J.diag_fixed then fix_depth() end
  local who = {}
  for i = 1, 3 do if r8(PLAYER[i]) ~= 0 then who[#who + 1] = string.format("P%d=char%d", i, r8(PLAYER[i] + 3)) end end
  log("J", "done ep=%d area=%d camx=%d (frame %d) players: %s", J.ep, r8(G.area), cam_x(), F, table.concat(who, " "))
  if J.god_saved then
    for i = 1, 3 do
      god[i] = J.god_saved[i]
      if not god[i] and r8(PLAYER[i]) == 2 then w8(PLAYER[i], 1) end
    end
  end
  J = nil
  T = 0
end

local function jump_step()
  if F > J.maxf then jump_fail("no result after " .. J.maxf .. " frames, phase " .. J.phase) return end
  local st = J.phase
  if (st == "walk" or st == "shift") and F % 60 == 0 then
    log("J", "%s: camx=%X streamed=%X cammode=%X go=%d next=%X", st, cam_x(), r16(G.sx2), r8(G.cammode), r8(G.go), r16(r32(G.nextev)))
  end
  if st == "boot" then
    cur = boot_input(F)
    if r8(G.ingame) ~= 0 and r8(G.mode) == 0x0C then
      if J.at_start and J.ep == 1 and J.area == 0 and J.x == 0 then cur = new_in(); jump_done(); return end
      cur = new_in(); J.phase = "ctrl1"; log("J", "in the level (phase ctrl1)")
    end
  elseif st == "ctrl1" then
    if controllable() then
      J.god_saved = { god[1], god[2], god[3] }
      god[1], god[2], god[3] = true, true, true       -- the players are safe while the jump moves them
      if J.ep == 1 then J.phase = "areas"
      else
        w8(G.stage, J.ep - 1); w8(G.sub, 0); w8(G.mode, 8)       -- story panel of the episode, then its start
        J.phase = "panel"; log("J", "poked stage=%d mode=8 (phase panel)", J.ep - 1)
      end
    end
  elseif st == "panel" then
    if r8(G.mode) == 8 then J.panel_seen = true end
    if J.panel_seen and r8(G.mode) == 0x0C then
      if J.at_start and J.area == 0 and J.x == 0 then jump_done(); return end
      J.phase = "ctrl2"
    end
  elseif st == "ctrl2" then
    if controllable() then J.phase = "areas" end
  elseif st == "areas" then
    local area = r8(G.area)
    if r8(G.stage) ~= J.ep - 1 then jump_fail("wrong episode " .. (r8(G.stage) + 1)) return end
    if area < J.area then
      -- fire the end-of-area event of this area
      local evs = camera_events()
      local endm = mode_sets()
      local endev
      for _, e in ipairs(evs) do if e.area == area and endm[e.mode] then endev = e; break end end
      if not endev then jump_fail("area " .. area .. " has no end-of-area event (last area?)") return end
      skip_spawns(0x8FFF, 0x8FFF)
      J.from_area = area; J.fired = F
      log("J", "end of area %d: event %d at x=%X mode %X", area, endev.idx, endev.x, endev.mode)
      local sub_area = (r8(G.stage) == 2 and endev.mode == 4) or (r8(G.stage) ~= 2 and endev.mode == 8)
      if sub_area then
        -- sub-area switch: the map is not reloaded, so the camera walks there and loads the events on the way
        -- (slopes and descents set the camera height); the end event itself is loaded when it is reached
        J.cx = cam_x(); J.target = endev.x; J.endaddr = endev.addr; J.phase = "walk"
      else
        w32(G.nextev, endev.addr); w8(G.evidx, endev.idx)
        w16(G.camx, endev.x); w16(G.camx + 2, 0)
        if endev.byy ~= 0 or r8(G.byy) ~= 0 then w16(G.camy, endev.y); w16(G.camy + 2, 0) end
        put_players_at(endev.x)
        J.phase = "leave"
      end
    else
      J.phase = "shift"; J.cx = nil; J.settle = 0
    end
  elseif st == "walk" then
    local _, _, lockmode = mode_sets()
    local nx = r32(G.nextev)
    local pending = (nx ~= J.endaddr) and (r16(nx) <= J.cx) and r16(nx) ~= 0xFFFF
    if pending or r8(G.cammode) == lockmode then w8(G.go, 1) end    -- load events on the way, release locks
    fix_diag()
    if step_cam(J.target) and not pending then J.phase = "leave"; J.fired = F end
  elseif st == "leave" then
    local m = r8(G.cammode)
    local endm = mode_sets()
    if r8(G.area) == J.from_area and endm[m] then w8(G.go, 1) end
    if r8(G.area) ~= J.from_area and controllable() then J.phase = "areas"
    elseif r8(G.area) == J.from_area and F - J.fired > 900 then jump_fail("area " .. J.from_area .. " did not end") return end
  elseif st == "shift" then
    local area = r8(G.area)
    if area ~= J.area then jump_fail("area " .. area .. " instead of " .. J.area) return end
    if J.x == 0 or r8(G.vertical) ~= 0 then
      if J.x ~= 0 then log("J", "camera X ignored: this area scrolls vertically") end
      jump_done(); return
    end
    if J.cx == nil then
      local endx = 0x7FFF
      local endm, stagee = mode_sets()
      for _, e in ipairs(camera_events()) do if e.area == area and (endm[e.mode] or stagee[e.mode]) then endx = e.x; break end end
      J.sx = J.x
      if J.sx > endx - 0x10 then J.sx = endx - 0x10; log("J", "camera X cut to %X (end of the area)", J.sx) end
      local skipped = skip_spawns(math.max(J.sx - 0x400, 0), math.max(J.sx - 1, 0))
      log("J", "spawn script moved past %d records (props kept from %X)", skipped, math.max(J.sx - 0x400, 0))
      J.cx = cam_x()
    end
    fix_diag()
    local reached = step_cam(J.sx)
    local nx = r32(G.nextev)
    local pending = (r16(nx) <= J.cx) and r16(nx) ~= 0xFFFF
    local _, _, lockmode = mode_sets()
    if pending or r8(G.cammode) == lockmode then w8(G.go, 1) end   -- load events on the way, release locks
    if reached and not pending and r8(G.cammode) ~= lockmode then J.settle = J.settle + 1 else J.settle = 0 end
    if J.settle > 12 then jump_done(); return end
  end
end

-- ---------------------------------------------------------------------------------------------
-- Scenarios: `snapshot` and `trace N` (head of this file; recorded by re/harness/probe/probe_record.sh)
-- ---------------------------------------------------------------------------------------------
-- The trace writer is the one of re/harness/record.lua, loaded as a library (DINO_LIB_ONLY).
local TW = nil
local function trace_writer()
  if TW then return TW end
  local path = env("PR_RECORD_LUA", "re/harness/record.lua")
  DINO_LIB_ONLY = true
  local chunk, err = loadfile(path, "t", _ENV)
  if not chunk then DINO_LIB_ONLY = nil; error("cannot load the trace writer " .. path .. ": " .. tostring(err)) end
  local ok, lib = pcall(chunk)
  DINO_LIB_ONLY = nil
  if not ok then error(lib) end
  TW = lib
  return TW
end

-- A game task (slots 0..14 of the task table at 0xFF0052) is still running or waiting: the game frame did not
-- finish (slowdown). Same rule as record.lua.
local function lagging()
  for t = 0, 14 do
    local s = r8(0xFF0052 + t * 16)
    if s == 4 or s == 8 or s == 0xC then return true end
  end
  return false
end

local STATE_CMDS = { poke = true, hold = true, release = true, spawn = true, hp = true, lives = true, score = true,
  god = true, time = true, killall = true, rank = true, level = true, aggr = true, objset = true }

-- `[FRAME] snapshot [control|control+N] [keep]`: arms the snapshot. It is taken by the vblank IRQ tap, at the
-- first game frame boundary (not lagging) after the line ran, and after `control` / `control+N` is true.
arm_snapshot = function(it)
  sn = { phase = "armed", k = 0, lag = 0, keep = false, when = nil, after = 0 }
  local desc = {}
  for _, a in ipairs(it.args) do
    local al = a:lower()
    local n = al:match("^control%+(%d+)$")
    if al == "keep" then sn.keep = true; desc[#desc + 1] = "keep"
    elseif al == "control" then sn.when = "control"; desc[#desc + 1] = "control"
    elseif n then sn.when = "control"; sn.after = tonumber(n); desc[#desc + 1] = al
    else log("ERR", "snapshot: unknown argument %s", a) end
  end
  if #items > 0 then log("ERR", "snapshot: %d setup lines have a later frame than the snapshot and do not run", #items) end
  log("X", "snapshot armed %s (trace %d frames)", table.concat(desc, " "), trace_n)
end

local function trace_paths()
  local dir = env("PR_TRACE_DIR", nil) or (OUTDIR .. "/" .. seg.rel:gsub("/$", ""))
  local name = env("PR_TRACE_NAME", nil) or seg.name
  return { state = dir .. "/" .. name .. ".state", trace = dir .. "/" .. name .. ".mame.trc",
    input = dir .. "/" .. name .. ".input.txt" }
end

-- Directives after the snapshot line, frames counted in game frames from the snapshot.
local function run_post(k)
  while #post_items > 0 and post_items[1].frame <= k do
    local it = table.remove(post_items, 1)
    it.frame = T or 0                       -- `watch` counts from the video frame it starts at
    if STATE_CMDS[it.cmd] then log("NOTE", "%s after the snapshot: the port does not run it", it.cmd) end
    exec(it)
  end
end

local function take_snapshot()
  local W = trace_writer()
  if not sn.keep then
    -- the setup helpers end here, like `god off` and `release` (holds are just dropped)
    local n = #holds
    for i = 1, 3 do
      if god[i] and r8(PLAYER[i]) == 2 then w8(PLAYER[i], 1) end
      if god[i] then n = n + 1 end
    end
    holds, god = {}, {}
    if n > 0 then log("X", "snapshot: %d holds / god modes ended", n) end
  end
  sn.paths = trace_paths()
  local snap = W.snapshot()
  local fh = assert(io.open(sn.paths.state, "wb"))
  W.write_header(fh, string.format(',"kind":"state","vblank":%d,"probe_segment":%s', irq_n, W.json_str(seg.name)))
  W.write_record(fh, -1, 0, irq_n, 0, nil, snap)
  fh:close()
  local inf = assert(io.open(sn.paths.input, "w"))
  inf:write("# probe segment ", seg.name, ": the key lines after the snapshot (frame 0 = first game frame after it)\n")
  for _, l in ipairs(postlines) do inf:write(l, "\n") end
  inf:close()
  sn.fh = assert(io.open(sn.paths.trace, "wb"))
  W.write_header(sn.fh, string.format(',"kind":"trace","clock":"game","sync_vblank":%d,"frames":%d,"input":%s,"probe_segment":%s',
    irq_n, trace_n, W.json_str(sn.paths.input), W.json_str(seg.name)))
  W.write_record(sn.fh, -1, 0, irq_n, 0, nil, snap)
  sn.prev, sn.phase, sn.k, sn.lag, sn.lagged = snap, "trace", 0, 0, 0
  log("SNAPSHOT", "state written (video frame %d, IRQ %d, game frame counter %d); tracing %d game frames", F, irq_n, r16(G.frame), trace_n)
  run_post(0)
  apply_holds()
  cur = script_at(postkeys, 0)
end

local control_gf = nil
local tap_keys = false      -- key lines before the snapshot (no jump) are set at the IRQ tap, by IRQ count, like record.lua
local function irq_tap()
  if tap_keys and (sn.phase == "" or sn.phase == "armed") then cur = script_at(keysegs, irq_n) end
  if sn.phase == "armed" then
    if lagging() then return end
    if sn.when == "control" then
      if control_gf == nil and controllable() then control_gf = r16(G.frame) end
      if control_gf == nil or ((r16(G.frame) - control_gf) & 0xFFFF) < sn.after then return end
    end
    take_snapshot()
  elseif sn.phase == "trace" then
    if lagging() then sn.lag = sn.lag + 1; return end
    local snap = TW.snapshot()
    TW.write_record(sn.fh, sn.k, (sn.lag > 0) and 1 or 0, irq_n, sn.lag, sn.prev, snap)
    sn.prev, sn.lagged, sn.lag, sn.k = snap, sn.lagged + sn.lag, 0, sn.k + 1
    if sn.k >= trace_n then
      sn.fh:close(); sn.fh = nil
      sn.phase, sn.msg = "done", string.format("trace %d frames (lag vblanks %d)", sn.k, sn.lagged)
      return
    end
    run_post(sn.k)
    apply_holds()
    cur = script_at(postkeys, sn.k)
  end
end
on_irq = function()
  local ok, err = pcall(irq_tap)
  if not ok and not sn.err then sn.err = tostring(err) end
end

-- ---------------------------------------------------------------------------------------------
-- Main loop (end of every video frame) and segments
-- ---------------------------------------------------------------------------------------------
local lastsig = ""
local function log_state()
  local sig = string.format("mode=%02x sub=%02x stage=%d area=%d ingame=%02x", r8(G.mode), r8(G.sub), r8(G.stage), r8(G.area), r8(G.ingame))
  if sig ~= lastsig then log("S", "%s", sig); lastsig = sig end
end

local segments = split_segments(read_file(env("PR_INPUT", nil)) .. "\n" .. read_file(env("PR_POKES", nil)) .. "\n" .. env("PR_LINES", ""))
local function has_snapshot(sg)
  for line in sg.text:gmatch("[^\n]+") do if line:match("^%s*%d*%s*snapshot%f[%W]") then return true end end
  return false
end
do   -- PR_ONLY=a,b (names) or * (every segment that has a snapshot line): run only those segments
  local only = env("PR_ONLY", "")
  if only ~= "" then
    local want, all, kept = {}, false, {}
    for n in only:gmatch("[^,]+") do if n == "*" then all = true else want[n] = true end end
    for _, sg in ipairs(segments) do
      if not sg.single and (want[sg.name] or (all and has_snapshot(sg))) then kept[#kept + 1] = sg end
    end
    segments = kept
  end
  for _, sg in ipairs(segments) do if has_snapshot(sg) then want_irq_tap = true end end
end
local seg_i = 0
local loading = false
local prev_time = 0
local J_end_F = nil
local summary = nil
local snap_limit = tonumber(env("PR_SNAP_MAXF", "12000"))   -- a snapshot segment fails when the snapshot is later
local seg_err = nil          -- setup error of a snapshot segment (bad jump line): the segment ends as FAILED

local function machine_time() return manager.machine.time:as_double() end

local function begin_segment(i)
  seg_i = i
  seg = segments[i]
  seg.rel = seg.single and "" or (seg.name .. "/")
  local dir = OUTDIR .. "/" .. seg.rel
  os.execute("mkdir -p '" .. dir .. "snap'")
  logf = io.open(dir .. "log.txt", "w")
  load_segment_text(seg)
  F, T, J_end_F, snapcount, lastsig = 0, nil, nil, 0, ""
  holds, god, watches = {}, {}, {}
  sn = { phase = "", k = 0, lag = 0 }
  control_gf, seg_err = nil, nil
  tap_keys = snap_cmd ~= nil and jump_item == nil
  cur = new_in()
  J = nil
  if jump_item then
    J = parse_jump(jump_item)
    if J == nil then
      log("ERR", "bad jump line: %s", jump_item.line)
      maxT = 0; T = 0
      if snap_cmd then seg_err = "bad jump line" end
    end
  end
  if not J then T = 0 end
  log("SEGMENT", "%s frames=%d", seg.name, maxT)
end

local function end_segment(msg)
  log("END", "%s", msg)
  if sn.fh then sn.fh:close(); sn.fh = nil end
  if sn.paths and not msg:match("^trace %d+ frames") then   -- never leave a partial scenario behind
    for _, path in pairs(sn.paths) do os.remove(path) end
  end
  logf:flush(); logf:close(); logf = nil
  if summary then summary:write(string.format("%s\t%s\n", seg.name, msg)); summary:flush() end
  if seg_i < #segments then
    manager.machine:load("power")          -- back to the power-on state for the next segment
    loading = true
  else
    manager.machine:exit()
  end
end

local function frame_body()
  if not installed then
    install()
    if #segments == 0 then
      io.stderr:write("probe.lua: no segment to run (PR_ONLY=" .. env("PR_ONLY", "") .. ")\n")
      manager.machine:exit()
      return
    end
    if #segments > 1 then
      manager.machine:save("power")        -- state at the end of the first frame
    end
    if #segments > 1 or env("PR_TRACE_DIR", nil) then summary = io.open(OUTDIR .. "/segments.txt", "w") end
    begin_segment(1)
  end
  local tm = machine_time()
  if loading then
    if tm < prev_time then                 -- the power-on state is back: this is frame 2 of a fresh boot
      loading = false
      prev_time = tm
      begin_segment(seg_i + 1)
      F = 1
    else
      prev_time = tm
      return
    end
  end
  prev_time = tm
  F = F + 1
  if J then
    T = nil
    jump_step()
    if jump_failed then
      local m = jump_failed
      jump_failed = nil
      end_segment("FAILED jump: " .. m)
      return
    end
    if J then log_state(); logf:flush(); return end
    J_end_F = J_end_F or F
  end
  -- (not before frame 3: the load of the power-on state is noticed by the machine time going back)
  if seg_err and F > 2 then end_segment("FAILED " .. seg_err); return end
  T = F - (J_end_F or 0)
  log_state()
  -- Once the snapshot is armed no more setup lines run; while the trace runs the vblank IRQ tap (irq_tap) sets
  -- the input and runs the directives at game frame boundaries.
  local tracing = sn.phase == "trace" or sn.phase == "done"
  if not tracing then
    -- directives of this frame
    while sn.phase == "" and #items > 0 and items[1].frame <= T do
      local it = table.remove(items, 1)
      exec(it)
    end
    apply_holds()
  end
  for _, w in ipairs(watches) do
    if T >= w.start and (T - w.start) % w.interval == 0 then sample(w) end
  end
  if snaps[T] or (every and T >= every[2] and T <= every[3] and (T - every[2]) % every[1] == 0) then take_snap() end
  if not tracing and not tap_keys then cur = script_at(keysegs, T) end
  logf:flush()
  if sn.err then end_segment("FAILED snapshot: " .. sn.err); return end
  if sn.phase == "done" then end_segment(sn.msg); return end
  if snap_cmd then
    if sn.phase ~= "trace" and T >= snap_limit then
      end_segment(string.format("FAILED snapshot not taken in %d frames", T))
    end
  elseif T >= maxT then end_segment(string.format("frames=%d", T)) end
end

-- A Lua error must not stop a long batch: report it, close the segment and go on with the next one.
emu.register_frame_done(function()
  local ok, err = pcall(frame_body)
  if not ok then
    io.stderr:write("probe.lua error: " .. tostring(err) .. "\n")
    if logf then pcall(end_segment, "LUA ERROR " .. tostring(err)) end
  end
end)
