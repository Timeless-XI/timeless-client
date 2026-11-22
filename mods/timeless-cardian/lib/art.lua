-- How a card looks: the game's own item window (a navy panel, a cream border, white text with a shadow), wearing a
-- header dressed for its family the way the shop windows dress for their shopkeeper (the Quadav's shell pattern,
-- the Goblin's patched leather, the guild's charcoal and gold for notorious monsters, Galkan stone and Hume planks
-- for townsfolk...), with its four numbers in a diamond over the picture's top-left corner, as in the old card
-- games. In the big card the picture is the monster's real model from your game files, turning slowly on a clear
-- backdrop (TimelessXI client); smaller cards show its element's crystal.
--
-- Art.card(c, x, y, s, id, o) draws card `id` at (x, y), s times 130 x 190 units. o: portrait (the live model),
-- count (a badge), missing (not in your collection: a dark silhouette), selected / staged (a highlight), owner (on
-- the board: 1 yours, blue; 2 theirs, red), sides (the numbers where it stands; else its printed ones), flipped
-- (it just changed sides: a flash), detail ("mini", "small" or "big"; by size if not given).

local Canvas = require("lib.canvas")
local cards = require("lib.cards")

local Art = {}

Art.W, Art.H = 130, 190

local function rgba(r, g, b, a) return { r, g, b, a or 255 } end
local function fade(c, k) return { c[1], c[2], c[3], math.floor((c[4] or 255) * k + 0.5) } end

Art.ELEMENT = {
  fire = rgba(216, 69, 46), ice = rgba(110, 195, 230), wind = rgba(79, 179, 92), earth = rgba(209, 169, 58),
  thunder = rgba(155, 95, 214), water = rgba(58, 116, 209), light = rgba(244, 241, 230), dark = rgba(90, 69, 128),
}
Art.ELEMENT_NAME = { fire = "Fire", ice = "Ice", wind = "Wind", earth = "Earth", thunder = "Thunder", water = "Water", light = "Light", dark = "Dark" }

-- Headers, after the shop windows' themes: bg, bg2 (the band's gradient), title, accent (edges, chips), muted
Art.THEMES = {
  quadav = { bg = rgba(150, 110, 50), bg2 = rgba(80, 56, 26), title = rgba(255, 236, 190), accent = rgba(240, 190, 80), muted = rgba(200, 180, 140), label = "Shell Hoard" },
  goblin = { bg = rgba(92, 64, 38), bg2 = rgba(58, 40, 24), title = rgba(232, 214, 150), accent = rgba(140, 196, 70), muted = rgba(190, 170, 130), label = "Sack o' Stuff" },
  orc = { bg = rgba(110, 30, 26), bg2 = rgba(40, 14, 12), title = rgba(250, 225, 205), accent = rgba(230, 120, 90), muted = rgba(200, 160, 150), label = "War Loot" },
  yagudo = { bg = rgba(84, 40, 96), bg2 = rgba(44, 20, 54), title = rgba(240, 220, 255), accent = rgba(200, 150, 240), muted = rgba(190, 170, 210), label = "Sacred Offerings" },
  tonberry = { bg = rgba(34, 58, 44), bg2 = rgba(12, 22, 16), title = rgba(230, 240, 210), accent = rgba(255, 210, 110), muted = rgba(150, 176, 160), label = "Grudge" },
  grave = { bg = rgba(52, 56, 92), bg2 = rgba(18, 20, 40), title = rgba(226, 230, 255), accent = rgba(170, 180, 240), muted = rgba(170, 176, 210), label = "After Dark" },
  wild = { bg = rgba(52, 86, 48), bg2 = rgba(22, 40, 22), title = rgba(226, 244, 210), accent = rgba(150, 210, 110), muted = rgba(170, 196, 160), label = "The Wilds" },
  sea = { bg = rgba(36, 70, 110), bg2 = rgba(14, 30, 56), title = rgba(214, 236, 255), accent = rgba(110, 180, 240), muted = rgba(160, 186, 210), label = "Tides" },
  ember = { bg = rgba(120, 50, 24), bg2 = rgba(50, 18, 10), title = rgba(255, 226, 190), accent = rgba(255, 150, 80), muted = rgba(210, 170, 140), label = "Arcana" },
  guild = { bg = rgba(44, 46, 54), bg2 = rgba(24, 26, 32), title = rgba(228, 192, 119), accent = rgba(228, 192, 119), muted = rgba(160, 168, 176), label = "Notorious" },
  dragon = { bg = rgba(90, 16, 20), bg2 = rgba(26, 6, 8), title = rgba(255, 214, 150), accent = rgba(240, 170, 80), muted = rgba(210, 160, 140), label = "Great Wyrm" },
  galka = { bg = rgba(104, 100, 96), bg2 = rgba(62, 58, 56), title = rgba(240, 226, 206), accent = rgba(214, 130, 60), muted = rgba(180, 172, 162), label = "Bastok" },
  hume = { bg = rgba(132, 92, 58), bg2 = rgba(92, 62, 38), title = rgba(250, 232, 196), accent = rgba(222, 180, 90), muted = rgba(190, 176, 156), label = "Bastok" },
  elvaan = { bg = rgba(30, 46, 92), bg2 = rgba(16, 24, 54), title = rgba(230, 236, 255), accent = rgba(228, 200, 130), muted = rgba(170, 180, 210), label = "San d'Oria" },
  tarutaru = { bg = rgba(62, 42, 112), bg2 = rgba(30, 20, 66), title = rgba(250, 236, 255), accent = rgba(250, 214, 110), muted = rgba(200, 186, 230), label = "Windurst" },
  mithra = { bg = rgba(122, 72, 40), bg2 = rgba(62, 34, 18), title = rgba(255, 234, 204), accent = rgba(244, 176, 92), muted = rgba(206, 176, 146), label = "On the Road" },
}

-- The board's colours: yours blue, theirs red (the window and its edge)
Art.OWNER = {
  [1] = { top = rgba(52, 92, 170), bot = rgba(18, 34, 78), edge = rgba(140, 190, 255) },
  [2] = { top = rgba(160, 52, 60), bot = rgba(70, 16, 22), edge = rgba(255, 150, 150) },
}

-- The foils: their frame and the light round the card
Art.FOIL = {
  [1] = { edge = rgba(255, 226, 130), glow = rgba(255, 214, 110, 90), label = "Gilded" },
  [2] = { edge = rgba(176, 150, 255), glow = rgba(140, 110, 240, 90), label = "Nightborne" },
  [3] = { edge = rgba(130, 236, 240), glow = rgba(110, 220, 240, 80), label = "Rift-touched" },
  [4] = { edge = rgba(240, 110, 90), glow = rgba(230, 90, 70, 80), label = "Juggernaut" },
}

local CREAM = rgba(221, 211, 182)
local INK = rgba(242, 239, 230)
local SKY = rgba(159, 208, 255)
local GOLDTXT = rgba(232, 215, 154)
local SHADOW = rgba(0, 0, 0, 160)

-- An elemental crystal (as the synthesis crystals), centred at (x, y), h tall
function Art.crystal(c, x, y, h, col, k)
  k = k or 1
  local w = h * 0.55
  local top, mid, bot = y - h / 2, y - h * 0.15, y + h / 2
  c:tri(x, top, x + w / 2, mid, x - w / 2, mid, fade(col, k))
  c:tri(x - w / 2, mid, x + w / 2, mid, x, bot, fade({ math.floor(col[1] * 0.75), math.floor(col[2] * 0.75), math.floor(col[3] * 0.75), 255 }, k))
  c:tri(x, top, x + w / 2, mid, x, mid + h * 0.08, fade(rgba(255, 255, 255), 0.3 * k))
end

-- The header band's ornaments (x, y: its corner; w, h: its size), after the shop windows
local DECO = {}
function DECO.quadav(c, x, y, w, h, s)
  local d = 6 * s
  for row = 0, 2 do
    local cy = y + 6 * s + row * 10 * s
    local off = (row % 2) * d
    for cx = x + d + off, x + w - d, d * 2 do
      c:tri(cx - d * 0.8, cy, cx, cy - d * 0.8, cx + d * 0.8, cy, rgba(60, 40, 16, 70))
      c:tri(cx - d * 0.8, cy, cx, cy + d * 0.8, cx + d * 0.8, cy, rgba(60, 40, 16, 70))
    end
  end
end
function DECO.goblin(c, x, y, w, h, s)
  c:rect(x + w - 34 * s, y + 3 * s, 26 * s, 13 * s, rgba(118, 84, 50), { round = 2 })
  for sx = x + w - 32 * s, x + w - 12 * s, 5 * s do
    c:line(sx, y + 5 * s, sx + 3 * s, y + 5 * s, rgba(232, 214, 150, 170), 1)
    c:line(sx, y + 14 * s, sx + 3 * s, y + 14 * s, rgba(232, 214, 150, 170), 1)
  end
  for _, p in ipairs({ { x + 4 * s, y + 4 * s }, { x + 4 * s, y + h - 4 * s }, { x + w - 4 * s, y + h - 4 * s } }) do
    c:circle(p[1], p[2], 2 * s, rgba(170, 150, 110))
  end
end
function DECO.orc(c, x, y, w, h, s)
  for tx = x + 6 * s, x + w - 6 * s, 9 * s do c:tri(tx - 3 * s, y + h, tx + 3 * s, y + h, tx, y + h - 6 * s, rgba(236, 222, 196, 120)) end
end
function DECO.yagudo(c, x, y, w, h, s)
  for fx = x + 8 * s, x + w, 14 * s do c:line(fx, y + h, fx + 8 * s, y + 2 * s, rgba(220, 190, 255, 60), 2 * s) end
end
function DECO.tonberry(c, x, y, w, h, s)
  local lx, ly = x + w - 12 * s, y + h / 2
  for r = 12, 4, -4 do c:circle(lx, ly, r * s, rgba(255, 210, 110, 26)) end
  c:rect(lx - 3 * s, ly - 4 * s, 6 * s, 8 * s, rgba(255, 220, 140, 230), { round = 1 })
end
function DECO.grave(c, x, y, w, h, s)
  c:circle(x + w - 12 * s, y + 9 * s, 6 * s, rgba(226, 230, 255, 220))
  c:circle(x + w - 9 * s, y + 7 * s, 5.4 * s, rgba(26, 28, 52, 255))
  for _, p in ipairs({ { 0.2, 0.25 }, { 0.38, 0.6 }, { 0.55, 0.2 }, { 0.7, 0.5 } }) do
    c:circle(x + w * p[1], y + h * p[2], 0.9 * s, rgba(226, 230, 255, 180))
  end
  for gx = x + 4 * s, x + w - 10 * s, 16 * s do c:rect(gx, y + h - 6 * s, 6 * s, 6 * s, rgba(10, 12, 24, 110), { round = 2 }) end
end
function DECO.wild(c, x, y, w, h, s)
  for lx = x + 10 * s, x + w - 6 * s, 18 * s do
    c:tri(lx, y + h - 2 * s, lx + 6 * s, y + h - 10 * s, lx + 10 * s, y + h - 2 * s, rgba(150, 210, 110, 50))
  end
end
function DECO.sea(c, x, y, w, h, s)
  for wx = x, x + w - 6 * s, 10 * s do
    c:line(wx, y + h - 4 * s, wx + 5 * s, y + h - 7 * s, rgba(160, 210, 255, 70), 1.5)
    c:line(wx + 5 * s, y + h - 7 * s, wx + 10 * s, y + h - 4 * s, rgba(160, 210, 255, 70), 1.5)
  end
end
function DECO.ember(c, x, y, w, h, s)
  for fx = x + 8 * s, x + w - 6 * s, 16 * s do c:tri(fx - 4 * s, y + h, fx + 4 * s, y + h, fx, y + h - 9 * s, rgba(255, 160, 80, 60)) end
end
function DECO.guild(c, x, y, w, h, s)
  local gx, gy = x + w - 12 * s, y + h / 2
  c:tri(gx - 5 * s, gy - 8 * s, gx + 5 * s, gy - 8 * s, gx, gy, rgba(228, 192, 119, 200))
  c:tri(gx - 5 * s, gy + 8 * s, gx + 5 * s, gy + 8 * s, gx, gy, rgba(228, 192, 119, 120))
  c:line(x + 3 * s, y + h - 1, x + w - 3 * s, y + h - 1, rgba(228, 192, 119, 90), 1)
end
function DECO.dragon(c, x, y, w, h, s)
  for k = 0, 2 do c:line(x + w - 30 * s + k * 7 * s, y + 3 * s, x + w - 36 * s + k * 7 * s, y + h - 3 * s, rgba(255, 190, 110, 110), 2 * s) end
  DECO.guild(c, x - 36 * s, y, w, h, s)
end
function DECO.galka(c, x, y, w, h, s)
  local bh = h / 3
  for row = 0, 2 do
    local by = y + row * bh
    c:line(x, by, x + w, by, rgba(40, 36, 34, 150), 1)
    for bx = x + (row % 2) * 15 * s, x + w, 30 * s do c:line(bx, by, bx, by + bh, rgba(40, 36, 34, 150), 1) end
  end
  c:circle(x + 5 * s, y + 5 * s, 2 * s, rgba(196, 140, 80))
  c:circle(x + w - 5 * s, y + 5 * s, 2 * s, rgba(196, 140, 80))
end
function DECO.hume(c, x, y, w, h, s)
  for py = y + h / 3, y + h - 1, h / 3 do c:line(x, py, x + w, py, rgba(60, 38, 20, 120), 1) end
  for nx = x + 20 * s, x + w - 10 * s, 40 * s do c:circle(nx, y + h / 6, 1 * s, rgba(226, 190, 120, 180)) end
end
function DECO.elvaan(c, x, y, w, h, s)
  for dx = x + 8 * s, x + w - 8 * s, 16 * s do
    c:tri(dx - 3 * s, y + h - 4 * s, dx, y + h - 7 * s, dx + 3 * s, y + h - 4 * s, rgba(228, 200, 130, 120))
  end
end
function DECO.tarutaru(c, x, y, w, h, s)
  for _, p in ipairs({ { 0.15, 0.3 }, { 0.45, 0.7 }, { 0.75, 0.25 } }) do
    local sx, sy, r = x + w * p[1], y + h * p[2], 3 * s
    c:tri(sx, sy - r, sx - r * 0.35, sy, sx + r * 0.35, sy, rgba(250, 214, 110, 150))
    c:tri(sx, sy + r, sx - r * 0.35, sy, sx + r * 0.35, sy, rgba(250, 214, 110, 150))
    c:tri(sx - r, sy, sx, sy - r * 0.35, sx, sy + r * 0.35, rgba(250, 214, 110, 150))
    c:tri(sx + r, sy, sx, sy - r * 0.35, sx, sy + r * 0.35, rgba(250, 214, 110, 150))
  end
end
function DECO.mithra(c, x, y, w, h, s)
  for _, p in ipairs({ { 0.62, 0.35 }, { 0.74, 0.62 }, { 0.86, 0.32 } }) do
    local px, py = x + w * p[1], y + h * p[2]
    c:circle(px, py, 2.2 * s, rgba(244, 176, 92, 90))
    for k = -1, 1 do c:circle(px + k * 2.2 * s, py - 2.8 * s, 0.9 * s, rgba(244, 176, 92, 90)) end
  end
end
Art.DECO = DECO

-- Pictures for the townsfolk (no single model to show), and a stand-in for a monster without one
local function picture(c, card, x, y, w, h, s)
  local cx, cy = x + w / 2, y + h / 2
  local k = card.key
  if k == "toma" then
    for r = 26, 8, -6 do c:circle(cx + 6 * s, cy - 8 * s, r * s, rgba(255, 207, 106, 22)) end
    c:line(cx - 10 * s, cy + 30 * s, cx - 10 * s, cy - 22 * s, rgba(46, 36, 22), 3 * s)
    c:line(cx - 10 * s, cy - 22 * s, cx + 6 * s, cy - 22 * s, rgba(46, 36, 22), 2.5 * s)
    c:rect(cx + 1 * s, cy - 18 * s, 10 * s, 14 * s, rgba(255, 231, 160), { round = 2 })
  elseif k == "hrodgar" then
    for i = -1, 1 do
      c:rect(cx - 12 * s + i * 14 * s, cy - 18 * s + math.abs(i) * 4 * s, 22 * s, 32 * s, rgba(239, 226, 196), { round = 3 })
      c:rect(cx - 12 * s + i * 14 * s, cy - 18 * s + math.abs(i) * 4 * s, 22 * s, 32 * s, rgba(59, 42, 28), { round = 3, line = 1 })
    end
    c:tri(cx + 2 * s, cy - 8 * s, cx + 8 * s, cy, cx + 2 * s, cy + 8 * s, rgba(179, 58, 42))
    c:tri(cx + 2 * s, cy - 8 * s, cx - 4 * s, cy, cx + 2 * s, cy + 8 * s, rgba(179, 58, 42))
  elseif k == "dagny" then
    for r = 22, 6, -6 do c:circle(cx, cy - 12 * s, r * s, rgba(255, 214, 120, 26)) end
    c:rect(cx - 5 * s, cy - 6 * s, 10 * s, 26 * s, rgba(244, 236, 214), { round = 2 })
    c:tri(cx - 3 * s, cy - 8 * s, cx + 3 * s, cy - 8 * s, cx, cy - 18 * s, rgba(255, 190, 80))
  elseif k == "fenwick" then
    c:tri(cx - 18 * s, cy - 18 * s, cx + 18 * s, cy - 18 * s, cx, cy + 24 * s, rgba(200, 190, 140))
    c:tri(cx - 14 * s, cy - 15 * s, cx + 14 * s, cy - 15 * s, cx, cy + 18 * s, rgba(40, 56, 104))
  elseif k == "shaali" then
    for i = -2, 2 do
      local ox = i * 9 * s
      c:rect(cx - 9 * s + ox, cy - 16 * s + math.abs(i) * 3 * s, 18 * s, 28 * s, rgba(239, 226, 196), { round = 3 })
      c:rect(cx - 9 * s + ox, cy - 16 * s + math.abs(i) * 3 * s, 18 * s, 28 * s, rgba(90, 50, 24), { round = 3, line = 1 })
    end
    c:circle(cx, cy - 2 * s, 5 * s, rgba(155, 95, 214))
  elseif k == "kipiko" then
    local r = 20 * s
    c:tri(cx, cy - r, cx - r * 0.3, cy, cx + r * 0.3, cy, rgba(250, 214, 110))
    c:tri(cx, cy + r, cx - r * 0.3, cy, cx + r * 0.3, cy, rgba(250, 214, 110))
    c:tri(cx - r, cy, cx, cy - r * 0.3, cx, cy + r * 0.3, rgba(250, 214, 110))
    c:tri(cx + r, cy, cx, cy - r * 0.3, cx, cy + r * 0.3, rgba(250, 214, 110))
  else
    Art.crystal(c, cx, cy - 2 * s, h * 0.62, Art.ELEMENT[card.el] or CREAM, 0.55)
  end
end

local function detail_for(s)
  if s < 0.62 then return "mini" end
  if s < 1.1 then return "small" end
  return "big"
end

-- The four numbers as a diamond: top, right, bottom, left (A for 10), size their print, centred at (cx, cy)
function Art.numbers(c, cx, cy, sides, size, colour)
  local dx, dy = size * 9, size * 10
  local o = { outline = SHADOW, bold = true, align = "center", size = size, colour = colour or INK }
  local h = size * 13 -- the print's height, about
  c:text(cx, cy - dy - h / 2, cards.label(sides[1]), o)
  c:text(cx + dx, cy - h / 2, cards.label(sides[2]), o)
  c:text(cx, cy + dy - h / 2, cards.label(sides[3]), o)
  c:text(cx - dx, cy - h / 2, cards.label(sides[4]), o)
end

-- A card as a strip, w x Art.STRIP: its crystal, name, level and numbers in print you can read. In the cardbook's
-- hand list. o: selected.
Art.STRIP = 24

function Art.strip(c, x, y, w, id, o)
  o = o or {}
  local card = cards.get(id)
  local h = Art.STRIP
  if not card then return end
  local th = Art.THEMES[card.theme] or Art.THEMES.guild
  local foil = card.variant and Art.FOIL[card.variant]
  if o.selected then
    c:rect(x - 2, y - 2, w + 4, h + 4, rgba(255, 230, 150, 150), { round = 6 })
  end
  c:rect(x, y, w, h, th.bg, { round = 5, to = th.bg2 })
  c:rect(x, y, w, h, foil and foil.edge or th.accent, { round = 5, line = foil and 1.6 or 1 })
  Art.crystal(c, x + 10, y + h / 2, h * 0.7, Art.ELEMENT[card.el] or CREAM)
  local sides = o.sides or cards.printed(id)
  local nums = table.concat({ cards.label(sides[1]), cards.label(sides[2]), cards.label(sides[3]), cards.label(sides[4]) }, " ")
  local nw = Canvas.measure(nums, 0.78, true) + 10
  c:rect(x + w - nw - 3, y + 3, nw, h - 6, rgba(0, 0, 0, 110), { round = 4 })
  c:text(x + w - nw / 2 - 3, y + 3, nums, { size = 0.78, align = "center", bold = true, colour = th.accent })
  local lv = "Lv" .. card.level
  local lw = Canvas.measure(lv, 0.62) + 6
  local right = x + w - nw - 6 - lw
  c:text(right + 2, y + 6, lv, { size = 0.62, colour = th.muted })
  c:text(x + 20, y + 4, Canvas.fit(card.name, right - x - 22, 0.74, true), { size = 0.74, bold = true, colour = th.title, outline = SHADOW })
end

function Art.card(c, x, y, s, id, o)
  o = o or {}
  local card = cards.get(id)
  local w, h = Art.W * s, Art.H * s
  if not card then return w, h end
  local th = Art.THEMES[card.theme] or Art.THEMES.guild
  local detail = o.detail or detail_for(s)
  -- Print never smaller than this on a small card (the hand, the cardbook's pages): the art shrinks, the words don't
  local function ts(k, least) return detail ~= "big" and math.max(k * s, least) or k * s end
  local foil = card.variant and Art.FOIL[card.variant]
  local legend = card.rarity == 3
  local own = o.owner and Art.OWNER[o.owner]
  -- The light round foils and legendaries, and the selection
  if o.flipped then
    c:rect(x - 4 * s, y - 4 * s, w + 8 * s, h + 8 * s, rgba(255, 255, 255, 120), { round = 10 * s })
  elseif o.selected or o.staged then
    c:rect(x - 3 * s, y - 3 * s, w + 6 * s, h + 6 * s, rgba(255, 230, 150, 160), { round = 9 * s })
  elseif foil then
    c:rect(x - 2.5 * s, y - 2.5 * s, w + 5 * s, h + 5 * s, foil.glow, { round = 9 * s })
  elseif legend then
    c:rect(x - 2.5 * s, y - 2.5 * s, w + 5 * s, h + 5 * s, rgba(242, 215, 122, 70), { round = 9 * s })
  end
  -- The window (on the board: its owner's colour)
  c:rect(x, y, w, h, own and own.top or rgba(44, 61, 102), { round = 7 * s, to = own and own.bot or rgba(14, 21, 43) })
  local edge = own and own.edge or (foil and foil.edge or CREAM)
  c:rect(x, y, w, h, edge, { round = 7 * s, line = (own or foil) and 1.8 or 1 })
  c:rect(x + 3 * s, y + 3 * s, w - 6 * s, h - 6 * s, fade(foil and foil.edge or CREAM, 0.45), { round = 5 * s, line = 1 })
  if o.missing then
    c:rect(x + 4 * s, y + 4 * s, w - 8 * s, h - 8 * s, rgba(6, 8, 14, 200), { round = 5 * s })
    c:text(x + w / 2, y + h / 2 - 12 * s, "?", { size = 1.6 * s, align = "center", bold = true, colour = rgba(120, 126, 150, 200) })
    if detail ~= "mini" then
      c:text(x + w / 2, y + h - 22 * s, Canvas.fit("No. " .. card.id, w - 8 * s, 0.65 * s), { size = 0.65 * s, align = "center", colour = rgba(120, 126, 150, 200) })
    end
    return w, h
  end
  -- The header band: crystal, name, level
  local pad = 6 * s
  local hb = (detail == "mini" and 22 or 30) * s
  local bx, by, bw = x + pad, y + pad, w - pad * 2
  c:rect(bx, by, bw, hb, th.bg, { round = 4 * s, to = th.bg2 })
  local deco = DECO[card.theme]
  local lvText = detail == "mini" and tostring(card.level) or ("Lv" .. card.level)
  local lvSize = ts(0.55, 0.6)
  local chip = Canvas.measure(lvText, lvSize, true) + 8 * s
  if deco then deco(c, bx, by, bw - chip - 5 * s, hb, s) end
  c:rect(bx, by, bw, hb, th.accent, { round = 4 * s, line = 1 })
  Art.crystal(c, bx + 7 * s, by + hb / 2, hb * 0.62, Art.ELEMENT[card.el] or CREAM)
  c:rect(bx + bw - chip - 3 * s, by + (hb - 14 * s) / 2, chip, 14 * s, rgba(0, 0, 0, 120), { round = 3 * s })
  c:text(bx + bw - chip / 2 - 3 * s, by + (hb - 14 * s) / 2 + 1.5 * s, lvText, { size = lvSize, align = "center", bold = true, colour = th.accent })
  local nameW = bw - 14 * s - chip - 6 * s
  local nameSize = detail == "mini" and math.max(0.52 * s, 0.5) or ts(0.62, 0.64)
  c:text(bx + 14 * s, by + (detail == "mini" and 4 or 3) * s, Canvas.fit(card.name, nameW, nameSize, true), { size = nameSize, bold = true, colour = th.title, outline = SHADOW })
  if detail ~= "mini" then
    c:text(bx + 14 * s, by + 16 * s, Canvas.fit((card.crest and "Set crest" or th.label or "") .. "  ·  " .. (Art.ELEMENT_NAME[card.el] or ""), nameW, ts(0.45, 0.5)), { size = ts(0.45, 0.5), colour = th.muted })
  end
  local cy = by + hb + 4 * s
  -- The voice, under the header (big cards)
  if detail == "big" then
    local bh = 14 * s
    c:rect(bx, cy + 3 * s, bw, bh, rgba(0, 0, 0, 90), { round = 4 * s })
    c:rect(bx, cy + 3 * s, bw, bh, th.accent, { round = 4 * s, line = 1 })
    c:tri(bx + 8 * s, cy + 3 * s, bx + 14 * s, cy + 3 * s, bx + 10 * s, cy - 1 * s, th.accent)
    c:text(bx + 5 * s, cy + 5 * s, Canvas.fit('"' .. card.say .. '"', bw - 10 * s, 0.45 * s), { size = 0.45 * s, colour = INK })
    cy = cy + bh + 7 * s
  end
  -- The picture, the numbers over its top-left corner
  local bottomH = detail == "big" and 40 * s or (detail == "small" and 18 * s or 0)
  local ah = y + h - pad - bottomH - cy
  c:rect(bx, cy, bw, ah, fade(own and own.bot or th.bg2, 0.9), { round = 3 * s, to = rgba(8, 10, 20, 230) })
  c:rect(bx, cy, bw, ah, rgba(221, 211, 182, 110), { round = 3 * s, line = 1 })
  if o.portrait and card.model and mod and mod.model_portraits then
    c:rect(bx + bw * 0.2, cy + ah * 0.86, bw * 0.6, ah * 0.07, rgba(0, 0, 0, 70), { round = ah * 0.035 })
    c:portrait(bx + 1, cy + 1, bw - 2, ah - 2, { model = card.model, spin = 18, body = true })
  else
    picture(c, card, bx + bw * 0.15, cy, bw * 0.85, ah, s)
  end
  local numSize = detail == "big" and 1.05 * s or math.max(1.15 * s, detail == "mini" and 0.9 or 0.82)
  local nx, ny = bx + numSize * 9 + 6 * s, cy + numSize * 10 + numSize * 7 + 2 * s
  c:rect(bx + 2 * s, cy + 2 * s, numSize * 18 + 12 * s, numSize * 20 + numSize * 13 + 4 * s, rgba(0, 0, 0, 120), { round = 4 * s })
  Art.numbers(c, nx, ny, o.sides or cards.printed(id), numSize, (o.sides and own) and INK or INK)
  cy = cy + ah + 3 * s
  -- What it is (small and big cards)
  if detail ~= "mini" then
    local tags = {}
    if card.rarity == 2 then tags[#tags + 1] = "Rare" end
    if card.rarity == 3 then tags[#tags + 1] = "Rare" tags[#tags + 1] = "Ex" end
    if foil then tags[#tags + 1] = foil.label end
    c:text(bx + 1, cy, Canvas.fit("[" .. card.fam .. "] " .. card.species, bw * (detail == "big" and 0.62 or 1), ts(0.5, 0.54)), { size = ts(0.5, 0.54), colour = GOLDTXT })
    if detail == "big" then
      local tx = bx + bw
      for i = #tags, 1, -1 do
        local tw = Canvas.measure(tags[i], 0.42 * s) + 6 * s
        tx = tx - tw
        c:rect(tx, cy, tw, 10 * s, rgba(0, 0, 0, 70), { round = 2 * s })
        c:rect(tx, cy, tw, 10 * s, CREAM, { round = 2 * s, line = 1 })
        c:text(tx + tw / 2, cy + 1, tags[i], { size = 0.42 * s, align = "center", colour = rgba(242, 215, 122) })
        tx = tx - 2 * s
      end
      cy = cy + 13 * s
      c:text(bx + 1, cy, ("Level %d  ·  %s"):format(card.level, Art.ELEMENT_NAME[card.el] or ""), { size = 0.48 * s, colour = SKY })
      if card.variant then
        cy = cy + 11 * s
        c:text(bx + 1, cy, Canvas.fit(cards.VARIANTS[card.variant].name .. ": " .. cards.VARIANTS[card.variant].text, bw, 0.44 * s), { size = 0.44 * s, colour = INK })
      end
    end
  end
  if o.count and o.count > 1 then
    local cw = 18 * s
    c:circle(x + w - 4 * s, y + 4 * s, cw / 2, rgba(30, 22, 14, 240))
    c:circle(x + w - 4 * s, y + 4 * s, cw / 2, th.accent, 1)
    c:text(x + w - 4 * s, y - 1 * s, "x" .. o.count, { size = ts(0.5, 0.56), align = "center", bold = true, colour = INK })
  end
  return w, h
end

-- A card back, for a hidden hand: Windurst's card soldiers' four suits on red
function Art.back(c, x, y, s)
  local w, h = Art.W * s, Art.H * s
  c:rect(x, y, w, h, rgba(201, 162, 90), { round = 7 * s, to = rgba(110, 80, 34) })
  c:rect(x + 3 * s, y + 3 * s, w - 6 * s, h - 6 * s, rgba(122, 38, 32), { round = 5 * s, to = rgba(58, 15, 12) })
  c:rect(x + 3 * s, y + 3 * s, w - 6 * s, h - 6 * s, rgba(201, 162, 90), { round = 5 * s, line = 1 })
  local cx, cy = x + w / 2, y + h * 0.45
  local r = 16 * s
  c:tri(cx, cy - r, cx - r * 0.6, cy, cx + r * 0.6, cy, rgba(232, 196, 122))
  c:tri(cx, cy + r, cx - r * 0.6, cy, cx + r * 0.6, cy, rgba(232, 196, 122))
  for _, p in ipairs({ { -1, -1 }, { 1, -1 }, { -1, 1 }, { 1, 1 } }) do c:circle(cx + p[1] * 22 * s, cy + p[2] * 30 * s, 4 * s, rgba(232, 196, 122, 200)) end
  if s >= 0.5 then
    c:text(cx, y + h - 26 * s, "CARDIAN", { size = 0.62 * s, align = "center", bold = true, colour = rgba(232, 196, 122) })
  end
  return w, h
end

return Art
