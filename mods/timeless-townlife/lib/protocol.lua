-- Reads the server's Town Life feed (modules/townlife):
--   TL1|C|<zone id>|<zone name>              a new zone: forget everything
--   TL1|B|<target index>|<text>              a speech bubble over that townsperson
--   TL1|L|<lamp>|<x>,<y>,<z>|<1 lit, 0 out>  a street lamp
--   TL1|S|<n>|<kind>|<x>,<y>,<z>|<range>     scenery: smoke, hammer, crowd, gulls
--   TL1|T|<section>|<text>                   a Town window row
--   TL1|E                                    the end of a snapshot
-- No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "TL1"

function P.new_model()
  return { zone = 0, zone_name = "", lamps = {}, scenery = {}, rows = {}, pending = nil, bubbles = {}, version = 0 }
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
    m.lamps, m.scenery, m.bubbles = {}, {}, {}
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
    if lit == "1" then m.lamps[tonumber(i)] = pos else m.lamps[tonumber(i)] = nil end
    return "lamp"
  elseif kind == "S" then
    local i, what, p, range = rest:match("^(%d+)|(%a+)|([^|]+)|(%d+)$")
    local pos = point(p)
    if not i or not pos then return true end
    m.scenery[tonumber(i)] = { kind = what, pos = pos, range = tonumber(range) or 0 }
    return "scenery"
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
