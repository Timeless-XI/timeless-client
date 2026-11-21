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
-- A high-quality synth flashes and chimes; a Masterwork bursts gold, eases the camera in, and saves a screenshot
-- under screenshots/masterworks (each can be turned off in the mod's settings).
--
--   /masterworks         open or close the ledger
--   /masterworks update  ask again now

local action = require("lib.action")
local protocol = require("lib.protocol")
local ledger = require("lib.ledger")
local moment = require("lib.moment")

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
  if protocol.feed(ledger.model, line, os.clock()) == "status" then
    local s = ledger.model.status
    if s and s.masterworks then
      moment.on_count(s.masterworks.made, os.clock(), function(id)
        local ok, r = pcall(res.item, id)
        return ok and r and r.name or "Masterwork"
      end)
    end
    ledger.render()
  end
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

-- High-quality and Masterwork moments (lib/moment.lua): after a high-quality synth, ask for the Masterwork count
events.on("synth_result", function(result, item, count, grade)
  if (result == 0 or result == 12) and (grade or 0) >= 1 then
    moment.on_hq(item, grade, os.clock(), function() st.count_at = os.clock() + 0.5 end)
  end
end)
-- The count to compare with, once logged in
local function ask_count() st.count_at = os.clock() + 3 end
ask_count()
events.on("login", ask_count)
events.on("unload", function() moment.clear() end)
events.on("zone_change", function() moment.clear() end)

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

timer.every(0.05, function() moment.update(os.clock()) end)

timer.every(0.1, function()
  local t = os.clock()
  if st.count_at and t >= st.count_at then
    st.count_at = nil
    pcall(action.send, "cb status 1")
  end
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
