-- Reads the Craft Plus status answer (modules/craft_plus, craft_book.lua: "cb status <n>"):
--   CB1|B|0|kind=status                 a fresh status starts
--   CB1|S|0|ww=<skill x10>:<rank>,sm=...,ck=...       the eight crafts
--   CB1|P|0|rested=,max=,gain=,insight=,ready=,lesson=,lmax=,l=<8 lessons, dotted>,loss=,t=<tier counts, dotted>
--   CB1|W|0|n=<Masterworks made>,on=,c=<Masterwork % per tier, dotted>,tn=<tier names>,hq=<HQ rate per tier>
--   CB1|A|<slot>|n=,c=,s=<skill x10>,r=<rank>,nx=<next rank>,ns=<its skill>,as=,al=,ws=,away=,job=
--   CB1|O|<slot>|r=,n=,c=,l=,q=,d=<0 open, 1 done, 2 done HQ>,who=,gil=
--   CB1|M|<recipe>|n=,made=,hq=
--   CB1|Z|0|kind=status                 complete
-- Other kinds (recipe pages, the craftbook's tabs) are ignored. No game or UI code here.

local P = {}

P.VERSION = "CB1"
P.CRAFTS = {
  { key = "ww", name = "Woodworking" }, { key = "sm", name = "Smithing" }, { key = "gs", name = "Goldsmithing" },
  { key = "cl", name = "Clothcraft" }, { key = "lt", name = "Leathercraft" }, { key = "bn", name = "Bonecraft" },
  { key = "al", name = "Alchemy" }, { key = "ck", name = "Cooking" },
}
P.RANKS = { [0] = "Amateur", "Recruit", "Initiate", "Novice", "Apprentice", "Journeyman", "Craftsman", "Artisan",
  "Adept", "Veteran", "Expert" }

function P.new_model()
  return { status = nil, pending = nil }
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

local function dots(s)
  local out = {}
  for v in (s or ""):gmatch("[^.]+") do out[#out + 1] = tonumber(v) or v end
  return out
end

local function num(v) return tonumber(v) or 0 end

-- Applies one line. Returns "status" when a status is complete, true for other lines of ours, false otherwise.
function P.feed(model, line, now)
  if type(line) ~= "string" then return false end
  local kind, id, body = line:match("^CB1|(%u)|(%d+)|(.*)$")
  if not kind then return false end
  id = tonumber(id)
  local f = fields(body)
  local st = model.pending
  if kind == "B" then
    model.pending = f.kind == "status" and { crafts = {}, orders = {}, mastery = {}, apprentices = {}, masterworks = nil } or nil
  elseif kind == "Z" then
    if st and f.kind == "status" then
      model.pending = nil
      st.at = now
      model.status = st
      return "status"
    end
  elseif st then
    if kind == "S" then
      for i, c in ipairs(P.CRAFTS) do
        local skill, rank = (f[c.key] or ""):match("^(%d+):(%d+)$")
        st.crafts[i] = { name = c.name, short = c.key:upper(), skill = num(skill) / 10, rank = num(rank) }
      end
    elseif kind == "P" then
      st.bonus = { rested = num(f.rested), max = num(f.max), gain = num(f.gain), insight = num(f.insight), ready = f.ready == "1",
        lesson = num(f.lesson), lmax = num(f.lmax), lessons = dots(f.l), loss = num(f.loss), tiers = dots(f.t) }
    elseif kind == "W" then
      st.masterworks = { made = num(f.n), on = f.on == "1", chance = dots(f.c), names = {}, hq = dots(f.hq) }
      for name in (f.tn or ""):gmatch("[^.]+") do st.masterworks.names[#st.masterworks.names + 1] = name end
    elseif kind == "A" then
      st.apprentices[#st.apprentices + 1] = { slot = id, name = f.n or "?", craft = num(f.c), skill = num(f.s) / 10, rank = f.r or "",
        next = f.nx ~= "" and f.nx or nil, next_skill = num(f.ns), assist = num(f.as), loss = num(f.al), workshop = num(f.ws),
        away = f.away == "1", job = num(f.job) }
    elseif kind == "O" then
      st.orders[#st.orders + 1] = { slot = id, recipe = num(f.r), name = f.n or "?", craft = num(f.c), level = num(f.l), qty = num(f.q),
        done = num(f.d), who = f.who or "", gil = num(f.gil) }
    elseif kind == "M" then
      st.mastery[#st.mastery + 1] = { recipe = id, name = f.n or "?", made = num(f.made), hq = num(f.hq) }
    end
  end
  return true
end

-- A recipe's mastery tier from the tier counts: index (0 = none yet), name, next count (nil at the top)
function P.tier(st, made)
  local tiers = st.bonus and st.bonus.tiers or { 25, 50, 100 }
  local names = st.masterworks and st.masterworks.names or { "Practiced", "Mastered", "Grandmaster" }
  local at = 0
  for i, t in ipairs(tiers) do if made >= t then at = i end end
  return at, names[at] or "Learning", tiers[at + 1]
end

-- The Masterwork chance (%) at a mastery tier index
function P.chance(st, at)
  local mw = st.masterworks
  if not mw or not mw.on or at == 0 then return 0 end
  return mw.chance[at] or 0
end

return P
