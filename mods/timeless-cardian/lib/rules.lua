-- The hand rules as the server holds them (modules/cardian), so the cardbook can say "no" before asking.
-- No game or UI code here.

local cards = require("lib.cards")

local R = {}

R.HAND_SIZE, R.HAND_LEGENDS = 5, 1

-- Why a hand won't do, or nil: five cards you own, one legendary at most
function R.hand_problem(ids, have)
  if #ids ~= R.HAND_SIZE then return ("A hand is %d cards (this one has %d)."):format(R.HAND_SIZE, #ids) end
  local seen, legends = {}, 0
  for _, id in ipairs(ids) do
    local c = cards.get(id)
    if not c then return "There's no such card." end
    seen[id] = (seen[id] or 0) + 1
    if seen[id] > (have[id] or 0) then return ("You have only %d %s."):format(have[id] or 0, c.name) end
    if c.rarity == 3 then legends = legends + 1 end
  end
  if legends > R.HAND_LEGENDS then return "Only one legendary in a hand." end
  return nil
end

return R
