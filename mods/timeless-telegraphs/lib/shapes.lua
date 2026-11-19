-- Where a telegraph lies on the ground, whether a point is inside it, and how it is drawn.
--
-- World coordinates: x east, z north, y the height pointing down (a smaller y is higher). Headings are radians,
-- 0 east and π/2 south, so a heading h points along (cos h, -sin h) on the ground. The server's cone is a triangle
-- (CTargetFind::findWithinCone): its apex at the monster, `radius` yalms long, `angle` degrees wide, turned the way
-- the monster faces (behind it for "b"). Circles are centred on the monster ("c") or on its target ("t").

local S = {}

S.CONE_IMG = "img/cone.png" -- a white triangle, apex at the bottom centre, 45° wide; tinted and stretched when drawn

function S.dir(h) return math.cos(h), -math.sin(h) end

-- The cone's three corners {x, z}: apex, left end, right end.
function S.cone_points(apex, heading, length, angle)
  local half = math.rad(angle) / 2
  local lx, lz = S.dir(heading - half)
  local rx, rz = S.dir(heading + half)
  return { x = apex.x, z = apex.z },
    { x = apex.x + lx * length, z = apex.z + lz * length },
    { x = apex.x + rx * length, z = apex.z + rz * length }
end

local function side(p, a, b)
  return (p.x - b.x) * (a.z - b.z) - (a.x - b.x) * (p.z - b.z)
end

function S.in_triangle(p, a, b, c)
  local d1, d2, d3 = side(p, a, b), side(p, b, c), side(p, c, a)
  local neg = d1 < 0 or d2 < 0 or d3 < 0
  local pos = d1 > 0 or d2 > 0 or d3 > 0
  return not (neg and pos)
end

-- Whether ground point p is inside telegraph t, placed by `where` (centre, apex, heading).
function S.inside(t, where, p)
  if not where or not p then return false end
  if t.shape == "c" or t.shape == "t" then
    local dx, dz = p.x - where.centre.x, p.z - where.centre.z
    return dx * dx + dz * dz <= t.radius * t.radius
  end
  local a, b, c = S.cone_points(where.centre, where.heading, t.radius, t.angle)
  return S.in_triangle(p, a, b, c)
end

local function clamp_colour(c, alpha)
  return { math.floor(c[1]), math.floor(c[2]), math.floor(c[3]), math.max(0, math.min(255, math.floor(alpha))) }
end

-- The draw ids a telegraph uses (all removed together).
function S.ids(key)
  return { key .. "f", key .. "e", key .. "p", key .. "l1", key .. "l2", key .. "l3", key .. "w", key .. "n" }
end

-- Draws telegraph t at `where`, `frac` (0..1) of the way to going off. o: colour, opacity (0..1), labels, flash,
-- cache (a table kept per telegraph, so the cone's picture is only redrawn when it moved).
function S.draw(key, t, where, frac, o)
  local col, op = o.colour, o.opacity or 1
  if o.flash then col = { 255, 236, 220 } end
  local y = where.centre.y - 0.08 -- a hair above the ground, so the floor doesn't hide it
  local centre = { x = where.centre.x, y = y, z = where.centre.z }
  if t.shape == "c" or t.shape == "t" then
    draw.ring(key .. "f", { at = centre, radius = t.radius, filled = true, segments = 72, colour = clamp_colour(col, 46 * op) })
    draw.ring(key .. "p", { at = centre, radius = math.max(0.05, t.radius * frac), filled = true, segments = 72,
      colour = clamp_colour(col, (o.flash and 150 or 70) * op) })
    draw.ring(key .. "e", { at = centre, radius = t.radius, segments = 72, colour = clamp_colour(col, 235 * op) })
  else
    local a, b, c = S.cone_points(centre, where.heading, t.radius, t.angle)
    local function p3(q) return { x = q.x, y = y, z = q.z } end
    -- The filled picture: only redrawn when the cone moved or turned (draw.image reads the file each call)
    local cache = o.cache or {}
    local moved = cache.x == nil or math.abs(cache.x - centre.x) > 0.15 or math.abs(cache.z - centre.z) > 0.15
      or math.abs((cache.h or 0) - where.heading) > 0.02 or cache.flash ~= o.flash
    if moved then
      local dx, dz = S.dir(where.heading)
      draw.image(key .. "f", {
        path = S.CONE_IMG,
        at = { x = centre.x + dx * t.radius / 2, y = y, z = centre.z + dz * t.radius / 2 },
        w = math.max(0.01, 2 * t.radius * math.tan(math.rad(t.angle) / 2)),
        h = math.max(0.01, t.radius),
        facing = "ground",
        yaw = where.heading,
        colour = clamp_colour(col, (o.flash and 255 or 200) * op),
      })
      cache.x, cache.z, cache.h, cache.flash = centre.x, centre.z, where.heading, o.flash
    end
    local edge = clamp_colour(col, 240 * op)
    draw.line(key .. "l1", { from = p3(a), to = p3(b), colour = edge })
    draw.line(key .. "l2", { from = p3(a), to = p3(c), colour = edge })
    draw.line(key .. "l3", { from = p3(b), to = p3(c), colour = edge })
    -- The front sweeping out from the monster as the move readies
    local w1 = { x = a.x + (b.x - a.x) * frac, z = a.z + (b.z - a.z) * frac }
    local w2 = { x = a.x + (c.x - a.x) * frac, z = a.z + (c.z - a.z) * frac }
    draw.line(key .. "w", { from = p3(w1), to = p3(w2), colour = clamp_colour({ 255, 240, 200 }, 255 * op) })
  end
  if o.labels and t.name ~= "" then
    local lx, lz = where.centre.x, where.centre.z
    if t.shape == "f" or t.shape == "b" then
      local dx, dz = S.dir(where.heading)
      lx, lz = lx + dx * t.radius * 0.6, lz + dz * t.radius * 0.6
    end
    draw.text(key .. "n", { at = { x = lx, y = where.centre.y - 1.2, z = lz }, text = t.name, size = 0.9,
      colour = clamp_colour({ 255, 228, 200 }, 255), outline = { 40, 0, 0, 230 } })
  end
end

function S.remove(key)
  for _, id in ipairs(S.ids(key)) do draw.remove(id) end
end

return S
