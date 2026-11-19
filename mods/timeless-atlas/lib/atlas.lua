-- The NM Atlas window: a search box and filters, the list of notorious monsters (up now, a timer, can pop, unknown),
-- and the one picked: its model turning slowly (TimelessXI client), where and how it spawns, whether it's up, what
-- it's weak to and what it drops.

local Canvas = require("lib.canvas")
local S = require("lib.style")
local P = require("lib.protocol")

local A = {}

A.WIN = "atlas"
A.W = 800
A.LIST_W = 290
A.ROW_H = 24
A.PORTRAIT = 176

A.model = nil
A.state = { query = "", up = false, here = false, band = 1, page = 1, selected = nil }
A.ask = nil -- set by init.lua: ask(text)

A.BANDS = { { "All", 0, 0 }, { "1-30", 1, 30 }, { "31-50", 31, 50 }, { "51-70", 51, 70 }, { "71+", 71, 0 } }

A.STATE = {
  [0] = { "Unknown", { 150, 156, 166, 255 } },
  [1] = { "Up now", { 120, 214, 140, 255 } },
  [2] = { "Timer", { 240, 190, 90, 255 } },
  [3] = { "Can pop", { 92, 196, 200, 255 } },
  [4] = { "Due", { 150, 220, 120, 255 } },
}

A.ELEMENTS = {
  { "fire", "Fire", { 232, 96, 72, 255 } }, { "ice", "Ice", { 130, 200, 240, 255 } },
  { "wind", "Wind", { 120, 214, 140, 255 } }, { "earth", "Earth", { 200, 160, 90, 255 } },
  { "thunder", "Thunder", { 190, 130, 240, 255 } }, { "water", "Water", { 80, 140, 230, 255 } },
  { "light", "Light", { 245, 240, 200, 255 } }, { "dark", "Dark", { 140, 110, 170, 255 } },
}

A.STATUSES = {
  { "light_sleep", "Sleep" }, { "dark_sleep", "Sleepga II" }, { "bind", "Bind" }, { "gravity", "Gravity" },
  { "paralyze", "Paralyze" }, { "slow", "Slow" }, { "silence", "Silence" }, { "blind", "Blind" },
  { "poison", "Poison" }, { "stun", "Stun" },
}

local IMMUNE = {
  light_sleep = "Sleep", dark_sleep = "Sleep", sleep = "Sleep", bind = "Bind", gravity = "Gravity", paralyze = "Paralyze",
  slow = "Slow", silence = "Silence", blind = "Blind", poison = "Poison", stun = "Stun", petrify = "Petrify",
  terror = "Terror", plague = "Plague", elegy = "Elegy", requiem = "Requiem", aspir = "Aspir", dispel = "Dispel",
  curse = "Curse", addle = "Addle",
}

local TYPES = {
  at_night = "only at night (20:00-4:00)", at_evening = "only in the evening", weather = "only in its weather",
  fog = "only in fog (early morning)", moonphase = "only at certain moon phases", windowed = "only at certain hours",
  scripted = "by its own rules (a pop, a quest or an event)",
}

local function title_case(s)
  return ((s or ""):gsub("(%a)([%w']*)", function(a, b) return a:upper() .. b end))
end

function A.zone_name(id)
  local ok, n = pcall(res.zone, id)
  if ok and type(n) == "string" and n ~= "" then return n end
  return "Zone " .. tostring(id)
end

function A.item_name(id)
  local ok, it = pcall(res.item, id)
  if ok and it and it.name then return it.name end
  return "Item " .. tostring(id)
end

local function now() return os.clock() end

function A.commas(n)
  local s = tostring(math.floor(n))
  local out = s:reverse():gsub("(%d%d%d)", "%1,"):reverse()
  return (out:gsub("^,", ""))
end

-----------------------------------------------------------------------------------------------------------------------
-- The bar: filters and pages
-----------------------------------------------------------------------------------------------------------------------
local function bar(c)
  local st = A.state
  local x = 0
  c:button(x, 0, 78, 24, "Up now", { key = "up", style = st.up and "on" or "off",
    tip = "Only the ones up right now.", on = function() st.up = not st.up; st.page = 1; A.refresh() end })
  x = x + 84
  c:button(x, 0, 88, 24, "This zone", { key = "here", style = st.here and "on" or "off",
    tip = "Only the ones in the zone you're in.", on = function() st.here = not st.here; st.page = 1; A.refresh() end })
  x = x + 100
  for i, b in ipairs(A.BANDS) do
    c:button(x, 0, 54, 24, b[1], { key = "band" .. i, style = st.band == i and "on" or "off",
      tip = i == 1 and "Every level." or ("Level " .. b[1] .. "."), on = function() st.band = i; st.page = 1; A.refresh() end })
    x = x + 58
  end
  local l = A.model and A.model.list or { page = 1, pages = 1, total = 0 }
  local right = A.W
  c:button(right - 28, 0, 28, 24, ">", { key = "next", enabled = l.page < l.pages,
    on = function() st.page = l.page + 1; A.refresh() end, tip = "Next page." })
  c:text(right - 34, 4, ("%d / %d"):format(l.page, l.pages), { size = 0.9, colour = S.MUTED, align = "right" })
  c:button(right - 108, 0, 28, 24, "<", { key = "prev", enabled = l.page > 1,
    on = function() st.page = l.page - 1; A.refresh() end, tip = "Previous page." })
  c:text(right - 116, 4, ("%d found"):format(l.total or 0), { size = 0.9, colour = S.MUTED, align = "right" })
end

-----------------------------------------------------------------------------------------------------------------------
-- The list
-----------------------------------------------------------------------------------------------------------------------
local function list(c, y0)
  local l = A.model and A.model.list
  local rows = l and l.rows or {}
  if #rows == 0 then
    c:para(0, y0 + 6, A.LIST_W, (A.model and A.model.list.at > 0) and "Nothing matches. Try fewer words or another filter."
      or "Asking the server...", { colour = S.MUTED })
    return y0 + 60
  end
  local y = y0
  for _, r in ipairs(rows) do
    local sel = A.state.selected == r.id
    local hot = c:region(0, y, A.LIST_W, A.ROW_H - 2, { key = "row" .. r.id, on = function() A.pick(r.id) end })
    if sel or hot then
      c:rect(0, y, A.LIST_W, A.ROW_H - 2, sel and S.CARD_HI or S.CARD, { round = 5 })
    end
    local st = A.STATE[r.state] or A.STATE[0]
    c:circle(10, y + 11, 4.5, st[2])
    c:text(22, y + 1, r.name, { bold = true, size = 0.9, w = A.LIST_W - 80 })
    local lv = r.lv[1] == r.lv[2] and ("Lv%d"):format(r.lv[1] or 0) or ("Lv%d-%d"):format(r.lv[1] or 0, r.lv[2] or 0)
    c:text(A.LIST_W - 6, y + 1, lv, { size = 0.82, colour = S.MUTED, align = "right" })
    c:text(22, y + 13, A.zone_name(r.zone), { size = 0.68, colour = S.FAINT, w = A.LIST_W - 80 })
    if r.state == 2 then
      local left = P.left(r.mins, l.at, now())
      c:text(A.LIST_W - 6, y + 13, P.span(left), { size = 0.68, colour = st[2], align = "right" })
    elseif r.state == 1 or r.state == 3 or r.state == 4 then
      c:text(A.LIST_W - 6, y + 13, st[1], { size = 0.68, colour = st[2], align = "right" })
    end
    y = y + A.ROW_H
  end
  return y
end

-----------------------------------------------------------------------------------------------------------------------
-- The detail
-----------------------------------------------------------------------------------------------------------------------
local function chip(c, x, y, text, colour)
  local w = Canvas.measure(text, 0.78) + 14
  c:rect(x, y, w, 18, S.fade(colour, 0.22), { round = 9 })
  c:text(x + 7, y + 2, text, { size = 0.78, colour = colour })
  return x + w + 6
end

local function rank_word(v)
  if v == nil then return "?", S.MUTED end
  if v <= -2 then return "Weak", { 236, 120, 100, 255 } end
  if v < 0 then return "A little weak", { 240, 170, 120, 255 } end
  if v == 0 then return "Normal", S.MUTED end
  if v < 4 then return "Resists", { 140, 180, 236, 255 } end
  return "Resists strongly", { 110, 150, 240, 255 }
end

local function status_word(v)
  if v == nil then return "?", S.MUTED end
  if v <= -2 then return "Easily", { 120, 214, 140, 255 } end
  if v < 0 then return "Fairly easily", { 160, 214, 150, 255 } end
  if v == 0 then return "Normally", S.MUTED end
  if v < 4 then return "Resists", { 240, 170, 120, 255 } end
  return "Rarely", { 236, 120, 100, 255 }
end

local function spawn_text(i)
  local sp = i.spawn or {}
  local lines = {}
  if i.flags:find("k", 1, true) then
    lines[#lines + 1] = "A Land King: it appears on its zone's own timer (a window of about a day), or is popped with the items below."
  end
  local function span_text(t)
    local a, b = math.floor(t[1] / 60), math.floor(t[2] / 60)
    if b - a < 5 then return "about " .. P.span(a) end
    return P.span(a) .. " to " .. P.span(b)
  end
  if sp.timed then
    local every = sp.timer and span_text(sp.timer) or "a set time"
    local ph = #sp.ph > 0 and (", taking the place of " .. table.concat(sp.ph, " or ")) or ""
    lines[#lines + 1] = ("On a timer: it appears on its own %s after it was last defeated%s. No placeholder hunting needed."):format(every, ph)
  elseif sp.chance > 0 then
    local ph = #sp.ph > 0 and table.concat(sp.ph, " or ") or "a placeholder"
    local cd = sp.cooldown > 0 and (", at most once every " .. P.span(math.floor(sp.cooldown / 60))) or ""
    lines[#lines + 1] = ("Lottery: when %s is defeated, a %d%% chance it appears in its place%s."):format(ph, sp.chance, cd)
  elseif sp.timer and not i.flags:find("k", 1, true) then
    lines[#lines + 1] = ("Respawns %s after it's defeated."):format(span_text(sp.timer))
  elseif sp.respawn > 0 then
    lines[#lines + 1] = ("Respawns about %s after it's defeated."):format(P.span(math.floor(sp.respawn / 60)))
  end
  for _, t in ipairs(sp.types or {}) do
    if t ~= "lottery" and TYPES[t] and not (t == "scripted" and (#i.pops > 0 or sp.timer or i.flags:find("k", 1, true))) then
      lines[#lines + 1] = "Spawns " .. TYPES[t] .. "."
    end
  end
  if sp.window then
    local a, b = sp.window:match("(%d+)%-(%d+)")
    if a then lines[#lines + 1] = ("Only between %s:00 and %s:00 Vana'diel time."):format(a, b) end
  end
  return lines
end

local function state_line(i)
  local st = A.STATE[i.state] or A.STATE[0]
  local text
  if i.state == 1 then
    text = "Up right now!"
  elseif i.state == 2 then
    local left = P.left(i.mins, i.at, now())
    text = left > 0 and ("Can spawn in " .. P.span(left)) or "Can spawn any time now"
  elseif i.state == 3 then
    text = "Can pop now: hunt its placeholders"
  elseif i.state == 4 then
    text = "Due: its timer is up, it appears within a few minutes (when its hour and weather allow)"
  else
    text = (#i.pops > 0) and "Pop it at the ??? when you're ready" or "Not seen since the server started"
  end
  if i.since and i.since >= 0 then
    text = text .. ("   ·   last defeated %s ago"):format(P.span(i.since))
  end
  return text, st[2]
end

local function detail(c, x0, y0, w)
  local i = A.model and A.model.info
  if not i then
    c:para(x0, y0 + 6, w, A.state.selected and "Looking it up..." or
      "Pick a notorious monster to see where it lives, how it spawns, what it drops and what it's weak to.",
      { colour = S.MUTED })
    return y0 + 60
  end
  local pw = A.PORTRAIT
  local tw = w - pw - 14
  -- The model, turning slowly
  local px = x0 + w - pw
  c:rect(px, y0, pw, pw, { 12, 14, 22, 230 }, { round = 8 })
  if mod and mod.model_portraits and i.model > 0 then
    c:portrait(px + 2, y0 + 2, pw - 4, pw - 4, { model = i.model, spin = 24, bg = { 12, 14, 22, 255 } })
  else
    c:para(px + 12, y0 + pw / 2 - 18, pw - 24, i.model > 0 and "Update your TimelessXI client to see the monster here."
      or "No model to show.", { size = 0.8, colour = S.FAINT })
  end
  c:rect(px, y0, pw, pw, S.EDGE, { round = 8, line = 1 })
  -- Name, place, level
  local y = y0
  c:text(x0, y, i.name, { bold = true, size = 1.25, colour = S.GOLD, w = tw })
  y = y + 26
  local lv = i.lv[1] == i.lv[2] and ("Level %d"):format(i.lv[1] or 0) or ("Level %d-%d"):format(i.lv[1] or 0, i.lv[2] or 0)
  c:text(x0, y, ("%s  ·  %s"):format(A.zone_name(i.zone), lv), { size = 0.92, w = tw })
  y = y + 18
  local kind = title_case(i.species ~= "" and i.species or i.family or "")
  local more = {}
  if kind ~= "" then more[#more + 1] = kind end
  if i.jobs ~= "" then more[#more + 1] = i.jobs end
  if i.hp > 0 then more[#more + 1] = A.commas(i.hp) .. " HP" end
  c:text(x0, y, table.concat(more, "  ·  "), { size = 0.85, colour = S.MUTED, w = tw })
  y = y + 22
  -- How it behaves
  local cx = x0
  if i.flags:find("a", 1, true) then cx = chip(c, cx, y, "Aggressive", { 236, 120, 100, 255 }) end
  if i.flags:find("l", 1, true) then cx = chip(c, cx, y, "Links", { 240, 190, 90, 255 }) end
  if i.flags:find("t", 1, true) then cx = chip(c, cx, y, "True sight", { 190, 130, 240, 255 }) end
  for _, d in ipairs(i.detects or {}) do
    if cx < x0 + tw - 60 then cx = chip(c, cx, y, "Detects by " .. d, { 150, 180, 220, 255 }) end
  end
  if i.flags:find("c", 1, true) and cx < x0 + tw - 70 then cx = chip(c, cx, y, "Charmable", { 120, 214, 140, 255 }) end
  y = y + 26
  -- Up or not
  local text, colour = state_line(i)
  c:circle(x0 + 6, y + 8, 5, colour)
  y = c:para(x0 + 18, y, tw - 18, text, { size = 0.9, colour = colour })
  y = math.max(y + 6, y0 + pw + 8)

  -- How it spawns
  y = c:heading(x0, y, w, "How it spawns")
  local sp = spawn_text(i)
  if #sp == 0 and #i.pops == 0 then sp = { "Nothing special known: it simply respawns." } end
  for _, line in ipairs(sp) do
    y = c:para(x0, y, w, line, { size = 0.88 }) + 2
  end
  for _, p in ipairs(i.pops) do
    local ix = x0
    c:text(ix, y + 3, "Trade at the ??? :", { size = 0.88, colour = S.MUTED })
    ix = ix + 112
    for _, it in ipairs(p.items) do
      c:icon(ix, y, 20, "item", it)
      local name = A.item_name(it)
      c:text(ix + 24, y + 3, name, { size = 0.82, w = 170 })
      c:tip(ix, y, 24 + math.min(170, Canvas.measure(name, 0.82)), 20, name, "Trade it at the ??? to pop " .. i.name .. ".")
      ix = ix + 30 + math.min(170, Canvas.measure(name, 0.82)) + 8
      if ix > x0 + w - 120 then ix = x0 + 112; y = y + 22 end
    end
    y = y + 24
  end
  y = y + 6

  -- Weak to
  y = c:heading(x0, y, w, "Weaknesses", "elements and enfeebles")
  local ew = w / 4
  for k, e in ipairs(A.ELEMENTS) do
    local col, row = (k - 1) % 4, (k - 1) // 4
    local ex, ey = x0 + col * ew, y + row * 22
    local v = i.weak.el and i.weak.el[e[1]]
    local word, wc = rank_word(v)
    c:circle(ex + 7, ey + 8, 6, e[3])
    c:text(ex + 18, ey, e[2], { size = 0.85, w = 60 })
    c:text(ex + 76, ey, word, { size = 0.8, colour = wc, w = ew - 80 })
    c:tip(ex, ey, ew - 4, 18, e[2], v and ("Its " .. e[2]:lower() .. " resistance rank: " .. v .. " (below 0: it takes more).") or "")
  end
  y = y + 48
  local sw = w / 2
  for k, s in ipairs(A.STATUSES) do
    local col, row = (k - 1) % 2, (k - 1) // 2
    local sx, sy = x0 + col * sw, y + row * 18
    local v = i.weak.st and i.weak.st[s[1]]
    local word, wc = status_word(v)
    local immune = false
    for _, im in ipairs(i.weak.im or {}) do if IMMUNE[im] == s[2] or im == s[1] then immune = true end end
    if immune then word, wc = "Immune", { 236, 120, 100, 255 } end
    c:text(sx, sy, s[2], { size = 0.82, colour = S.MUTED, w = 90 })
    c:text(sx + 92, sy, word, { size = 0.82, colour = wc, w = sw - 96 })
  end
  y = y + 5 * 18 + 4
  local extra = {}
  for _, im in ipairs(i.weak.im or {}) do
    local n = IMMUNE[im] or title_case(im:gsub("_", " "))
    local listed = false
    for _, s in ipairs(A.STATUSES) do if s[2] == n then listed = true end end
    if not listed then extra[#extra + 1] = n end
  end
  if #extra > 0 then
    y = c:para(x0, y, w, "Also immune to: " .. table.concat(extra, ", ") .. ".", { size = 0.82, colour = { 236, 150, 130, 255 } }) + 2
  end
  local phys = {}
  for _, k in ipairs({ "slashing", "piercing", "blunt", "h2h" }) do
    local v = i.weak.ph and i.weak.ph[k]
    if v and v ~= 0 then
      phys[#phys + 1] = ("%s %s%d%%"):format(k == "h2h" and "Hand-to-hand" or title_case(k), v > 0 and "+" or "", v)
    end
  end
  if i.weak.mg and i.weak.mg.all and i.weak.mg.all ~= 0 then
    phys[#phys + 1] = ("Magic %s%d%%"):format(i.weak.mg.all > 0 and "+" or "", i.weak.mg.all)
  end
  if #phys > 0 then
    y = c:para(x0, y, w, "Damage taken: " .. table.concat(phys, ",  ") .. ".", { size = 0.82, colour = S.MUTED }) + 2
  end
  y = y + 8

  -- Drops
  y = c:heading(x0, y, w, "Drops", #i.steal > 0 and ("steal: " .. A.item_name(i.steal[1])) or nil)
  if #i.drops == 0 then
    y = c:para(x0, y, w, "Nothing listed.", { size = 0.85, colour = S.MUTED })
  end
  local dw = w / 2
  local col = 0
  local function drop_row(item, pct, indent)
    local dx = x0 + col * dw + (indent or 0)
    c:icon(dx, y, 20, "item", item)
    local name = A.item_name(item)
    c:text(dx + 24, y + 3, name, { size = 0.84, w = dw - 90 - (indent or 0) })
    c:text(x0 + col * dw + dw - 10, y + 3, pct, { size = 0.84, colour = S.GOLD, align = "right" })
    c:tip(dx, y, dw - 8 - (indent or 0), 20, name, "Drop rate " .. pct .. ".")
    col = col + 1
    if col == 2 then col = 0; y = y + 22 end
  end
  for _, d in ipairs(i.drops) do
    if d.item then
      drop_row(d.item, P.percent(d.rate))
    elseif d.one then
      if col == 1 then col = 0; y = y + 22 end
      c:text(x0, y + 2, ("One of these (%s):"):format(P.percent(d.rate)), { size = 0.8, colour = S.MUTED })
      y = y + 18
      for _, o in ipairs(d.one) do
        drop_row(o.item, P.percent(math.floor(d.rate * o.w / 100 + 0.5)), 8)
      end
      if col == 1 then col = 0; y = y + 22 end
    end
  end
  if col == 1 then y = y + 22 end
  return y + 6
end

function A.draw(c)
  bar(c)
  local y0 = 34
  local ly = list(c, y0)
  c:line(A.LIST_W + 7, y0, A.LIST_W + 7, math.max(ly, y0 + 20 * A.ROW_H), { 255, 255, 255, 24 }, 1)
  local dy = detail(c, A.LIST_W + 16, y0, A.W - A.LIST_W - 16)
  c:grow(math.max(ly, dy, y0 + 20 * A.ROW_H))
end

-----------------------------------------------------------------------------------------------------------------------
-- Asking
-----------------------------------------------------------------------------------------------------------------------
function A.list_query()
  local st = A.state
  local band = A.BANDS[st.band] or A.BANDS[1]
  local zone = 0
  if st.here then
    local ok, z = pcall(game.zone)
    zone = ok and tonumber(z) or 0
  end
  local q = (st.query or ""):lower():gsub("[^%w%s']", " "):gsub("%s+", " ")
  local text = ("na list %d %d %d %d %d %s"):format(st.page, st.up and 1 or 0, zone, band[2], band[3], q)
  return (text:gsub("%s+$", ""))
end

function A.refresh()
  if A.ask then A.ask(A.list_query()) end
  A.render()
end

function A.pick(id)
  A.state.selected = id
  if A.model then A.model.info = nil end
  if A.ask then A.ask("na info " .. id) end
  A.render()
end

function A.setup(model)
  A.model = model
  ui.window(A.WIN, {
    title = "NM Atlas",
    width = A.W + 20,
    help = "Every notorious monster of the open world: search by name, by what it drops or by its placeholder; see whether it's up, how it spawns, what it's weak to and what it drops.",
    elements = {
      { type = "input", id = "q", label = "Search", text = "", filter = true,
        tip = "A name, an item it drops, a placeholder or a family: 'lizzy', 'bounding boots', 'rock lizard', 'behemoth'." },
      { type = "canvas", id = "atlas", w = A.W, h = 560, draw = {} },
    },
  })
  Canvas.add(A.WIN, "atlas", A.W, A.draw)
  ui.on(A.WIN, function(win, e)
    if e.kind == "close" then A.forget() return end
    if e.id == "q" and (e.kind == "edit" or e.kind == "change") then
      A.state.query = e.text or ""
      A.state.page = 1
      A.state.typed_at = now()
      if e.kind == "change" then A.state.typed_at = nil; A.refresh() end
      return
    end
    Canvas.event(win, e)
  end)
end

-- Closed: let go of the answers (the list, the detail); the search and the pick stay, and are asked again on opening.
-- The model turning in the portrait goes too (the client drops a preview no window has asked for).
function A.forget()
  local m = A.model
  if not m then return end
  m.list = { page = 1, pages = 1, total = 0, rows = {}, at = 0 }
  m.pending_list, m.info, m.pending_info = nil, nil, nil
  A.state.typed_at = nil
  if A.on_forget then A.on_forget() end
end

function A.render()
  if ui.is_open(A.WIN) then Canvas.render(A.WIN, "atlas") end
end

function A.toggle()
  if ui.is_open(A.WIN) then
    ui.close(A.WIN)
    A.forget()
  else
    ui.open(A.WIN)
    A.refresh()
    if A.state.selected and A.ask then A.ask("na info " .. A.state.selected) end
  end
end

return A
