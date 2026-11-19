-- Timeless Reputation: your fame in San d'Oria, Bastok, Windurst, Jeuno, Selbina and Rabao, and Norg.
--
-- Open it from the Timeless menu (the Reputation card). A window shows each area's crest, its fame level (1-9) and
-- what people there make of you, how far it is to the next level, and how fame there grows; plus your nation and rank.
-- It asks the server with a packet (not chat) when it opens and every 30 s while open; the answers are taken out of
-- the chat log. Needs the server's reputation module.

local action = require("lib.action")
local protocol = require("lib.protocol")
local window = require("lib.window")

local EVERY = 30

local model = protocol.new_model()
local st = { queue = {}, next_ask = nil }

local function ask()
  st.queue[#st.queue + 1] = "rp"
  st.next_ask = os.clock() + EVERY
end

window.setup(model)
window.ask = ask

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  if protocol.feed(model, line) == "done" then window.render() end
  return false
end)

events.on("login", function() st.next_ask = os.clock() + 5 end) -- for the menu card's summary

-- Opened by the Timeless menu's card (and a key, if you set one)
commands.register("reputation", function() window.toggle() end)

local bound
local function apply_key()
  if bound then pcall(binds.unset, bound) bound = nil end
  local ok, key = pcall(settings.get, "key_reputation")
  if ok and type(key) == "string" and key ~= "" and pcall(binds.set, key, window.toggle) then bound = key end
end
apply_key()
events.on("settings_changed", apply_key)

mods.provide("timeless_menu", function()
  return {
    title = "Reputation",
    order = 25,
    open = "//reputation",
    desc = window.summary() or "Your fame in San d'Oria, Bastok, Windurst, Jeuno, Selbina and Rabao, and Norg.",
  }
end)

timer.every(0.1, function()
  local t = os.clock()
  if st.next_ask and t >= st.next_ask and (ui.is_open(window.WIN) or not model.loaded) then
    ask()
    if not ui.is_open(window.WIN) then st.next_ask = nil end -- the login ask, once
  end
  if #st.queue > 0 then pcall(action.send, table.remove(st.queue, 1)) end
end)
