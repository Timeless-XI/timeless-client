-- Timeless Masterworks: your crafting ledger (/masterworks).
--
-- A window in a workshop's walnut and brass: the eight crafts with your skill and guild rank (your best one lit),
-- the recipes you've made most with their mastery (Practiced, Mastered, Grandmaster marked on each bar) and the
-- Masterwork chance each has reached, how many Masterworks you've made, your workbench (rested synths, Flash of
-- Insight, lessons, guild rank), today's craft orders, and your apprentice's rank and progress.
--
-- It asks the server's Craft Plus module (a packet, not chat: "cb status 20") when it opens, every 30 s while it's
-- open, and just after each synth; the lines are taken out of the chat log.
--
--   /masterworks         open or close the ledger
--   /masterworks update  ask again now

local action = require("lib.action")
local protocol = require("lib.protocol")
local ledger = require("lib.ledger")

local REFRESH = 30
local st = { next_ask = nil, queue = {}, was_busy = false }

local function ask_soon(seconds)
  local at = os.clock() + (seconds or 0)
  if st.next_ask == nil or at < st.next_ask then st.next_ask = at end
end

ledger.setup()

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  if protocol.feed(ledger.model, line, os.clock()) == "status" then ledger.render() end
  return false
end)

local function toggle()
  if ui.is_open(ledger.WIN) then
    ui.close(ledger.WIN)
  else
    ui.open(ledger.WIN)
    ledger.render()
    ask_soon(0)
  end
end

local function masterworks_command(sub)
  sub = (sub or ""):lower()
  if sub == "" then
    toggle()
  elseif sub == "update" or sub == "refresh" then
    ask_soon(0)
  else
    print("Masterworks: /masterworks | update")
  end
end
commands.register("masterworks", masterworks_command)
commands.register("timelessmasterworks", masterworks_command)

local bound
local function apply_key()
  if bound then pcall(binds.unset, bound) bound = nil end
  local ok, key = pcall(settings.get, "key_masterworks")
  if ok and type(key) == "string" and key ~= "" and pcall(binds.set, key, toggle) then bound = key end
end
apply_key()
events.on("settings_changed", apply_key)

mods.provide("timeless_menu", function()
  local s = ledger.model.status
  local made = s and s.masterworks and s.masterworks.made or nil
  return {
    title = "Masterworks",
    desc = made and ("Your crafting ledger: mastery, Masterwork chances, orders and apprentice. Masterworks made: %d."):format(made)
      or "Your crafting ledger: mastery, Masterwork chances, today's orders and your apprentice.",
    order = 45,
    open = "//masterworks",
  }
end)

timer.every(0.1, function()
  local t = os.clock()
  local open = ui.is_open(ledger.WIN)
  -- Just after a synth, while the ledger is open: ask again
  local ok, synth = pcall(game.synth)
  local busy = ok and type(synth) == "table" and synth.busy or false
  if open and st.was_busy and not busy then ask_soon(1.5) end
  st.was_busy = busy
  if open and st.next_ask and t >= st.next_ask then
    st.next_ask = t + REFRESH
    pcall(action.send, "cb status 20")
  elseif not open then
    st.next_ask = nil
  end
end)
