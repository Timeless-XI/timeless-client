-- High-quality and Masterwork moments. A high-quality synth: a bright flash round you and a chime. A Masterwork: a
-- golden burst with rays and a ring, a fanfare, the camera easing in on you and back, a golden light on you (Modern
-- night lighting), and a screenshot under screenshots/masterworks, noted in the ledger.
--
-- Telling a Masterwork: after a high-quality synth the mod asks the server for its Masterwork count (cb status 1);
-- one more than before is a Masterwork. No game or UI code beyond the render, audio and client calls.

local M = {}

M.FLASH_SECONDS = 0.8
M.GOLD_SECONDS = 3.6
M.CAMERA_IN, M.CAMERA_HOLD, M.CAMERA_OUT = 0.9, 1.6, 1.3
M.CAMERA_SCALE = 0.55   -- of the distance it was, no nearer than 0.5
M.SHOT_AT = 1.1         -- seconds into the burst: the camera in, the rays full
M.CHECK_SECONDS = 6     -- how long to wait for the server's count after a high-quality synth
M.KEEP = 20             -- moments kept in the ledger's list

M.FLASH_WGSL = [[
@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let c = textureSample(scene, samp, in.uv);
  // post.p[0]: x, y the centre (0..1), z the strength (0..1), w the grade (1..3)
  let asp = post.size.x / max(post.size.y, 1.0);
  let d = distance(vec2<f32>(in.uv.x * asp, in.uv.y), vec2<f32>(post.p[0].x * asp, post.p[0].y));
  let glow = post.p[0].z * (exp(-d * d * 16.0) * 0.85 + 0.1);
  let tint = mix(vec3<f32>(1.0, 0.98, 0.92), vec3<f32>(0.86, 0.94, 1.0), clamp(post.p[0].w - 1.0, 0.0, 2.0) * 0.4);
  return vec4<f32>(c.rgb + tint * glow, c.a);
}
]]

M.GOLD_WGSL = [[
@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let c = textureSample(scene, samp, in.uv);
  // post.p[0]: x, y the centre (0..1), z the strength (0..1), w the ring's radius
  let s = post.p[0].z;
  let asp = post.size.x / max(post.size.y, 1.0);
  let v = vec2<f32>((in.uv.x - post.p[0].x) * asp, in.uv.y - post.p[0].y);
  let d = length(v);
  let a = atan2(v.y, v.x);
  let rays = pow(0.5 + 0.5 * sin(a * 14.0 + post.time * 0.7), 6.0) * 0.6 + pow(0.5 + 0.5 * sin(a * 23.0 - post.time), 8.0) * 0.4;
  let ring = exp(-pow((d - post.p[0].w) * 9.0, 2.0)) * 0.55;
  let core = exp(-d * d * 20.0);
  let gold = vec3<f32>(1.0, 0.78, 0.32);
  let glow = s * (core * 1.05 + rays * exp(-d * 2.2) * 0.5 + ring);
  let warm = mix(c.rgb, c.rgb * vec3<f32>(1.08, 1.0, 0.86), s * 0.6);
  let vig = 1.0 - s * 0.35 * smoothstep(0.45, 1.1, length((in.uv - vec2<f32>(0.5, 0.5)) * vec2<f32>(asp, 1.0)));
  return vec4<f32>(warm * vig + gold * glow, c.a);
}
]]

local st = { flash = nil, gold = nil, check = nil, base = nil, cam = nil }
M.state = st

local function setting(k)
  local ok, v = pcall(settings.get, k)
  if not ok or v == nil then return true end
  return v
end

-- Where you are on screen, 0..1 (your chest)
local function centre()
  local me = game.player()
  local okS, sc = pcall(hud.screen)
  if me and me.pos and okS and sc and sc.width and sc.width > 0 then
    local ok, p = pcall(draw.to_screen, { x = me.pos.x, y = me.pos.y - 1.1, z = me.pos.z })
    if ok and p and p.on_screen ~= false then
      return math.max(0, math.min(1, p.x / sc.width)), math.max(0, math.min(1, p.y / sc.height))
    end
  end
  return 0.5, 0.55
end

local function ease(t) t = math.max(0, math.min(1, t)) return t * t * (3 - 2 * t) end

function M.safe_name(text)
  return ((text or "Masterwork"):gsub("[^%w%s%-']", ""):gsub("%s+", " "))
end

-- A high-quality synth: the flash and the chime; then ask whether it was a Masterwork
function M.on_hq(item, grade, now, ask)
  if setting("hq_moment") then
    pcall(render.shader, "mwflash", M.FLASH_WGSL)
    st.flash = { t0 = now, grade = grade or 1 }
    pcall(audio.play, "sounds/chime.wav", 0.6)
  end
  if st.base ~= nil then
    st.check = { item = item, base = st.base, until_ = now + M.CHECK_SECONDS }
    ask()
  end
end

-- A status came in: the Masterwork count. More than before, while checking: a Masterwork.
function M.on_count(made, now, item_name)
  local was = st.check
  st.base = made
  if was and now <= was.until_ and made > was.base then
    st.check = nil
    M.masterwork(item_name(was.item), now)
    return true
  end
  if was and now > was.until_ then st.check = nil end
  return false
end

function M.masterwork(name, now)
  if not setting("masterwork_moment") then return end
  pcall(render.shader, "mwgold", M.GOLD_WGSL)
  st.gold = { t0 = now, name = name, shot = false }
  pcall(audio.play, "sounds/fanfare.wav", 0.7)
  local me = game.player()
  if me and me.index then
    pcall(render.light, "mwglow", { entity = me.index, height = 1.2, colour = { 255, 200, 110 }, range = 6, intensity = 14, halo = 0.6 })
  end
  if setting("camera_moment") and type(render.set_camera_distance) == "function" then
    local ok, d0 = pcall(render.camera_distance)
    d0 = ok and type(d0) == "number" and d0 or 1
    st.cam = { t0 = now, from = d0, to = math.max(0.5, d0 * M.CAMERA_SCALE) }
  end
end

-- Each frame (or tick): drives the passes, the camera and the screenshot
function M.update(now)
  if st.flash then
    local k = (now - st.flash.t0) / M.FLASH_SECONDS
    if k >= 1 then
      pcall(render.shader, "mwflash", nil)
      st.flash = nil
    else
      local x, y = centre()
      local s = (k < 0.12 and k / 0.12 or (1 - (k - 0.12) / 0.88)) * (0.55 + 0.15 * st.flash.grade)
      pcall(render.shader_params, "mwflash", { { x, y, s, st.flash.grade } })
    end
  end
  if st.gold then
    local t = now - st.gold.t0
    if t >= M.GOLD_SECONDS then
      pcall(render.shader, "mwgold", nil)
      pcall(render.light, "mwglow", nil)
      st.gold = nil
    else
      local x, y = centre()
      local s = t < 0.25 and t / 0.25 or (t > M.GOLD_SECONDS - 1.2 and (M.GOLD_SECONDS - t) / 1.2 or 1)
      pcall(render.shader_params, "mwgold", { { x, y, s, 0.08 + t * 0.55 } })
      if not st.gold.shot and t >= M.SHOT_AT then
        st.gold.shot = true
        if setting("screenshot") then
          local ok, file = pcall(client.screenshot, ("masterworks/%s %s"):format(os.date("%Y-%m-%d %H%M%S"), M.safe_name(st.gold.name)))
          M.remember(st.gold.name, ok and file or nil)
        else
          M.remember(st.gold.name, nil)
        end
      end
    end
  end
  if st.cam then
    local t = now - st.cam.t0
    local c = st.cam
    local d
    if t < M.CAMERA_IN then
      d = c.from + (c.to - c.from) * ease(t / M.CAMERA_IN)
    elseif t < M.CAMERA_IN + M.CAMERA_HOLD then
      d = c.to
    elseif t < M.CAMERA_IN + M.CAMERA_HOLD + M.CAMERA_OUT then
      d = c.to + (c.from - c.to) * ease((t - M.CAMERA_IN - M.CAMERA_HOLD) / M.CAMERA_OUT)
    else
      d = c.from
      st.cam = nil
    end
    pcall(render.set_camera_distance, d)
  end
end

-- The ledger's list of moments (newest first), kept per character
function M.remember(name, file)
  local list = storage.character.moments or {}
  table.insert(list, 1, { item = name, file = file and ("screenshots/" .. file) or nil, at = os.date("%Y-%m-%d") })
  while #list > M.KEEP do table.remove(list) end
  storage.character.moments = list
  pcall(storage.save)
end

function M.moments()
  return storage.character.moments or {}
end

function M.clear()
  pcall(render.shader, "mwflash", nil)
  pcall(render.shader, "mwgold", nil)
  pcall(render.light, "mwglow", nil)
  if st.cam then pcall(render.set_camera_distance, st.cam.from) end
  st.flash, st.gold, st.cam = nil, nil, nil
end

return M
