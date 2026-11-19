-- Timeless Gauges: your job's own meters on screen (replaces the timelessgauges Ashita addon): Faith, Beast Essence and
-- Unleash, Ki, Hot Streak, Shadow Step, Attunement and the rest of the TimelessXI job perks.
--
-- An edge window shows the gauges of your main job (jobs without any, like Paladin, show nothing); drag it along the
-- screen's edges like the Timers, or fold it to a tab. Full or ready gauges glow; point at one to see what it does.
--
-- The server sends only what changes, about once a second, and only while this mod asks: it asks with a packet (not
-- chat), says "still here" every 20 s (the server stops a minute after the last one), and the lines are taken out of
-- the chat log. Needs the server's gauges module.
--
--   /gauges            show or hide the bar
--   /gauges labels     show or hide the labels;  /gauges job  the job name
--   /gauges width <n>  gauge width (100-400);    /gauges opacity <n>  the bar's opacity (0-1)
--   /gauges demo       sample gauges, to place the bar without a fight
--   /gauges update     ask the server for everything again

local action = require("lib.action")
local protocol = require("lib.protocol")
local gauges = require("lib.gauges")

local BEAT = 20 -- seconds between "still here" requests

local ga = { next_beat = os.clock() + 2, full_at = os.clock() + 1, queue = {}, last_draw = 0, last_second = -1 }

local function ask(full)
  ga.queue[#ga.queue + 1] = full and "ga on full" or "ga on"
  ga.next_beat = os.clock() + BEAT
end

local function ask_full_soon(seconds)
  local at = os.clock() + seconds
  if ga.full_at == nil or at < ga.full_at then ga.full_at = at end
end

gauges.setup()

-- The feed: read it and take it out of the chat log (into the real model while the demo shows).
filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  protocol.feed(gauges.demo or gauges.model, line, os.clock())
  if not gauges.demo then gauges.dirty = true end
  return false
end)

-- Zoning in: the server's timers may have stopped; ask for everything again.
events.on("zone_change", function() ask_full_soon(3) end)
events.on("login", function() ask_full_soon(3) end)

-- Tell the server to stop now (it would anyway, a minute later).
events.on("unload", function() pcall(action.send, "ga off") end)

local function msg(text) print("Job Gauges: " .. text) end

local function gauges_command(sub, value)
  local p = gauges.prefs()
  sub = (sub or ""):lower()
  local n = tonumber(value)
  if sub == "" then
    p.visible = not p.visible
    msg(p.visible and "shown." or "hidden (/gauges to show them).")
    if p.visible then ask(true) end
  elseif sub == "labels" then
    p.labels = not p.labels
  elseif sub == "job" then
    p.job = not p.job
  elseif sub == "width" and n then
    p.width = math.max(100, math.min(400, n))
  elseif sub == "opacity" and n then
    pcall(ui.opacity, gauges.WIN, math.max(0, math.min(1, n)))
  elseif sub == "demo" then
    gauges.toggle_demo()
    msg(gauges.demo and "sample gauges shown. /gauges demo again to go back." or "back to your own gauges.")
  elseif sub == "update" or sub == "refresh" then
    ask(true)
  else
    msg("/gauges | labels | job | width <n> | opacity <n> | demo | update")
    return
  end
  storage.save()
  gauges.define()
  gauges.update()
end
commands.register("gauges", gauges_command)
commands.register("timelessgauges", gauges_command)

-- A key from the settings.
local bound
local function apply_key()
  if bound then pcall(binds.unset, bound) bound = nil end
  local ok, key = pcall(settings.get, "key_gauges")
  if ok and type(key) == "string" and key ~= "" and pcall(binds.set, key, function()
    local p = gauges.prefs()
    p.visible = not p.visible
    storage.save()
    gauges.update()
  end) then bound = key end
end
apply_key()
events.on("settings_changed", apply_key)

-- TimelessMenu's card.
mods.provide("timeless_menu", function()
  return {
    title = "Job Gauges",
    desc = "Your job's own meters (Faith, Essence, Ki, Hot Streak...) on the screen's edge. Full or ready gauges glow.",
    order = 30,
    open = "//gauges",
    buttons = {
      { label = "Demo", command = "//gauges demo", tip = "Sample gauges, to place the bar without a fight." },
      { label = "Labels", command = "//gauges labels", tip = "Show or hide the gauges' names." },
      { label = "Update", command = "//gauges update", tip = "Ask the server for your gauges again." },
    },
  }
end)

-- Every tenth of a second: send what is queued, the full ask when due and the heartbeat; redraw the bar when its data
-- changed, often while something on it moves, and once a second otherwise (countdowns).
timer.every(0.1, function()
  local t = os.clock()
  if ga.full_at and t >= ga.full_at then
    ga.full_at = nil
    ask(true)
  elseif t >= ga.next_beat then
    ask(false)
  end
  if #ga.queue > 0 then
    local text = table.remove(ga.queue, 1)
    pcall(action.send, text)
  end
  -- The demo's timers keep running: start it over when they run out.
  if gauges.demo and t - gauges.model.received > 30 then
    gauges.model = gauges.demo_model()
  end
  local second = math.floor(t)
  if gauges.dirty or gauges.animated() or second ~= ga.last_second then
    gauges.dirty = false
    ga.last_second = second
    gauges.update()
  end
end)
