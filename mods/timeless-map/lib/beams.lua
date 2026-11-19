-- Pillars of light over what is happening in your own zone: an open rift, a roaming Champion, a warband captain, a
-- Fomor band. The server says where each one is (WM1|P lines, every 10 s); while one is in sight, its live position
-- is used instead. Each pillar has a label with what it is and how far, and none is drawn over something right next
-- to you (you have found it).

local B = {}

B.HEIGHT = 30   -- yalms tall
B.NEAR = 12     -- no pillar within this many yalms
B.LABELS = {
  R = { "Time Rift", "Ancient Rift", "Hero's Rift" },
  C = { "Champion", "Elite Champion", "Legendary Champion" },
}
B.COLOURS = {
  R = { 82, 219, 204, 220 }, W = { 236, 108, 96, 220 }, F = { 184, 122, 250, 220 }, C = { 247, 204, 97, 220 },
}
local LEGENDARY = { 255, 140, 60, 230 }

B.drawn = {} -- id -> true

function B.label(h)
  if h.k == "W" then return ("Warband captain (Lv. %d)"):format(h.l or 0) end
  if h.k == "F" then return "Fomor band" end
  local names = B.LABELS[h.k] or {}
  local text = names[h.c] or names[1] or "?"
  if h.k == "C" then text = ("%s (Lv. %d)"):format(text, h.l or 0) end
  return text
end

-- Where it is now: the live entity when the client knows it, else where the server last saw it.
function B.position(h)
  local ok, e = pcall(game.entity, h.index)
  if ok and e and e.pos then return e.pos end
  return { x = h.x, y = h.y, z = h.z }
end

function B.update(here, enabled)
  local keep = {}
  local me = game.player()
  if enabled and me and me.pos then
    for i, h in ipairs(here or {}) do
      local pos = B.position(h)
      local dx, dy, dz = pos.x - me.pos.x, pos.y - me.pos.y, pos.z - me.pos.z
      local dist = math.sqrt(dx * dx + dy * dy + dz * dz)
      if dist >= B.NEAR then
        local colour = (h.k == "C" and h.c == 3) and LEGENDARY or B.COLOURS[h.k] or B.COLOURS.C
        local id = "beam" .. i
        local top = { x = pos.x, y = pos.y - B.HEIGHT, z = pos.z } -- y points down: up is minus
        draw.line(id, { from = { x = pos.x, y = pos.y, z = pos.z }, to = top, colour = colour })
        draw.line(id .. "b", { from = { x = pos.x + 0.15, y = pos.y, z = pos.z }, to = { x = top.x + 0.15, y = top.y, z = top.z },
          colour = { colour[1], colour[2], colour[3], 120 } })
        draw.text(id .. "t", { at = { x = top.x, y = top.y - 1, z = top.z }, text = ("%s  %dy"):format(B.label(h), math.floor(dist + 0.5)),
          size = 0.95, colour = { 255, 246, 228, 255 }, outline = { 0, 0, 0, 230 } })
        keep[id], keep[id .. "b"], keep[id .. "t"] = true, true, true
      end
    end
  end
  for id in pairs(B.drawn) do
    if not keep[id] then draw.remove(id) end
  end
  B.drawn = keep
end

function B.clear()
  for id in pairs(B.drawn) do draw.remove(id) end
  B.drawn = {}
end

return B
