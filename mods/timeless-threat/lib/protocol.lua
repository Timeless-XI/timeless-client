-- Reads the server's enmity feed (modules/combat_feed). Every line is one of:
--   EN1|C                                    forget everything
--   EN1|M|<mob index>|h=<hp%>,t=<index it attacks>,me=<1 = your target>
--   EN1|L|<mob index>|<index>:<CE>:<VE>;...  its enmity list, highest first
--   EN1|X|<mob index>                        stop showing it
-- Indexes are zone target indexes (game.entity takes them). No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "EN1"

function P.new_model()
  return { mobs = {}, received = 0 }
end

-- The feed line in a chat message, or nil.
function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

local function pairs_of(body)
  local out = {}
  for k, v in (body or ""):gmatch("([%w_]+)=([^,]*)") do out[k] = v end
  return out
end

local function mob(model, index)
  local m = model.mobs[index]
  if not m then
    m = { index = index, hpp = 100, holder = 0, mine = false, rows = {} }
    model.mobs[index] = m
  end
  return m
end

-- Applies one line to the model. Returns true when the line was ours.
function P.feed(model, line, now)
  if type(line) ~= "string" then return false end
  local kind, rest = line:match("^EN1|(%u)|?(.*)$")
  if not kind then return false end
  model.received = now or model.received
  if kind == "C" then
    model.mobs = {}
    return true
  end
  local index, body = rest:match("^(%d+)|?(.*)$")
  index = tonumber(index)
  if not index then return true end
  if kind == "X" then
    model.mobs[index] = nil
  elseif kind == "M" then
    local m, f = mob(model, index), pairs_of(body)
    m.hpp = tonumber(f.h) or m.hpp
    m.holder = tonumber(f.t) or 0
    m.mine = f.me == "1"
    m.at = now
  elseif kind == "L" then
    local m = mob(model, index)
    m.rows = {}
    for a, ce, ve in body:gmatch("(%d+):(%d+):(%d+)") do
      m.rows[#m.rows + 1] = { index = tonumber(a), ce = tonumber(ce), ve = tonumber(ve) }
    end
  end
  return true
end

-- The monsters to show: your target first, then by index.
function P.list(model)
  local out = {}
  for _, m in pairs(model.mobs) do out[#out + 1] = m end
  table.sort(out, function(a, b)
    if a.mine ~= b.mine then return a.mine end
    return a.index < b.index
  end)
  return out
end

-- A row's total enmity (CE + VE).
function P.total(row) return (row.ce or 0) + (row.ve or 0) end

-- The row of whoever the monster is attacking (it has the hate), else the highest.
function P.holder_row(m)
  for _, r in ipairs(m.rows) do
    if r.index == m.holder then return r end
  end
  return m.rows[1]
end

-- A row's share of the holder's enmity, in percent (100 = the holder; above 100 means it is about to turn).
function P.percent(m, row)
  local h = P.holder_row(m)
  local top = h and P.total(h) or 0
  if top <= 0 then return 0 end
  return math.floor(P.total(row) * 100 / top + 0.5)
end

return P
