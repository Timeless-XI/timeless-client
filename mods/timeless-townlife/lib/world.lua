-- The town in the world: speech bubbles over townsfolk, glowing street lamps, chimney smoke and sounds by place.

local W = {}

W.BUBBLE_SECONDS = 5
W.LAMP_RANGE = 60   -- yalms: lamps further than this aren't drawn
W.SMOKE_RANGE = 80
W.SOUND_EVERY = { hammer = 5, crowd = 4.5, gulls = 6 } -- seconds between plays (about the sound's length)
W.SOUND_VOLUME = { hammer = 0.45, crowd = 0.35, gulls = 0.4 }

-- Real lights (TimelessXI's Night Lighting Modern: render.light / render.lamp_zone): the lamplighter's lamps light up
-- one by one and the native lamps beside them follow, his lantern and the guards' torches give light. Without them
-- (Original lighting, an older client) the lamps glow as drawn sprites.
W.LAMP_LIGHT = { colour = { 255, 186, 110 }, range = 10, intensity = 16, halo = 0.9, height = 2.6, fade = 1.5 }
W.CARRY = {
  lantern = { colour = { 255, 196, 120 }, range = 7, intensity = 7, halo = 0.45, height = 1.3 },
  torch = { colour = { 255, 150, 70 }, range = 9, intensity = 10, halo = 0.6, height = 2.0 },
}
W.ZONE_RADIUS = 6 -- yalms round a lamplighter's lamp in which the zone's own lamps follow it

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
    local L = W.LAMP_LIGHT
    for i, pos in pairs(model.all_lamps or {}) do
      local lit = model.lamps[i] ~= nil
      local zid = "tlz" .. i
      pcall(render.lamp_zone, zid, { x = pos.x, y = pos.y, z = pos.z, radius = W.ZONE_RADIUS, level = lit and 1 or 0, fade = L.fade })
      keep_z[zid] = true
      if lit then
        local at = model.lit_at and model.lit_at[i]
        local k = at and math.min(1, math.max(0, now - at) / L.fade) or 1
        local id = "tll" .. i
        pcall(render.light, id, { x = pos.x, y = pos.y - L.height, z = pos.z, colour = L.colour, range = L.range,
          intensity = L.intensity * k, halo = L.halo * k })
        keep_l[id] = true
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

-- Lamps and smoke: redrawn a few times a second (the smoke drifts up)
function W.scene(model, now, me)
  local keep = {}
  W.lights(model, now)
  if me and me.pos then
    if setting("lamps") and not W.real_lights() then
      local flicker = 0.9 + 0.1 * math.sin(now * 7.3)
      for i, pos in pairs(model.lamps) do
        if dist(pos, me.pos) <= W.LAMP_RANGE then
          local id = "lamp" .. i
          local top = { x = pos.x, y = pos.y - 2.6, z = pos.z } -- y points down: up is minus
          draw.image(id, { path = "img/glow.png", at = top, w = 1.6 * flicker, h = 1.6 * flicker, facing = "camera",
            colour = { 255, 236, 200, math.floor(230 * flicker) } })
          draw.ring(id .. "g", { at = { x = pos.x, y = pos.y - 0.05, z = pos.z }, radius = 3.2, filled = true, segments = 32,
            colour = { 255, 196, 110, math.floor(34 * flicker) } })
          keep[id], keep[id .. "g"] = true, true
        end
      end
    end
    if setting("smoke") then
      for i, s in pairs(model.scenery) do
        if s.kind == "smoke" and dist(s.pos, me.pos) <= W.SMOKE_RANGE then
          for k = 0, 3 do
            local t = (now * 0.25 + k / 4 + i * 0.37) % 1 -- each puff rises, grows and fades, then starts again
            local id = ("smoke%d_%d"):format(i, k)
            local at = { x = s.pos.x + math.sin(t * 3 + k) * 0.6 * t, y = s.pos.y - t * 6, z = s.pos.z + t * 1.5 }
            draw.image(id, { path = "img/smoke.png", at = at, w = 1.2 + t * 2.4, h = 1.2 + t * 2.4, facing = "camera",
              colour = { 255, 255, 255, math.floor(170 * math.sin(t * math.pi)) } })
            keep[id] = true
          end
        end
      end
    end
  end
  for id in pairs(drawn) do
    if not keep[id] then draw.remove(id) end
  end
  drawn = keep
end

-- Sounds by place: louder the closer you are
function W.sounds(model, now, me)
  if not (me and me.pos) or not setting("sounds") then return end
  for i, s in pairs(model.scenery) do
    local every = W.SOUND_EVERY[s.kind]
    if every and s.range > 0 then
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
