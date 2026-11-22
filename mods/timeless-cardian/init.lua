-- Timeless Cardian: Vana'diel's card game (the server's cardian module).
--
--   /cardbook   your cards (and the ones still to find), Champions' foils and your hand of five; also on the Timeless menu
--   /cardian    the table, while a game is on (talk to a card player at the Steaming Sheep after dark, or to Shaali
--               the travelling card shark, to sit down)
--   How to play: a button on the table and in the cardbook (and it opens by itself with your first game)
--   Card: with a Card Binder (Shaali sells it), a button above the screen's middle while the monster you're fighting
--         is at 20% HP or less: it becomes its card (no experience for it)
--   Play cards: a button by a Card Club member or your job's master when you target them, close by
--
-- Cards wear the game's own window with a header dressed for their family, as the shop windows dress for their
-- shopkeeper; the big card shows the monster's real model from your game files, turning slowly (TimelessXI client).
-- The mod asks the server with a packet (not chat): "cd on" every 20 s, and the feed lines are taken out of the log.

local action = require("lib.action")
local protocol = require("lib.protocol")
local book = require("lib.book")
local tbl = require("lib.table")
local found = require("lib.found")
local howto = require("lib.howto")
local invite = require("lib.invite")

local BEAT = 20
local CARD_BTN = "cardian_bind"
local NPC_BTN = "cardian_npc"
local NPC_RANGE = 6
local cards = require("lib.cards")

local model = protocol.new_model()
local st = { next_beat = os.clock() + 2, queue = {}, version = 0 }

local function ask(text)
  st.queue[#st.queue + 1] = text
  if text == "cd on" then st.next_beat = os.clock() + BEAT end
end

local function sound(name, vol)
  local ok, v = pcall(settings.get, "sounds")
  if ok and v == false then return end
  pcall(audio.play, ("sounds/%s.wav"):format(name), vol or 0.5)
end

book.setup(model)
book.send = ask
tbl.setup(model)
tbl.send = ask
found.setup()
howto.setup()
invite.setup()
invite.send = ask
book.howto = function() howto.open() end
tbl.howto = function() howto.open() end
book.to_table = function() ui.open(tbl.WIN) tbl.render() end
found.open_book = function() if not ui.is_open(book.WIN) then book.toggle() end end

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  local what = protocol.feed(model, line, os.clock())
  if what == "book" then
    book.on_book()
  elseif what == "message" then
    book.render()
  elseif what == "game" then
    tbl.reset()
    ui.open(tbl.WIN)
    sound("shuffle", 0.45)
    -- The first game: the rules open beside the table
    if storage and storage.character and not storage.character.cardian_rules_seen then
      storage.character.cardian_rules_seen = true
      pcall(storage.save)
      howto.open(1)
    end
  elseif what == "state" then
    local n = model.game and model.game.now
    if n and n.last and #n.last.flips > 0 then sound("flip", 0.5) end
    tbl.state.waiting = false
    tbl.render()
  elseif what == "result" then
    tbl.state.waiting = false
    if model.result.winner == 1 then sound("win", 0.55) elseif model.result.winner == 2 then sound("lose", 0.45) end
    tbl.render()
    if ui.is_open(book.WIN) then ask("cd book") end
  elseif what == "invite" then
    invite.show(model.invite, os.clock())
    sound("flip", 0.5)
  elseif what == "found" then
    local f = model.found[#model.found]
    local ok, show = pcall(settings.get, "popup")
    if not (ok and show == false) then found.show(f.id, f.how, os.clock()) end
    sound("found", 0.55)
    if ui.is_open(book.WIN) then ask("cd book") end
  end
  return false
end)

events.on("login", function() st.next_beat = os.clock() + 3 end)
events.on("zone_change", function() st.next_beat = os.clock() + 3 end)
events.on("unload", function()
  pcall(action.send, "cd off")
  pcall(hud.remove, CARD_BTN)
  pcall(hud.remove, CARD_BTN .. "_t")
  pcall(hud.remove, NPC_BTN)
  pcall(hud.remove, NPC_BTN .. "_t")
end)
events.on("target_change", function() if ui.is_open(book.WIN) then book.render() end end)

commands.register("cardbook", function() book.toggle() end)
commands.register("cardian", function()
  if model.game and not model.game.over then
    if ui.is_open(tbl.WIN) then ui.close(tbl.WIN) else ui.open(tbl.WIN) tbl.render() end
  else
    book.toggle()
  end
end)

mods.provide("timeless_menu", function()
  return {
    title = "Cardian",
    order = 18,
    open = "//cardbook",
    command = "/cardbook",
    desc = "Your Cardian cards, sets, quests (the Card Club, job masters, lost cards) and hand. Monsters drop their cards (some only at night, in a weather or under the beastmen); bind one with a Card Binder; play the Steaming Sheep's regulars after dark, Shaali the card shark, or other players.",
  }
end)

-- The Card button: a Card Binder, and the monster you're fighting is weak enough
local BIND_HPP, BIND_RANGE = 20, 20
st.card_shown = false
local function card_target()
  if not (model.book.binder) then return nil end
  local ok, t = pcall(game.target)
  if not ok or not t or t.kind ~= "mob" or t.me then return nil end
  if (t.status or 0) ~= 1 or (t.hpp or 100) > BIND_HPP or (t.hpp or 0) <= 0 or (t.distance or 99) > BIND_RANGE then return nil end
  return t
end
function st.card_button()
  local t = card_target()
  if not t then
    if st.card_shown then pcall(hud.remove, CARD_BTN) pcall(hud.remove, CARD_BTN .. "_t") st.card_shown = false end
    return
  end
  local s = hud.screen()
  local k = s.ui_scale or 1
  local w, h = math.floor(120 * k), math.floor(34 * k)
  local x, y = math.floor((s.width - w) / 2), math.floor(s.height * 0.62)
  hud.rect(CARD_BTN, { x = x, y = y, w = w, h = h, colour = { 120, 86, 40, 220 }, layer = 5,
    tip = { title = "Card", text = ("Turn %s into its card with your Card Binder. You get no experience for it."):format(t.name) } })
  hud.text(CARD_BTN .. "_t", { x = x + w / 2, y = y + math.floor(8 * k), text = "Card", size = 1.15 * k, colour = { 255, 236, 196, 255 },
    align = "center", layer = 6 })
  if not st.card_shown then
    hud.on(CARD_BTN, function(_, e)
      if e.kind == "click" and e.button == "left" then
        local tt = card_target()
        if tt then ask("cd bind " .. tt.id) sound("flip", 0.5) end
      end
    end)
  end
  st.card_shown = true
end

-- The Play cards button: your target is a club member you've met (or are about to), or your job's master
local function norm(name) return (name or ""):gsub("[%s_]", ""):lower() end
local function npc_target()
  local b = model.book
  local ok, t = pcall(game.target)
  if not ok or not t or t.kind ~= "npc" or (t.distance or 99) > NPC_RANGE then return nil end
  local n = norm(t.name)
  for r, cl in ipairs(cards.CLUB) do
    local state = (b.club or {})[r]
    if (state == "next" or state == "beaten") and norm(cl.npc) == n then return t, cl.title .. ": " .. cl.name end
  end
  for _, m in ipairs(cards.MASTERS) do
    if m[1] == b.job and norm(m[3]) == n then return t, ("%s master: %s"):format(m[2], (m[3]:gsub("_", " "))) end
  end
end
st.npc_shown = false
function st.npc_button()
  local t, who = npc_target()
  if not t or (model.game and not model.game.over) then
    if st.npc_shown then pcall(hud.remove, NPC_BTN) pcall(hud.remove, NPC_BTN .. "_t") st.npc_shown = false end
    return
  end
  local s = hud.screen()
  local k = s.ui_scale or 1
  local w, h = math.floor(150 * k), math.floor(34 * k)
  local x, y = math.floor((s.width - w) / 2), math.floor(s.height * 0.62)
  hud.rect(NPC_BTN, { x = x, y = y, w = w, h = h, colour = { 60, 80, 130, 220 }, layer = 5,
    tip = { title = "Play cards", text = who .. ". A game of Cardian." } })
  hud.text(NPC_BTN .. "_t", { x = x + w / 2, y = y + math.floor(8 * k), text = "Play cards", size = 1.1 * k, colour = { 230, 238, 255, 255 },
    align = "center", layer = 6 })
  if not st.npc_shown then
    hud.on(NPC_BTN, function(_, e)
      if e.kind == "click" and e.button == "left" then
        local tt = npc_target()
        if tt then ask("cd npc " .. tt.id) end
      end
    end)
  end
  st.npc_shown = true
end

timer.every(0.1, function()
  local t = os.clock()
  if t >= st.next_beat then ask("cd on") end
  if #st.queue > 0 then pcall(action.send, table.remove(st.queue, 1)) end
  found.tick(t)
  invite.tick(t)
  pcall(st.card_button)
  pcall(st.npc_button)
end)
