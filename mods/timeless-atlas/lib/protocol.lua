-- Reads the server's NM Atlas feed (modules/nm_atlas). A list:
--   NA1|H|<page>|<pages>|<total>   NA1|R|<id>|<zone>|<lv1>|<lv2>|<state>|<minutes>|<name>   NA1|E
-- A monster's detail:
--   NA1|D|<id>|<zone>|<lv1>|<lv2>|<model>|<hp>|<flags>|<element>|<jobs>|<name>
--   NA1|K|<ecosystem>|<family>|<species>|<detects>
--   NA1|W|el|fire=0,...  NA1|W|st|...  NA1|W|ph|slashing=-12,...  NA1|W|mg|all=50  NA1|W|im|stun,silence
--   NA1|S|<types>|<respawn s>|<window>|<lottery chance>|<cooldown s>|<placeholders>|<timer min-max s>
--        (types has "timed" for a former lottery NM the server spawns on a timer; the timer field is newer: may be absent)
--   NA1|P|<???>|<item ids>   NA1|X|<state>|<minutes>|<minutes since defeated>
--   NA1|G|<rate>|<item>   NA1|O|<rate>|<item>:<w>,...   NA1|T|<item>   NA1|Z
-- States: 0 unknown, 1 up, 2 a timer (minutes to go), 3 can pop now. No game or UI code here.

local P = {}

P.VERSION = "NA1"

function P.new_model()
  return { list = { page = 1, pages = 1, total = 0, rows = {}, at = 0 }, pending_list = nil,
    info = nil, pending_info = nil, version = 0 }
end

function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

local function split(s, sep)
  local out = {}
  for part in ((s or "") .. sep):gmatch("(.-)" .. sep:gsub("%p", "%%%0")) do out[#out + 1] = part end
  return out
end

local function kv(body)
  local out = {}
  for k, v in (body or ""):gmatch("([%w_]+)=(%-?%d+)") do out[k] = tonumber(v) end
  return out
end

local function ids(body)
  local out = {}
  for n in (body or ""):gmatch("%d+") do out[#out + 1] = tonumber(n) end
  return out
end

-- Applies a line. Returns "list" or "info" when one is complete, true for other lines of ours, false otherwise.
function P.feed(m, line, now)
  if type(line) ~= "string" then return false end
  local kind, rest = line:match("^NA1|(%u)|?(.*)$")
  if not kind then return false end
  local f = split(rest, "|")
  if kind == "H" then
    m.pending_list = { page = tonumber(f[1]) or 1, pages = tonumber(f[2]) or 1, total = tonumber(f[3]) or 0, rows = {} }
  elseif kind == "R" and m.pending_list then
    local r = { id = tonumber(f[1]), zone = tonumber(f[2]), lv = { tonumber(f[3]), tonumber(f[4]) },
      state = tonumber(f[5]) or 0, mins = tonumber(f[6]) or 0, name = table.concat(f, "|", 7) }
    m.pending_list.rows[#m.pending_list.rows + 1] = r
  elseif kind == "E" and m.pending_list then
    m.list = m.pending_list
    m.list.at = now or 0
    m.pending_list = nil
    m.version = m.version + 1
    return "list"
  elseif kind == "D" then
    m.pending_info = { id = tonumber(f[1]), zone = tonumber(f[2]), lv = { tonumber(f[3]), tonumber(f[4]) },
      model = tonumber(f[5]) or 0, hp = tonumber(f[6]) or 0, flags = f[7] or "", element = f[8] or "",
      jobs = f[9] or "", name = table.concat(f, "|", 10), weak = {}, pops = {}, drops = {}, steal = {} }
  elseif m.pending_info then
    local i = m.pending_info
    if kind == "K" then
      i.ecosystem, i.family, i.species = f[1], f[2], f[3]
      i.detects = f[4] ~= "" and split(f[4], ",") or {}
    elseif kind == "W" then
      if f[2] and f[1] == "im" then
        i.weak.im = f[2] ~= "" and split(f[2], ",") or {}
      else
        i.weak[f[1]] = kv(f[2])
      end
    elseif kind == "S" then
      i.spawn = { types = f[1] ~= "" and split(f[1], ",") or {}, respawn = tonumber(f[2]) or 0, window = f[3] ~= "" and f[3] or nil,
        chance = tonumber(f[4]) or 0, cooldown = tonumber(f[5]) or 0, ph = (f[6] or "") ~= "" and split(f[6], ",") or {} }
      local lo, hi = (f[7] or ""):match("^(%d+)%-(%d+)$")
      if lo then i.spawn.timer = { tonumber(lo), tonumber(hi) } end
      for _, t in ipairs(i.spawn.types) do if t == "timed" then i.spawn.timed = true end end
    elseif kind == "P" then
      i.pops[#i.pops + 1] = { npc = f[1], items = ids(f[2]) }
    elseif kind == "X" then
      i.state, i.mins, i.since = tonumber(f[1]) or 0, tonumber(f[2]) or 0, tonumber(f[3]) or -1
    elseif kind == "G" then
      i.drops[#i.drops + 1] = { rate = tonumber(f[1]) or 0, item = tonumber(f[2]) }
    elseif kind == "O" then
      local group = { rate = tonumber(f[1]) or 0, one = {} }
      for item, w in (f[2] or ""):gmatch("(%d+):(%d+)") do group.one[#group.one + 1] = { item = tonumber(item), w = tonumber(w) } end
      i.drops[#i.drops + 1] = group
    elseif kind == "T" then
      i.steal[#i.steal + 1] = tonumber(f[1])
    elseif kind == "Z" then
      i.at = now or 0
      m.info = i
      m.pending_info = nil
      m.version = m.version + 1
      return "info"
    end
  elseif kind == "Z" then
    return "info" -- an unknown id: nothing came
  end
  return true
end

-- Minutes left now, from minutes when it came and the seconds since
function P.left(mins, at, now)
  if not mins or mins <= 0 then return 0 end
  return math.max(0, mins - math.floor(((now or 0) - (at or 0)) / 60))
end

-- "3h 12m", "45m", "2d 4h"
function P.span(mins)
  mins = math.max(0, math.floor(mins or 0))
  if mins >= 2880 then return ("%dd %dh"):format(mins // 1440, (mins % 1440) // 60) end
  if mins >= 60 then return ("%dh %02dm"):format(mins // 60, mins % 60) end
  return ("%dm"):format(mins)
end

-- A drop's chance as text: 240 -> "24%", 5 -> "0.5%"
function P.percent(permille)
  if permille >= 1000 then return "100%" end
  if permille % 10 == 0 then return ("%d%%"):format(permille // 10) end
  return ("%.1f%%"):format(permille / 10)
end

return P
