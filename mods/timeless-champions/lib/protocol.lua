-- Reads the server's Champion lines (modules/champions/lua/champions_feed.lua):
--   CH1|C                          forget everything
--   CH1|M|<mob index>|r=<c|e|l>,a=<affix keys, dot separated>[,n=<whole name>][,e=<element>][,f=1][,rg=0|1][,ud=0|1][,rn=0|1]
--                                  [,sh=0|1][,pk=<yalms>][,rl=<index.index>]
--   CH1|X|<mob index>              stop showing it
-- No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "CH1"
P.RANKS = { c = "champion", e = "elite", l = "legendary" }

function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

local function flag(v)
  if v == nil then return nil end
  return v == "1"
end

local function list(v, number)
  local out = {}
  for w in (v or ""):gmatch("[^%.]+") do
    if number then
      local n = tonumber(w)
      if n then out[#out + 1] = math.floor(n) end
    else
      out[#out + 1] = w
    end
  end
  return out
end

-- A table for one line, or nil when it isn't a usable one:
--   {op = "C"} | {op = "X", index} | {op = "M", index, rank, affixes = {keys}, has = {key = true}, name, element, fighting,
--    raged, undying, regen, shadows, pack, rallied = {indexes}}
-- raged, undying, regen and shadows are nil when the Champion lacks that affix.
function P.parse(line)
  if type(line) ~= "string" then return nil end
  if line == "CH1|C" then return { op = "C" } end
  local x = line:match("^CH1|X|(%d+)$")
  if x then return { op = "X", index = tonumber(x) } end
  local index, body = line:match("^CH1|M|(%d+)|(.*)$")
  if not index then return nil end
  local f = {}
  for k, v in body:gmatch("([%w_]+)=([^,]*)") do f[k] = v end
  local rank = P.RANKS[f.r or ""]
  if not rank then return nil end
  local c = {
    op = "M",
    index = tonumber(index),
    rank = rank,
    affixes = list(f.a),
    name = (f.n ~= nil and f.n ~= "") and f.n:sub(1, 60) or nil,
    element = (f.e ~= nil and f.e ~= "") and f.e or nil,
    fighting = f.f == "1",
    raged = flag(f.rg),
    undying = flag(f.ud),
    regen = flag(f.rn),
    shadows = flag(f.sh),
    pack = tonumber(f.pk),
    rallied = list(f.rl, true),
  }
  c.has = {}
  for _, k in ipairs(c.affixes) do c.has[k] = true end
  if c.pack then c.pack = math.max(1, math.min(c.pack, 40)) end
  return c
end

return P
