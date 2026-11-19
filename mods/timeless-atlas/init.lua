-- Timeless NM Atlas: every notorious monster of the open world, on the Timeless menu.
--
-- Search by name, by an item it drops or by its placeholder; filter to the ones up now, the ones in your zone or a
-- level band. Pick one to see its model turning slowly (TimelessXI client build 6 and up), where it lives, its level
-- and family, how it spawns (lottery from placeholders, a timer, the items to trade at its ???), whether it is up or
-- how long until it can spawn, what it is weak to (elements, enfeebles, immunities, damage types) and what it drops,
-- with each item's chance.
--
-- It asks the server with a packet (not chat) and the answers are taken out of the chat log. Needs the server's
-- nm_atlas module.

local action = require("lib.action")
local protocol = require("lib.protocol")
local atlas = require("lib.atlas")

local REFRESH = 60

local model = protocol.new_model()
local st = { queue = {}, last_refresh = 0 }

local function ask(text)
  -- One list question at a time in the queue (the newest search wins)
  if text:sub(1, 8) == "na list " then
    for i = #st.queue, 1, -1 do
      if st.queue[i]:sub(1, 8) == "na list " then table.remove(st.queue, i) end
    end
  end
  st.queue[#st.queue + 1] = text
end

atlas.setup(model)
atlas.ask = ask
atlas.on_forget = function() st.queue = {} end

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  if not ui.is_open(atlas.WIN) then return false end -- a late answer after closing: hidden, not kept
  local what = protocol.feed(model, line, os.clock())
  if what == "list" or what == "info" then atlas.render() end
  return false
end)

-- Opened from the Timeless menu's card (and a key, if you set one)
commands.register("atlas", function() atlas.toggle() end)

local bound
local function apply_key()
  if bound then pcall(binds.unset, bound) bound = nil end
  local ok, key = pcall(settings.get, "key_atlas")
  if ok and type(key) == "string" and key ~= "" and pcall(binds.set, key, atlas.toggle) then bound = key end
end
apply_key()
events.on("settings_changed", apply_key)

mods.provide("timeless_menu", function()
  return {
    title = "NM Atlas",
    order = 17,
    open = "//atlas",
    desc = "Every notorious monster: whether it's up, how it spawns, what it's weak to and what it drops. Search by name or item.",
  }
end)

timer.every(0.1, function()
  -- Closed with nothing to send: nothing to do
  if #st.queue == 0 and not atlas.state.typed_at and not ui.is_open(atlas.WIN) then return end
  local t = os.clock()
  -- A search typed: asked a moment after the last key
  if atlas.state.typed_at and t - atlas.state.typed_at >= 0.35 then
    atlas.state.typed_at = nil
    atlas.refresh()
  end
  -- Fresh statuses while the window is open
  if ui.is_open(atlas.WIN) then
    if t - st.last_refresh >= REFRESH then
      st.last_refresh = t
      ask(atlas.list_query())
      if atlas.state.selected then ask("na info " .. atlas.state.selected) end
    end
    -- The countdowns tick every few seconds
    if t - (st.last_draw or 0) >= 5 then
      st.last_draw = t
      atlas.render()
    end
  end
  if #st.queue > 0 then pcall(action.send, table.remove(st.queue, 1)) end
end)
