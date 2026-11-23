-- TimelessMenu's window: the TimelessXI mods by category (Pinned, Companions, World, Combat, Crafting & Shops,
-- Games, Other), each a compact row (what it does, Open and its tab buttons), and the server's commands.
--
-- The cards come from the mods themselves: a loaded mod that offers "timeless_menu" (mods.provide) gives its card(s)
-- when the menu draws, so a new mod shows up here without a change to the menu. The TimelessXI mods the menu knows
-- also get a card when they are installed but not running (with Load) or not installed yet.

local S = require("lib.style")
local Canvas = require("lib.canvas")

local M = {}

M.WIN = "timeless_menu"
M.WIDTH = 720
local CW = 696

-- The TimelessXI mods: their cards when they aren't running to give their own.
M.KNOWN = {
  {
    id = "timeless-companions", order = 10,
    cards = {
      { title = "Companion Manager", open = "//cmgr", command = "/cmgr",
        desc = "Call, keep, save and set up your companions.",
        buttons = {
          { label = "Party", command = "//cmgr party" }, { label = "Call", command = "//cmgr call" },
          { label = "Kept", command = "//cmgr kept" }, { label = "Presets", command = "//cmgr presets" },
          { label = "Tactics", command = "//cmgr tactics" }, { label = "Settings", command = "//cmgr settings" },
        } },
      { title = "Companion View", open = "//cv", command = "/cv",
        desc = "Your companions' gear, stats and role." },
    },
  },
  {
    id = "timeless-journal", order = 20,
    cards = { { title = "Journal", open = "//ql", command = "/ql",
      desc = "Your missions and quests, and what to do next." } },
  },
  {
    id = "timeless-gauges", order = 30,
    cards = { { title = "Job Gauges", open = "//gauges", command = "/gauges",
      desc = "Your job's own meters (Faith, Essence, Ki, Hot Streak...) on the screen's edge." } },
  },
}

-- The server's commands, said in /say; the answers come in chat.
M.SERVER = {
  { label = "Welcome", command = "/say !welcome", tip = "What TimelessXI adds, and where to start." },
  { label = "Companions", command = "/say !companion status", tip = "Your companions, bond and Everglass." },
  { label = "Guild", command = "/say !guild", tip = "The Artisan Guild: your standing and its stock." },
  { label = "Expedition", command = "/say !expedition", tip = "The guild's gathering expeditions." },
  { label = "Dig", command = "/say !dig", tip = "Chocobo digging: your skill and today's finds." },
  { label = "Chocobo", command = "/say !chocobo", tip = "Your chocobo: training, colour and whistle." },
  { label = "Journal", command = "/say !journal", tip = "Your progress, in chat." },
}

M.state = { note = nil, note_at = 0 }

-- Runs a command line (set by init.lua; commands.run in the game).
M.run = function(line) commands.run(line) end

-- A card list from what a mod's "timeless_menu" gave: one card, or a list of them.
local function as_cards(v)
  if type(v) ~= "table" then return {} end
  if v.title then return { v } end
  local out = {}
  for _, c in ipairs(v.cards or v) do
    if type(c) == "table" and c.title then out[#out + 1] = c end
  end
  return out
end

-- Every mod's entry: { id, name, order, state = "running" | "off" | "missing", cards }.
function M.entries()
  local installed, running = {}, {}
  for _, m in ipairs(mods.installed()) do installed[m.id] = m end
  for _, m in ipairs(mods.list()) do running[m.id] = m end
  local out, seen = {}, {}
  -- Running mods that give their own cards (any mod, TimelessXI's or not)
  for id, m in pairs(running) do
    local gives = false
    for _, name in ipairs(m.provides or {}) do
      if name == "timeless_menu" then gives = true end
    end
    if gives and id ~= "timeless-menu" then
      local okc, v = pcall(mods.call, id, "timeless_menu")
      local cards = okc and as_cards(v) or {}
      if #cards > 0 then
        out[#out + 1] = { id = id, name = m.name, order = tonumber(cards[1].order) or 50, state = "running", cards = cards }
        seen[id] = true
      end
    end
  end
  -- TimelessXI's mods that didn't give one: running (an older version), off, or not installed
  for _, k in ipairs(M.KNOWN) do
    if not seen[k.id] then
      local state = running[k.id] and "running" or (installed[k.id] and "off" or "missing")
      out[#out + 1] = { id = k.id, name = installed[k.id] and installed[k.id].name or k.id, order = k.order,
        state = state, cards = k.cards }
    end
  end
  table.sort(out, function(a, b)
    if a.order ~= b.order then return a.order < b.order end
    return a.id < b.id
  end)
  return out
end

-----------------------------------
-- The layout: categories down the left, the chosen one's mods on the right as compact rows (title, one line of
-- what it does with the rest in its tip, Open and its tabs). A mod names its category in its card
-- (category = "Combat"); the menu knows TimelessXI's own; anything else goes under Other. Pin a mod (the star)
-- and it's also under Pinned, which opens first. The window keeps one height: its tallest category's.
-----------------------------------
M.CATEGORIES = {
  { key = "Pinned", tip = "The mods you've pinned (the star on a row)." },
  { key = "Companions", tip = "Your companions." },
  { key = "World", tip = "The world around you: maps, towns, notorious monsters, your quests and fame." },
  { key = "Combat", tip = "In a fight: your gauges, enmity, monsters' moves and casts, Champions." },
  { key = "Crafting & Shops", tip = "Shops, crafting and Masterworks." },
  { key = "Games", tip = "Cardian and other games." },
  { key = "Other", tip = "Mods that don't name a category." },
  { key = "Server", tip = "The server's commands; the answers come in chat." },
}

-- TimelessXI's mods, by card title, for cards that don't say (older versions)
M.CATEGORY_OF = {
  ["Companion Manager"] = "Companions", ["Companion View"] = "Companions",
  ["World Map"] = "World", ["Town"] = "World", ["NM Atlas"] = "World", ["Journal"] = "World", ["Reputation"] = "World",
  ["Job Gauges"] = "Combat", ["Enmity"] = "Combat", ["Telegraphs"] = "Combat", ["Champions"] = "Combat", ["Cast Bars"] = "Combat",
  ["Shops"] = "Crafting & Shops", ["Masterworks"] = "Crafting & Shops",
  ["Cardian"] = "Games",
}

local SIDE = 150  -- the categories' column
local GAP = 12
local RX = SIDE + GAP
local RW = CW - RX
local PAD = 10
local BH = 24 -- button height

local function prefs()
  storage.account.menu = storage.account.menu or {}
  local p = storage.account.menu
  p.pinned = p.pinned or {}
  return p
end
M.prefs = prefs

local function category(card)
  local c = card.category or M.CATEGORY_OF[card.title]
  for _, k in ipairs(M.CATEGORIES) do
    if k.key == c and c ~= "Pinned" and c ~= "Server" then return c end
  end
  return "Other"
end
M.category = category

-- Every card with its entry, by category: { [category] = { { card, entry }, ... } }
function M.grouped()
  local out = {}
  for _, k in ipairs(M.CATEGORIES) do out[k.key] = {} end
  local pinned = prefs().pinned
  for _, entry in ipairs(M.entries()) do
    for _, card in ipairs(entry.cards) do
      table.insert(out[category(card)], { card = card, entry = entry })
      if pinned[card.title] then table.insert(out.Pinned, { card = card, entry = entry }) end
    end
  end
  return out
end

local function button_w(it) return math.max(52, Canvas.measure(it.label, 0.85, it.primary) + 18) end

-- How many rows a row of buttons takes in w.
local function rows(items, w)
  if #items == 0 then return 0 end
  local cx, n = 0, 1
  for _, it in ipairs(items) do
    local bw = button_w(it)
    if cx > 0 and cx + bw > w then cx, n = 0, n + 1 end
    cx = cx + bw + 5
  end
  return n
end

-- A row of buttons from x, wrapping onto new rows. Returns the y below.
local function button_row(c, x, y, w, items, enabled, key)
  local cx, cy = x, y
  for i, it in ipairs(items) do
    local bw = button_w(it)
    if cx > x and cx + bw > x + w then
      cx, cy = x, cy + BH + 5
    end
    c:button(cx, cy, bw, BH, it.label, { style = it.primary and "primary" or "plain", enabled = enabled, tip = it.tip,
      bold = it.primary, size = 0.85, key = key .. i, on = function()
        M.run(it.command)
        if it.after then it.after() end
      end })
    cx = cx + bw + 5
  end
  return cy + BH
end

-- A card's buttons: Open and its own when the mod runs, Load when it's installed but off, none when it's missing.
local function card_items(card, entry)
  local items = {}
  if entry.state == "running" then
    if card.open then items[#items + 1] = { label = "Open", command = card.open, primary = true, tip = "Show or hide " .. card.title .. "." } end
    for _, b in ipairs(card.buttons or {}) do items[#items + 1] = b end
  elseif entry.state == "off" then
    items[1] = { label = "Load", primary = true, command = "/lua load " .. entry.id,
      tip = "Start " .. (entry.name or entry.id) .. " now.", after = function() M.state.refresh_at = os.clock() + 1 end }
  end
  return items
end

local function row_height(card, entry)
  local r = rows(card_items(card, entry), RW - PAD * 2)
  local h = PAD + 18 + 16
  if r > 0 then h = h + 6 + r * BH + (r - 1) * 5 end
  return h + PAD
end

local function draw_row(c, y, card, entry, n)
  local h = row_height(card, entry)
  c:card(RX, y, RW, h)
  local x, inner = RX + PAD, RW - PAD * 2
  local ty = y + PAD
  local p = prefs()
  local pinned = p.pinned[card.title]
  -- The pin: a small button at the right
  c:button(RX + RW - PAD - 46, ty - 2, 46, 20, pinned and "Pinned" or "Pin", { style = pinned and "primary" or "plain", size = 0.72,
    key = "pin" .. n, tip = pinned and "Take it out of Pinned." or "Also show it under Pinned, which opens first.",
    on = function()
      p.pinned[card.title] = not pinned or nil
      pcall(storage.save)
      M.render()
    end })
  local tw = c:text(x, ty, card.title, { bold = true, colour = entry.state == "running" and S.GOLD or S.MUTED, w = inner - 120 })
  local right = card.command or ""
  if entry.state == "off" then right = "installed, not running"
  elseif entry.state == "missing" then right = "not installed yet" end
  c:text(RX + RW - PAD - 54, ty + 1, right, { size = 0.8, colour = S.FAINT, align = "right", w = inner - tw - 70 })
  -- What it does: one line here, all of it in the tip
  local desc = card.desc or ""
  local line = Canvas.fit(desc, inner, 0.85)
  c:region(x, ty + 18, inner, 16, { key = "desc" .. n, title = card.title, tip = line ~= desc and desc or nil })
  c:text(x, ty + 18, line, { size = 0.85, colour = S.MUTED, w = inner })
  local items = card_items(card, entry)
  if #items > 0 then button_row(c, x, ty + 40, inner, items, true, "c" .. n .. "_") end
  return y + h + 6
end

-- The height of a category's list, to keep the window one size
local function list_height(list)
  local h = 0
  for _, it in ipairs(list) do h = h + row_height(it.card, it.entry) + 6 end
  return h
end

local function server_height()
  return 26 + rows(M.SERVER, RW) * (BH + 5) + 40
end

-- The category shown: the one picked; Pinned first when anything is pinned
function M.current(groups)
  local p = prefs()
  local cur = M.state.category or p.category
  if cur == nil or (cur ~= "Server" and #(groups[cur] or {}) == 0) then
    cur = #groups.Pinned > 0 and "Pinned" or "Companions"
    if #(groups[cur] or {}) == 0 then
      for _, k in ipairs(M.CATEGORIES) do
        if k.key ~= "Server" and #groups[k.key] > 0 then cur = k.key break end
      end
    end
  end
  return cur
end

function M.draw(c)
  local groups = M.grouped()
  local cur = M.current(groups)
  -- The categories: the ones with something in them (and Server)
  local y = 0
  local tallest = 0
  for i, k in ipairs(M.CATEGORIES) do
    local list = groups[k.key] or {}
    if k.key == "Server" or #list > 0 then
      local lit = k.key == cur
      c:button(0, y, SIDE, 30, k.key, { style = lit and "on" or "off", key = "cat" .. i, tip = k.tip,
        sub = nil, on = (not lit) and function()
          M.state.category = k.key
          local p = prefs()
          p.category = k.key
          pcall(storage.save)
          M.render()
        end or nil })
      if k.key ~= "Server" then
        c:text(SIDE - 10, y + 7, tostring(#list), { size = 0.78, colour = lit and S.TEXT or S.FAINT, align = "right" })
      end
      y = y + 34
      tallest = math.max(tallest, k.key == "Server" and server_height() or list_height(list))
    end
  end
  c:para(0, y + 6, SIDE, "Drag the hourglass anywhere; right-click it to hide it (/tm button).", { size = 0.72, colour = S.FAINT })
  -- The chosen category
  local ry = 0
  if cur == "Server" then
    ry = c:heading(RX, 0, RW, "Server", "answers come in chat")
    ry = button_row(c, RX, ry, RW, M.SERVER, true, "srv")
  else
    for n, it in ipairs(groups[cur] or {}) do
      ry = draw_row(c, ry, it.card, it.entry, n)
    end
  end
  if M.state.note and os.clock() - M.state.note_at < 8 then
    c:text(RX, ry + 8, M.state.note, { size = 0.8, colour = S.FAINT, w = RW })
  end
  -- One height for every category, so the window doesn't jump about
  c:grow(math.max(tallest, y + 60))
end

function M.setup()
  ui.window(M.WIN, {
    title = "TimelessMenu",
    width = M.WIDTH,
    help = "Every TimelessXI mod in one place: open them and jump to their tabs, and the server's commands.",
    elements = { { type = "canvas", id = "cards", w = CW, h = 200, draw = {} } },
  })
  Canvas.add(M.WIN, "cards", CW, M.draw)
  ui.on(M.WIN, function(win, e) Canvas.event(win, e) end)
end

function M.render()
  if ui.is_open(M.WIN) then Canvas.render(M.WIN, "cards") end
end

function M.is_open() return ui.is_open(M.WIN) end

function M.open()
  ui.show(M.WIN)
  M.render()
end

function M.toggle()
  if ui.is_open(M.WIN) then ui.close(M.WIN) else M.open() end
end

return M
