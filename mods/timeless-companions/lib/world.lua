-- Companions in the world: name plates in their role's colour, a ring at their feet, and what they say in party
-- chat as a speech bubble over their head.

local S = require("lib.style")

local W = {}

W.BUBBLE_SECONDS = 5

local plated = {}  -- server id -> role key of the plate set
local ringed = {}  -- server id -> true while its ring is drawn
local bubbles = {} -- server id -> { text, until_ }
local bubbled = {} -- server id -> true while its bubble elements exist

local function setting(k)
  local ok, v = pcall(settings.get, k)
  return ok and v
end

-- A party chat line: if a companion said it, it gets a bubble.
function W.on_chat(msg, sender, kind, members)
  if kind ~= 4 or not sender or sender == "" then return end
  for _, m in ipairs(members) do
    if m.name == sender then
      local plain = msg
      if type(text) == "table" and text.plain then
        local ok, p = pcall(text.plain, msg)
        if ok and p then plain = p end
      end
      local n = utf8.len(plain) or #plain
      bubbles[m.id] = { text = plain, until_ = os.clock() + W.BUBBLE_SECONDS + math.min(4, n / 30) }
      return
    end
  end
end

-- Wraps text to lines of at most n letters.
function W.wrap(text, n)
  local lines, line = {}, ""
  for word in text:gmatch("%S+") do
    if line == "" then
      line = word
    elseif (utf8.len(line) or #line) + 1 + (utf8.len(word) or #word) <= n then
      line = line .. " " .. word
    else
      lines[#lines + 1] = line
      line = word
    end
  end
  if line ~= "" then lines[#lines + 1] = line end
  if #lines > 3 then
    lines = { lines[1], lines[2], lines[3] .. "…" }
  end
  return lines
end

local function clear_bubble(id)
  if bubbled[id] then
    hud.remove("bub_" .. id)
    hud.remove("bubt_" .. id)
    bubbled[id] = nil
  end
end

function W.update(members)
  local here = {}
  for _, m in ipairs(members) do here[m.id] = m end

  -- Name plates
  local plates = setting("name_colours")
  for id, key in pairs(plated) do
    local m = here[id]
    if not plates or not m or (m.role and m.role.key ~= key) then
      pcall(render.entity, id, nil)
      plated[id] = nil
    end
  end
  if plates then
    for id, m in pairs(here) do
      if m.entity and not plated[id] and m.role then
        pcall(render.entity, id, { name_colour = m.role.colour })
        plated[id] = m.role.key
      end
    end
  end

  -- Rings at their feet
  local rings = setting("rings")
  for id in pairs(ringed) do
    if not rings or not here[id] or not here[id].entity then
      pcall(draw.remove, "ring_" .. id)
      ringed[id] = nil
    end
  end
  if rings then
    for id, m in pairs(here) do
      if m.entity and m.entity.pos and m.role then
        pcall(draw.ring, "ring_" .. id, { at = m.entity.pos, radius = 0.6, segments = 40,
          colour = S.fade(m.role.colour, (m.hpp or 0) == 0 and 0.25 or 0.75) })
        ringed[id] = true
      end
    end
  end

  -- Speech bubbles
  local now = os.clock()
  local u = 1
  local okS, sc = pcall(hud.screen)
  if okS and sc then u = sc.ui_scale or 1 end
  for id, b in pairs(bubbles) do
    local m = here[id]
    local e = m and m.entity
    local p
    if e and e.pos and setting("bubbles") and now < b.until_ then
      local top = { x = e.pos.x, y = e.pos.y - (e.height or 1.6) - 0.3, z = e.pos.z }
      local ok, sp = pcall(draw.to_screen, top)
      if ok and sp and sp.on_screen ~= false then p = sp end
    end
    if not p then
      clear_bubble(id)
      if now >= b.until_ then bubbles[id] = nil end
    else
      local lines = W.wrap(b.text, 34)
      local longest = 0
      for _, l in ipairs(lines) do longest = math.max(longest, utf8.len(l) or #l) end
      local w = math.floor((longest * 6.6 + 24) * u)
      local h = math.floor((#lines * 15 + 12) * u)
      local x, y = p.x - w / 2, p.y - h - 26 * u
      hud.bubble("bub_" .. id, { x = x, y = y, w = w, h = h, round = 9 * u, colour = { 14, 16, 26, 225 },
        edge = S.fade(m.role and m.role.colour or S.GOLD, 0.8), edge_w = 1.5 * u, tail = { x = p.x, y = p.y - 6 * u },
        layer = 4 })
      hud.text("bubt_" .. id, { x = p.x, y = y + 5 * u, text = table.concat(lines, "\n"), align = "center", size = 0.82,
        colour = S.TEXT, layer = 5 })
      bubbled[id] = true
    end
  end
end

function W.clear()
  for id in pairs(plated) do pcall(render.entity, id, nil) end
  for id in pairs(ringed) do pcall(draw.remove, "ring_" .. id) end
  for id in pairs(bubbled) do clear_bubble(id) end
  plated, ringed, bubbles = {}, {}, {}
end

return W
