-- A recipe card: what it makes (NQ / HQ1-3), its crafts against your skill, your chances, the crystal and materials
-- with what you have in your bags and in storage, how many to craft, Craft and Favorite.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local C = require("lib.core")
local L = C.L

local K = {}

K.WIN = "artisan_recipe"
K.CW = 400
K.recipe = nil
K.n = 1

K.on_change = function() end -- set by init.lua: redraw the other windows (a favorite changed)

local function pct_colour(x)
  if x >= 0.9 then return S.GOOD elseif x >= 0.6 then return S.WARN end
  return S.BAD
end

local function row(c, y, label, value, colour, tip)
  c:text(12, y, label, { size = 0.8, colour = S.MUTED })
  c:text(K.CW - 12, y, value, { size = 0.85, colour = colour or S.TEXT, align = "right", bold = true })
  if tip then c:region(0, y - 2, K.CW, 18, { tip = tip }) end
  return y + 18
end

function K.draw(c)
  local r = K.recipe
  if not r then return end
  local o, chk = C.odds(r), C.check(r)
  local y = 0
  -- head
  c:icon(0, 0, 40, "item", r.result)
  c:text(48, 0, r.name .. (r.qty > 1 and (" x" .. r.qty) or ""), { bold = true, colour = S.GOLD, w = K.CW - 80 })
  local fav = C.is_fav(r.id)
  local hot = c:region(K.CW - 24, 0, 24, 22, { key = "fav", on = function() C.toggle_fav(r.id) K.render() K.on_change() end,
    tip = fav and "In your Favorites: click to take it out." or "Add to Favorites." })
  c:text(K.CW - 12, -1, "★", { align = "center", size = 1.3, colour = (fav or hot) and S.GOLD or S.FAINT })
  local parts = { ("%s Lv %d"):format(L.CRAFTS[r.craft].name, r.level) }
  for _, s in ipairs(L.subcrafts(r)) do parts[#parts + 1] = ("%s Lv %d"):format(L.CRAFTS[s.craft].name, s.level) end
  c:text(48, 20, table.concat(parts, " · "), { size = 0.75, colour = S.MUTED, w = K.CW - 48 })
  y = 48
  -- results
  local res = r.res
  local x = 0
  for i = 1, 4 do
    local id, q = res[i * 2 - 1], res[i * 2]
    if id and id > 0 then
      local label = i == 1 and "NQ" or ("HQ" .. (i - 1))
      c:card(x, y, 96, 40, { fill = i == 1 and S.CARD or { 228, 192, 119, 18 } })
      c:icon(x + 6, y + 6, 28, "item", id)
      c:text(x + 38, y + 4, label, { size = 0.7, colour = i == 1 and S.MUTED or S.GOLD, bold = true })
      c:text(x + 38, y + 19, "x" .. (q or 1), { size = 0.8 })
      c:region(x, y, 96, 40, { title = C.name(id), tip = i == 1 and "What a normal synth makes." or ("What a tier-%d high-quality synth makes."):format(i - 1) })
      x = x + 100
    end
  end
  y = y + 48
  -- chances
  y = c:heading(0, y, K.CW, "Your chances", C.ST.ready and nil or "waiting for the server")
  if C.ST.ready then
    if o.too_hard then
      c:para(12, y, K.CW - 24, "More than 15 levels above your skill: it would almost always break.", { colour = S.BAD, size = 0.85 })
      y = y + 22
    end
    y = row(c, y, "Success", L.pct(o.success), pct_colour(o.success), "Every craft of the recipe rolls on its own; the chances multiply.")
    y = row(c, y, "High quality", o.hq > 0 and ("%s  (HQ1 %s · HQ2 %s · HQ3 %s)"):format(L.pct(o.hq), L.pct(o.hq1), L.pct(o.hq2), L.pct(o.hq3)) or "none",
      o.hq > 0 and S.GOLD or S.FAINT, o.hq > 0 and "The further your skill is over the recipe, the better: 11, 31 and 51 levels over each raise it." or
      "A craft is at or above your skill, or a craft ring is on: no high quality.")
    for i, cr in pairs(o.crafts) do
      local why = cr.why == "cap" and "at your rank's cap" or (cr.why == "max" and "maxed" or (cr.why == "low" and "too easy" or nil))
      y = row(c, y, ("%s skill-up (you %s, needs %d)"):format(L.CRAFTS[i].name, L.skill_text(cr.skill), cr.need),
        why or L.pct(cr.skillup or 0), why and S.FAINT or S.FOCUS)
    end
    y = row(c, y, "On a break, each material lost", L.pct(o.loss), S.MUTED)
    local extra = {}
    if o.rested then extra[#extra + 1] = "rested" end
    if (o.lessons or 0) > 0 then extra[#extra + 1] = ("%d lesson%s"):format(o.lessons, o.lessons == 1 and "" or "s") end
    if o.tier then extra[#extra + 1] = ("mastery (%d made)"):format(o.made or 0) end
    if #extra > 0 then
      c:text(12, y, "Counted in: " .. table.concat(extra, ", "), { size = 0.75, colour = S.FAINT, w = K.CW - 24 })
      y = y + 18
    end
  end
  y = y + 6
  -- materials
  y = c:heading(0, y, K.CW, "Materials", "bags · storage")
  local crystal_have = 0
  for _, id in ipairs(L.CRYSTAL_SETS[r.element] or {}) do crystal_have = crystal_have + C.have(id) end
  local cl = r.element > 0 and C.stored(L.cluster_of(r.element)) or 0
  local crystal_stored = 0
  for _, id in ipairs(L.CRYSTAL_SETS[r.element] or {}) do crystal_stored = crystal_stored + C.stored(id) end
  local lines = { { id = r.crystal, need = 1, bags = crystal_have, stored = crystal_stored + 12 * cl,
    note = cl > 0 and ("(%d cluster%s)"):format(cl, cl == 1 and "" or "s") or nil } }
  for _, g in ipairs(r.ing) do lines[#lines + 1] = { id = g.id, need = g.qty, bags = C.have(g.id), stored = C.stored(g.id) } end
  for _, m in ipairs(lines) do
    local enough = m.bags + m.stored >= m.need
    c:icon(12, y, 18, "item", m.id)
    c:text(36, y + 1, ("%s%s"):format(m.need > 1 and (m.need .. "x ") or "", C.name(m.id)), { size = 0.85, w = 200, colour = enough and S.TEXT or S.BAD })
    if m.note then c:text(240, y + 2, m.note, { size = 0.7, colour = S.FAINT }) end
    c:text(K.CW - 12, y + 1, ("%s · %s"):format(C.commas(m.bags), C.commas(m.stored)), { size = 0.85, align = "right", colour = enough and S.MUTED or S.BAD })
    y = y + 22
  end
  if r.ki ~= 0 then
    local ok = C.has_ki(r.ki)
    c:text(12, y, ok and "Needs a guild key item: you have it." or "Needs a key item from the guild that you don't have.", { size = 0.8, colour = ok and S.GOOD or S.WARN, w = K.CW - 24 })
    y = y + 20
  end
  y = y + 8
  -- craft
  local room = C.queue_room()
  local max = math.max(1, math.min(room, chk.max > 0 and chk.max or 1, L.MAX_QUEUE))
  K.n = math.max(1, math.min(K.n, max))
  c:card(0, y, K.CW, 48)
  c:text(12, y + 6, "How many", { size = 0.75, colour = S.MUTED })
  c:button(12, y + 20, 24, 22, "-", { key = "minus", enabled = K.n > 1, on = function() K.n = K.n - 1 K.render() end })
  c:text(62, y + 22, tostring(K.n), { align = "center", bold = true })
  c:button(88, y + 20, 24, 22, "+", { key = "plus", enabled = K.n < max, on = function() K.n = K.n + 1 K.render() end })
  c:button(118, y + 20, 44, 22, "Max", { key = "max", size = 0.8, enabled = K.n < max, on = function() K.n = max K.render() end,
    tip = ("As many as your materials and the queue allow (%d)."):format(max) })
  local why = C.block(r)
  c:button(K.CW - 172, y + 8, 160, 32, K.n > 1 and ("Craft %d"):format(K.n) or "Craft", { style = "primary", key = "craft", enabled = why == nil,
    tip = why or "Queues it: synths run one every 15 seconds, using your inventory first, then storage.",
    on = function() C.craft(r, K.n) end })
  y = y + 52
  if why then
    y = c:para(0, y, K.CW, why, { size = 0.8, colour = S.WARN })
  end
  c:grow(y)
end

function K.render()
  if ui.is_open(K.WIN) then Canvas.render(K.WIN, "card") end
end

function K.setup()
  ui.window(K.WIN, {
    title = "Recipe",
    width = K.CW + 20,
    help = "A recipe with your chances and materials. Craft queues it (12 at most); the crystal and materials come from your inventory first, then storage.",
    elements = { { type = "canvas", id = "card", w = K.CW, h = 400, draw = {} } },
  })
  Canvas.add(K.WIN, "card", K.CW, K.draw)
  ui.on(K.WIN, function(win, e)
    if e.kind == "close" then return end
    Canvas.event(win, e)
  end)
end

function K.open(r)
  if K.recipe ~= r then K.n = 1 end
  K.recipe = r
  ui.open(K.WIN)
  K.render()
end

return K
