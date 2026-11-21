-- The Masterworks ledger: a workshop's walnut and brass, the eight crafts as medallions, your recipe mastery with
-- the tiers marked on each bar and the Masterwork chance each one has reached, and beside it the workbench (rested
-- synths, Flash of Insight, lessons), today's craft orders and your apprentice.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local protocol = require("lib.protocol")

local L = {}

L.WIN = "masterworks"
L.CW, L.CH = 640, 452
L.model = protocol.new_model()

local WOOD = { 46, 32, 22, 252 }
local WOOD2 = { 70, 48, 30, 255 }
local BRASS = { 214, 172, 96, 255 }
local BRASS_DIM = { 214, 172, 96, 110 }
local PARCH = { 236, 222, 190, 255 }
local INK = { 70, 52, 34, 255 }
local GOOD = { 140, 210, 120, 255 }
local GLOW = { 255, 210, 120, 255 }

local function anvil(c, x, y, col)
  c:rect(x, y, 26, 7, col, { round = 2 })
  c.draw[#c.draw + 1] = { type = "tri", x1 = x, y1 = y, x2 = x - 8, y2 = y, x3 = x, y3 = y + 5, colour = col }
  c:rect(x + 9, y + 7, 8, 7, col)
  c:rect(x + 4, y + 14, 18, 4, col, { round = 1 })
end

local function hammer(c, x, y, col)
  c:line(x, y + 16, x + 12, y + 4, { 150, 104, 60, 255 }, 3)
  c:rect(x + 8, y - 2, 12, 7, col, { round = 1 })
end

function L.draw_head(c, st)
  c:rect(0, 0, L.CW, 52, WOOD2, { to = WOOD })
  for y = 12, 48, 12 do c:line(0, y, L.CW, y, { 30, 20, 12, 90 }, 1) end
  anvil(c, 22, 18, BRASS)
  hammer(c, 34, 6, { 200, 200, 210, 255 })
  c:text(62, 8, "Masterworks", { size = 1.35, bold = true, colour = PARCH, outline = { 0, 0, 0, 160 } })
  local mw = st and st.masterworks
  local okM, moment = pcall(require, "lib.moment")
  local last = okM and moment.moments()[1] or nil
  local sub = mw and (mw.on and "Your crafting ledger" or "Masterworks are off on this server") or "Your crafting ledger"
  if last then
    sub = ("Latest Masterwork: %s, %s%s"):format(last.item or "?", last.at or "", last.file and " (screenshot saved)" or "")
  end
  c:text(62, 31, sub, { size = 0.82, colour = last and GLOW or BRASS_DIM, w = L.CW - 250 })
  if last and last.file then
    c:tip(62, 28, L.CW - 250, 18, "Masterwork screenshot", last.file)
  end
  local made = mw and mw.made or 0
  c:rect(L.CW - 176, 10, 162, 32, { 0, 0, 0, 90 }, { round = 8 })
  c:rect(L.CW - 176, 10, 162, 32, BRASS, { round = 8, line = 1.2 })
  c:text(L.CW - 168, 14, tostring(made), { size = 1.4, bold = true, colour = made > 0 and GLOW or PARCH })
  c:text(L.CW - 132, 19, made == 1 and "Masterwork made" or "Masterworks made", { size = 0.8, colour = PARCH, w = 116 })
  c:line(0, 52, L.CW, 52, BRASS, 1.5)
end

function L.draw_crafts(c, st, y)
  local w = (L.CW - 16) / 8
  local best = 0
  for _, k in ipairs(st.crafts) do best = math.max(best, k.skill) end
  for i, k in ipairs(st.crafts) do
    local x = 8 + (i - 1) * w
    local top = k.skill > 0 and k.skill == best
    local cx = x + w / 2
    c:circle(cx, y + 18, 16, top and { 120, 84, 40, 255 } or { 0, 0, 0, 90 })
    c:circle(cx, y + 18, 16, top and GLOW or BRASS_DIM, top and 2 or 1.2)
    c:text(cx, y + 11, k.short, { size = 0.8, bold = true, align = "center", colour = k.skill > 0 and PARCH or { 150, 140, 120, 200 } })
    c:text(cx, y + 38, k.skill > 0 and ("%.1f"):format(k.skill) or "-", { size = 0.82, bold = true, align = "center",
      colour = top and GLOW or PARCH })
    c:text(cx, y + 52, protocol.RANKS[k.rank] or "", { size = 0.66, align = "center", colour = BRASS_DIM, w = w - 4 })
    c:tip(x, y, w, 64, k.name, k.skill > 0 and ("Skill %.1f, guild rank %s."):format(k.skill, protocol.RANKS[k.rank] or "?")
      or "Not started yet.")
  end
  return y + 70
end

-- A mastery bar: made against the top tier, with the tiers marked
local function mastery_bar(c, st, x, y, w, made)
  local tiers = st.bonus and st.bonus.tiers or { 25, 50, 100 }
  local top = tiers[#tiers] or 100
  c:rect(x, y, w, 8, { 0, 0, 0, 120 }, { round = 4 })
  local frac = math.min(1, made / top)
  if frac > 0 then c:rect(x, y, math.max(6, w * frac), 8, made >= top and GLOW or BRASS, { round = 4 }) end
  for _, t in ipairs(tiers) do
    local tx = x + w * (t / top)
    c:line(tx, y - 2, tx, y + 10, made >= t and PARCH or { 120, 100, 80, 220 }, 1.2)
  end
end

function L.draw_mastery(c, st, y)
  local w = 360
  c:heading(10, y, w, "Recipe mastery", "made · HQ · Masterwork")
  y = y + 28
  if #st.mastery == 0 then
    local tiers = st.bonus and st.bonus.tiers or { 25, 50, 100 }
    y = c:para(10, y, w, ("Nothing mastered yet. Every successful synth of a recipe counts: Practiced at %d, Mastered at %d, Grandmaster at %d. From Mastered, a high-quality piece of gear can come out a Masterwork."):format(tiers[1] or 25, tiers[2] or 50, tiers[3] or 100),
      { size = 0.82, colour = PARCH })
    return y
  end
  for i, m in ipairs(st.mastery) do
    if y > L.CH - 60 then
      c:text(10, y, ("...and %d more."):format(#st.mastery - i + 1), { size = 0.76, colour = BRASS_DIM })
      break
    end
    local at, name, nextAt = protocol.tier(st, m.made)
    local chance = protocol.chance(st, at)
    c:text(10, y, m.name, { size = 0.84, bold = true, colour = PARCH, w = 190 })
    c:text(206, y + 1, name, { size = 0.74, colour = at >= 2 and GLOW or BRASS_DIM, w = 80 })
    c:text(w + 10, y + 1, ("%d · %d"):format(m.made, m.hq), { size = 0.74, align = "right", colour = PARCH })
    mastery_bar(c, st, 10, y + 17, w - 70, m.made)
    if chance > 0 then
      c:rect(w - 52, y + 13, 62, 15, { 120, 84, 40, 220 }, { round = 7 })
      c:text(w - 21, y + 14, chance .. "% MW", { size = 0.7, bold = true, align = "center", colour = GLOW })
    else
      c:text(w + 10, y + 15, nextAt and ((nextAt - m.made) .. " to go") or "", { size = 0.68, align = "right", colour = BRASS_DIM })
    end
    c:tip(10, y, w, 30, m.name, ("%d made, %d high quality. %s.%s%s"):format(m.made, m.hq, name,
      nextAt and (" %d more to the next tier."):format(nextAt - m.made) or " The top tier.",
      chance > 0 and (" A high-quality piece of gear from it has a %d%% chance to come out a Masterwork."):format(chance) or ""))
    y = y + 34
  end
  return y
end

function L.draw_side(c, st, top)
  local x, w = 392, L.CW - 400
  local y = top
  c:card(x, y, w, 96, { fill = { 0, 0, 0, 80 }, edge = BRASS_DIM })
  c:text(x + 10, y + 6, "Workbench", { bold = true, colour = BRASS })
  local b = st.bonus or {}
  c:text(x + 10, y + 26, ("Rested  %d / %d  (+%d%% skill-ups)"):format(b.rested or 0, b.max or 0, b.gain or 0), { size = 0.78, colour = PARCH, w = w - 20 })
  c:text(x + 10, y + 42, b.ready and "Flash of Insight: ready!" or ("Flash of Insight: %d%% a synth"):format(b.insight or 0),
    { size = 0.78, colour = b.ready and GLOW or PARCH, w = w - 20 })
  local les = {}
  for i, n in ipairs(b.lessons or {}) do
    if (tonumber(n) or 0) > 0 then les[#les + 1] = protocol.CRAFTS[i].key:upper() .. " " .. n end
  end
  c:text(x + 10, y + 58, #les > 0 and ("Lessons: " .. table.concat(les, ", ")) or "Lessons: none waiting", { size = 0.78, colour = PARCH, w = w - 20 })
  c:text(x + 10, y + 74, (b.loss or 0) > 0 and ("Guild rank: %d%% fewer materials lost"):format(b.loss) or "Guild rank: no break bonus yet",
    { size = 0.74, colour = BRASS_DIM, w = w - 20 })
  y = y + 104

  c:card(x, y, w, 30 + math.max(1, #st.orders) * 30, { fill = { 0, 0, 0, 80 }, edge = BRASS_DIM })
  c:text(x + 10, y + 6, "Today's orders", { bold = true, colour = BRASS })
  y = y + 26
  if #st.orders == 0 then
    c:text(x + 10, y + 2, "No orders today.", { size = 0.78, colour = BRASS_DIM })
    y = y + 30
  end
  for _, o in ipairs(st.orders) do
    local mark = o.done == 2 and "★" or (o.done == 1 and "✓" or "·")
    c:text(x + 10, y, mark, { size = 0.9, bold = true, colour = o.done > 0 and GOOD or BRASS_DIM })
    c:text(x + 24, y, ("%s%s"):format(o.name, o.qty > 1 and (" x" .. o.qty) or ""), { size = 0.78, bold = true,
      colour = o.done > 0 and S.fade(PARCH, 0.6) or PARCH, w = w - 90 })
    c:text(x + w - 10, y + 1, ("%d gil"):format(o.gil), { size = 0.72, align = "right", colour = BRASS })
    c:text(x + 24, y + 14, ("%s Lv.%d · %s"):format(protocol.CRAFTS[o.craft] and protocol.CRAFTS[o.craft].name or "?", o.level, o.who),
      { size = 0.66, colour = BRASS_DIM, w = w - 34 })
    y = y + 30
  end
  y = y + 8

  local aps = st.apprentices
  c:card(x, y, w, 30 + math.max(1, #aps) * 46, { fill = { 0, 0, 0, 80 }, edge = BRASS_DIM })
  c:text(x + 10, y + 6, "Apprentice", { bold = true, colour = BRASS })
  y = y + 26
  if #aps == 0 then
    c:para(x + 10, y, w - 20, "None. !companion apprentice <name> <craft> takes a kept companion on.", { size = 0.74, colour = BRASS_DIM })
  end
  for _, a in ipairs(aps) do
    local craft = protocol.CRAFTS[a.craft] and protocol.CRAFTS[a.craft].name or "?"
    c:text(x + 10, y, a.name, { size = 0.84, bold = true, colour = PARCH, w = w - 90 })
    c:text(x + w - 10, y + 1, a.away and "away" or "with you", { size = 0.7, align = "right", colour = a.away and BRASS_DIM or GOOD })
    c:text(x + 10, y + 15, ("%s %s · %.1f"):format(a.rank, craft, a.skill), { size = 0.72, colour = BRASS_DIM, w = w - 20 })
    local frac = a.next and math.min(1, a.skill / math.max(1, a.next_skill)) or 1
    c:rect(x + 10, y + 31, w - 20, 6, { 0, 0, 0, 120 }, { round = 3 })
    c:rect(x + 10, y + 31, math.max(4, (w - 20) * frac), 6, BRASS, { round = 3 })
    c:tip(x, y, w, 40, a.name, ("%s, %s %s (skill %.1f).%s Assist: +%d%% success%s%s."):format(a.name, a.rank, craft, a.skill,
      a.next and (" %s at %d."):format(a.next, a.next_skill) or "", a.assist, a.loss > 0 and (", -%d%% materials lost"):format(a.loss) or "",
      a.workshop > 0 and (", workshop up to %d synths"):format(a.workshop) or ""))
    y = y + 46
  end
end

function L.draw(c)
  local st = L.model.status
  c:rect(0, 0, L.CW, L.CH, WOOD, { round = 6 })
  L.draw_head(c, st)
  if not st then
    c:text(14, 70, "Opening your ledger...", { colour = PARCH })
    return
  end
  local y = L.draw_crafts(c, st, 60)
  L.draw_mastery(c, st, y)
  L.draw_side(c, st, y)
  local mw = st.masterworks
  if mw and mw.on then
    local names = mw.names
    local text = ("From %s, a high-quality piece of gear may come out a Masterwork: %d%%, or %d%% at %s. It keeps your name and gains augments."):format(
      names[2] or "Mastered", mw.chance[2] or 0, mw.chance[3] or 0, names[3] or "Grandmaster")
    c:line(0, L.CH - 26, L.CW, L.CH - 26, BRASS_DIM, 1)
    c:text(10, L.CH - 20, text, { size = 0.7, colour = BRASS_DIM, w = L.CW - 20 })
  end
end

function L.render()
  if ui.is_open(L.WIN) then Canvas.render(L.WIN, "ledger") end
end

function L.setup()
  ui.window(L.WIN, {
    title = "Masterworks",
    width = L.CW + 20,
    help = "Your crafts, recipe mastery and Masterwork chances, today's orders and your apprentice. Point at anything for the details.",
    elements = { { type = "canvas", id = "ledger", w = L.CW, h = L.CH, draw = {} } },
  })
  Canvas.add(L.WIN, "ledger", L.CW, L.draw)
  ui.on(L.WIN, function(win, e) Canvas.event(win, e) end)
end

return L
