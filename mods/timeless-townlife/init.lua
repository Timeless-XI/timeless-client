-- Timeless Town Life: Bastok, lived in. The server's townlife module walks the townsfolk around (a stall keeper's
-- shift change, the lamplighter, guards, kids, fishermen, the caravan, market day); this mod lets you see and hear it:
--
--   speech bubbles over townsfolk as they talk (the same lines are in chat for everyone)
--   the lamp posts on the lamplighter's round dark until he lights them (Modern night lighting), chimney smoke
--   sounds by place: hammering by the furnaces, crowds on the plaza and in the Steaming Sheep, gulls on the docks
--   /town  the Town window: headlines, who has the stall, the caravan, market day, your companions in town, gossip
--
-- It asks the server with a packet (not chat): "tl on" every 20 s (the server stops 60 s after the last one), and the
-- feed lines are taken out of the chat log. Without the server's module it shows nothing.

local action = require("lib.action")
local protocol = require("lib.protocol")
local world = require("lib.world")
local town = require("lib.town")

local BEAT = 20

local model = protocol.new_model()
local st = { next_beat = os.clock() + 2, queue = {}, last_scene = 0 }

local function ask(what)
  st.queue[#st.queue + 1] = "tl " .. what
  if what == "on" then st.next_beat = os.clock() + BEAT end
end

town.setup(model)
town.ask = function() ask("town") end

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  local what = protocol.feed(model, line, os.clock())
  if what == "rows" then town.render() end
  return false
end)

events.on("zone_change", function()
  world.clear()
  model.lamps, model.scenery, model.bubbles, model.rows, model.carry = {}, {}, {}, {}, {}
  model.all_lamps = {}
  model.zone, model.zone_name = 0, ""
  town.render()
  st.next_beat = os.clock() + 3 -- the server sends this zone's snapshot when it hears from us in it
end)
events.on("login", function() st.next_beat = os.clock() + 3 end)
events.on("unload", function()
  pcall(action.send, "tl off")
  world.clear()
end)

commands.register("town", function(sub)
  sub = (sub or ""):lower()
  if sub == "update" or sub == "refresh" then
    ask("town")
  else
    town.toggle()
  end
end)
commands.register("townlife", function() town.toggle() end)

mods.provide("timeless_menu", function()
  return {
    title = "Town",
    order = 16,
    open = "//town",
    command = "/town",
    desc = model.zone_name ~= "" and ("What's going on in %s: the stall, the caravan, market day and the gossip."):format(model.zone_name)
      or "What's going on in town: headlines, the caravan, market day and the gossip.",
  }
end)

timer.every(0.1, function()
  local t = os.clock()
  if t >= st.next_beat then ask("on") end
  if #st.queue > 0 then pcall(action.send, table.remove(st.queue, 1)) end
  world.bubbles(model, t)
  if t - st.last_scene >= 0.2 then
    st.last_scene = t
    local me = game.player()
    world.scene(model, t, me)
    world.sounds(model, t, me)
  end
end)
