-- Reads the server's reputation feed (modules/reputation):
--   RP1|B|n=<nation>,r=<rank>,p=<rank points>
--   RP1|A|<area>|f=<fame>,l=<level>,lo=<fame at this level>,hi=<fame at the next, 0 at the top>
--   RP1|E
-- No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "RP1"

function P.new_model()
  return { loaded = false, nation = nil, rank = 0, points = 0, areas = {}, pending = nil, version = 0 }
end

function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

local function fields(body)
  local out = {}
  for k, v in (body or ""):gmatch("(%a+)=(%-?%d+)") do out[k] = tonumber(v) end
  return out
end

-- Applies one line: "done" when a picture is complete, true for other lines of ours, false otherwise.
function P.feed(m, line)
  if type(line) ~= "string" then return false end
  local kind, rest = line:match("^RP1|(%u)|?(.*)$")
  if not kind then return false end
  if kind == "B" then
    local f = fields(rest)
    m.pending = { nation = f.n, rank = f.r or 0, points = f.p or 0, areas = {} }
  elseif kind == "A" then
    local area, body = rest:match("^(%d+)|(.*)$")
    if area and m.pending then
      local f = fields(body)
      m.pending.areas[tonumber(area)] = { fame = f.f or 0, level = f.l or 1, lo = f.lo or 0, hi = f.hi or 0 }
    end
  elseif kind == "E" and m.pending then
    m.nation, m.rank, m.points, m.areas = m.pending.nation, m.pending.rank, m.pending.points, m.pending.areas
    m.pending = nil
    m.loaded = true
    m.version = m.version + 1
    return "done"
  end
  return true
end

-- How far through the current level (0..1); 1 at the top
function P.progress(a)
  if not a then return 0 end
  if a.hi == nil or a.hi <= a.lo then return 1 end
  return math.max(0, math.min(1, (a.fame - a.lo) / (a.hi - a.lo)))
end

return P
