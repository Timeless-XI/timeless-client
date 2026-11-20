-- Reads the server's cast lines (modules/combat_feed, "cb on"):
--   CB1|<mob index>|k=m,i=<spell id>,ms=<cast time>,t=<target index>   a spell, and how long until it lands
--   CB1|<mob index>|k=s,i=<move id>,ms=<ready time>,t=<target index>    a move being readied
--   CB1|<mob index>|k=x                                                 its cast was interrupted
-- No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "CB1"
local KINDS = { m = "spell", s = "skill", x = "cut" }

function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

-- {index, kind = "spell" | "skill" | "cut", id, secs, target}, or nil when it isn't a usable line.
function P.parse(line)
  if type(line) ~= "string" then return nil end
  local index, body = line:match("^CB1|(%d+)|(.*)$")
  if not index then return nil end
  local f = {}
  for k, v in body:gmatch("([%w_]+)=([^,]*)") do f[k] = v end
  local kind = KINDS[f.k or ""]
  if not kind then return nil end
  local c = { index = tonumber(index), kind = kind }
  if kind ~= "cut" then
    c.id = tonumber(f.i)
    local ms = tonumber(f.ms)
    if not c.id or not ms or ms <= 0 then return nil end
    c.secs = math.min(ms, 60000) / 1000
    local t = tonumber(f.t)
    c.target = (t and t > 0) and t or nil
  end
  return c
end

return P
