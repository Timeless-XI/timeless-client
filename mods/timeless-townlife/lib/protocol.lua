-- Reads the server's Town Life feed (modules/townlife):
--   TL1|C|<zone id>|<zone name>              a new zone: forget everything
--   TL1|B|<target index>|<text>              a speech bubble over that townsperson
--   TL1|L|<lamp>|<x>,<y>,<z>|<1 lit, 0 out>  a street lamp
--   TL1|S|<n>|<kind>|<x>,<y>,<z>|<range>     scenery: smoke, hammer, crowd, gulls, music, cards, docks
--   TL1|O|<n>|<1 on, 0 off>                  scenery that comes and goes (the tavern's music while the bard plays)
--   TL1|A|<target index>|<lantern, torch, ->  what a townsperson carries that gives light (- nothing)
--   TL1|W|<target index>|<0-100>|<craft>|<sound>  an apprentice at the furnace, how far along (TL1|W|<index>|- : stopped)
--   TL1|T|<section>|<text>                   a Town window row
--   TL1|E                                    the end of a snapshot
-- No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "TL1"

function P.new_model()
  return { zone = 0, zone_name = "", lamps = {}, scenery = {}, rows = {}, pending = nil, bubbles = {}, carry = {}, all_lamps = {}, work = {}, version = 0 }
end

function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

local function point(s)
  local x, y, z = (s or ""):match("^(%-?[%d%.]+),(%-?[%d%.]+),(%-?[%d%.]+)$")
  if not x then return nil end
  return { x = tonumber(x), y = tonumber(y), z = tonumber(z) }
end

-- Applies one line. Returns what changed: "zone", "bubble", "lamp", "scenery", "rows" (a snapshot ended), true for
-- other lines of ours, false for anything else.
function P.feed(m, line, now)
  if type(line) ~= "string" then return false end
  local kind, rest = line:match("^TL1|(%u)|?(.*)$")
  if not kind then return false end
  if kind == "C" then
    local id, name = rest:match("^(%d+)|?(.*)$")
    m.zone, m.zone_name = tonumber(id) or 0, name or ""
    m.lamps, m.scenery, m.bubbles, m.carry, m.all_lamps, m.work = {}, {}, {}, {}, {}, {}
    m.pending = {}
    return "zone"
  elseif kind == "B" then
    local index, text = rest:match("^(%d+)|(.*)$")
    if not index then return true end
    m.bubbles[tonumber(index)] = { text = text, at = now }
    return "bubble"
  elseif kind == "L" then
    local i, p, lit = rest:match("^(%d+)|([^|]+)|([01])$")
    local pos = point(p)
    if not i or not pos then return true end
    local n = tonumber(i)
    -- Lit just now (not in a snapshot): when, for the fade in
    m.lit_at = m.lit_at or {}
    if lit == "1" and not m.lamps[n] and m.pending == nil then m.lit_at[n] = now end
    if lit ~= "1" then m.lit_at[n] = nil end
    if lit == "1" then m.lamps[n] = pos else m.lamps[n] = nil end
    m.all_lamps = m.all_lamps or {}
    m.all_lamps[tonumber(i)] = pos
    return "lamp"
  elseif kind == "A" then
    local index, what = rest:match("^(%d+)|([%a%-]+)$")
    if not index then return true end
    m.carry[tonumber(index)] = what ~= "-" and what or nil
    return "carry"
  elseif kind == "S" then
    local i, what, p, range = rest:match("^(%d+)|(%a+)|([^|]+)|(%d+)$")
    local pos = point(p)
    if not i or not pos then return true end
    m.scenery[tonumber(i)] = { kind = what, pos = pos, range = tonumber(range) or 0 }
    return "scenery"
  elseif kind == "O" then
    local i, on = rest:match("^(%d+)|([01])$")
    local s = i and m.scenery[tonumber(i)]
    if s then s.off = on == "0" end
    return "scenery"
  elseif kind == "W" then
    local index, pct, craft, sound = rest:match("^(%d+)|(%d+)|(%a+)|(%a+)$")
    m.work = m.work or {}
    if index then
      m.work[tonumber(index)] = { pct = math.min(100, tonumber(pct)), craft = craft, sound = sound }
    else
      local gone = rest:match("^(%d+)|%-$")
      if gone then m.work[tonumber(gone)] = nil end
    end
    return "work"
  elseif kind == "T" then
    local section, text = rest:match("^(%a+)|(.*)$")
    if section then
      m.pending = m.pending or {}
      m.pending[#m.pending + 1] = { section = section, text = text }
    end
    return true
  elseif kind == "E" then
    m.rows = m.pending or {}
    m.pending = nil
    m.version = m.version + 1
    return "rows"
  end
  return true
end

return P
