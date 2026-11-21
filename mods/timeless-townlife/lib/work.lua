-- Apprentices at the furnace (the server's townlife_workshop.lua, feed lines W): over each one a progress bar with
-- their craft, sparks off the anvil (steam over a pot or flask), the furnace's glow on them (Modern night lighting)
-- and their craft's sound: hammering, a loom, bubbling.

local K = {}

K.CRAFTS = {
  wood = { name = "Woodworking", colour = { 196, 150, 96 }, fx = "chips" },
  smith = { name = "Smithing", colour = { 255, 150, 60 }, fx = "sparks" },
  gold = { name = "Goldsmithing", colour = { 255, 210, 90 }, fx = "sparks" },
  cloth = { name = "Clothcraft", colour = { 210, 170, 230 }, fx = nil },
  leather = { name = "Leathercraft", colour = { 180, 120, 80 }, fx = nil },
  bone = { name = "Bonecraft", colour = { 235, 225, 200 }, fx = "chips" },
  alchemy = { name = "Alchemy", colour = { 120, 220, 170 }, fx = "steam" },
  cook = { name = "Cooking", colour = { 240, 190, 110 }, fx = "steam" },
}
K.SOUND_EVERY = { hammer = 5, loom = 4, bubble = 5 }
K.SOUND_VOLUME = { hammer = 0.4, loom = 0.4, bubble = 0.35 }
K.SOUND_RANGE = 22
K.PARTICLES = 10   -- per apprentice
K.GRAVITY = 9      -- yalms per second squared (y points down)

local parts = {}    -- index -> particles
local shown = {}    -- index -> true (hud and draw ids made)
local next_sound = {}
local lights = {}

local function setting(k)
  local ok, v = pcall(settings.get, k)
  if not ok or v == nil then return true end
  return v
end

local function clamp255(v) return math.max(0, math.min(255, math.floor(v + 0.5))) end

local function clear(index)
  hud.remove("tlw_" .. index)
  hud.remove("tlwb_" .. index)
  hud.remove("tlwt_" .. index)
  for k = 1, K.PARTICLES do pcall(draw.remove, ("tlws_%d_%d"):format(index, k)) end
  if lights[index] then pcall(render.light, "tlwl_" .. index, nil) lights[index] = nil end
  parts[index], shown[index], next_sound[index] = nil, nil, nil
end

-- Where they work: in front of them, at bench height
local function anvil(e)
  local h = e.heading or 0
  return { x = e.pos.x + math.cos(h) * 0.7, y = e.pos.y - 0.9, z = e.pos.z - math.sin(h) * 0.7 }
end

local function emit(kind, at)
  if kind == "sparks" then
    local a = math.random() * math.pi * 2
    local s = 1.5 + math.random() * 2.5
    return { x = at.x, y = at.y, z = at.z, vx = math.cos(a) * s, vy = -(1.5 + math.random() * 2.5), vz = math.sin(a) * s,
      life = 0.35 + math.random() * 0.35, age = 0, gravity = K.GRAVITY }
  elseif kind == "chips" then
    local a = math.random() * math.pi * 2
    return { x = at.x, y = at.y, z = at.z, vx = math.cos(a) * 1.2, vy = -(1 + math.random()), vz = math.sin(a) * 1.2,
      life = 0.5, age = 0, gravity = K.GRAVITY }
  else -- steam: slow, rising, drifting
    return { x = at.x + (math.random() - 0.5) * 0.3, y = at.y, z = at.z + (math.random() - 0.5) * 0.3,
      vx = (math.random() - 0.5) * 0.3, vy = -(0.4 + math.random() * 0.3), vz = (math.random() - 0.5) * 0.3,
      life = 1.4 + math.random() * 0.6, age = 0, gravity = 0 }
  end
end

-- The particles a little later: the live ones moved on, new ones from the anvil
function K.step(list, kind, at, dt)
  local out = {}
  for _, p in ipairs(list) do
    p.age = p.age + dt
    if p.age < p.life then
      p.vy = p.vy + p.gravity * dt
      p.x, p.y, p.z = p.x + p.vx * dt, p.y + p.vy * dt, p.z + p.vz * dt
      out[#out + 1] = p
    end
  end
  -- Sparks come in bursts, as the hammer falls; steam steadily
  local burst = kind == "steam" and (math.random() < 0.5 and 1 or 0) or (math.random() < 0.25 and 4 or 0)
  for _ = 1, burst do
    if #out < K.PARTICLES then out[#out + 1] = emit(kind, at) end
  end
  return out
end

function K.update(model, now, dt, me)
  local on = setting("workshop")
  local u = 1
  local okS, sc = pcall(hud.screen)
  if okS and sc then u = sc.ui_scale or 1 end
  for index in pairs(shown) do
    if not (on and model.work and model.work[index]) then clear(index) end
  end
  if not on then return end
  local real = type(render) == "table" and type(render.night) == "function" and type(render.light) == "function"
    and (function() local ok, n = pcall(render.night) return ok and type(n) == "table" and n.modern == true end)()
  for index, w in pairs(model.work or {}) do
    local ok, e = pcall(game.entity, index)
    if not (ok and e and e.pos) then
      clear(index)
    else
      local c = K.CRAFTS[w.craft] or K.CRAFTS.smith
      shown[index] = true
      -- The bar over their head
      local top = { x = e.pos.x, y = e.pos.y - (e.height or 1.6) - 0.15, z = e.pos.z }
      local okp, sp = pcall(draw.to_screen, top)
      if okp and sp and sp.on_screen ~= false then
        local bw, bh = math.floor(76 * u), math.floor(7 * u)
        local x, y = math.floor(sp.x - bw / 2), math.floor(sp.y - 16 * u)
        hud.rect("tlwb_" .. index, { x = x - 2, y = y - 2, w = bw + 4, h = bh + 4, colour = { 30, 22, 14, 190 }, layer = 3 })
        hud.bar("tlw_" .. index, { x = x, y = y, w = bw, h = bh, value = w.pct / 100, colour = { c.colour[1], c.colour[2], c.colour[3], 235 }, layer = 3 })
        hud.text("tlwt_" .. index, { x = sp.x, y = y - math.floor(14 * u), text = ("%s %d%%"):format(c.name, w.pct), align = "center",
          size = 0.72, colour = { 250, 236, 210, 255 }, layer = 3 })
      else
        hud.remove("tlw_" .. index) hud.remove("tlwb_" .. index) hud.remove("tlwt_" .. index)
      end
      -- Sparks, chips or steam
      if c.fx then
        local at = anvil(e)
        parts[index] = K.step(parts[index] or {}, c.fx, at, dt)
        for k = 1, K.PARTICLES do
          local p = parts[index][k]
          local id = ("tlws_%d_%d"):format(index, k)
          if p then
            local fade = 1 - p.age / p.life
            local col
            if c.fx == "sparks" then
              col = { 255, clamp255(170 + 80 * fade), clamp255(60 * fade), clamp255(255 * fade) }
            elseif c.fx == "chips" then
              col = { c.colour[1], c.colour[2], c.colour[3], clamp255(230 * fade) }
            else
              col = { 235, 240, 245, clamp255(120 * fade) }
            end
            local tail = c.fx == "steam" and 0.12 or 0.05
            pcall(draw.line, id, { from = { x = p.x, y = p.y, z = p.z },
              to = { x = p.x - p.vx * tail, y = p.y - p.vy * tail, z = p.z - p.vz * tail }, colour = col })
          else
            pcall(draw.remove, id)
          end
        end
      end
      -- The furnace's glow, flickering
      if real and (c.fx == "sparks" or c.fx == "chips") then
        local flicker = 0.75 + 0.25 * math.sin(now * 11 + index) * math.sin(now * 7.3)
        pcall(render.light, "tlwl_" .. index, { entity = index, height = 1.0, colour = { 255, 140, 60 }, range = 5,
          intensity = 9 * flicker, halo = 0.3 })
        lights[index] = true
      elseif lights[index] then
        pcall(render.light, "tlwl_" .. index, nil)
        lights[index] = nil
      end
      -- Their craft's sound
      if setting("sounds") and me and me.pos then
        local dx, dy, dz = e.pos.x - me.pos.x, e.pos.y - me.pos.y, e.pos.z - me.pos.z
        local d = math.sqrt(dx * dx + dy * dy + dz * dz)
        local every = K.SOUND_EVERY[w.sound]
        if every and d <= K.SOUND_RANGE and now >= (next_sound[index] or 0) then
          next_sound[index] = now + every
          local vol = K.SOUND_VOLUME[w.sound] * (1 - d / K.SOUND_RANGE) ^ 1.5
          if vol > 0.02 then pcall(audio.play, ("sounds/%s.wav"):format(w.sound), vol) end
        end
      end
    end
  end
end

function K.clear()
  for index in pairs(shown) do clear(index) end
  parts, shown, next_sound, lights = {}, {}, {}, {}
end

return K
