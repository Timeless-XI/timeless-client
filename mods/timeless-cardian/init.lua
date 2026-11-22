-- Timeless Cardian: Vana'diel's card game (the server's cardian module).
--
--   /cardbook   your cards (and the ones still to find), Champions' foils and your hand of five; also on the Timeless menu
--   /cardian    the table, while a game is on (talk to a card player at the Steaming Sheep after dark, or to Shaali
--               the travelling card shark, to sit down)
--   How to play: a button on the table and in the cardbook (and it opens by itself with your first game)
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
events.on("unload", function() pcall(action.send, "cd off") end)
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
    desc = "Your Cardian cards and hand. Monsters drop their cards; play the Steaming Sheep's regulars after dark, Shaali the card shark, or other players.",
  }
end)

timer.every(0.1, function()
  local t = os.clock()
  if t >= st.next_beat then ask("cd on") end
  if #st.queue > 0 then pcall(action.send, table.remove(st.queue, 1)) end
  found.tick(t)
  invite.tick(t)
end)
