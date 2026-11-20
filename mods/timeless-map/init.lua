-- Timeless Map: what is happening across Vana'diel right now (/worldmap).
--
-- A window with the conquest regions drawn as a map, each in its owner's colour (beastman-held ones marked), and
-- markers for what is going on there: open Time, Ancient and Hero's Rifts (with the minutes until they close),
-- beastman warbands, Fomor bands and roaming Champions. A panel lists them, everywhere or for the region you click,
-- with the moon and the day. In your own zone, pillars of light stand over each of them, with what and how far.
-- When a rift, a warband, a Fomor band, a Legendary or a Gilded Champion appears anywhere, a line in chat says so.
--
-- The server sends a fresh picture only when something changed (checked every 10 s), and only while this mod asks:
-- it asks with a packet (not chat), says "still here" every 30 s (the server stops 90 s after the last one), and the
-- lines are taken out of the chat log. Needs the server's world_map module.
--
--   /worldmap          open or close the map (also /wmap)
--   /worldmap beams    the pillars of light on or off
--   /worldmap alerts   the chat lines on or off
--   /worldmap update   ask the server for everything again

local action = require("lib.action")
local protocol = require("lib.protocol")
local map = require("lib.worldmap")
local beams = require("lib.beams")

local BEAT = 30

local st = { next_beat = os.clock() + 2, full_at = os.clock() + 1, queue = {}, last_draw = 0, dirty = false }

local function ask(full)
  st.queue[#st.queue + 1] = full and "wm on full" or "wm on"
  st.next_beat = os.clock() + BEAT
end

local function ask_full_soon(seconds)
  local at = os.clock() + seconds
  if st.full_at == nil or at < st.full_at then st.full_at = at end
end

map.setup()

local ALERT = { R = true, W = true, F = true, G = true }

local function alert(fresh)
  if not map.prefs().alerts then return end
  for _, e in ipairs(fresh or {}) do
    if ALERT[e.k] or (e.k == "C" and e.c == 3) then
      print(("World map: %s (%s)."):format(map.describe(e, os.clock()), map.region_name(e.region)))
    end
  end
end

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  local what, fresh = protocol.feed(map.model, line, os.clock())
  if what == "snapshot" then
    alert(fresh)
    st.dirty = true
  elseif what == "here" then
    st.beams_dirty = true
  end
  return false
end)

events.on("zone_change", function()
  map.model.here = {}
  beams.clear()
  ask_full_soon(3)
end)
events.on("login", function() ask_full_soon(3) end)
events.on("unload", function()
  pcall(action.send, "wm off")
  beams.clear()
end)

local function msg(text) print("World map: " .. text) end

local function worldmap_command(sub)
  local p = map.prefs()
  sub = (sub or ""):lower()
  if sub == "" then
    map.toggle()
    if ui.is_open(map.WIN) and map.model.version == 0 then ask(true) end
    return
  elseif sub == "beams" then
    p.beams = not p.beams
    msg(p.beams and "pillars of light on." or "pillars of light off.")
    st.beams_dirty = true
  elseif sub == "alerts" then
    p.alerts = not p.alerts
    msg(p.alerts and "alerts on." or "alerts off.")
  elseif sub == "update" or sub == "refresh" then
    ask(true)
  else
    msg("/worldmap | beams | alerts | update")
    return
  end
  storage.save()
  map.render()
end
commands.register("worldmap", worldmap_command)
commands.register("wmap", worldmap_command)

local bound
local function apply_key()
  if bound then pcall(binds.unset, bound) bound = nil end
  local ok, key = pcall(settings.get, "key_worldmap")
  if ok and type(key) == "string" and key ~= "" and pcall(binds.set, key, map.toggle) then bound = key end
end
apply_key()
events.on("settings_changed", apply_key)

mods.provide("timeless_menu", function()
  local n = #map.model.events
  return {
    title = "World Map",
    desc = n > 0 and (("%d things are happening across Vana'diel right now: rifts, warbands, Champions."):format(n))
      or "Conquest owners, open rifts, beastman warbands, Fomor bands and roaming Champions, across Vana'diel.",
    order = 15,
    open = "//worldmap",
    buttons = {
      { label = "Beams", command = "//worldmap beams", tip = "Pillars of light over what is happening in your zone." },
      { label = "Alerts", command = "//worldmap alerts", tip = "A chat line when a rift, warband or Legendary appears." },
    },
  }
end)

timer.every(0.1, function()
  local t = os.clock()
  if st.full_at and t >= st.full_at then
    st.full_at = nil
    ask(true)
  elseif t >= st.next_beat then
    ask(false)
  end
  if #st.queue > 0 then pcall(action.send, table.remove(st.queue, 1)) end
  -- The window: at once on news, otherwise every 5 s for the countdowns
  if ui.is_open(map.WIN) and (st.dirty or t - st.last_draw >= 5) then
    st.dirty = false
    st.last_draw = t
    map.render()
  end
  -- The pillars: four times a second (their labels count the yalms), and at once when the list changed
  if st.beams_dirty or t - (st.last_beams or 0) >= 0.25 then
    st.beams_dirty = false
    st.last_beams = t
    beams.update(map.model.here, map.prefs().beams)
  end
end)
