-- Reads the server's world map feed (modules/world_map):
--   WM1|B|v=<version>,n=<count>,o=<owners>   a snapshot starts; o: 19 digits, regions 0..18's owners
--                                            (0 San d'Oria, 1 Bastok, 2 Windurst, 3 beastmen, - unknown)
--   WM1|E|<zone>|r=<region>,k=<kind>,l=<level>,c=<code>,m=<minutes left>
--        k: C a roaming Champion (c 1 Champion, 2 Elite, 3 Legendary), W a warband (l its captain's level),
--           F a Fomor band, R an open rift (c 1 Time, 2 Ancient, 3 Hero's; m minutes until it closes),
--           G a Gilded (treasure) Champion (l its level)
--   WM1|D|v=<version>                        the snapshot is complete: it replaces the last one
--   WM1|H|n=<count>                          what is in your own zone starts over
--   WM1|P|k=<kind>,i=<index>,x=,y=,z=,l=,c=  one of them, and where it is
-- No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "WM1"

function P.new_model()
  return { version = 0, owners = {}, events = {}, here = {}, received = 0, pending = nil }
end

function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

local function fields(body)
  local out = {}
  for k, v in (body or ""):gmatch("([%w_]+)=([^,]*)") do out[k] = v end
  return out
end

-- An event's identity, to tell what is new between snapshots.
function P.key(e) return table.concat({ e.k, e.zone, e.c or 0, e.l or 0 }, ":") end

-- Applies one line. Returns "snapshot" when a snapshot was completed (with the list of events that weren't in the
-- last one), "here" when your zone's list changed, true for other lines of ours, false for anything else.
function P.feed(model, line, now)
  if type(line) ~= "string" then return false end
  local kind, rest = line:match("^WM1|(%u)|?(.*)$")
  if not kind then return false end
  model.received = now or model.received
  if kind == "B" then
    local f = fields(rest)
    local owners = {}
    local o = f.o or ""
    for i = 1, #o do owners[i - 1] = tonumber(o:sub(i, i)) end
    model.pending = { version = tonumber(f.v) or 0, owners = owners, events = {} }
  elseif kind == "E" then
    local zone, body = rest:match("^(%d+)|(.*)$")
    if zone and model.pending then
      local f = fields(body)
      local e = { zone = tonumber(zone), region = tonumber(f.r) or 255, k = f.k, l = tonumber(f.l), c = tonumber(f.c),
        m = tonumber(f.m), at = now }
      if e.k == "C" or e.k == "W" or e.k == "F" or e.k == "R" or e.k == "G" then
        model.pending.events[#model.pending.events + 1] = e
      end
    end
  elseif kind == "D" then
    local snap = model.pending
    model.pending = nil
    if not snap then return true end
    local before = {}
    for _, e in ipairs(model.events) do before[P.key(e)] = true end
    local fresh = {}
    for _, e in ipairs(snap.events) do
      if not before[P.key(e)] then fresh[#fresh + 1] = e end
    end
    local first = model.version == 0
    model.version, model.owners, model.events = snap.version, snap.owners, snap.events
    return "snapshot", first and {} or fresh
  elseif kind == "H" then
    model.here = {}
    return "here"
  elseif kind == "P" then
    local f = fields(rest)
    model.here[#model.here + 1] = { k = f.k, index = tonumber(f.i), x = tonumber(f.x) or 0, y = tonumber(f.y) or 0,
      z = tonumber(f.z) or 0, l = tonumber(f.l) or 0, c = tonumber(f.c) or 0 }
    return "here"
  end
  return true
end

-- A rift's minutes left now (the server sends it once; it counts down here).
function P.minutes_left(e, now)
  if not e.m then return nil end
  return math.max(0, math.ceil(e.m - ((now or 0) - (e.at or 0)) / 60))
end

-- Events of one region (or all when region is nil), in the order the server sent them.
function P.events(model, region)
  local out = {}
  for _, e in ipairs(model.events) do
    if region == nil or e.region == region then out[#out + 1] = e end
  end
  return out
end

return P
