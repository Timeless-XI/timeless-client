-- How a card looks: the game's own item window (a navy panel, a cream border, white text with a shadow, abilities
-- in gear wording), wearing a header dressed for its family the way the shop windows dress for their shopkeeper
-- (the Quadav's shell pattern, the Goblin's patched leather, the guild's charcoal and gold for notorious monsters,
-- Galkan stone and Hume planks for townsfolk...). In the big card the picture is the monster's real model from your
-- game files, turning slowly on a clear backdrop (TimelessXI client); small cards show its element's crystal.
--
-- Art.card(c, x, y, s, id, o) draws card `id` at (x, y), s times 130 x 190 units. o: portrait (the live model),
-- count (a badge), missing (not in your collection: a dark silhouette), selected / staged (a highlight), power and
-- bonus (its power on the table, and how much of it is bonus), detail ("mini", "small" or "big"; by size if not given).

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

function Art.card(c, x, y, s, id, o)
  o = o or {}
  local card = cards.get(id)
  local w, h = Art.W * s, Art.H * s
  if not card then return w, h end
  local th = Art.THEMES[card.theme] or Art.THEMES.guild
  local detail = o.detail or detail_for(s)
  local foil = card.variant and Art.FOIL[card.variant]
  local legend = card.rarity == 3
  -- The light round foils and legendaries, and the selection
  if o.selected or o.staged then
    c:rect(x - 3 * s, y - 3 * s, w + 6 * s, h + 6 * s, o.staged and rgba(120, 214, 140, 120) or rgba(255, 230, 150, 140), { round = 9 * s })
  elseif foil then
    c:rect(x - 2.5 * s, y - 2.5 * s, w + 5 * s, h + 5 * s, foil.glow, { round = 9 * s })
  elseif legend then
    c:rect(x - 2.5 * s, y - 2.5 * s, w + 5 * s, h + 5 * s, rgba(242, 215, 122, 70), { round = 9 * s })
  end
  -- The window
  c:rect(x, y, w, h, rgba(44, 61, 102), { round = 7 * s, to = rgba(14, 21, 43) })
  c:rect(x, y, w, h, foil and foil.edge or CREAM, { round = 7 * s, line = foil and 1.6 or 1 })
  c:rect(x + 3 * s, y + 3 * s, w - 6 * s, h - 6 * s, fade(foil and foil.edge or CREAM, 0.45), { round = 5 * s, line = 1 })
  if o.missing then
    c:rect(x + 4 * s, y + 4 * s, w - 8 * s, h - 8 * s, rgba(6, 8, 14, 200), { round = 5 * s })
    c:text(x + w / 2, y + h / 2 - 12 * s, "?", { size = 1.6 * s, align = "center", bold = true, colour = rgba(120, 126, 150, 200) })
    if detail ~= "mini" then
      c:text(x + w / 2, y + h - 22 * s, Canvas.fit("No. " .. card.id, w - 8 * s, 0.65 * s), { size = 0.65 * s, align = "center", colour = rgba(120, 126, 150, 200) })
    end
    return w, h
  end
  -- The header band
  local pad = 6 * s
  local hb = (detail == "mini" and 22 or 30) * s
  local bx, by, bw = x + pad, y + pad, w - pad * 2
  c:rect(bx, by, bw, hb, th.bg, { round = 4 * s, to = th.bg2 })
  local deco = DECO[card.theme]
  local chip = 16 * s
  -- The dressing keeps clear of the cost chip
  if deco then deco(c, bx, by, bw - chip - 5 * s, hb, s) end
  c:rect(bx, by, bw, hb, th.accent, { round = 4 * s, line = 1 })
  Art.crystal(c, bx + 7 * s, by + hb / 2, hb * 0.62, Art.ELEMENT[card.el] or CREAM)
  local cost = card.cost
  c:rect(bx + bw - chip - 3 * s, by + (hb - chip) / 2, chip, chip, rgba(0, 0, 0, 120), { round = 3 * s })
  c:rect(bx + bw - chip - 3 * s, by + (hb - chip) / 2, chip, chip, th.accent, { round = 3 * s, line = 1 })
  c:text(bx + bw - chip / 2 - 3 * s, by + (hb - chip) / 2 + 1.5 * s, tostring(cost), { size = 0.7 * s, align = "center", bold = true, colour = th.accent })
  local nameW = bw - 14 * s - chip - 6 * s
  local nameSize = (detail == "mini" and 0.52 or 0.62) * s
  c:text(bx + 14 * s, by + (detail == "mini" and 4 or 3) * s, Canvas.fit(card.name, nameW, nameSize, true), { size = nameSize, bold = true, colour = th.title, outline = SHADOW })
  if detail ~= "mini" then
    c:text(bx + 14 * s, by + 16 * s, Canvas.fit((th.label or "") .. "  ·  " .. (Art.ELEMENT_NAME[card.el] or ""), nameW, 0.45 * s), { size = 0.45 * s, colour = th.muted })
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
  -- The picture
  local wordy = #(card.text or {}) + (card.variant and 1 or 0) >= 2
  local ah = (detail == "mini" and 0.36 or (detail == "big" and (wordy and 0.29 or 0.36) or 0.42)) * h
  c:rect(bx, cy, bw, ah, fade(th.bg2, 0.9), { round = 3 * s, to = rgba(8, 10, 20, 230) })
  c:rect(bx, cy, bw, ah, rgba(221, 211, 182, 110), { round = 3 * s, line = 1 })
  if o.portrait and card.model and mod and mod.model_portraits then
    c:rect(bx + bw * 0.2, cy + ah * 0.84, bw * 0.6, ah * 0.08, rgba(0, 0, 0, 70), { round = ah * 0.04 })
    c:portrait(bx + 1, cy + 1, bw - 2, ah - 2, { model = card.model, spin = 18, body = true })
  else
    picture(c, card, bx, cy, bw, ah, s)
  end
  cy = cy + ah + 3 * s
  -- What it is
  if detail ~= "mini" then
    local tags = {}
    if card.rarity == 2 then tags[#tags + 1] = "Rare" end
    if card.rarity == 3 then tags[#tags + 1] = "Rare" tags[#tags + 1] = "Ex" end
    if foil then tags[#tags + 1] = foil.label end
    c:text(bx + 1, cy, Canvas.fit("[" .. card.fam .. "] " .. card.species, bw * 0.62, 0.5 * s), { size = 0.5 * s, colour = GOLDTXT })
    local tx = bx + bw
    for i = #tags, 1, -1 do
      local tw = Canvas.measure(tags[i], 0.42 * s) + 6 * s
      tx = tx - tw
      c:rect(tx, cy, tw, 10 * s, rgba(0, 0, 0, 70), { round = 2 * s })
      c:rect(tx, cy, tw, 10 * s, CREAM, { round = 2 * s, line = 1 })
      c:text(tx + tw / 2, cy + 1, tags[i], { size = 0.42 * s, align = "center", colour = rgba(242, 215, 122) })
      tx = tx - 2 * s
    end
    cy = cy + 12 * s
  end
  -- What it does (big cards; small ones show the first ability's name)
  local textBottom = y + h - pad - 16 * s
  if detail == "big" then
    local lines = {}
    for _, t in ipairs(card.text) do lines[#lines + 1] = { t[1], t[2] } end
    if card.variant then lines[#lines + 1] = { cards.VARIANTS[card.variant].name, cards.VARIANTS[card.variant].text } end
    -- The words shrink a little when a card has a lot to say (a foil adds its own line)
    local size, step
    for _, k in ipairs({ 0.48, 0.44, 0.40, 0.37 }) do
      size, step = k * s, (k + 0.015) / 0.48 * 9.5 * s
      local need = 0
      for _, t in ipairs(lines) do need = need + #Canvas.wrap(t[1] .. ": " .. t[2], bw - 2 * s, size) * step + 1.5 * s end
      if cy + need <= textBottom + 2 * s then break end
    end
    for _, t in ipairs(lines) do
      local wrapped = Canvas.wrap(t[1] .. ": " .. t[2], bw - 2 * s, size)
      for i, ln in ipairs(wrapped) do
        if cy + step - 0.5 * s > textBottom then break end
        if i == 1 then
          local head = t[1] .. ":"
          c:text(bx + 1, cy, head, { size = size, colour = SKY })
          c:text(bx + 1 + Canvas.measure(head .. " ", size), cy, Canvas.fit(ln:sub(#head + 2), bw - Canvas.measure(head .. " ", size), size), { size = size, colour = INK })
        else
          c:text(bx + 1, cy, ln, { size = size, colour = INK })
        end
        cy = cy + step
      end
      cy = cy + 1.5 * s
    end
  elseif detail == "small" and card.text[1] then
    c:text(bx + 1, cy, Canvas.fit(card.text[1][1], bw, 0.5 * s), { size = 0.5 * s, colour = SKY })
  end
  -- Power, in the theme's chip (on the table: its power now, the bonus in green or red)
  local pw = o.power or card.power
  local label = detail == "mini" and tostring(pw) or ("Power " .. pw)
  local lw = Canvas.measure(label, 0.6 * s, true) + 10 * s
  local px, py = x + w - pad - lw, y + h - pad - 13 * s
  c:rect(px, py, lw, 13 * s, th.bg, { round = 3 * s, to = th.bg2 })
  c:rect(px, py, lw, 13 * s, th.accent, { round = 3 * s, line = 1 })
  c:text(px + lw / 2, py + 1.5 * s, label, { size = 0.6 * s, align = "center", bold = true, colour = th.accent })
  if o.bonus and o.bonus ~= 0 then
    local b = (o.bonus > 0 and "+" or "") .. o.bonus
    local bw2 = Canvas.measure(b, 0.5 * s, true) + 6 * s
    c:rect(px - bw2 - 2 * s, py + 1.5 * s, bw2, 10 * s, o.bonus > 0 and rgba(47, 107, 58) or rgba(140, 46, 40), { round = 3 * s })
    c:text(px - bw2 / 2 - 2 * s, py + 2 * s, b, { size = 0.5 * s, align = "center", bold = true, colour = rgba(240, 248, 236) })
  end
  if o.silenced then
    c:text(x + pad + 1, py + 1.5 * s, "silenced", { size = 0.45 * s, colour = rgba(236, 160, 150) })
  end
  if o.count and o.count > 1 then
    local cw = 18 * s
    c:circle(x + w - 4 * s, y + 4 * s, cw / 2, rgba(30, 22, 14, 240))
    c:circle(x + w - 4 * s, y + 4 * s, cw / 2, th.accent, 1)
    c:text(x + w - 4 * s, y - 1 * s, "x" .. o.count, { size = 0.5 * s, align = "center", bold = true, colour = INK })
  end
  return w, h
end

-- A card back (the Steaming Sheep's), for the opponent's hand and face-down plays
function Art.back(c, x, y, s)
  local w, h = Art.W * s, Art.H * s
  c:rect(x, y, w, h, rgba(201, 162, 90), { round = 7 * s, to = rgba(110, 80, 34) })
  c:rect(x + 3 * s, y + 3 * s, w - 6 * s, h - 6 * s, rgba(122, 38, 32), { round = 5 * s, to = rgba(58, 15, 12) })
  c:rect(x + 3 * s, y + 3 * s, w - 6 * s, h - 6 * s, rgba(201, 162, 90), { round = 5 * s, line = 1 })
  local cx, cy = x + w / 2, y + h * 0.45
  for _, p in ipairs({ { -10, 4 }, { 0, -2 }, { 10, 4 }, { -5, 10 }, { 5, 10 } }) do c:circle(cx + p[1] * s, cy + p[2] * s, 9 * s, rgba(239, 226, 196)) end
  c:circle(cx + 19 * s, cy + 2 * s, 6 * s, rgba(59, 42, 28))
  if s >= 0.5 then
    c:text(cx, y + h - 26 * s, "IRONHAND", { size = 0.62 * s, align = "center", bold = true, colour = rgba(232, 196, 122) })
  end
  return w, h
end

return Art
