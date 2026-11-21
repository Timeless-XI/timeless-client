-- The table: a game of Ironhand at the Steaming Sheep. Three fronts, their cards above and yours below with each
-- card's power now (the bonus in green, or red); your hand along the bottom; a big card on the right with the
-- monster's model turning slowly. Pick a card, then a front to put it on (click it again to take it back); Reveal
-- sends your plays and the round ends. The world panel says what the day, the hour and the moon are doing.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local cards = require("lib.cards")
local Art = require("lib.art")
local R = require("lib.rules")

local T = {}

T.WIN = "ironhand"
T.W = 1040
T.FW = 236 -- a front's width
T.ROWS = 6  -- cards a side shows on a front (strips: Art.strip)
T.HS = 0.84 -- the hand's cards
T.state = { pick = nil, staged = {}, waiting = false, look = nil }
T.send = function() end
T.play = function() end
T.howto = function() end

local DAY = { fire = "Firesday", earth = "Earthsday", water = "Watersday", wind = "Windsday", ice = "Iceday",
  thunder = "Lightningday", light = "Lightsday", dark = "Darksday" }

local function now() return T.model and T.model.game and T.model.game.now end

function T.reset()
  T.state.pick, T.state.staged, T.state.waiting, T.state.look = nil, {}, false, nil
end

function T.stage(f)
  local st, n = T.state, now()
  if not st.pick or st.waiting then return end
  local ok, why = R.can_stage(n, st.staged, st.pick, f)
  if not ok then st.note = "Can't: " .. why T.render() return end
  st.staged[#st.staged + 1] = { h = st.pick, f = f }
  st.pick, st.note = nil, nil
  T.render()
end

function T.unstage(i)
  table.remove(T.state.staged, i)
  T.render()
end

function T.reveal()
  local st = T.state
  if st.waiting or not now() or now().over then return end
  local parts = {}
  for _, p in ipairs(st.staged) do parts[#parts + 1] = p.h .. ":" .. p.f end
  T.send("ih reveal " .. table.concat(parts, ","))
  st.staged, st.pick, st.waiting = {}, nil, true
  T.render()
end

local function staged_on(f)
  local out = {}
  for i, p in ipairs(T.state.staged) do
    if p.f == f then out[#out + 1] = { i = i, p = p } end
  end
  return out
end

local function is_staged(h)
  for _, p in ipairs(T.state.staged) do if p.h == h then return true end end
  return false
end

function T.draw(c)
  local m = T.model
  local g = m and m.game
  local n = g and g.now
  local st = T.state
  if not g or not n or not n.round then
    c:para(0, 10, T.W, "No game on. Visit the Steaming Sheep in Port Bastok after dark and talk to one of the card players at the game table.", { colour = S.MUTED })
    return
  end
  -- Across the top: them, the round, the stakes
  c:text(0, 0, g.opponent, { size = 1.25, bold = true, colour = S.GOLD })
  local x = 12 + Canvas.measure(g.opponent, 1.25, true)
  for i = 1, math.min(n.their_hand or 0, 8) do Art.back(c, x + (i - 1) * 14, 2, 0.14) end
  c:text(x + math.min(n.their_hand or 0, 8) * 14 + 12, 5, ("%d in hand"):format(n.their_hand or 0), { size = 0.8, colour = S.MUTED })
  c:text(T.W, 0, ("Round %d of 6"):format(n.round), { size = 1.05, bold = true, colour = S.TEXT, align = "right" })
  c:text(T.W, 20, g.stakes == "ante" and "For a card: the loser hands over a common" or "A friendly game", { size = 0.75, colour = S.MUTED, align = "right" })
  -- The world
  local w = g.world or {}
  local parts = {}
  if w.day then parts[#parts + 1] = ("%s: %s +1"):format(DAY[w.day] or w.day, Art.ELEMENT_NAME[w.day] or w.day) end
  parts[#parts + 1] = w.night and "Night: night cards are awake" or "Day: night cards sleep"
  if w.moon then parts[#parts + 1] = w.moon end
  parts[#parts + 1] = "Element edge +1"
  c:text(0, 26, table.concat(parts, "  ·  "), { size = 0.78, colour = { 200, 214, 236, 255 }, w = 700 })
  -- The fronts
  local fy = 50
  local sw, sh = T.FW - 16, Art.STRIP + 4
  local side = T.ROWS * sh
  local fh = 40 + side + 6 + 34 + side + 8
  for f = 1, 3 do
    local fr = n.fronts[f] or {}
    local fx = (f - 1) * (T.FW + 10)
    local can = st.pick and R.can_stage(n, st.staged, st.pick, f)
    local hot = c:region(fx, fy, T.FW, fh, { on = can and function() T.stage(f) end or nil, enabled = can and true or false, key = "front" .. f })
    c:rect(fx, fy, T.FW, fh, can and { 120, 214, 140, hot and 40 or 18 } or { 0, 0, 0, 70 }, { round = 10 })
    c:rect(fx, fy, T.FW, fh, can and { 120, 214, 140, 200 } or S.EDGE, { round = 10, line = 1 })
    c:text(fx + T.FW / 2, fy + 6, fr.name or "", { size = 0.9, bold = true, colour = S.TEXT, align = "center", w = T.FW - 10 })
    c:text(fx + T.FW / 2, fy + 23, Canvas.fit(fr.twist or "", T.FW - 10, 0.72), { size = 0.72, colour = S.MUTED, align = "center" })
    -- One side's cards as strips; theirs build down from the top, yours up from the score
    local function row(list, sideNo, y0, extra)
      local all = {}
      for _, u in ipairs(list or {}) do all[#all + 1] = { u = u } end
      for _, sp in ipairs(extra or {}) do all[#all + 1] = { sp = sp } end
      for i, e in ipairs(all) do
        if i > T.ROWS then
          c:text(fx + T.FW - 10, y0 + side - 12, ("+%d more"):format(#all - T.ROWS), { size = 0.66, colour = S.MUTED, align = "right" })
          break
        end
        local cx, cy = fx + 8, y0 + (i - 1) * sh
        if e.u then
          local u = e.u
          local hk = c:region(cx, cy, sw, Art.STRIP, { on = function() st.look = u.id T.render() end, key = ("f%ds%di%d"):format(f, sideNo, i - 1),
            title = cards.get(u.id) and cards.get(u.id).name, tip = "Click to see it in the big card." })
          Art.strip(c, cx, cy, sw, u.id, { power = u.power, bonus = u.bonus, silenced = u.silenced, selected = hk or st.look == u.id })
        else
          local sp = e.sp
          c:region(cx, cy, sw, Art.STRIP, { on = function() T.unstage(sp.i) end, key = "staged" .. sp.i, title = "To be played", tip = "Click to take it back." })
          Art.strip(c, cx, cy, sw, n.hand[sp.p.h], { staged = true })
        end
      end
    end
    row(fr.cards and fr.cards[2], 2, fy + 40)
    local ty = fy + 40 + side + 6
    local mine, theirs = fr.mine or 0, fr.theirs or 0
    c:rect(fx + 8, ty, T.FW - 16, 28, { 0, 0, 0, 90 }, { round = 6 })
    c:text(fx + 20, ty + 5, tostring(theirs), { size = 1.1, bold = true, colour = theirs > mine and S.GOLD or S.TEXT })
    c:text(fx + T.FW / 2, ty + 7, theirs > mine and "they lead" or (mine > theirs and "you lead" or "even"), { size = 0.78, colour = S.MUTED, align = "center" })
    c:text(fx + T.FW - 20, ty + 5, tostring(mine), { size = 1.1, bold = true, colour = mine > theirs and S.GOLD or S.TEXT, align = "right" })
    row(fr.cards and fr.cards[1], 1, ty + 34, staged_on(f))
  end
  -- The hand
  local hy = fy + fh + 30
  local spent = R.spent(n, st.staged)
  c:text(0, hy - 18, ("Your hand  ·  crystals %d of %d  ·  %d cards left in your deck"):format(n.crystals - spent, n.crystals, n.deck_left or 0),
    { size = 0.8, colour = S.MUTED, w = 720 })
  local hw = Art.W * T.HS
  for h, id in ipairs(n.hand) do
    local hx = (h - 1) * (hw + 8)
    if hx + hw <= 728 then
      local card = cards.get(id)
      local affordable = card and card.cost <= n.crystals - spent
      local placed = is_staged(h)
      c:region(hx, hy, hw, Art.H * T.HS, { on = (not placed and not st.waiting) and function() st.pick = (st.pick == h) and nil or h st.look = nil T.render() end or nil,
        key = "hand" .. h, enabled = not placed })
      Art.card(c, hx, hy + (st.pick == h and -6 or 0), T.HS, id, { selected = st.pick == h, staged = placed, detail = "small" })
      if not affordable and not placed then c:rect(hx, hy, hw, Art.H * T.HS, { 0, 0, 0, 110 }, { round = 5 }) end
    end
  end
  -- The big card, the buttons and the result
  local rx = 750
  local look = st.look or (st.pick and n.hand[st.pick]) or n.hand[1]
  if look then Art.card(c, rx + 22, fy, 1.6, look, { portrait = true, detail = "big" }) end
  local by = fy + Art.H * 1.6 + 14
  if m.result then
    local r = m.result
    c:text(rx + 145, by, r.winner == 1 and "You win!" or (r.winner == 2 and (g.opponent .. " wins") or "A draw"),
      { size = 1.3, bold = true, colour = r.winner == 1 and S.GOOD or (r.winner == 2 and S.BAD or S.TEXT), align = "center" })
    c:para(rx, by + 28, 290, r.text, { size = 0.82, colour = S.TEXT })
    c:button(rx + 90, by + 110, 110, 28, "Close", { on = function() ui.close(T.WIN) end, key = "close" })
    c:button(rx + 90, by + 146, 110, 26, "How to play", { on = function() T.howto() end, key = "howto" })
  else
    local pvp = g.key == "pvp"
    c:button(rx, by, 140, 34, st.waiting and (pvp and "Waiting..." or "Revealing...") or "Reveal", { style = "primary", on = (not st.waiting) and T.reveal or nil,
      enabled = not st.waiting, key = "reveal", tip = "Turn your cards over. Theirs turn over too, and the round ends." })
    c:button(rx + 150, by, 140, 34, "Fold", { style = "danger", on = function() T.send("ih concede") end, key = "fold",
      tip = g.stakes == "ante" and "Give up the game, and the card you put up." or "Give up the game." })
    local hint = st.note or (st.pick and "Now pick a front for it." or (#st.staged > 0 and "Reveal when you're ready." or "Pick a card from your hand."))
    if pvp and st.waiting then
      hint = ("Waiting for %s to reveal..."):format(g.opponent)
    elseif pvp and g.ready and not st.note then
      hint = ("%s is ready. %s"):format(g.opponent, hint)
    end
    local hy2 = c:para(rx, by + 44, 290, hint, { size = 0.8, colour = S.MUTED })
    c:button(rx, math.max(hy2 + 8, by + 84), 140, 26, "How to play", { on = function() T.howto() end, key = "howto",
      tip = "The rules: fronts, crystals, the world's bonuses, and collecting cards." })
  end
end

function T.setup(model)
  T.model = model
  ui.window(T.WIN, {
    title = "Ironhand",
    width = T.W + 20,
    help = "A game of Ironhand: take two of the three fronts. Pick a card, then a front; Reveal to play the round.",
    elements = { { type = "canvas", id = "table", w = T.W, h = 50 + (40 + 2 * T.ROWS * (Art.STRIP + 4) + 48) + 30 + Art.H * T.HS + 12, draw = {} } },
  })
  Canvas.add(T.WIN, "table", T.W, T.draw)
  ui.on(T.WIN, function(win, e)
    if e.kind == "close" then return end -- the game waits: /ironhand opens it again
    Canvas.event(win, e)
  end)
end

function T.render()
  if ui.is_open(T.WIN) then Canvas.render(T.WIN, "table") end
end

return T
