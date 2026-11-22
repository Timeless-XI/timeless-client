-- The table: a game of Cardian. Their hand down the left (shown with Open), the 3x3 board in the middle with a
-- front at the head of each column, your hand down the right, and a big card with the monster's model turning
-- slowly. On your turn, pick a card from your hand, then an empty square; cards it beats flip blue. The cards that
-- just flipped flash. Own two squares of a column to take its front; two fronts win.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local cards = require("lib.cards")
local Art = require("lib.art")

local T = {}

T.WIN = "cardian"
T.W = 1000
T.CS = 0.8  -- the board's cards
T.HS = 0.6  -- the hands'
T.state = { pick = nil, waiting = false, look = nil, note = nil }
T.send = function() end
T.howto = function() end

local DAY = { fire = "Firesday", earth = "Earthsday", water = "Watersday", wind = "Windsday", ice = "Iceday",
  thunder = "Lightningday", light = "Lightsday", dark = "Darksday" }
local RULE_NAMES = { open = "Open", same = "Same", plus = "Plus", elemental = "Elemental" }

-- Where things go
local BX, BY = 130, 104                         -- the board's corner
local CW, CH = Art.W * 0.8 + 8, Art.H * 0.8 + 8 -- a square
local HX = BX + 3 * CW + 22                     -- your hand's column
local RX = HX + Art.W * 0.6 + 28                -- the right-hand panel

local function now() return T.model and T.model.game and T.model.game.now end

function T.reset()
  T.state.pick, T.state.waiting, T.state.look, T.state.note = nil, false, nil, nil
end

function T.place(cell)
  local st, n = T.state, now()
  if not n or n.over or not n.mine or st.waiting or not st.pick or n.board[cell] then return end
  T.send(("cd place %d %d"):format(st.pick, cell))
  st.pick, st.waiting, st.note = nil, true, nil
  T.render()
end

local function square_rect(cell)
  local col, row = (cell - 1) % 3, (cell - 1) // 3
  return BX + col * CW, BY + row * CH
end

function T.draw(c)
  local m = T.model
  local g = m and m.game
  local n = g and g.now
  local st = T.state
  if not g or not n or not n.placed then
    c:para(0, 10, T.W, "No game on. Visit the Steaming Sheep in Port Bastok after dark, or find Shaali the card shark on her travels, and sit down to play.", { colour = S.MUTED })
    return
  end
  -- Across the top: them, the stakes, the house rules, the world
  c:text(0, 0, g.opponent, { size = 1.25, bold = true, colour = S.GOLD })
  local rules = {}
  for _, k in ipairs({ "open", "same", "plus", "elemental" }) do if g.rules[k] then rules[#rules + 1] = RULE_NAMES[k] end end
  if g.rules.same or g.rules.plus then rules[#rules + 1] = "Combo" end
  c:text(T.W, 0, "House rules: " .. (#rules > 0 and table.concat(rules, " · ") or "none"), { size = 0.9, bold = true, colour = S.TEXT, align = "right" })
  c:text(T.W, 20, g.stakes == "ante" and "For a card: the winner takes the loser's best" or "A friendly game", { size = 0.75, colour = S.MUTED, align = "right" })
  local w = g.world or {}
  local parts = {}
  if w.day then parts[#parts + 1] = ("%s: %s cards +1"):format(DAY[w.day] or w.day, Art.ELEMENT_NAME[w.day] or w.day) end
  if w.moon then parts[#parts + 1] = w.moon end
  if w.night then parts[#parts + 1] = "Night: Nightborne +1" end
  parts[#parts + 1] = "A marked square: its element +1, others -1"
  c:text(0, 26, table.concat(parts, "  ·  "), { size = 0.78, colour = { 200, 214, 236, 255 }, w = RX - 20 })

  -- Their hand, down the left
  local hs = T.HS
  local step = 70
  c:text(0, 52, ("Theirs (%d)"):format(#n.theirs), { size = 0.8, colour = S.MUTED })
  for i, id in ipairs(n.theirs) do
    local y = 72 + (i - 1) * step
    if id and id > 0 then
      local hot = c:region(0, y, Art.W * hs, step, { on = function() st.look = id T.render() end, key = "their" .. i, title = cards.get(id) and cards.get(id).name })
      Art.card(c, 0, y, hs, id, { owner = 2, selected = hot, detail = "small" })
    else
      Art.back(c, 0, y, hs)
    end
  end

  -- The fronts at the head of the columns, and who holds each
  for f = 1, 3 do
    local fr = g.fronts[f] or {}
    local fx = BX + (f - 1) * CW
    c:rect(fx + 2, 52, CW - 4, 46, { 0, 0, 0, 80 }, { round = 6 })
    c:text(fx + CW / 2, 55, Canvas.fit(fr.name or "", CW - 8, 0.82, true), { size = 0.82, bold = true, colour = S.TEXT, align = "center" })
    c:text(fx + CW / 2, 73, Canvas.fit(fr.twist or "", CW - 8, 0.66), { size = 0.66, colour = S.MUTED, align = "center" })
    local mine, theirs = 0, 0
    for r = 0, 2 do
      local o = n.board[r * 3 + f]
      if o then if o.owner == 1 then mine = mine + 1 else theirs = theirs + 1 end end
    end
    local held = mine >= 2 and "Yours" or (theirs >= 2 and "Theirs" or "Open")
    local sy = BY + 3 * CH + 4
    c:rect(fx + 2, sy, CW - 4, 26, mine >= 2 and { 52, 92, 170, 160 } or (theirs >= 2 and { 160, 52, 60, 160 } or { 0, 0, 0, 80 }), { round = 6 })
    c:text(fx + CW / 2, sy + 5, ("%s  ·  %d-%d"):format(held, mine, theirs), { size = 0.82, bold = true, colour = S.TEXT, align = "center" })
  end

  -- The board
  local flipped = {}
  if n.last then for _, fl in ipairs(n.last.flips) do flipped[fl.cell] = fl.how end end
  local canPlace = n.mine and not n.over and not st.waiting and st.pick ~= nil
  for cell = 1, 9 do
    local x, y = square_rect(cell)
    local o = n.board[cell]
    local el = g.squares[cell]
    if o then
      local hot = c:region(x + 4, y + 4, CW - 8, CH - 8, { on = function() st.look = o.id T.render() end, key = "sq" .. cell,
        title = cards.get(o.id) and cards.get(o.id).name, tip = flipped[cell] and ("Just flipped (" .. flipped[cell] .. ").") or nil })
      Art.card(c, x + 4, y + 4, T.CS, o.id, { owner = o.owner, sides = o.sides, flipped = flipped[cell] ~= nil, selected = hot, detail = "mini" })
      if n.last and n.last.cell == cell then
        c:rect(x + 2, y + 2, CW - 4, CH - 4, { 255, 240, 190, 200 }, { round = 9, line = 2 })
      end
      if el then Art.crystal(c, x + CW - 14, y + CH - 18, 16, Art.ELEMENT[el]) end
    else
      local hot = c:region(x + 4, y + 4, CW - 8, CH - 8, { on = canPlace and function() T.place(cell) end or nil, enabled = canPlace, key = "sq" .. cell,
        tip = el and ("Marked: %s cards +1 here, others -1."):format(Art.ELEMENT_NAME[el] or el) or nil })
      c:rect(x + 4, y + 4, CW - 8, CH - 8, canPlace and { 140, 190, 255, hot and 60 or 22 } or { 0, 0, 0, 70 }, { round = 8 })
      c:rect(x + 4, y + 4, CW - 8, CH - 8, canPlace and { 140, 190, 255, 160 } or S.EDGE, { round = 8, line = 1 })
      if el then
        Art.crystal(c, x + CW / 2, y + CH / 2 - 8, 40, Art.ELEMENT[el], 0.8)
        c:text(x + CW / 2, y + CH / 2 + 18, Art.ELEMENT_NAME[el] or el, { size = 0.72, colour = S.MUTED, align = "center" })
      end
    end
  end

  -- Your hand, down the right of the board
  c:text(HX, 52, ("Yours (%d)"):format(#n.hand), { size = 0.8, colour = S.MUTED })
  for i, hc in ipairs(n.hand) do
    local y = 72 + (i - 1) * step
    local hot = c:region(HX, y, Art.W * hs, i == #n.hand and Art.H * hs or step, { on = function()
      st.pick = (st.pick == i) and nil or i
      st.look = hc.id
      T.render()
    end, key = "hand" .. i, title = cards.get(hc.id) and cards.get(hc.id).name })
    Art.card(c, HX + (st.pick == i and -10 or 0), y, hs, hc.id, { owner = 1, sides = hc.sides, selected = st.pick == i or hot, detail = "small" })
  end

  -- The big card, then whose turn it is, or how it ended
  local look = st.look or (st.pick and n.hand[st.pick] and n.hand[st.pick].id) or (n.hand[1] and n.hand[1].id)
  local bs = 1.7
  if look then Art.card(c, RX, 52, bs, look, { portrait = true, detail = "big" }) end
  local by = 52 + Art.H * bs + 14
  if m.result then
    local r = m.result
    c:text(RX + Art.W * bs / 2, by, r.winner == 1 and "You win!" or (r.winner == 2 and (g.opponent .. " wins") or "A draw"),
      { size = 1.3, bold = true, colour = r.winner == 1 and S.GOOD or (r.winner == 2 and S.BAD or S.TEXT), align = "center" })
    local ey = c:para(RX, by + 28, T.W - RX, r.text, { size = 0.82, colour = S.TEXT })
    c:button(RX, ey + 10, 110, 28, "Close", { on = function() ui.close(T.WIN) end, key = "close" })
    c:button(RX + 120, ey + 10, 120, 28, "How to play", { on = function() T.howto() end, key = "howto" })
  else
    local pvp = g.key == "pvp"
    local turn
    if n.mine and not st.waiting then
      turn = st.note or (st.pick and "Now pick an empty square." or "Your turn: pick a card from your hand.")
    elseif pvp then
      turn = ("Waiting for %s..."):format(g.opponent)
    else
      turn = ("%s is thinking..."):format(g.opponent)
    end
    c:text(RX, by, n.mine and not st.waiting and "Your turn" or "Their turn", { size = 1.1, bold = true, colour = n.mine and not st.waiting and S.GOOD or S.TEXT })
    local ty = c:para(RX, by + 22, T.W - RX, turn, { size = 0.82, colour = S.MUTED })
    c:button(RX, ty + 10, 110, 28, "Fold", { style = "danger", on = function() T.send("cd concede") end, key = "fold",
      tip = g.stakes == "ante" and "Give up the game, and the card you put up." or "Give up the game." })
    c:button(RX + 120, ty + 10, 120, 28, "How to play", { on = function() T.howto() end, key = "howto",
      tip = "The rules: capturing, Same, Plus and Combo, the fronts, the world's numbers." })
  end
end

function T.setup(model)
  T.model = model
  ui.window(T.WIN, {
    title = "Cardian",
    width = T.W + 20,
    help = "A game of Cardian: on your turn, pick a card from your hand, then an empty square. A higher touching number flips an enemy card. Two squares of a column take its front; two fronts win.",
    elements = { { type = "canvas", id = "table", w = T.W, h = BY + 3 * CH + 40, draw = {} } },
  })
  Canvas.add(T.WIN, "table", T.W, T.draw)
  ui.on(T.WIN, function(win, e)
    if e.kind == "close" then return end -- the game waits: the cardbook's Back to the game opens it again
    Canvas.event(win, e)
  end)
end

function T.render()
  if ui.is_open(T.WIN) then Canvas.render(T.WIN, "table") end
end

return T
