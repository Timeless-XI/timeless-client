-- Timeless Threat: exact enmity from the TimelessXI server, on screen.
--
-- An edge window shows each monster you, your party, your companions and your pets are fighting (your target first):
-- its HP, and a bar for everyone on its enmity list, measured against whoever it is attacking (►). The numbers are
-- the server's own (cumulative + volatile enmity), not a guess from the chat log. Over 90% of the holder a bar warns,
-- and when it's yours a line on screen tells you to ease off. The window hides itself out of combat.
--
-- The server sends only what changes, once a second, and only while this mod asks: it asks with a packet (not chat),
-- says "still here" every 20 s (the server stops a minute after the last one), and the lines are taken out of the
-- chat log. Needs the server's combat_feed module.
--
--   /threat            show or hide the window
--   /threat compact    only your target
--   /threat rows <n>   names per monster (2-7);   /threat width <n>   (160-360)
--   /threat warn       the on-screen warning on or off
--   /threat demo       a sample fight, to place the window
--   /threat update     ask the server for everything again

local action = require("lib.action")
local protocol = require("lib.protocol")
local threat = require("lib.threat")

local BEAT = 20

local st = { next_beat = os.clock() + 2, full_at = os.clock() + 1, queue = {}, last_second = -1 }

local function ask(full)
  st.queue[#st.queue + 1] = full and "en on full" or "en on"
  st.next_beat = os.clock() + BEAT
end

local function ask_full_soon(seconds)
  local at = os.clock() + seconds
  if st.full_at == nil or at < st.full_at then st.full_at = at end
end

threat.setup()

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  protocol.feed(threat.demo or threat.model, line, os.clock())
  if not threat.demo then threat.dirty = true end
  return false
end)

events.on("zone_change", function()
  threat.model = threat.demo and threat.model or protocol.new_model()
  ask_full_soon(3)
end)
events.on("login", function() ask_full_soon(3) end)
events.on("unload", function()
  pcall(action.send, "en off")
  pcall(hud.remove, "threat_warn")
end)

local function msg(text) print("Enmity: " .. text) end

local function threat_command(sub, value)
  local p = threat.prefs()
  sub = (sub or ""):lower()
  local n = tonumber(value)
  if sub == "" then
    p.visible = not p.visible
    msg(p.visible and "shown when you fight." or "hidden (/threat to show it).")
  elseif sub == "compact" then
    p.compact = not p.compact
    msg(p.compact and "only your target." or "every monster you're fighting.")
  elseif sub == "rows" and n then
    p.rows = n
  elseif sub == "width" and n then
    p.width = n
  elseif sub == "warn" then
    p.warn = not p.warn
    msg(p.warn and "the on-screen warning is on." or "the on-screen warning is off.")
  elseif sub == "demo" then
    threat.toggle_demo()
    msg(threat.demo and "a sample fight. /threat demo again to go back." or "back to your own fights.")
  elseif sub == "update" or sub == "refresh" then
    ask(true)
  else
    msg("/threat | compact | rows <n> | width <n> | warn | demo | update")
    return
  end
  storage.save()
  threat.prefs()
  threat.define()
  threat.update()
end
commands.register("threat", threat_command)
commands.register("timelessthreat", threat_command)

local bound
local function apply_key()
  if bound then pcall(binds.unset, bound) bound = nil end
  local ok, key = pcall(settings.get, "key_threat")
  if ok and type(key) == "string" and key ~= "" and pcall(binds.set, key, function()
    local p = threat.prefs()
    p.visible = not p.visible
    storage.save()
    threat.update()
  end) then bound = key end
end
apply_key()
events.on("settings_changed", apply_key)

mods.provide("timeless_menu", function()
  return {
    title = "Enmity",
    desc = "Exact enmity for every monster you fight, from the server. Warns you before you pull hate.",
    order = 35,
    open = "//threat",
    buttons = {
      { label = "Demo", command = "//threat demo", tip = "A sample fight, to place the window." },
      { label = "Compact", command = "//threat compact", tip = "Only your target, or every monster." },
      { label = "Warning", command = "//threat warn", tip = "The on-screen warning on or off." },
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
  if #st.queue > 0 then
    pcall(action.send, table.remove(st.queue, 1))
  end
  local second = math.floor(t * 4) -- four times a second at most when idle
  if threat.dirty or threat.animated() or second ~= st.last_second then
    threat.dirty = false
    st.last_second = second
    threat.update()
  end
end)
