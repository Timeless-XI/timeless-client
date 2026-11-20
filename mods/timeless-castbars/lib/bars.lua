-- Draws a bar over each casting monster's head: the spell or move's name, how long until it lands, and a fill in its
-- element's colour (moves orange). A red frame when it is aimed at you or hits an area. Interrupted, it turns red
-- for a moment ("Interrupted!"); landed, it flashes full.
-- HUD elements are made once and then only changed; what wasn't drawn this frame is removed.

local B = {}

B.huds = {}
local now_hud = {}

B.ELEMENT = {
  [0] = { 255, 110, 60 },  -- fire
  [1] = { 140, 210, 255 }, -- ice
  [2] = { 120, 230, 140 }, -- wind
  [3] = { 210, 170, 90 },  -- earth
  [4] = { 200, 130, 255 }, -- lightning
  [5] = { 80, 150, 255 },  -- water
  [6] = { 255, 245, 190 }, -- light
  [7] = { 170, 110, 230 }, -- dark
}
B.SPELL = { 150, 170, 255 }
B.SKILL = { 255, 160, 60 }
B.CUT = { 255, 70, 60 }

local function put(kind, id, spec)
  now_hud[id] = kind
  if B.huds[id] == kind then
    hud.set(id, spec)
  else
    hud[kind](id, spec)
  end
end

function B.begin() now_hud = {} end

function B.finish()
  for id in pairs(B.huds) do if not now_hud[id] then pcall(hud.remove, id) end end
  B.huds = now_hud
end

function B.clear()
  B.begin()
  B.finish()
end

local function a(c, alpha) return { c[1], c[2], c[3], alpha } end

-- What a cast is called and how it looks: {name, colour, area}
function B.describe(c)
  if c.kind == "spell" then
    local s = res.spell(c.id)
    local colour = s and B.ELEMENT[s.element] or B.SPELL
    return { name = (s and s.name) or ("Spell " .. c.id), colour = colour, area = s and s.area or false }
  end
  local name
  if c.id >= 256 then
    local ok, n = pcall(res.mob_skill, c.id)
    name = ok and n or nil
  else
    local ab = res.ability(c.id)
    name = ab and ab.name or nil
  end
  return { name = name or "a move", colour = B.SKILL, area = false }
end

-- One bar. o: {scale, me (our index), now, progress(c) -> frac, left}
function B.draw(key, c, e, o)
  local p = draw.to_screen({ x = e.pos.x, y = e.pos.y - math.max(1, math.min(e.height or 2, 12)) - 0.4, z = e.pos.z })
  if not p or not p.on_screen then return end
  local d = B.describe(c)
  local s = o.scale
  local w = math.floor(130 * s + 0.5)
  local h = math.max(4, math.floor(8 * s + 0.5))
  local x = math.floor(p.x - w / 2)
  local y = math.floor(p.y)
  local frac, left = o.progress(c)
  local colour = d.colour
  local fill = frac or 1
  local alpha = 230
  local label = d.name
  local right = left and ("%.1f"):format(left) or ""
  if c.state == "cut" then
    colour, fill, label, right = B.CUT, frac or 1, "Interrupted!", ""
  elseif c.state == "done" then
    fill, right = 1, ""
    alpha = math.floor(230 * math.max(0, 1 - (o.now - c.at) / 0.6))
  elseif not frac then
    alpha = math.floor(120 + 100 * (0.5 + 0.5 * math.sin(o.now * 6))) -- time not known: it pulses
  end
  local aimed = c.target ~= nil and c.target == o.me
  if aimed and c.state == "casting" then label = label .. "  > you" end
  local warn = c.state == "casting" and (aimed or d.area)
  put("rect", key .. "bg", { x = x - 1, y = y - 1, w = w + 2, h = h + 2, colour = { 8, 10, 16, 200 }, round = 2, layer = 1 })
  put("rect", key .. "fill", { x = x, y = y, w = math.max(1, math.floor(w * fill + 0.5)), h = h, colour = a(colour, alpha),
    round = 2, layer = 2 })
  put("rect", key .. "warn", { x = x - 2, y = y - 2, w = w + 4, h = h + 4, colour = { 255, 70, 50, 230 }, line = 2, round = 3,
    visible = warn or false, layer = 3 })
  local ts = math.max(0.6, 0.85 * s)
  put("text", key .. "name", { x = x, y = y - math.floor(15 * s + 0.5), text = label, size = ts, colour = { 245, 240, 230, 255 },
    outline = { 0, 0, 0, 220 }, layer = 4 })
  put("text", key .. "left", { x = x + w, y = y - math.floor(15 * s + 0.5), text = right, size = ts, align = "right",
    colour = { 255, 230, 160, 255 }, outline = { 0, 0, 0, 220 }, layer = 4 })
  if d.area and c.state == "casting" then
    put("text", key .. "area", { x = math.floor(p.x), y = y + h + 2, text = "AREA", size = ts * 0.85, align = "center",
      colour = { 255, 120, 90, 255 }, outline = { 0, 0, 0, 220 }, layer = 4 })
  end
end

return B
