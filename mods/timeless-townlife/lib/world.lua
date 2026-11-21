-- The town in the world: speech bubbles over townsfolk, the lamplighter's lamps, carried lights and sounds by place. (The chimney smoke
-- it once drew floated where no chimney stands: the zone has its own.)

local W = {}

W.BUBBLE_SECONDS = 5
W.SOUND_EVERY = { hammer = 5, crowd = 4.5, gulls = 6, music = 12, cards = 6, docks = 5 } -- seconds between plays (about the sound's length)
W.SOUND_VOLUME = { hammer = 0.45, crowd = 0.35, gulls = 0.4, music = 0.5, cards = 0.55, docks = 0.5 }

-- Real lights (TimelessXI's Night Lighting Modern: render.lamp_zone / render.light): the zone's lamp posts by the
-- lamplighter's round stay dark at night until he lights them, then fade on; his lantern and the guards' torches give
-- light. Without them (Original lighting, an older client) the lamps are the game's own.
W.LAMP_FADE = 1.5 -- seconds a lamp takes to come on
W.CARRY = {
  lantern = { colour = { 255, 196, 120 }, range = 7, intensity = 7, halo = 0.45, height = 1.3 },
  torch = { colour = { 255, 150, 70 }, range = 9, intensity = 10, halo = 0.6, height = 2.0 },
  candle = { colour = { 255, 180, 100 }, range = 4, intensity = 4, halo = 0.3, height = 0.9 }, -- on the card table
}
W.SNAP = 8          -- yalms: a lamplighter's lamp is the zone's own lamp post this close to it (none: nothing shows)
W.ZONE_RADIUS = 1.5 -- yalms round that lamp post's glow

local drawn = {}    -- draw id -> true
local lights = {}   -- render.light id -> true
local zones = {}    -- render.lamp_zone id -> true
local bubbled = {}  -- index -> true
local next_sound = {} -- scenery n -> clock

local function setting(k)
  local ok, v = pcall(settings.get, k)
  if not ok or v == nil then return true end
  return v
end

local function dist(a, b)
  local dx, dy, dz = a.x - b.x, a.y - b.y, a.z - b.z
  return math.sqrt(dx * dx + dy * dy + dz * dz)
end

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
  if #lines > 3 then lines = { lines[1], lines[2], lines[3] .. "…" } end
  return lines
end

local function clear_bubble(index)
  if bubbled[index] then
    hud.remove("tlb_" .. index)
    hud.remove("tlbt_" .. index)
    bubbled[index] = nil
  end
end

function W.bubbles(model, now)
  local u = 1
  local okS, sc = pcall(hud.screen)
  if okS and sc then u = sc.ui_scale or 1 end
  local on = setting("bubbles")
  for index, b in pairs(model.bubbles) do
    local n = utf8.len(b.text) or #b.text
    local until_ = b.at + W.BUBBLE_SECONDS + math.min(4, n / 30)
    local p
    if on and now < until_ then
      local ok, e = pcall(game.entity, index)
      if ok and e and e.pos then
        local top = { x = e.pos.x, y = e.pos.y - (e.height or 1.6) - 0.3, z = e.pos.z }
        local okp, sp = pcall(draw.to_screen, top)
        if okp and sp and sp.on_screen ~= false then p = sp end
      end
    end
    if not p then
      clear_bubble(index)
      if now >= until_ then model.bubbles[index] = nil end
    else
      local lines = W.wrap(b.text, 34)
      local longest = 0
      for _, l in ipairs(lines) do longest = math.max(longest, utf8.len(l) or #l) end
      local w = math.floor((longest * 6.6 + 24) * u)
      local h = math.floor((#lines * 15 + 12) * u)
      local x, y = p.x - w / 2, p.y - h - 26 * u
      hud.bubble("tlb_" .. index, { x = x, y = y, w = w, h = h, round = 9 * u, colour = { 250, 244, 228, 235 },
        edge = { 150, 110, 60, 220 }, edge_w = 1.5 * u, tail = { x = p.x, y = p.y - 6 * u }, layer = 4 })
      hud.text("tlbt_" .. index, { x = p.x, y = y + 5 * u, text = table.concat(lines, "\n"), align = "center", size = 0.82,
        colour = { 48, 36, 24, 255 }, layer = 5 })
      bubbled[index] = true
    end
  end
end

-- Real lights to use: the client has them and Night Lighting is Modern
function W.real_lights()
  if type(render) ~= "table" or type(render.light) ~= "function" or type(render.night) ~= "function" then return false end
  local ok, n = pcall(render.night)
  return ok and type(n) == "table" and n.modern == true
end

-- The lamplighter's lamps and the carried lights as real lights; drops the ones gone
function W.lights(model, now)
  local keep_l, keep_z = {}, {}
  if W.real_lights() and setting("lamps") then
    -- Each of the lamplighter's lamps is the zone's own lamp post nearest it (its glow): out until he lights it,
    -- then fading on. A spot with no lamp post near it shows nothing (no light floating in the air).
    local glows = {}
    if type(render.lamps) == "function" then
      local ok, list = pcall(render.lamps)
      if ok and type(list) == "table" then glows = list end
    end
    for i, pos in pairs(model.all_lamps or {}) do
      local best, bd = nil, W.SNAP
      for _, g in ipairs(glows) do
        local dx, dz = g.x - pos.x, g.z - pos.z
        local d = math.sqrt(dx * dx + dz * dz)
        if d <= bd then best, bd = g, d end
      end
      if best then
        local zid = "tlz" .. i
        pcall(render.lamp_zone, zid, { x = best.x, y = best.y, z = best.z, radius = W.ZONE_RADIUS,
          level = model.lamps[i] ~= nil and 1 or 0, fade = W.LAMP_FADE })
        keep_z[zid] = true
      end
    end
    for index, kind in pairs(model.carry or {}) do
      local c = W.CARRY[kind]
      if c then
        local id = "tlc" .. index
        pcall(render.light, id, { entity = index, height = c.height, colour = c.colour, range = c.range,
          intensity = c.intensity, halo = c.halo, night_only = true })
        keep_l[id] = true
      end
    end
  end
  for id in pairs(lights) do
    if not keep_l[id] then pcall(render.light, id, nil) end
  end
  for id in pairs(zones) do
    if not keep_z[id] then pcall(render.lamp_zone, id, nil) end
  end
  lights, zones = keep_l, keep_z
end

-- The lamplighter's lamps and the carried lights, a few times a second
function W.scene(model, now, me)
  W.lights(model, now)
  -- Anything an older version drew (lamp glows, smoke) goes.
  for id in pairs(drawn) do draw.remove(id) end
  drawn = {}
end

-- Sounds by place: louder the closer you are
function W.sounds(model, now, me)
  if not (me and me.pos) or not setting("sounds") then return end
  for i, s in pairs(model.scenery) do
    local every = W.SOUND_EVERY[s.kind]
    if every and s.range > 0 and not s.off then
      local d = dist(s.pos, me.pos)
      if d <= s.range and now >= (next_sound[i] or 0) then
        next_sound[i] = now + every
        local vol = W.SOUND_VOLUME[s.kind] * (1 - d / s.range) ^ 1.5
        if vol > 0.02 then pcall(audio.play, ("sounds/%s.wav"):format(s.kind), vol) end
      end
    end
  end
end

function W.clear()
  for id in pairs(lights) do pcall(render.light, id, nil) end
  for id in pairs(zones) do pcall(render.lamp_zone, id, nil) end
  lights, zones = {}, {}
  for id in pairs(drawn) do draw.remove(id) end
  for index in pairs(bubbled) do clear_bubble(index) end
  drawn, bubbled, next_sound = {}, {}, {}
end

return W
