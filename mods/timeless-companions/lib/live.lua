-- Live companion data straight from the client, between server feeds: HP, MP and TP from the party list, what each
-- companion fights (its entity's target), and what it is doing (spells, weapon skills and abilities from the action
-- events). The server's feeds say who the companions are and what they wear; this says how they are doing now.

local S = require("lib.style")
local link = require("lib.link")

local L = {}

L.ACTION_SECONDS = 4 -- how long a finished action's line stays

local doing = {} -- server id -> { text, colour, until_ }

local function now() return os.clock() end

local function spell_name(id)
  local ok, s = pcall(res.spell, id)
  return ok and s and s.name or nil
end

local function ability_name(id)
  local ok, a = pcall(res.ability, id)
  return ok and a and a.name or nil
end

local function entity_name(id)
  if not id or id == 0 then return nil end
  local ok, e = pcall(game.entity, id)
  return ok and e and e.name or nil
end

-- What an action tells about its actor: a line and its colour, and how long it stays (nil: until the next).
function L.describe(a)
  local t = a.targets and a.targets[1]
  local r = t and t.results and t.results[1]
  local target = t and entity_name(t.id)
  local to = (target and t.id ~= a.actor) and (" → " .. target) or ""
  if a.category == 8 then
    if a.param == 28787 then return "Interrupted", S.BAD, 2 end
    local name = spell_name((r and r.value or 0) & 0x3FF)
    if name then return "Casting " .. name .. to, S.FOCUS, 12 end
  elseif a.category == 4 then
    local name = spell_name(a.param)
    if name then return name .. to, S.MP, L.ACTION_SECONDS end
  elseif a.category == 7 then
    if a.param == 28787 then return "Interrupted", S.BAD, 2 end
    local name = ability_name(r and r.value or 0)
    if name then return "Readying " .. name, S.GOLD, 4 end
  elseif a.category == 3 then
    local name = ability_name(a.param)
    if name then return name .. "!" .. to, S.GOLD, L.ACTION_SECONDS end
  elseif a.category == 6 then
    local name = ability_name(a.param + 512) or ability_name(a.param)
    if name then return name .. to, S.BOND, L.ACTION_SECONDS end
  end
  return nil
end

function L.on_action(a)
  if not a or not a.actor then return end
  local text, colour, seconds = L.describe(a)
  if text then
    doing[a.actor] = { text = text, colour = colour, until_ = now() + (seconds or L.ACTION_SECONDS) }
  end
end

-- The live line for a companion: what it's doing, else what it fights, else nil.
function L.activity(id, entity)
  if id == nil then return nil end
  local d = doing[id]
  if d and now() < d.until_ then return d.text, d.colour end
  doing[id] = nil
  if entity and entity.status == 1 then
    local target = entity_name(entity.target_id) or (entity.target_index and entity_name(entity.target_index))
    if target then return "Fighting " .. target, S.MUTED end
    return "Fighting", S.MUTED
  end
  return nil
end

-- Who the server says is a companion: target index -> true (both feeds).
local function companion_indexes()
  local set = {}
  for _, id in ipairs(link.cv.order or {}) do set[id] = true end
  for _, p in ipairs(link.cm.party or {}) do set[p.id] = true end
  return set
end

-- The companions in our party now, in party order: party member fields (hp, hpp, mp, mpp, tp, jobs, levels) plus
-- entity (its live entity, or nil out of range), role (from the manager feed or a guess), kept / bond (from the feeds).
function L.members()
  local ok, party = pcall(game.party)
  if not ok or not party then return {} end
  local known = companion_indexes()
  local out = {}
  for _, m in ipairs(party) do
    if m.party == 0 and m.slot ~= 0 then
      local okE, e = pcall(game.entity, m.id)
      e = okE and e or nil
      if known[m.index] or (e and e.trust) then
        local rec = { id = m.id, index = m.index, name = m.name, hp = m.hp, hpp = m.hpp, mp = m.mp, mpp = m.mpp,
          tp = m.tp, job = m.main_job, lv = m.main_level, sjob = m.sub_job, slv = m.sub_level, in_zone = m.in_zone,
          hp_max = m.hp_max, buffs = m.buffs, entity = e, companion = known[m.index] == true }
        local job = link.cm.jobs and link.cm.jobs[m.main_job]
        local cvrec = link.cv.companions and link.cv.companions[m.index]
        if job and job.role then
          rec.role = S.ROLE[job.role] or S.ROLE.DPS
        else
          rec.role = S.role_guess(m.main_job, cvrec and cvrec.tank)
        end
        if cvrec then rec.bond, rec.kept = cvrec.bond, cvrec.kept end
        for _, p in ipairs(link.cm.party or {}) do
          if p.id == m.index then
            rec.bond = (p.bond or "") ~= "" and p.bond or rec.bond
            rec.kept = (p.kept or 0) > 0
          end
        end
        out[#out + 1] = rec
      end
    end
  end
  return out
end

-- One companion's live record by target index, or nil.
function L.by_index(index)
  for _, m in ipairs(L.members()) do
    if m.index == index then return m end
  end
  return nil
end

-- A short signature of what the windows show live, to redraw only on a change.
function L.signature(members)
  local t = {}
  for _, m in ipairs(members) do
    local act = L.activity(m.id, m.entity) or ""
    t[#t + 1] = table.concat({ m.index, m.hp or 0, m.mp or 0, m.tp or 0, m.hpp or 0, act }, ":")
  end
  return table.concat(t, "|")
end

return L
