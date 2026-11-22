-- The cardbook (/cardbook, the Timeless menu): every card you have, the ones still to find as silhouettes, the
-- foils from Champions, a big card with the monster's model turning slowly, your hand of five to pick and save,
-- where Shaali the card shark is today, and challenging another player.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local cards = require("lib.cards")
local Art = require("lib.art")
local R = require("lib.rules")

local B = {}

B.WIN = "cardbook"
B.W = 900
B.COLS, B.ROWS, B.S = 5, 2, 0.82
B.state = { tab = 1, page = 1, pick = 1, edit = nil, note = nil, confirm = nil }
B.send = function() end -- "cd ..." to the server, set by init.lua
B.howto = function() end -- opens How to play, set by init.lua
B.to_table = function() end -- opens the table, set by init.lua
B.RANGE = 15 -- yalms: how close another player must be to challenge (the server's config.pvp.range)

local TABS = {
  { label = "All", tip = "Every card, the ones you have and the ones still to find." },
  { label = "Commons", tip = "Field monsters' cards." },
  { label = "Rares", tip = "Notorious monsters' cards." },
  { label = "Legendaries", tip = "A great wyrm's, and the card players' own (beat one three times)." },
  { label = "Foils", tip = "Champions' cards: Gilded, Nightborne, Rift-touched and Juggernaut. The ones you have." },
  { label = "Sets", tip = "A region's (or kind's) cards: own one of each and its crest card is yours." },
}

local EL = { fire = "fire", ice = "ice", wind = "wind", earth = "earth", thunder = "thunder", water = "water", light = "light", dark = "dark" }

-- Where a card comes from, in a few words (the world's own: when they drop)
function B.where(card)
  if card.crest then
    for _, set in ipairs(cards.SETS) do
      if set.crest == card.id then return ("Own every card of %s: its crest is yours."):format(set.name) end
    end
  end
  if card.nm then return "A notorious monster's: hunt it." end
  if card.key == "shaali" then return "Beat Shaali, the travelling card shark, three times." end
  if card.rarity == 3 then return card.fam == "Townsfolk" and "Beat its owner three times at the Steaming Sheep." or "A legend's: hunt it." end
  local w = card.when
  if w then
    if w.night then return "Its monsters drop it only at night (20:00 to 6:00)." end
    if w.weather then return ("Its monsters drop it only in %s weather."):format(EL[w.weather] or w.weather) end
    if w.beastHeld then return "Its monsters drop it only while the beastmen hold their region (conquest)." end
    if w.region then
      local r = w.region:lower():gsub("_", " "):gsub("^%l", string.upper)
      return ("Its monsters drop it only in %s."):format(r)
    end
  end
  return "Its monsters drop it now and then, or bind one with a Card Binder."
end

-- What Card Mod breaks a card into (the server's ca.modYield), in words; nil when it won't
local CRYSTAL = { fire = "Fire", ice = "Ice", wind = "Wind", earth = "Earth", thunder = "Lightning", water = "Water", light = "Light", dark = "Dark" }
local FOIL_ITEM = { "a gold beastcoin", "a moonstone", "a petrified log", "a darksteel ingot" }
function B.mod_yield(id)
  local c = cards.get(id)
  if not c or c.rarity == 3 or c.crest then return nil end
  local el = CRYSTAL[c.el] or "Fire"
  local out = c.rarity == 2 and ("a %s cluster"):format(el)
    or ("%d %s crystals"):format(c.level <= 2 and 2 or (c.level <= 4 and 4 or 6), el)
  if c.variant then out = out .. " and " .. FOIL_ITEM[c.variant] end
  return out
end

function B.set_progress(set)
  local h = B.model and B.model.book.have or {}
  local n = 0
  for _, id in ipairs(set.cards) do if (h[id] or 0) > 0 then n = n + 1 end end
  return n, #set.cards
end

local REGULARS = { { "hrodgar", "Hrodgar" }, { "fenwick", "Fenwick" }, { "kipiko", "Kipiko" }, { "dagny", "Dagny" }, { "shaali", "Shaali" } }

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
  if t == 6 then return out end
  for _, c in ipairs(cards.list) do
    if (t == 1 and not c.crest) or (t > 1 and c.rarity == t - 1 and not c.crest) then out[#out + 1] = c.id end
  end
  return out
end

function B.hand()
  return B.state.edit or (B.model and B.model.book.hand) or {}
end

local function in_hand(id)
  local n = 0
  for _, d in ipairs(B.hand()) do if d == id then n = n + 1 end end
  return n
end

local function edit()
  if not B.state.edit then
    B.state.edit = {}
    for i, id in ipairs(B.model and B.model.book.hand or {}) do B.state.edit[i] = id end
  end
  return B.state.edit
end

function B.add(id)
  local e = edit()
  if #e >= R.HAND_SIZE then B.state.note = "Your hand has five already: take one out first." B.render() return end
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
  local why = R.hand_problem(e, have())
  if why then B.state.note = why B.render() return end
  local ids = {}
  for i, id in ipairs(e) do ids[i] = tostring(id) end
  B.send("cd hand " .. table.concat(ids, ","))
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
  c:switch(B.W - 540, 0, 26, TABS, st.tab, { on = function(i) st.tab, st.page = i, 1 B.render() end, key = "tab" })
  local list = B.list()
  local per = B.COLS * B.ROWS
  local pages = math.max(1, math.ceil(#list / per))
  st.page = math.min(st.page, pages)
  local cw, ch = Art.W * B.S, Art.H * B.S
  local gx, gy = 4, 40
  if st.tab == 6 then
    B.draw_sets(c, gx, gy)
  elseif #list == 0 then
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
    c:text(ix + 26, iy, ("You have %d  ·  %d in your hand"):format(n, in_hand(pick)), { size = 0.85, colour = S.TEXT })
    local card = cards.get(pick)
    local can_add = in_hand(pick) < n
    c:button(ix + 26, iy + 22, 110, 26, "Add to hand", { style = "primary", on = can_add and function() B.add(pick) end or nil,
      enabled = can_add, key = "add", tip = can_add and "Puts it in the hand you're picking." or "Every copy you have is in your hand." })
    c:button(ix + 144, iy + 22, 110, 26, "Take out", { on = in_hand(pick) > 0 and function() B.remove(pick) end or nil,
      enabled = in_hand(pick) > 0, key = "out" })
    local yield = B.mod_yield(pick)
    if yield then
      local here = m and m.book.mod_here
      local sure = st.confirm == pick
      c:button(ix + 26, iy + 54, 228, 26, sure and "Break it? Click again" or "Card Mod", { style = sure and "danger" or nil, key = "mod", enabled = here,
        on = here and function()
          if st.confirm == pick then
            st.confirm = nil
            B.send("cd mod " .. pick)
          else
            st.confirm = pick
            B.render()
          end
        end or nil,
        tip = here and ("Breaks one copy into %s, for crafting."):format(yield) or ("In a town, this breaks one copy into %s, for crafting."):format(yield) })
    elseif card.rarity == 3 and card.fam == "Townsfolk" then
      c:text(ix + 26, iy + 54, "Won from its owner at the table.", { size = 0.8, colour = S.MUTED, w = 270 })
    elseif card.crest then
      c:text(ix + 26, iy + 54, "A set's crest: yours for good.", { size = 0.8, colour = S.MUTED, w = 270 })
    end
  else
    c:para(ix + 26, iy, 260, B.where(cards.get(pick)), { size = 0.85, colour = S.MUTED })
  end
  -- The hand of five you bring to the table
  local dy = 470
  local hand = B.hand()
  local why = R.hand_problem(hand, h)
  c:heading(0, dy, B.W, ("Your hand (%d of %d)"):format(#hand, R.HAND_SIZE), st.edit and "Changed: save it to play with it" or "The five cards you bring to the table")
  local hs = 0.62
  for i = 1, R.HAND_SIZE do
    local x, y = 2 + (i - 1) * (Art.W * hs + 12), dy + 30
    local id = hand[i]
    if id then
      local hot = c:region(x, y, Art.W * hs, Art.H * hs, { on = function() st.pick = id B.render() end, key = "hand" .. i,
        title = cards.get(id).name, tip = "Click to look at it; take it out from the big card." })
      Art.card(c, x, y, hs, id, { selected = hot or st.pick == id, detail = "small" })
    else
      c:rect(x, y, Art.W * hs, Art.H * hs, S.SLOT, { round = 6 })
      c:rect(x, y, Art.W * hs, Art.H * hs, S.EDGE, { round = 6, line = 1 })
    end
  end
  -- Where the card shark is, beside the hand
  local sx = 2 + R.HAND_SIZE * (Art.W * hs + 12) + 10
  if m and m.shark then
    c:text(sx, dy + 34, "On the road", { size = 0.95, bold = true, colour = S.GOLD })
    local ey = c:para(sx, dy + 54, B.W - sx, ("Shaali Nine-Lives, the travelling card shark, has her table in %s today, and goes to %s tomorrow. She plays Open, Plus and Elemental, with notorious monsters' cards."):format(m.shark.town, m.shark.next), { size = 0.82, colour = S.MUTED })
    local binder = m.book.binder
    c:para(sx, ey + 6, B.W - sx, binder and "You have a Card Binder: fight a monster down to 20% HP and the Card button by your target turns it into its card (no experience for it)."
      or ("She sells a Card Binder (%d gil): it turns a weakened monster into its card, so you can hunt the card you want."):format(m.book.binder_price or 5000),
      { size = 0.82, colour = binder and S.GOOD or S.MUTED })
  end
  local by = dy + 30 + Art.H * hs + 10
  c:button(0, by, 110, 26, "Save hand", { style = "primary", on = (st.edit and not why) and B.save or nil, enabled = st.edit ~= nil and not why, key = "save" })
  local g = m and m.game
  if g and g.now and not g.now.over and not (m.result) then
    c:button(B.W - 270, by, 140, 26, "Back to the game", { style = "primary", on = function() B.to_table() end, key = "totable",
      tip = "Your game is still on." })
  end
  c:button(B.W - 120, by, 120, 26, "How to play", { on = function() B.howto() end, key = "howto",
    tip = "The rules: capturing, the fronts, house rules, the world's numbers, and collecting cards." })
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
  c:text(0, wy, "Wins (three for their own card):  " .. table.concat(parts, "  ·  "),
    { size = 0.8, colour = S.MUTED, w = B.W })
  -- Another player: target them (close by) and challenge them
  local cy = wy + 24
  local t = game and game.target and game.target()
  local busy = g and g.now and not g.now.over and not m.result
  if t and t.kind == "player" and not t.me then
    local close = (t.distance or 99) <= B.RANGE
    local on = close and not busy
    c:button(0, cy, 200, 26, "Challenge " .. t.name, { style = "primary", key = "challenge", enabled = on,
      on = on and function() B.send(("cd challenge %d"):format(t.id)) end or nil,
      tip = close and "A friendly game: nothing changes hands." or "Stand closer to them to play." })
    c:button(208, cy, 150, 26, "For a card", { key = "challenge_ante", enabled = on,
      on = on and function() B.send(("cd challenge %d ante"):format(t.id)) end or nil,
      tip = "You each put up the best card in your hand you have a spare of; the winner takes the loser's." })
    if not close then c:text(370, cy + 5, "Stand closer to " .. t.name .. " to play.", { size = 0.8, colour = S.MUTED }) end
  else
    c:text(0, cy + 5, "To play another player: target them while you stand close by, then challenge them here.", { size = 0.8, colour = S.MUTED, w = B.W })
  end
end

-- The Sets tab: each set, how much of it you have, and its crest (click a set to see its crest big)
function B.draw_sets(c, x, y)
  local st = B.state
  local h = have()
  local rh = 38
  for i, set in ipairs(cards.SETS) do
    local ry = y + (i - 1) * rh
    local n, total = B.set_progress(set)
    local done = (h[set.crest] or 0) > 0
    local missing = {}
    for _, id in ipairs(set.cards) do
      if (h[id] or 0) == 0 then missing[#missing + 1] = cards.get(id).name end
    end
    local hot = c:region(x, ry, 580, rh - 4, { key = "set" .. i, on = function() st.pick = set.crest B.render() end,
      title = set.name, tip = #missing == 0 and "Every card found." or ("Still to find: " .. table.concat(missing, ", ") .. ".") })
    c:rect(x, ry, 580, rh - 4, (hot or st.pick == set.crest) and S.EDGE_HI or S.EDGE, { round = 5, line = 1 })
    c:text(x + 10, ry + 4, (set.name:gsub("^%l", string.upper)), { size = 0.9, bold = true, colour = done and S.GOLD or S.TEXT })
    c:text(x + 10, ry + 20, done and ("Complete: " .. cards.get(set.crest).name .. " is yours") or ("Crest: " .. cards.get(set.crest).name),
      { size = 0.75, colour = done and S.GOOD or S.MUTED })
    c:bar(x + 330, ry + 12, 170, 8, n / total, done and S.GOOD or S.GOLD)
    c:text(x + 570, ry + 9, ("%d / %d"):format(n, total), { size = 0.85, colour = S.TEXT, align = "right" })
  end
end

function B.setup(model)
  B.model = model
  ui.window(B.WIN, {
    title = "Cardbook",
    width = B.W + 20,
    help = "Your Cardian cards: the ones you have, the ones still to find, Champions' foils, and the hand of five you bring to the table.",
    elements = { { type = "canvas", id = "book", w = B.W, h = 790, draw = {} } },
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
    local same = #st.edit == #B.model.book.hand
    for i, id in ipairs(st.edit) do if B.model.book.hand[i] ~= id then same = false end end
    if same then st.edit = nil st.note = "Hand saved." end
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
    B.send("cd book")
    B.render()
  end
end

return B
