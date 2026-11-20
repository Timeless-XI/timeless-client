-- Draws the Champions: auras (a ground ring and a light), affix badges over their heads, hazards and rally lines.
-- Every shape, light and HUD element drawn in a frame is noted; what was drawn last frame but not this one is removed,
-- so a Champion that dies, leaves or turns a feature off leaves nothing behind.
--
-- World coordinates: x east, z north, y the height pointing down (a smaller y is higher).

local A = require("lib.affixes")

local S = {}

S.drawn, S.huds, S.lights = {}, {}, {}
local now_draw, now_hud, now_light = {}, {}, {}

local TAU = 2 * math.pi

local function clamp(v) return math.max(0, math.min(255, math.floor(v + 0.5))) end
local function col(c, a) return { clamp(c[1]), clamp(c[2]), clamp(c[3]), clamp(a) } end
local function mix(a, b, t) return { a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t } end
local function at(p, up) return { x = p.x, y = p.y - (up or 0), z = p.z } end

local function ring(id, spec) now_draw[id] = true draw.ring(id, spec) end
local function line(id, spec) now_draw[id] = true draw.line(id, spec) end
local function text(id, spec) now_draw[id] = true draw.text(id, spec) end

-- A HUD element: made the first time, then only changed (so its tooltip stays up while the pointer rests on it).
local function put(kind, id, spec)
  now_hud[id] = kind
  if S.huds[id] == kind then
    hud.set(id, spec)
  else
    hud[kind](id, spec)
  end
end

function S.begin()
  now_draw, now_hud, now_light = {}, {}, {}
end

-- Removes what this frame didn't draw.
function S.finish()
  for id in pairs(S.drawn) do if not now_draw[id] then draw.remove(id) end end
  for id in pairs(S.huds) do if not now_hud[id] then pcall(hud.remove, id) end end
  for id in pairs(S.lights) do if not now_light[id] then render.light(id, nil) end end
  S.drawn, S.huds = now_draw, now_hud
  local kept = {}
  for id in pairs(now_light) do kept[id] = S.lights[id] end
  S.lights = kept
end

function S.clear()
  S.begin()
  S.finish()
end

-- The Champion's colour now: its rank's, pulsing toward red while enraged. Also the pulse (0..1).
function S.colour(c, t)
  local rank = A.RANK[c.rank] or A.RANK.champion
  if c.raged then
    local p = 0.5 + 0.5 * math.sin(t * 7)
    return mix(rank.colour, A.RAGE, 0.55 + 0.45 * p), p
  end
  return rank.colour, 0.5 + 0.5 * math.sin(t * 2)
end

local function radius(e) return math.max(0.8, math.min(e.hitbox or 1, 6)) + 0.8 end
local function height(e) return math.max(1, math.min(e.height or 2, 12)) end

-- The aura: a disc and its edge on the ground, an outer ring and turning marks for Elite and Legendary, and a light.
function S.aura(key, c, e, o, t)
  local rank = A.RANK[c.rank] or A.RANK.champion
  local colour, pulse = S.colour(c, t)
  local op = o.opacity
  local R = radius(e) * (c.raged and (1 + 0.08 * pulse) or 1)
  local g = at(e.pos, 0.06)
  ring(key .. "a_disc", { at = g, radius = R, filled = true, segments = 64, colour = col(colour, (24 + 16 * pulse) * op) })
  ring(key .. "a_edge", { at = g, radius = R, segments = 64, colour = col(colour, (170 + 60 * pulse) * op) })
  if c.element and A.ELEMENT[c.element] then
    ring(key .. "a_elem", { at = g, radius = math.max(0.3, R - 0.3), segments = 64, colour = col(A.ELEMENT[c.element].colour, 200 * op) })
  end
  if c.rank ~= "champion" then
    ring(key .. "a_out", { at = g, radius = R + 0.35, segments = 64, colour = col(colour, 90 * op) })
  end
  if c.rank == "legendary" then
    local turn = t * 0.6
    for i = 1, 6 do
      local h = turn + (i - 1) * TAU / 6
      local dx, dz = math.cos(h), -math.sin(h)
      line(key .. "a_rune" .. i, { from = { x = g.x + dx * (R + 0.35), y = g.y, z = g.z + dz * (R + 0.35) },
        to = { x = g.x + dx * (R + 0.85), y = g.y, z = g.z + dz * (R + 0.85) }, colour = col(colour, 210 * op) })
    end
  end
  if o.lights then
    local id = key .. "light"
    now_light[id] = true
    local intensity = rank.intensity * (c.raged and (0.8 + 0.6 * pulse) or 1)
    local light = { entity = e.id, height = height(e) * 0.5, colour = { clamp(c.raged and colour[1] or rank.light[1]),
      clamp(c.raged and colour[2] or rank.light[2]), clamp(c.raged and colour[3] or rank.light[3]) }, range = rank.range,
      intensity = intensity, halo = rank.halo }
    local last = S.lights[id]
    -- Lights are only sent again when they change enough (a rage pulses ten times a second)
    if not last or last.entity ~= e.id or math.abs(last.intensity - intensity) > 0.4 or last.colour[1] ~= light.colour[1]
      or last.colour[2] ~= light.colour[2] or (o.clock or 0) - (last.at or 0) > 5 then
      light.at = o.clock
      S.lights[id] = light
      render.light(id, { entity = light.entity, height = light.height, colour = light.colour, range = light.range,
        intensity = light.intensity, halo = light.halo })
    end
  end
end

-- What its affixes do around it, drawn where it matters.
function S.hazards(key, c, e, o, t, find)
  local op = o.opacity
  local R = radius(e)
  local H = height(e)
  local g = at(e.pos, 0.06)

  -- Alpha before its fight: how far it calls its kind from
  if c.pack and not c.fighting then
    local C = { 255, 150, 60 }
    ring(key .. "h_pk_fill", { at = g, radius = c.pack, filled = true, segments = 96, colour = col(C, 14 * op) })
    ring(key .. "h_pk", { at = g, radius = c.pack, segments = 96, colour = col(C, 130 * op) })
    if o.labels then
      text(key .. "h_pk_t", { at = { x = g.x, y = g.y - 0.4, z = g.z - c.pack }, text = "Rally range", size = 0.8,
        colour = { 255, 210, 160, 230 }, outline = { 60, 20, 0, 220 } })
    end
  end

  -- Thorned: thorns round it at arm's length, turning
  if c.has.thorned then
    local C = { 255, 120, 60 }
    for i = 1, 12 do
      local h = -t * 0.8 + (i - 1) * TAU / 12
      local dx, dz = math.cos(h), -math.sin(h)
      local y = g.y - 0.25
      line(key .. "h_th" .. i, { from = { x = g.x + dx * (R + 0.05), y = y, z = g.z + dz * (R + 0.05) },
        to = { x = g.x + dx * (R + 0.75), y = y - 0.35, z = g.z + dz * (R + 0.75) }, colour = col(C, 200 * op) })
    end
  end

  -- Shielded and Warded: a dome of rings, steel for weapons, blue for magic
  local domes = {}
  if c.has.shielded then domes[#domes + 1] = { id = "sh", colour = { 235, 215, 150 }, grow = 0 } end
  if c.has.warded then domes[#domes + 1] = { id = "wd", colour = { 120, 175, 255 }, grow = #domes > 0 and 0.3 or 0 } end
  for _, d in ipairs(domes) do
    local shimmer = 0.7 + 0.3 * math.sin(t * 3 + #d.id)
    for i, f in ipairs({ { 0.25, 1.0 }, { 0.6, 0.85 }, { 0.9, 0.5 } }) do
      ring(key .. "h_" .. d.id .. i, { at = at(e.pos, H * f[1]), radius = (R + d.grow) * f[2], segments = 48,
        colour = col(d.colour, 85 * shimmer * op) })
    end
  end

  -- Venomous: a green haze round it
  if c.has.venomous then
    local C = { 120, 230, 90 }
    local w = R + 1.3 + 0.15 * math.sin(t * 2.3)
    ring(key .. "h_vn_fill", { at = g, radius = w, filled = true, segments = 48, colour = col(C, 22 * op) })
    ring(key .. "h_vn", { at = g, radius = w, segments = 48, colour = col(C, 80 * op) })
  end

  -- Renewing: while it heals, green rings rise up it
  if c.has.regen and c.regen then
    local C = { 110, 255, 150 }
    for i = 1, 2 do
      local f = ((t * 0.7) + (i - 1) * 0.5) % 1
      ring(key .. "h_rn" .. i, { at = at(e.pos, H * f), radius = R * (0.75 - 0.3 * f), segments = 40,
        colour = col(C, 150 * (1 - f) * op) })
    end
  end

  -- Blinking with shadows up: pale rings drifting round its middle
  if c.has.blinking and c.shadows then
    for i = 1, 3 do
      local h = t * 1.5 + (i - 1) * TAU / 3
      ring(key .. "h_bl" .. i, { at = { x = e.pos.x + math.cos(h) * 0.35, y = e.pos.y - H * 0.5, z = e.pos.z - math.sin(h) * 0.35 },
        radius = R * 0.7, segments = 40, colour = col({ 230, 235, 255 }, 70 * op) })
    end
  end

  -- Vampiric in a fight: the life it drains, from whoever it fights to it
  if c.has.vampiric and c.fighting and e.target_index then
    local v = find(e.target_index)
    if v then
      local a = at(e.pos, H * 0.55)
      local b = at(v.pos, height(v) * 0.55)
      local C = { 210, 30, 60 }
      line(key .. "h_vp", { from = b, to = a, colour = col(C, (110 + 60 * math.sin(t * 4)) * op) })
      for i = 1, 3 do
        local f = ((t * 0.9) + (i - 1) / 3) % 1
        ring(key .. "h_vp" .. i, { at = { x = b.x + (a.x - b.x) * f, y = b.y + (a.y - b.y) * f, z = b.z + (a.z - b.z) * f },
          radius = 0.18, filled = true, segments = 12, colour = col(C, 230 * op) })
      end
    end
  end
end

-- Lines from an Alpha to the monsters it rallied, and a ring under each.
function S.rally(key, c, e, o, t, find)
  if #(c.rallied or {}) == 0 then return end
  local op = o.opacity
  local C = { 255, 160, 70 }
  local a = at(e.pos, height(e) * 0.5)
  for _, index in ipairs(c.rallied) do
    local m = find(index)
    if m and (m.hpp or 1) > 0 then
      local b = at(m.pos, height(m) * 0.5)
      line(key .. "r_l" .. index, { from = a, to = b, colour = col(C, (80 + 40 * math.sin(t * 3)) * op) })
      ring(key .. "r_r" .. index, { at = at(m.pos, 0.06), radius = radius(m) - 0.3, segments = 40, colour = col(C, 120 * op) })
    end
  end
end

-- The badges over its head: the game's icons for its affixes, each with a tooltip.
function S.badges(key, c, e, o)
  local list = A.badges(c)
  if #list == 0 then return end
  local p = draw.to_screen(at(e.pos, height(e) + 1.1))
  if not p or not p.on_screen then return end
  local s = hud.screen()
  local size = math.floor(20 * (s.ui_scale or 1) + 0.5)
  local gap = math.max(2, math.floor(size / 6))
  local w = #list * size + (#list - 1) * gap
  local x0 = math.floor(p.x - w / 2)
  local y = math.floor(p.y - size)
  local rank = A.RANK[c.rank] or A.RANK.champion
  for i, b in ipairs(list) do
    local x = x0 + (i - 1) * (size + gap)
    local id = key .. "b" .. i
    local tip = { title = ("%s  (%s)"):format(b.title, rank.name), text = b.text }
    local frame = b.alert and { 255, 70, 50, 240 } or { 0, 0, 0, 190 }
    put("rect", id .. "f", { x = x - 2, y = y - 2, w = size + 4, h = size + 4, colour = frame, line = 2, round = 3, layer = 1 })
    put("icon", id, { x = x, y = y, kind = "status", id = b.icon, size = size, tip = tip, layer = 2 })
    put("rect", id .. "d", { x = x, y = y, w = size, h = size, colour = { 10, 10, 14, 160 }, visible = b.dim, layer = 3 })
  end
end

return S
