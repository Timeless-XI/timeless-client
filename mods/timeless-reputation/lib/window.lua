-- The Reputation window: your fame in San d'Oria, Bastok, Windurst, Jeuno, Selbina and Rabao, and Norg, each with its
-- emblem, its fame level (1-9, as pips and a word) and how far to the next, plus your nation and rank.
-- The four with a flag in the game (San d'Oria, Bastok, Windurst, Jeuno) show that flag's own icon, drawn by the client
-- from the player's game files (items 181-184), so nothing of the game's art is copied here. Selbina & Rabao and Norg
-- have no emblem in the game, so they get a simple drawn one (a wave, an anchor).

local Canvas = require("lib.canvas")
local S = require("lib.style")
local protocol = require("lib.protocol")

local R = {}

R.WIN = "reputation"
R.W = 540

R.NATIONS = { [0] = "San d'Oria", [1] = "Bastok", [2] = "Windurst" }

R.AREAS = {
  { id = 0, name = "San d'Oria", colour = { 196, 70, 66, 255 }, glyph = "cross", item = 181, -- San d'Orian Flag
    how = "Quests in San d'Oria: the city's people, its knights and Chateau d'Oraguille." },
  { id = 1, name = "Bastok", colour = { 82, 132, 214, 255 }, glyph = "gear", item = 182, -- Bastokan Flag
    how = "Quests in Bastok: the Markets, the Mines, the Port and the Metalworks." },
  { id = 2, name = "Windurst", colour = { 92, 176, 104, 255 }, glyph = "star", item = 183, -- Windurstian Flag
    how = "Quests in Windurst: the Woods, the Waters, the Walls and Port Windurst." },
  { id = 3, name = "Jeuno", colour = { 156, 108, 210, 255 }, glyph = "tower", item = 184, -- Jeunoan Flag
    how = "Grows with your fame in San d'Oria, Bastok and Windurst together.", shared = true },
  { id = 4, name = "Selbina & Rabao", colour = { 74, 176, 176, 255 }, glyph = "wave",
    how = "Grows with your fame in San d'Oria and Bastok.", shared = true },
  { id = 5, name = "Norg", colour = { 214, 122, 64, 255 }, glyph = "anchor",
    how = "Quests for the pirates of Norg and the Tenshodo." },
}

-- What people make of you, by fame level
R.WORDS = { "Unknown", "Heard of", "Known", "Respected", "Well regarded", "Admired", "Celebrated", "Renowned", "Legendary" }

R.model = nil
R.ask = nil -- set by init.lua

local function glyph(c, kind, x, y, col)
  local w = { 255, 250, 240, 235 }
  if kind == "cross" then
    c:rect(x - 2, y - 11, 4, 22, w, { round = 1 })
    c:rect(x - 9, y - 4, 18, 4, w, { round = 1 })
  elseif kind == "gear" then
    c:circle(x, y, 9, w, 2)
    c:circle(x, y, 4, w)
    for i = 0, 5 do
      local a = i * math.pi / 3
      c:line(x + math.cos(a) * 9, y + math.sin(a) * 9, x + math.cos(a) * 13, y + math.sin(a) * 13, w, 3)
    end
  elseif kind == "star" then
    for i = 0, 4 do
      local a1, a2 = -math.pi / 2 + i * 2 * math.pi / 5, -math.pi / 2 + (i + 2) * 2 * math.pi / 5
      c:line(x + math.cos(a1) * 11, y + math.sin(a1) * 11, x + math.cos(a2) * 11, y + math.sin(a2) * 11, w, 2)
    end
  elseif kind == "tower" then
    c:rect(x - 5, y - 8, 10, 18, w, { round = 1 })
    c:rect(x - 8, y - 12, 16, 5, w, { round = 1 })
    c:rect(x - 2, y + 3, 4, 7, col)
  elseif kind == "wave" then
    for i = 0, 1 do
      local yy = y - 4 + i * 8
      c:line(x - 11, yy, x - 5, yy - 4, w, 2)
      c:line(x - 5, yy - 4, x + 1, yy, w, 2)
      c:line(x + 1, yy, x + 7, yy - 4, w, 2)
      c:line(x + 7, yy - 4, x + 11, yy - 1, w, 2)
    end
  elseif kind == "anchor" then
    c:line(x, y - 11, x, y + 9, w, 3)
    c:line(x - 7, y - 6, x + 7, y - 6, w, 2)
    c:line(x - 10, y + 3, x, y + 10, w, 2)
    c:line(x + 10, y + 3, x, y + 10, w, 2)
    c:circle(x, y - 12, 2.5, w, 1.5)
  end
end

local function card(c, x, y, w, h, info, a)
  c:card(x, y, w, h, { fill = S.fade(info.colour, 0.08), edge = S.fade(info.colour, 0.45) })
  -- The crest
  local cx, cy = x + 30, y + 32
  if info.item then
    c:circle(cx, cy, 21, { 0, 0, 0, 110 })
    c:circle(cx, cy, 21, S.fade(info.colour, 0.9), 2)
    c:icon(cx - 16, cy - 16, 32, "item", info.item)
  else
    c:circle(cx, cy, 21, S.fade(info.colour, 0.85))
    c:circle(cx, cy, 21, { 255, 240, 205, 160 }, 2)
    glyph(c, info.glyph, cx, cy, S.fade(info.colour, 0.85))
  end
  -- Name and what they make of you
  local level = a and a.level or 1
  c:text(x + 60, y + 10, info.name, { bold = true, colour = S.TEXT, w = w - 70 })
  c:text(x + 60, y + 29, a and R.WORDS[level] or "...", { size = 0.92, colour = S.fade(info.colour, 1), w = w - 70 })
  -- Nine pips
  for i = 1, 9 do
    local px = x + 64 + (i - 1) * 14
    if a and i <= level then
      c:circle(px, y + 54, 4.5, info.colour)
    else
      c:circle(px, y + 54, 4.5, { 255, 255, 255, 60 }, 1)
    end
  end
  c:text(x + 64 + 9 * 14, y + 47, ("%d/9"):format(level), { size = 0.85, colour = S.MUTED })
  -- To the next level
  local frac = protocol.progress(a)
  c:bar(x + 12, y + 72, w - 24, 8, frac, info.colour)
  local note
  if not a then
    note = "Asking the town criers..."
  elseif a.hi == 0 or a.hi <= a.lo then
    note = "As famous as anyone gets."
  else
    note = ("%d to %s"):format(a.hi - a.fame, R.WORDS[level + 1] or "the next level")
  end
  c:text(x + 12, y + 84, note, { size = 0.85, colour = S.MUTED, w = w - 24 })
  c:para(x + 12, y + 100, w - 24, info.how, { size = 0.78, colour = S.FAINT })
  c:region(x, y, w, h, { title = info.name .. (a and (": fame level %d"):format(level) or ""),
    tip = info.how .. (a and ("\nFame %d (this level from %d%s)."):format(a.fame, a.lo, a.hi > a.lo and (", the next at " .. a.hi) or "") or "") })
end

function R.draw(c)
  local m = R.model
  local y = 0
  c:text(0, y, "Reputation", { bold = true, size = 1.2, colour = S.GOLD })
  c:button(R.W - 90, y, 90, 24, "Refresh", { key = "refresh", on = function() if R.ask then R.ask() end end,
    tip = "Ask again (it also updates every 30 seconds while open)." })
  y = y + 26
  local nation = m and m.loaded and R.NATIONS[m.nation]
  if nation then
    c:text(0, y, ("A citizen of %s, rank %d."):format(nation, m.rank or 0), { size = 0.95, colour = S.TEXT })
  else
    c:text(0, y, m and m.loaded and "" or "Waiting for the server...", { size = 0.95, colour = S.MUTED })
  end
  y = y + 28
  local cw, ch, gap = (R.W - 12) / 2, 134, 12
  for i, info in ipairs(R.AREAS) do
    local col, row = (i - 1) % 2, math.floor((i - 1) / 2)
    local a = m and m.loaded and m.areas[info.id] or nil
    card(c, col * (cw + gap), y + row * (ch + gap), cw, ch, info, a)
  end
  c:grow(y + 3 * (ch + gap))
end

-- A short line for the Timeless menu's card
function R.summary()
  local m = R.model
  if not (m and m.loaded) then return nil end
  local parts = {}
  for _, info in ipairs(R.AREAS) do
    local a = m.areas[info.id]
    if a then parts[#parts + 1] = ("%s %d"):format(info.name, a.level) end
  end
  return table.concat(parts, ", ")
end

function R.setup(model)
  R.model = model
  ui.window(R.WIN, {
    title = "Reputation",
    width = R.W + 20,
    help = "Your fame in each area: what people there think of you, and how far to the next level. Fame opens up quests and better treatment.",
    elements = { { type = "canvas", id = "rep", w = R.W, h = 480, draw = {} } },
  })
  Canvas.add(R.WIN, "rep", R.W, R.draw)
  ui.on(R.WIN, function(win, e)
    if e.kind == "close" then return end
    Canvas.event(win, e)
  end)
end

function R.render()
  if ui.is_open(R.WIN) then Canvas.render(R.WIN, "rep") end
end

function R.toggle()
  if ui.is_open(R.WIN) then
    ui.close(R.WIN)
  else
    ui.open(R.WIN)
    Canvas.render(R.WIN, "rep")
    if R.ask then R.ask() end
  end
end

return R
