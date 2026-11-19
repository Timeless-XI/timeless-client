-- TimelessMenu's window: one card per TimelessXI mod (what it does, Open and its tab buttons), then the server's
-- commands.
--
-- The cards come from the mods themselves: a loaded mod that offers "timeless_menu" (mods.provide) gives its card(s)
-- when the menu draws, so a new mod shows up here without a change to the menu. The TimelessXI mods the menu knows
-- also get a card when they are installed but not running (with Load) or not installed yet.

local S = require("lib.style")
local Canvas = require("lib.canvas")

local M = {}

M.WIN = "timeless_menu"
M.WIDTH = 560
local CW = 536

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

local PAD = 12
local BH = 26 -- button height

local function button_w(it) return math.max(60, Canvas.measure(it.label, 0.9, it.primary) + 24) end

-- How many rows a row of buttons takes in w.
local function rows(items, w)
  if #items == 0 then return 0 end
  local cx, n = 0, 1
  for _, it in ipairs(items) do
    local bw = button_w(it)
    if cx > 0 and cx + bw > w then cx, n = 0, n + 1 end
    cx = cx + bw + 6
  end
  return n
end

-- A row of buttons from x, wrapping onto new rows. Returns the y below.
local function button_row(c, x, y, w, items, enabled, key)
  local cx, cy = x, y
  for i, it in ipairs(items) do
    local bw = button_w(it)
    if cx > x and cx + bw > x + w then
      cx, cy = x, cy + BH + 6
    end
    c:button(cx, cy, bw, BH, it.label, { style = it.primary and "primary" or "plain", enabled = enabled, tip = it.tip,
      bold = it.primary, key = key .. i, on = function()
        M.run(it.command)
        if it.after then it.after() end
      end })
    cx = cx + bw + 6
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

-- One card's height and drawing.
local function card_height(card, entry)
  local inner = CW - PAD * 2
  local h = PAD + 20 + Canvas.wrapped_height(card.desc or "", inner, 0.95)
  local r = rows(card_items(card, entry), inner)
  if r > 0 then h = h + 8 + r * BH + (r - 1) * 6 end
  return h + PAD
end

local function draw_card(c, y, card, entry, n)
  local h = card_height(card, entry)
  c:card(0, y, CW, h)
  local x, inner = PAD, CW - PAD * 2
  local ty = y + PAD
  local tw = c:text(x, ty, card.title, { bold = true, colour = entry.state == "running" and S.GOLD or S.MUTED, w = inner - 140 })
  local right = card.command or ""
  if entry.state == "off" then right = "installed, not running"
  elseif entry.state == "missing" then right = "not installed yet" end
  c:text(CW - PAD, ty + 1, right, { size = 0.85, colour = S.FAINT, align = "right", w = inner - tw - 16 })
  ty = c:para(x, ty + 20, inner, card.desc or "", { size = 0.95, colour = S.MUTED })
  local items = card_items(card, entry)
  if #items > 0 then button_row(c, x, ty + 8, inner, items, true, "c" .. n .. "_") end
  return y + h + 8
end

function M.draw(c)
  local y = 0
  local n = 0
  for _, entry in ipairs(M.entries()) do
    for _, card in ipairs(entry.cards) do
      n = n + 1
      y = draw_card(c, y, card, entry, n)
    end
  end
  y = c:heading(0, y + 4, CW, "Server", "answers come in chat")
  y = button_row(c, 0, y, CW, M.SERVER, true, "srv")
  if M.state.note and os.clock() - M.state.note_at < 8 then
    c:text(0, y + 8, M.state.note, { size = 0.85, colour = S.FAINT, w = CW })
  end
  c:para(0, y + (M.state.note and 28 or 8), CW,
    "Drag the hourglass button anywhere; right-click it to hide it (/tm button brings it back).",
    { size = 0.8, colour = S.FAINT })
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
