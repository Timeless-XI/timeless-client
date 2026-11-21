-- The cardbook (/cardbook, the Timeless menu): every card you have, the ones still to find as silhouettes, the
-- foils from Champions, a big card with the monster's model turning slowly, and your deck of fifteen to build and save.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local cards = require("lib.cards")
local Art = require("lib.art")
local R = require("lib.rules")

local B = {}

B.WIN = "cardbook"
B.W = 900
B.COLS, B.ROWS, B.S = 5, 2, 0.82
B.state = { tab = 1, page = 1, pick = 1, edit = nil, note = nil }
B.send = function() end -- "ih ..." to the server, set by init.lua
B.howto = function() end -- opens How to play, set by init.lua
B.to_table = function() end -- opens the table, set by init.lua

local TABS = {
  { label = "All", tip = "Every card, the ones you have and the ones still to find." },
  { label = "Commons", tip = "Field monsters' cards." },
  { label = "Rares", tip = "Notorious monsters' cards." },
  { label = "Legendaries", tip = "A great wyrm's, and the Steaming Sheep regulars' own (beat one three times)." },
  { label = "Foils", tip = "Champions' cards: Gilded, Nightborne, Rift-touched and Juggernaut. The ones you have." },
}

local REGULARS = { { "hrodgar", "Hrodgar" }, { "fenwick", "Fenwick" }, { "kipiko", "Kipiko" }, { "dagny", "Dagny" } }

local function have() return B.model and B.model.book.have or {} end

-- The cards a tab lists
function B.list()
  local out = {}
  local t = B.state.tab
  if t == 5 then
    for id, n in pairs(have()) do
      if id > 1000 and n > 0 then out[#out + 1] = id end
    end
    table.sort(out, function(a, b) return (a % 1000) * 10 + a // 1000 < (b % 1000) * 10 + b // 1000 end)
    return out
  end
  for _, c in ipairs(cards.list) do
    if t == 1 or c.rarity == t - 1 then out[#out + 1] = c.id end
  end
  return out
end

function B.deck()
  return B.state.edit or (B.model and B.model.book.deck) or {}
end

local function in_deck(id)
  local n = 0
  for _, d in ipairs(B.deck()) do if d == id then n = n + 1 end end
  return n
end

local function edit()
  if not B.state.edit then
    B.state.edit = {}
    for i, id in ipairs(B.model and B.model.book.deck or {}) do B.state.edit[i] = id end
  end
  return B.state.edit
end

function B.add(id)
  local e = edit()
  if #e >= R.DECK_SIZE then B.state.note = "Your deck has fifteen already: take one out first." return end
  e[#e + 1] = id
  table.sort(e, function(a, b) return (a % 1000) < (b % 1000) or ((a % 1000) == (b % 1000) and a < b) end)
  B.state.note = nil
  B.render()
end

function B.remove(id)
  local e = edit()
  for i = #e, 1, -1 do
    if e[i] == id then table.remove(e, i) break end
  end
  B.state.note = nil
  B.render()
end

function B.save()
  local e = B.state.edit
  if not e then return end
  local why = R.deck_problem(e, have())
  if why then B.state.note = why B.render() return end
  local ids = {}
  for i, id in ipairs(e) do ids[i] = tostring(id) end
  B.send("ih deck " .. table.concat(ids, ","))
  B.state.note = "Saving..."
  B.render()
end

function B.draw(c)
  local m = B.model
  local h = have()
  local st = B.state
  -- The header: counts and tabs
  local owned, foils = 0, 0
  for id, n in pairs(h) do
    if n > 0 then
      if id > 1000 then foils = foils + 1 else owned = owned + 1 end
    end
  end
  c:text(0, 2, "Cardbook", { size = 1.3, bold = true, colour = S.GOLD })
  c:text(110, 7, ("%d of %d cards  ·  %d foils"):format(owned, #cards.list, foils), { size = 0.85, colour = S.MUTED })
  c:switch(B.W - 470, 0, 26, TABS, st.tab, { on = function(i) st.tab, st.page = i, 1 B.render() end, key = "tab" })
  local list = B.list()
  local per = B.COLS * B.ROWS
  local pages = math.max(1, math.ceil(#list / per))
  st.page = math.min(st.page, pages)
  local cw, ch = Art.W * B.S, Art.H * B.S
  local gx, gy = 4, 40
  if #list == 0 then
    c:para(gx, gy + 20, 560, st.tab == 5 and "No foils yet. Champions drop them: a Gilded, Nightborne, Rift-touched or Juggernaut card of their family."
      or "Nothing here yet.", { colour = S.MUTED })
  end
  for i = 1, per do
    local id = list[(st.page - 1) * per + i]
    if id then
      local col, row = (i - 1) % B.COLS, (i - 1) // B.COLS
      local x, y = gx + col * (cw + 10), gy + row * (ch + 14)
      local n = h[id] or 0
      local hot = c:region(x, y, cw, ch, { on = function() st.pick = id B.render() end, key = "card" .. id,
        tip = n > 0 and (("You have %d."):format(n)) or "Not found yet.", title = n > 0 and cards.get(id).name or ("No. " .. (id % 1000)) })
      Art.card(c, x, y, B.S, id, { count = n, missing = n == 0, selected = st.pick == id or hot })
    end
  end
  local py = gy + B.ROWS * (ch + 14)
  if pages > 1 then
    c:button(gx, py, 70, 24, "Back", { on = st.page > 1 and function() st.page = st.page - 1 B.render() end or nil, enabled = st.page > 1, key = "prev" })
    c:text(gx + 120, py + 4, ("Page %d of %d"):format(st.page, pages), { size = 0.85, colour = S.MUTED, align = "center" })
    c:button(gx + 170, py, 70, 24, "Next", { on = st.page < pages and function() st.page = st.page + 1 B.render() end or nil, enabled = st.page < pages, key = "next" })
  end
  -- The big card
  local ix = 600
  local pick = cards.get(st.pick) and st.pick or 1
  local n = h[pick] or 0
  local bs = 1.75
  Art.card(c, ix + 26, gy, bs, pick, { portrait = n > 0, missing = n == 0, detail = "big" })
  local iy = gy + Art.H * bs + 10
  if n > 0 then
    c:text(ix + 26, iy, ("You have %d  ·  %d in your deck"):format(n, in_deck(pick)), { size = 0.85, colour = S.TEXT })
    local card = cards.get(pick)
    local can_add = in_deck(pick) < n
    c:button(ix + 26, iy + 22, 110, 26, "Add to deck", { style = "primary", on = can_add and function() B.add(pick) end or nil,
      enabled = can_add, key = "add", tip = can_add and "Puts it in the deck you're building." or "Every copy you have is in the deck." })
    c:button(ix + 144, iy + 22, 110, 26, "Take out", { on = in_deck(pick) > 0 and function() B.remove(pick) end or nil,
      enabled = in_deck(pick) > 0, key = "out" })
    if card.rarity == 3 and card.fam == "Townsfolk" then
      c:text(ix + 26, iy + 54, "Won at the Steaming Sheep's table.", { size = 0.8, colour = S.MUTED, w = 270 })
    end
  else
    local card = cards.get(pick)
    local how = card.nm and "A notorious monster's: hunt it." or (card.rarity == 3 and "Beat its owner three times at the Steaming Sheep."
      or "Its monsters drop it now and then.")
    c:para(ix + 26, iy, 260, how, { size = 0.85, colour = S.MUTED })
  end
  -- The deck
  local dy = 470
  local deck = B.deck()
  local why = R.deck_problem(deck, h)
  c:heading(0, dy, B.W, ("Your deck (%d of %d)"):format(#deck, R.DECK_SIZE), st.edit and "Changed: save it to play with it" or "The deck you play at the table")
  -- Fifteen strips in three columns, so the names can be read
  local sw = (B.W - 4 - 2 * 12) / 3
  for i = 1, R.DECK_SIZE do
    local col, rowi = (i - 1) // 5, (i - 1) % 5
    local x, y = 2 + col * (sw + 12), dy + 30 + rowi * (Art.STRIP + 4)
    local id = deck[i]
    if id then
      local hot = c:region(x, y, sw, Art.STRIP, { on = function() st.pick = id B.render() end, key = "deck" .. i,
        title = cards.get(id).name, tip = "Click to look at it; take it out from the big card." })
      Art.strip(c, x, y, sw, id, { selected = hot or st.pick == id })
    else
      c:rect(x, y, sw, Art.STRIP, S.SLOT, { round = 5 })
      c:rect(x, y, sw, Art.STRIP, S.EDGE, { round = 5, line = 1 })
    end
  end
  local by = dy + 30 + 5 * (Art.STRIP + 4) + 8
  c:button(0, by, 110, 26, "Save deck", { style = "primary", on = (st.edit and not why) and B.save or nil, enabled = st.edit ~= nil and not why, key = "save" })
  local g = m and m.game
  if g and g.now and not g.now.over and not (m.result) then
    c:button(B.W - 270, by, 140, 26, "Back to the game", { style = "primary", on = function() B.to_table() end, key = "totable",
      tip = "Your game at the Steaming Sheep is still on." })
  end
  c:button(B.W - 120, by, 120, 26, "How to play", { on = function() B.howto() end, key = "howto",
    tip = "The rules: fronts, crystals, the world's bonuses, and collecting cards." })
  c:button(118, by, 110, 26, "Undo changes", { on = st.edit and function() st.edit, st.note = nil, nil B.render() end or nil, enabled = st.edit ~= nil, key = "undo" })
  local note = st.note or (st.edit and why) or (m and m.message and m.message.text)
  if note then c:text(240, by + 5, note, { size = 0.85, colour = (why and st.edit) and S.WARN or S.MUTED, w = B.W - 240 - 280 }) end
  -- The regulars
  local wy = by + 38
  local parts = {}
  for _, r in ipairs(REGULARS) do
    local wins = m and m.book.wins[r[1]] or 0
    parts[#parts + 1] = ("%s %d/3"):format(r[2], math.min(wins, 3))
  end
  c:text(0, wy, "At the Steaming Sheep (after dark):  " .. table.concat(parts, "  ·  ") .. "  ·  three wins for their own card",
    { size = 0.8, colour = S.MUTED, w = B.W })
end

function B.setup(model)
  B.model = model
  ui.window(B.WIN, {
    title = "Cardbook",
    width = B.W + 20,
    help = "Your Ironhand cards: the ones you have, the ones still to find, Champions' foils, and the deck you play at the Steaming Sheep's table.",
    elements = { { type = "canvas", id = "book", w = B.W, h = 740, draw = {} } },
  })
  Canvas.add(B.WIN, "book", B.W, B.draw)
  ui.on(B.WIN, function(win, e)
    if e.kind == "close" then return end
    Canvas.event(win, e)
  end)
end

-- The server's cardbook came in: an edit that's now saved is done
function B.on_book()
  local st = B.state
  if st.edit and B.model then
    local same = #st.edit == #B.model.book.deck
    for i, id in ipairs(st.edit) do if B.model.book.deck[i] ~= id then same = false end end
    if same then st.edit = nil st.note = "Deck saved." end
  end
  if st.note == "Saving..." then st.note = nil end
  B.render()
end

function B.render()
  if ui.is_open(B.WIN) then Canvas.render(B.WIN, "book") end
end

function B.toggle()
  if ui.is_open(B.WIN) then
    ui.close(B.WIN)
  else
    ui.open(B.WIN)
    B.send("ih book")
    B.render()
  end
end

return B
