-- Companion View (/cv): each companion's job, role, HP / MP / TP, equipment and stats.
--
-- One canvas: a toolbar, the companions as cards on the left, and the chosen one on the right (header, role or
-- gear switch, bars, the 4 x 4 equipment grid with item tooltips, combat stats and attributes).

local S = require("lib.style")
local Canvas = require("lib.canvas")
local link = require("lib.link")
local feed = require("lib.feed_cv")
local live = require("lib.live")
local jobicons = require("lib.jobicons")

local V = {}

V.WIN = "companion_view"
V.WIDTH = 640
local CW = V.WIDTH - 24 -- the canvas's width inside the window's padding

local state = { selected = nil, switching_until = nil }
V.state = state
V.open_manager = function() end -- set by init.lua

local SLOT_LABELS = {
  main = "Main", sub = "Sub", ranged = "Range", ammo = "Ammo", head = "Head", neck = "Neck", ear1 = "Ear",
  ear2 = "Ear", body = "Body", hands = "Hands", ring1 = "Ring", ring2 = "Ring", back = "Back", waist = "Waist",
  legs = "Legs", feet = "Feet",
}

-- Warriors, Paladins and Dark Knights: tank or deal damage (!companion war|pld|drk tank|dps). mode: the server's
-- switch; on means the job's other role.
V.ROLE_SWITCH = {
  [1] = { mode = "wardps", cmd = "war", default = "tank" },
  [7] = { mode = "plddps", cmd = "pld", default = "tank" },
  [8] = { mode = "drktank", cmd = "drk", default = "dps" },
}

local function role_of(rec)
  return S.role_guess(rec.mjob, rec.tank)
end

-- "WAR DRK PLD" from an item's job bits, or "All Jobs".
local function jobs_text(bits)
  if not bits or bits == 0 then return nil end
  local names, n = {}, 0
  for j = 1, 22 do
    if bits & (1 << j) ~= 0 then
      n = n + 1
      names[#names + 1] = S.job_short(j)
    end
  end
  if n >= 22 then return "All Jobs" end
  return table.concat(names, " ")
end

-- An item's tooltip: its name, then the description, weapon damage and delay, level and jobs.
function V.item_tip(slot, id)
  if not id or id == 0 then return SLOT_LABELS[slot], "Nothing equipped." end
  local ok, item = pcall(res.item, id)
  if not ok or not item then return ("Item %d"):format(id), SLOT_LABELS[slot] end
  local lines = {}
  if (item.description or "") ~= "" then lines[#lines + 1] = item.description end
  if (item.damage or 0) > 0 then
    lines[#lines + 1] = ("DMG:%d  Delay:%d"):format(item.damage, item.delay or 0)
  end
  local lv = (item.level or 0) > 0 and ("Lv.%d "):format(item.level) or ""
  local jobs = jobs_text(item.jobs)
  if lv ~= "" or jobs then lines[#lines + 1] = lv .. (jobs or "") end
  local flags = {}
  if (item.flags or 0) & 0x8000 ~= 0 then flags[#flags + 1] = "Rare" end
  if (item.flags or 0) & 0x4000 ~= 0 then flags[#flags + 1] = "Ex" end
  if #flags > 0 then lines[#lines + 1] = table.concat(flags, " ") end
  return item.name or ("Item %d"):format(id), table.concat(lines, "\n")
end

local function switching()
  return state.switching_until ~= nil and os.clock() < state.switching_until
end

-- A switch was pressed: send it quietly, then show "Switching..." until the companion is back with its new gear.
local function switch_to(cmd)
  link.send(cmd, link.SLOW, false, true)
  state.switching_until = os.clock() + link.SLOW
  V.render()
end

local function pill(c, x, y, text, colour)
  local w = Canvas.measure(text, 0.8, true) + 14
  c:rect(x, y, w, 17, S.fade(colour, 0.22), { round = 8 })
  c:rect(x, y, w, 17, S.fade(colour, 0.7), { round = 8, line = 1 })
  c:text(x + w / 2, y + 2, text, { size = 0.8, bold = true, align = "center", colour = colour })
  return w
end

-- HP / MP / TP now: the party list's live values when the companion is in it, else the feed's.
function V.vitals(rec, m)
  local s = rec.stats
  if not m then return s.hp or 0, s.mhp or 0, s.mp or 0, s.mmp or 0, s.tp or 0 end
  local mhp = (s.mhp or 0) > 0 and s.mhp or (m.hp_max or 0)
  local mmp = s.mmp or 0
  if mmp == 0 and (m.mp or 0) > 0 and (m.mpp or 0) > 0 then mmp = math.floor(m.mp * 100 / m.mpp + 0.5) end
  return m.hp or 0, mhp, m.mp or 0, mmp, m.tp or 0
end

-- The companions as cards, down the left: name, job, what they're doing now, HP.
local function draw_list(c, model, x, y, w, lives)
  for _, id in ipairs(model.order) do
    local rec = model.companions[id]
    local m = lives[id]
    local role = role_of(rec)
    local sel = state.selected == id
    local hot = c:region(x, y, w, 64, { key = "card" .. id, on = function()
      state.selected = id
      V.render()
    end })
    c:rect(x, y, w, 64, (sel or hot) and S.CARD_HI or S.CARD, { round = 8 })
    c:rect(x, y, w, 64, sel and S.EDGE_HI or S.EDGE, { round = 8, line = sel and 1.5 or 1 })
    c:rect(x + 1, y + 8, 3, 48, role.colour, { round = 1 })
    jobicons.draw(c, x + 9, y + 7, 28, rec.mjob, { role = role })
    c:text(x + 43, y + 6, rec.name, { bold = true, colour = sel and S.GOLD or S.TEXT, w = w - 50 })
    local sub = (rec.sjob or 0) ~= 0 and (" / %s%d"):format(S.job_short(rec.sjob), rec.slv) or ""
    c:text(x + 43, y + 23, ("%s%d%s"):format(S.job_short(rec.mjob), rec.mlv, sub), { size = 0.85, colour = role.colour })
    local act, colour = live.activity(m and m.id, m and m.entity)
    if m and (m.hpp or 1) == 0 then act, colour = "Knocked out", S.BAD end
    if act then c:text(x + 12, y + 39, act, { size = 0.75, colour = colour, w = w - 20 }) end
    local hp, mhp = V.vitals(rec, m)
    c:bar(x + 12, y + 54, w - 24, 4, mhp > 0 and hp / mhp or ((m and m.hpp or 0) / 100), S.HP)
    y = y + 70
  end
  return y
end

-- One bar with its label and value above it.
local function stat_bar(c, x, y, w, label, cur, max, colour, text)
  cur, max = cur or 0, max or 0
  c:text(x, y, label, { size = 0.85, bold = true, colour = colour })
  c:text(x + w, y, text or ("%d / %d"):format(cur, max), { size = 0.85, colour = S.TEXT, align = "right" })
  c:bar(x, y + 17, w, 6, max > 0 and cur / max or 0, colour)
end

local function draw_equipment(c, rec, x, y)
  local cell, gap = 44, 5
  for i, slot in ipairs(feed.SLOTS) do
    local col, row = (i - 1) % 4, (i - 1) // 4
    local cx, cy = x + col * (cell + gap), y + row * (cell + gap)
    local id = rec.gear[slot] or 0
    local title, text = V.item_tip(slot, id)
    local hot = c:over(cx, cy, cell, cell)
    c:rect(cx, cy, cell, cell, S.SLOT, { round = 6 })
    c:rect(cx, cy, cell, cell, hot and S.EDGE_HI or S.EDGE, { round = 6, line = hot and 1.5 or 1 })
    if id ~= 0 then
      c:icon(cx + 4, cy + 4, cell - 8, "item", id)
    else
      c:text(cx + cell / 2, cy + cell / 2 - 7, SLOT_LABELS[slot], { size = 0.75, colour = S.FAINT, align = "center" })
    end
    c:region(cx, cy, cell, cell, { key = "slot" .. i, title = title, tip = text })
  end
  return y + 4 * cell + 3 * gap
end

local COMBAT = {
  { "att", "Attack" }, { "def", "Defense" }, { "acc", "Accuracy" }, { "eva", "Evasion" },
  { "ratt", "R. Attack" }, { "racc", "R. Accuracy" },
}

local function draw_stats(c, rec, x, y, w)
  local half = (w - 8) / 2
  local h = 22 + 7 * 18 + 6
  -- Combat
  c:card(x, y, half, h)
  c:text(x + 10, y + 6, "Combat", { bold = true, colour = S.GOLD })
  for i, row in ipairs(COMBAT) do
    local ry = y + 26 + (i - 1) * 18
    c:text(x + 10, ry, row[2], { size = 0.85, colour = S.MUTED })
    local v = rec.stats[row[1]]
    c:text(x + half - 10, ry, v and tostring(v) or "-", { size = 0.85, colour = S.TEXT, align = "right" })
  end
  -- Attributes: base, and the gear's bonus in green or red
  local ax = x + half + 8
  c:card(ax, y, half, h)
  c:text(ax + 10, y + 6, "Attributes", { bold = true, colour = S.GOLD })
  for i, key in ipairs(feed.ATTRIBUTES) do
    local ry = y + 26 + (i - 1) * 18
    c:text(ax + 10, ry, key:upper(), { size = 0.85, colour = S.MUTED })
    local a = rec.attributes[key]
    if not a then
      c:text(ax + half - 10, ry, "-", { size = 0.85, align = "right" })
    else
      if a.bonus ~= 0 then
        c:text(ax + half - 10, ry, ("%+d"):format(a.bonus), { size = 0.85, align = "right",
          colour = a.bonus > 0 and S.GOOD or S.BAD })
        c:text(ax + half - 44, ry, tostring(a.base), { size = 0.85, align = "right" })
      else
        c:text(ax + half - 10, ry, tostring(a.base), { size = 0.85, align = "right" })
      end
    end
  end
  return y + h
end

local function draw_detail(c, model, rec, x, y, w, m)
  local role = role_of(rec)
  -- Header: the job's emblem, name, role, bond
  jobicons.draw(c, x, y - 2, 34, rec.mjob, { role = role })
  c:tip(x, y - 2, 34, 34, S.job_long(rec.mjob), (rec.sjob or 0) ~= 0 and ("Support job: " .. S.job_long(rec.sjob)) or nil)
  x, w = x + 42, w - 42
  local nw = c:text(x, y, rec.name, { size = 1.45, bold = true, colour = S.GOLD, w = w - 230 })
  if m then
    c:button(x + w - 64, y, 64, 24, "Target", { key = "target", size = 0.85, tip = ("Target %s."):format(rec.name),
      on = function() pcall(actions.target, m.id) end })
  end
  local px = x + nw + 10
  px = px + pill(c, px, y + 5, role.label, role.colour) + 6
  if (rec.bond or "") ~= "" then
    pill(c, px, y + 5, rec.bond, S.BOND)
    c:tip(px, y + 5, Canvas.measure(rec.bond, 0.8, true) + 14, 17, "Kept companion",
      "A companion you keep, and your bond with them. Bond grows with every monster you defeat together.")
  end
  x, w = x - 42, w + 42
  y = y + 34
  local job = ("Lv%d %s"):format(rec.mlv, S.job_long(rec.mjob))
  if (rec.sjob or 0) ~= 0 then job = job .. (" / Lv%d %s"):format(rec.slv, S.job_long(rec.sjob)) end
  if S.RACES[rec.race] then job = job .. "  ·  " .. S.RACES[rec.race] end
  c:text(x, y, job, { colour = S.TEXT, w = w })
  y = y + 20
  local act, colour = live.activity(m and m.id, m and m.entity)
  if m and (m.hpp or 1) == 0 then act, colour = "Knocked out", S.BAD end
  if not m then act, colour = "Not in range", S.FAINT end
  c:text(x, y, act or "Standing by", { size = 0.85, colour = act and colour or S.FAINT, w = w })
  y = y + 22

  -- Thieves: Treasure Hunter or damage gear. Warriors, Paladins, Dark Knights: tank or damage.
  local modes = model.modes or {}
  local sw = V.ROLE_SWITCH[rec.mjob]
  if rec.mjob == 6 or sw then
    c:text(x, y + 5, rec.mjob == 6 and "Gear" or "Role", { size = 0.85, bold = true, colour = S.MUTED })
    if switching() then
      c:text(x + 44, y + 5, rec.mjob == 6 and "Switching gear..." or "Switching role...", { colour = S.GOLD })
    elseif rec.mjob == 6 then
      local th = modes.th == true
      c:switch(x + 44, y, 24, {
        { label = "Treasure Hunter", tip = "Wear the gear with the most Treasure Hunter (better drops)." },
        { label = "Damage", tip = "Wear the gear that does the most damage." },
      }, th and 1 or 2, { key = "th", on = function(i) switch_to(i == 1 and "!companion th on" or "!companion th off") end })
    else
      local tanking = (sw.default == "tank") ~= (modes[sw.mode] == true)
      c:switch(x + 44, y, 24, {
        { label = "Tank", tip = "Shield (great sword for a Dark Knight) and tanking gear; holds the monster." },
        { label = "Damage", tip = "Two-handed weapon and damage gear." },
      }, tanking and 1 or 2, { key = "role", on = function(i)
        switch_to(("!companion %s %s"):format(sw.cmd, i == 1 and "tank" or "dps"))
      end })
    end
    y = y + 34
  end

  -- HP, MP, TP side by side, live
  local hp, mhp, mp, mmp, tp = V.vitals(rec, m)
  local cols = mmp > 0 and 3 or 2
  local bw = (w - (cols - 1) * 14) / cols
  local bx = x
  stat_bar(c, bx, y, bw, "HP", hp, mhp, S.HP)
  bx = bx + bw + 14
  if cols == 3 then
    stat_bar(c, bx, y, bw, "MP", mp, mmp, S.MP)
    bx = bx + bw + 14
  end
  stat_bar(c, bx, y, bw, "TP", tp, 3000, S.TP, tostring(tp))
  y = y + 36

  -- Equipment and stats
  local gy = draw_equipment(c, rec, x, y)
  local gx = x + 4 * 44 + 3 * 5 + 14
  local sy = draw_stats(c, rec, gx, y, x + w - gx)
  return math.max(gy, sy)
end

function V.draw(c)
  local model = link.cv
  -- Toolbar
  c:button(0, 0, 84, 26, "Refresh", { style = "primary", key = "refresh", tip = "Ask the server for fresh data.",
    enabled = not link.waiting.cv, on = function() link.ask("cv") V.render() end })
  local status, bad = link.status("cv")
  c:text(96, 5, status, { colour = bad and S.BAD or S.MUTED, w = CW - 96 - 140 })
  c:button(CW - 128, 0, 128, 26, "Companion Manager", { key = "manager", tip = "Call, keep and set up companions (/cmgr).",
    on = function() V.open_manager() end, size = 0.85 })
  local y = 38

  if #model.order == 0 then
    c:card(0, y, CW, 70)
    c:text(CW / 2, y + 16, link.waiting.cv and "Asking the server for your companions..." or "No companions with you.",
      { align = "center", colour = S.TEXT })
    c:text(CW / 2, y + 38, "Call some from the Companion Manager, then press Refresh.",
      { align = "center", size = 0.85, colour = S.MUTED })
    return
  end

  if model.companions[state.selected] == nil then state.selected = model.order[1] end
  local lives = {}
  for _, m in ipairs(live.members()) do lives[m.index] = m end
  local ly = draw_list(c, model, 0, y, 170, lives)
  local dy = draw_detail(c, model, model.companions[state.selected], 186, y, CW - 186, lives[state.selected])
  c:grow(math.max(ly, dy))
end

function V.render()
  if ui.is_open(V.WIN) then Canvas.render(V.WIN, "body") end
end

function V.setup()
  ui.window(V.WIN, {
    title = "Companion View",
    width = V.WIDTH,
    help = "Your companions' gear, stats and role. Point at an item for its details.",
    elements = { { type = "canvas", id = "body", w = CW, h = 120, draw = {} } },
  })
  Canvas.add(V.WIN, "body", CW, V.draw)
  ui.on(V.WIN, function(win, e)
    Canvas.event(win, e)
  end)
end

function V.is_open() return ui.is_open(V.WIN) end

function V.open()
  ui.show(V.WIN)
  link.ask("cv")
  Canvas.render(V.WIN, "body")
end

function V.toggle()
  if ui.is_open(V.WIN) then ui.close(V.WIN) else V.open() end
end

return V
