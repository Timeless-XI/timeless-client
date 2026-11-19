-- Reads the server's telegraph lines (modules/combat_feed):
--   TG1|<mob index>|s=<shape>,r=<yalms>,a=<degrees>,t=<target index>,ms=<ready time>,n=<move name>
--     s: c a circle around the monster, t a circle around its target, f a cone ahead, b a cone behind
-- Sent the moment a monster starts readying an area move. No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "TG1"
P.SHAPES = { c = true, t = true, f = true, b = true }

function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

-- A telegraph from a line received at `now` (seconds), or nil when the line isn't a usable one.
function P.parse(line, now)
  if type(line) ~= "string" then return nil end
  local mob, body = line:match("^TG1|(%d+)|(.*)$")
  if not mob then return nil end
  local f = {}
  for k, v in body:gmatch("([%w_]+)=([^,]*)") do f[k] = v end
  local shape, radius, ms = f.s, tonumber(f.r), tonumber(f.ms)
  if not P.SHAPES[shape or ""] or not radius or radius <= 0 or not ms or ms <= 0 then return nil end
  return {
    mob = tonumber(mob),
    shape = shape,
    radius = math.min(radius, 60),
    angle = math.max(5, math.min(180, tonumber(f.a) or 45)),
    target = tonumber(f.t) or 0,
    name = f.n or "",
    start = now,
    ends = now + math.min(ms, 15000) / 1000,
  }
end

return P
