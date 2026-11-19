-- The shop window: a header dressed for the shopkeeper's race, what they say, the shop's sections on the left, the
-- items of the chosen section as cards, and a preview of the chosen item (its icon, the game's description, level
-- and jobs, stock, why it can't be bought) with the price and a Buy button. Buy asks once more before it buys.
-- "Old menu" opens the shop's classic menus (for anything the window doesn't show, like guild projects).

local S = require("lib.style")
local Canvas = require("lib.canvas")
local protocol = require("lib.protocol")
local themes = require("lib.themes")

local W = {}

W.WIN = "shop"
W.CW, W.CH = 660, 432
W.HEAD = 60
W.LIST_W = 120
W.GRID_X, W.GRID_W = 128, 300
W.PREV_X = 436
W.CARD_H = 44

W.model = protocol.new_model()
W.state = { section = 1, item = nil, page = 1, confirm = nil, voice = nil, waiting = false, token = nil, demo = false }

W.send = function() end -- set by init.lua: sends a "sh ..." request

local function theme() return themes.for_shop(W.model.shop) end

-- Item names and descriptions from the game's own tables, when the item is one
function W.item_info(item)
  if not item or (item.id or 0) == 0 then return nil end
  local ok, r = pcall(res.item, item.id)
  return ok and type(r) == "table" and r or nil
end

local function jobs_text(mask)
  if not mask or mask == 0 then return nil end
  if mask & 0x7FFFFE == 0x7FFFFE then return "All jobs" end
  local out = {}
  for j = 1, 22 do
    if mask & (1 << j) ~= 0 then out[#out + 1] = S.job_short(j) end
  end
  return #out > 0 and table.concat(out, " ") or nil
end

-- Little pictures for things that aren't items
function W.glyph(c, x, y, s, g, col)
  local H = themes.helpers
  local cx, cy = x + s / 2, y + s / 2
  if g == "sack" then
    c:circle(cx, cy + s * 0.12, s * 0.34, col)
    H.tri(c, cx - s * 0.18, y + s * 0.18, cx + s * 0.18, y + s * 0.18, cx, cy, col)
    c:line(cx - s * 0.16, y + s * 0.3, cx + s * 0.16, y + s * 0.3, { 40, 24, 10, 220 }, 2)
    c:text(cx, cy - s * 0.04, "?", { size = s / 26, align = "center", bold = true, colour = { 40, 24, 10, 255 } })
  elseif g == "tonic" then
    c:rect(cx - s * 0.2, cy - s * 0.05, s * 0.4, s * 0.42, col, { round = 3 })
    c:rect(cx - s * 0.08, y + s * 0.12, s * 0.16, s * 0.22, col)
    c:rect(cx - s * 0.11, y + s * 0.06, s * 0.22, s * 0.08, { 160, 110, 70, 255 }, { round = 1 })
  elseif g == "lure" then
    H.star(c, cx, cy, s * 0.4, col)
  elseif g == "focus" then
    c:circle(cx, cy, s * 0.36, col, 2)
    c:circle(cx, cy, s * 0.14, col)
  elseif g == "feast" then
    c:circle(cx, cy + s * 0.12, s * 0.38, { col[1], col[2], col[3], 110 })
    c:circle(cx, cy + s * 0.04, s * 0.26, col)
    c:circle(cx, cy - s * 0.24, s * 0.05, col)
  elseif g == "gil" then
    c:circle(cx, cy, s * 0.36, col)
    c:text(cx, cy - s * 0.24, "G", { size = s / 26, align = "center", bold = true, colour = { 60, 40, 10, 255 } })
  elseif g == "slot" then
    c:circle(cx, y + s * 0.32, s * 0.16, col)
    c:rect(cx - s * 0.26, cy + s * 0.04, s * 0.52, s * 0.3, col, { round = 6 })
  elseif g == "recall" then
    c:circle(cx, cy, s * 0.32, col, 2.5)
    H.tri(c, cx + s * 0.22, cy - s * 0.3, cx + s * 0.42, cy - s * 0.06, cx + s * 0.14, cy - s * 0.02, col)
  elseif g == "hourglass" then
    H.tri(c, cx - s * 0.26, y + s * 0.14, cx + s * 0.26, y + s * 0.14, cx, cy, col)
    H.tri(c, cx - s * 0.26, y + s * 0.86, cx + s * 0.26, y + s * 0.86, cx, cy, { col[1], col[2], col[3], 150 })
  else
    H.diamond(c, cx, cy, s * 0.3, col)
  end
end

local function draw_icon(c, item, x, y, s, t)
  c:rect(x, y, s, s, { 0, 0, 0, 70 }, { round = 4 })
  local icon = (item.icon or 0) > 0 and item.icon or item.id
  if (icon or 0) > 0 then
    c:icon(x + 2, y + 2, s - 4, "item", icon)
  else
    W.glyph(c, x + 3, y + 3, s - 6, item.glyph, t.colours.accent)
  end
end

local function current_section()
  local shop = W.model.shop
  return shop and shop.sections[W.state.section]
end

local function selected_item()
  local sec = current_section()
  return sec and W.state.item and sec.items[W.state.item] or nil
end

-- What the shopkeeper is saying now
function W.speech()
  local shop, t, v = W.model.shop, theme(), W.state.voice
  if v and v.kind == "server" then return shop and shop.greeting or themes.say(t, "greet", v.n) end
  if v then return themes.say(t, v.kind, v.n) end
  return themes.say(t, "greet", 1)
end

function W.draw_head(c, t)
  local shop = W.model.shop
  local col = t.colours
  c:rect(0, 0, W.CW, W.HEAD, col.bg, { to = col.bg2 })
  t.deco(c, W.CW, W.HEAD)
  local title = (shop and shop.title and t.key == "guild") and shop.title or t.shop(shop and shop.keeper)
  c:text(16, 10, title, { size = 1.35, bold = true, colour = col.title, outline = t.title_on_light and nil or { 0, 0, 0, 160 }, w = W.CW - 230 })
  c:text(16, 34, ("%s  ·  %s"):format(shop and shop.keeper or "", t.label), { size = 0.85, colour = t.title_on_light and col.title or col.muted, w = W.CW - 230 })
end

function W.draw_bubble(c, t)
  local col = t.colours
  local y = W.HEAD + 8
  c:rect(10, y, W.CW - 220, 30, { 0, 0, 0, 90 }, { round = 8 })
  c:rect(10, y, W.CW - 220, 30, col.edge, { round = 8, line = 1 })
  themes.helpers.tri(c, 24, y, 34, y, 26, y - 7, col.edge)
  c:text(22, y + 7, "\"" .. W.speech() .. "\"", { size = 0.92, colour = col.text, w = W.CW - 250 })
  local shop = W.model.shop
  if shop then
    local money = t.money(shop.balance, shop.currency)
    c:text(W.CW - 14, y + 2, "You have", { size = 0.75, align = "right", colour = col.muted })
    c:text(W.CW - 14, y + 14, money, { size = 1.0, align = "right", bold = true, colour = col.price, w = 190 })
  end
end

function W.draw_sections(c, t)
  local col, shop = t.colours, W.model.shop
  local y = W.HEAD + 48
  for i, sec in ipairs(shop.sections) do
    local sel = i == W.state.section
    local hot = c:region(8, y, W.LIST_W - 8, 24, { key = "sec" .. i, on = function()
      W.state.section, W.state.item, W.state.page, W.state.confirm = i, 1, 1, nil
      W.render()
    end })
    c:rect(8, y, W.LIST_W - 8, 24, sel and col.card_hi or (hot and col.card or { 0, 0, 0, 0 }), { round = 5 })
    if sel then c:rect(8, y + 4, 3, 16, col.accent, { round = 1 }) end
    c:text(16, y + 5, sec.name, { size = 0.86, bold = sel, colour = sel and col.text or col.muted, w = W.LIST_W - 40 })
    c:text(W.LIST_W - 6, y + 6, tostring(#sec.items), { size = 0.72, align = "right", colour = col.muted })
    y = y + 27
    if y > W.CH - 60 then break end
  end
end

function W.per_page()
  local rows = math.floor((W.CH - 40 - (W.HEAD + 48)) / (W.CARD_H + 6))
  return rows * 2
end

function W.draw_grid(c, t)
  local col, shop = t.colours, W.model.shop
  local sec = current_section()
  if not sec then return end
  local per = W.per_page()
  local pages = math.max(1, math.ceil(#sec.items / per))
  W.state.page = math.max(1, math.min(W.state.page, pages))
  local cw = (W.GRID_W - 6) / 2
  local first = (W.state.page - 1) * per + 1
  for n = first, math.min(#sec.items, first + per - 1) do
    local item = sec.items[n]
    local k = n - first
    local x = W.GRID_X + (k % 2) * (cw + 6)
    local y = W.HEAD + 48 + math.floor(k / 2) * (W.CARD_H + 6)
    local sel = W.state.item == n
    local okbuy = protocol.affordable(shop, item)
    local hot = c:region(x, y, cw, W.CARD_H, { key = "item" .. n, on = function()
      W.state.item, W.state.confirm = n, nil
      W.render()
    end })
    c:rect(x, y, cw, W.CARD_H, sel and col.card_hi or (hot and col.card_hi or col.card), { round = 6 })
    c:rect(x, y, cw, W.CARD_H, sel and col.accent or col.edge, { round = 6, line = sel and 1.6 or 1 })
    if t.card then t.card(c, x, y, cw, W.CARD_H, sel) end
    draw_icon(c, item, x + 6, y + 6, 32, t)
    local fade = item.can and 1 or 0.5
    c:text(x + 44, y + 7, item.label, { size = 0.82, bold = true, colour = S.fade(col.text, fade), w = cw - 50 })
    c:text(x + 44, y + 24, t.money(item.price, shop.currency, true), { size = 0.76, colour = okbuy and col.price or S.fade(col.price, 0.55), w = cw - 84 })
    if item.left >= 0 then
      c:text(x + cw - 6, y + 25, item.left == 0 and "sold out" or (item.left .. " left"), { size = 0.7, align = "right",
        colour = item.left == 0 and col.bad or col.muted })
    end
    if not item.can and item.why then c:tip(x, y, cw, W.CARD_H, item.label, item.why) end
  end
  if pages > 1 then
    local y = W.CH - 58
    c:text(W.GRID_X + W.GRID_W / 2, y + 3, ("%d / %d"):format(W.state.page, pages), { size = 0.78, align = "center", colour = col.muted })
    c:button(W.GRID_X, y, 54, 20, "‹ Prev", { size = 0.76, key = "prev", enabled = W.state.page > 1, on = function()
      W.state.page = W.state.page - 1 W.render()
    end })
    c:button(W.GRID_X + W.GRID_W - 54, y, 54, 20, "Next ›", { size = 0.76, key = "next", enabled = W.state.page < pages, on = function()
      W.state.page = W.state.page + 1 W.render()
    end })
  end
end

function W.draw_preview(c, t)
  local col, shop = t.colours, W.model.shop
  local x, w = W.PREV_X, W.CW - W.PREV_X - 8
  local top = W.HEAD + 48
  c:rect(x, top, w, W.CH - 40 - top, { 0, 0, 0, 80 }, { round = 8 })
  c:rect(x, top, w, W.CH - 40 - top, col.edge, { round = 8, line = 1 })
  local item = selected_item()
  if not item then
    c:para(x + 12, top + 14, w - 24, themes.say(t, "browse", 1), { size = 0.86, colour = col.muted })
    return
  end
  local info = W.item_info(item)
  draw_icon(c, item, x + 12, top + 12, 40, t)
  local name = info and info.name or item.label
  local y = top + 14
  for _, line in ipairs(Canvas.wrap(name, w - 70, 0.95, true)) do
    c:text(x + 60, y, line, { size = 0.95, bold = true, colour = col.text })
    y = y + 16
  end
  y = math.max(y, top + 58)
  local facts = {}
  if info and (info.level or 0) > 0 then facts[#facts + 1] = "Lv. " .. info.level end
  local jt = info and jobs_text(info.jobs)
  if jt then facts[#facts + 1] = jt end
  if item.left >= 0 then facts[#facts + 1] = item.left == 0 and "Sold out" or (item.left .. " left") end
  if #facts > 0 then
    y = c:para(x + 12, y, w - 24, table.concat(facts, "  ·  "), { size = 0.76, colour = col.accent })
  end
  local desc = (info and info.description and info.description ~= "") and info.description or item.desc
  if desc then
    y = c:para(x + 12, y + 2, w - 24, desc, { size = 0.8, colour = col.muted }) + 2
  end
  if item.desc and info and info.description and info.description ~= "" then
    y = c:para(x + 12, y, w - 24, item.desc, { size = 0.78, colour = col.muted }) + 2
  end
  local bottom = W.CH - 40 - 12
  if not item.can and item.why then
    c:para(x + 12, bottom - 66, w - 24, item.why, { size = 0.78, colour = col.bad })
  elseif item.can and shop.balance < item.price then
    c:para(x + 12, bottom - 66, w - 24, themes.say(t, "poor", 1), { size = 0.78, colour = col.bad })
  end
  c:text(x + 12, bottom - 30, t.money(item.price, shop.currency), { size = 1.05, bold = true, colour = col.price, w = w - 24 })
  local can = protocol.affordable(shop, item) and not W.state.waiting
  local key = item.s .. "." .. item.i
  local confirming = W.state.confirm == key
  local bw = confirming and (w - 24 - 70) or (w - 24)
  local label = W.state.waiting and "..." or (confirming and ("Buy for " .. item.price .. "?") or "Buy")
  local hot = c:region(x + 12, bottom - 10, bw, 26, { key = "buy", enabled = can, on = function()
    if W.state.confirm == key then
      W.state.confirm, W.state.waiting = nil, true
      W.send(("buy %d %d %d"):format(shop.token, item.s, item.i))
    else
      W.state.confirm = key
    end
    W.render()
  end })
  local fill = can and (hot and S.fade(col.buy_edge, 0.55) or col.buy) or { 80, 80, 80, 90 }
  c:rect(x + 12, bottom - 10, bw, 26, fill, { round = 6 })
  c:rect(x + 12, bottom - 10, bw, 26, can and col.buy_edge or { 140, 140, 140, 120 }, { round = 6, line = 1.2 })
  c:text(x + 12 + bw / 2, bottom - 4, label, { size = 0.9, bold = true, align = "center", colour = can and col.text or S.fade(col.text, 0.5), w = bw - 6 })
  if confirming then
    c:button(x + w - 12 - 64, bottom - 10, 64, 26, "Cancel", { size = 0.82, key = "cancel", on = function()
      W.state.confirm = nil W.render()
    end })
  end
end

function W.draw_footer(c, t)
  local col = t.colours
  local y = W.CH - 28
  local r = W.model.result
  if r then
    c:text(10, y + 5, r.msg, { size = 0.78, colour = r.ok and col.accent or col.bad, w = W.CW - 220 })
  end
  c:button(W.CW - 196, y, 92, 22, "Old menu", { size = 0.8, key = "classic", tip = "The shop's classic menus (guild projects and the rest).",
    enabled = not W.state.demo, on = function()
      W.send(("classic %d"):format(W.model.shop.token))
      W.close()
    end })
  c:button(W.CW - 98, y, 90, 22, "Close", { size = 0.8, key = "close", on = function() W.close(true) end })
end

function W.draw(c)
  local t = theme()
  c:rect(0, 0, W.CW, W.CH, t.colours.body, { round = 6 })
  if not W.model.shop then
    c:text(16, 16, "No shop open. Talk to a shopkeeper.", { colour = S.MUTED })
    return
  end
  W.draw_head(c, t)
  W.draw_bubble(c, t)
  W.draw_sections(c, t)
  W.draw_grid(c, t)
  W.draw_preview(c, t)
  W.draw_footer(c, t)
end

function W.render()
  if ui.is_open(W.WIN) then Canvas.render(W.WIN, "shop") end
end

function W.setup()
  ui.window(W.WIN, {
    title = "Shop",
    width = W.CW + 20,
    help = "Choose a section on the left and an item to see it. Buy asks once more before it buys.",
    elements = { { type = "canvas", id = "shop", w = W.CW, h = W.CH, draw = {} } },
  })
  Canvas.add(W.WIN, "shop", W.CW, W.draw)
  ui.on(W.WIN, function(win, e)
    if e.kind == "close" then
      if not W.state.demo then W.send("close") end
      return
    end
    Canvas.event(win, e)
  end)
end

-- A shop came in: open it (a new shop starts at its first section and the keeper's greeting; a refresh after a
-- purchase keeps the place)
function W.opened()
  local shop = W.model.shop
  local same = W.state.token ~= nil and W.state.keeper == shop.keeper and W.state.open
  W.state.token, W.state.keeper = shop.token, shop.keeper
  W.state.waiting = false
  if not same then
    W.state.section, W.state.item, W.state.page, W.state.confirm = 1, 1, 1, nil
    W.state.voice = shop.greeting and { kind = "server" } or { kind = "greet", n = math.random(1, 3) }
    W.model.result = nil
  end
  local sec = shop.sections[W.state.section]
  if not sec then W.state.section, W.state.item = 1, 1 end
  if not ui.is_open(W.WIN) then ui.open(W.WIN) end
  W.state.open = true
  W.render()
end

-- A purchase's answer: what the keeper says about it
function W.answered()
  local r = W.model.result
  W.state.waiting = false
  local kind = "buy"
  if not r.ok then
    local m = r.msg:lower()
    kind = (m:find("costs") or m:find("everglass; you have")) and "poor" or ((m:find("sold") or m:find("gone")) and "sold" or nil)
  end
  W.state.voice = kind and { kind = kind, n = math.random(1, 3) } or W.state.voice
  W.render()
end

-- Closes the window; tell: let the server forget the shop (not after "Old menu", which already did)
function W.close(tell)
  if tell and W.state.open and not W.state.demo then W.send("close") end
  W.state.open = false
  if ui.is_open(W.WIN) then ui.close(W.WIN) end
end

return W
