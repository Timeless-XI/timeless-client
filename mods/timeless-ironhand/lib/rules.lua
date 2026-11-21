-- The deck rules and the table's limits as the server holds them (modules/ironhand), so the cardbook and the table
-- can say "no" before asking. No game or UI code here.

local cards = require("lib.cards")

local R = {}

R.DECK_SIZE, R.DECK_COPIES, R.DECK_LEGENDS, R.CAPACITY = 15, 2, 1, 4

-- Why a deck won't do, or nil
function R.deck_problem(ids, have)
  if #ids ~= R.DECK_SIZE then return ("A deck is %d cards (this one has %d)."):format(R.DECK_SIZE, #ids) end
  local seen, base, legends = {}, {}, 0
  for _, id in ipairs(ids) do
    local c = cards.get(id)
    if not c then return "There's no such card." end
    seen[id] = (seen[id] or 0) + 1
    local b = c.base or c.id
    base[b] = (base[b] or 0) + 1
    if seen[id] > (have[id] or 0) then return ("You have only %d %s."):format(have[id] or 0, c.name) end
    if base[b] > R.DECK_COPIES then return ("At most %d of %s."):format(R.DECK_COPIES, cards.byId[b].name) end
    if c.rarity == 3 then legends = legends + 1 end
  end
  if legends > R.DECK_LEGENDS then return "Only one legendary in a deck." end
  return nil
end

-- Whether hand card h can go to front f, with the plays already staged; and why not
function R.can_stage(now, staged, h, f)
  local id = now and now.hand[h]
  if not id then return false, "no such card" end
  local spent, here = 0, 0
  for _, p in ipairs(staged) do
    if p.h == h then return false, "already played" end
    local c = cards.get(now.hand[p.h])
    spent = spent + (c and c.cost or 0)
    if p.f == f then here = here + 1 end
  end
  local c = cards.get(id)
  if spent + c.cost > (now.crystals or 0) then return false, "not enough crystals" end
  local fr = now.fronts[f]
  local cap = R.CAPACITY + (c.ab.leap and 1 or 0)
  if fr and #(fr.cards and fr.cards[1] or {}) + here >= cap then return false, "that front is full" end
  return true
end

function R.spent(now, staged)
  local n = 0
  for _, p in ipairs(staged) do
    local c = cards.get(now.hand[p.h])
    n = n + (c and c.cost or 0)
  end
  return n
end

return R
