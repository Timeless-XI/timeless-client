-- Timeless Companions: TimelessXI's companions for vana-client, live.
--
--   /cv  Companion View: each companion's gear, stats and role (replaces the companionview Ashita addon)
--   /cmgr  Companion Manager: call, keep, save and set up companions (replaces companionmanager)
--   In the world: name plates and a ring in each companion's role colour, and their party chat in speech bubbles.
--
-- The windows talk to the server's companions module the way the Ashita addons do: a button sends its !companion
-- command in a 0x0B5 action packet (lib/action.lua), and the server answers with feed lines (CV1 / CM1) that the
-- chat filter reads and hides (lib/link.lua). What changes moment to moment (HP, MP, TP, targets, spells, weapon
-- skills, status effects) comes from the client itself (lib/live.lua), so it is live without asking the server.

local link = require("lib.link")
local live = require("lib.live")
local view = require("lib.view")
local manager = require("lib.manager")
local world = require("lib.world")

view.setup()
manager.setup()

link.is_open.cv = view.is_open
link.is_open.cm = manager.is_open
view.open_manager = function() manager.open() end
manager.open_view = function() view.open() end

filters.chat(function(text)
  return link.on_line(text)
end)

events.on("action", function(a) live.on_action(a) end)

local members = {}
events.on("chat_message", function(msg, sender, kind)
  world.on_chat(msg, sender, kind, members)
end)

local function cv_command(sub)
  sub = (sub or ""):lower()
  if sub == "update" or sub == "refresh" then
    if not view.is_open() then view.open() else link.ask("cv") end
  else
    view.toggle()
  end
end
commands.register("cv", cv_command)
commands.register("companionview", cv_command)

local function cm_command(sub)
  sub = (sub or ""):lower()
  local tab = manager.tab_index(sub)
  if sub == "update" or sub == "refresh" then
    if not manager.is_open() then manager.open() else link.ask("cm") end
  elseif tab then
    manager.open(tab)
  else
    manager.toggle()
  end
end
commands.register("cmgr", cm_command)
commands.register("companionmanager", cm_command)

-- Keys from the settings
local bound = {}
local function apply_keys()
  for _, k in ipairs(bound) do pcall(binds.unset, k) end
  bound = {}
  for setting, fn in pairs({ key_view = view.toggle, key_manager = manager.toggle }) do
    local ok, key = pcall(settings.get, setting)
    if ok and type(key) == "string" and key ~= "" then
      if pcall(binds.set, key, fn) then bound[#bound + 1] = key end
    end
  end
end
apply_keys()
events.on("settings_changed", function()
  apply_keys()
end)

-- Every tenth of a second: send queued actions and ask again when due; draw the world; redraw a window
-- when its feed changed, or (a few times a second) when what it shows live did.
local last_second, last_live, live_sig = 0, 0, ""
timer.every(0.1, function()
  local ok, auto = pcall(settings.get, "auto_refresh")
  link.tick(ok and auto or 0)

  members = live.members()
  world.update(members)

  if link.dirty.cm and view.is_open() and link.cm.loaded and not link.waiting.cv then
    -- The manager's feed shows a different party (someone called or dismissed): Companion View asks again
    local inparty, seen = {}, {}
    for _, p in ipairs(link.cm.party) do inparty[p.id] = true end
    local same = true
    for _, id in ipairs(link.cv.order) do
      seen[id] = true
      if not inparty[id] then same = false end
    end
    for id in pairs(inparty) do
      if not seen[id] then same = false end
    end
    if not same then link.ask("cv") end
  end

  local redraw_view, redraw_manager = false, false
  if link.dirty.cv then
    link.dirty.cv = false
    redraw_view = true
  end
  if link.dirty.cm or link.dirty.messages then
    link.dirty.cm = false
    link.dirty.messages = false
    manager.refresh()
  end

  local t = os.clock()
  if t - last_live >= 0.3 then
    last_live = t
    local sig = live.signature(members)
    if sig ~= live_sig then
      live_sig = sig
      redraw_view = true
      redraw_manager = manager.state.tab == 1
    end
  end

  -- Once a second: countdowns tick, old answers fade, "Switching..." ends
  local s = math.floor(t)
  if s ~= last_second then
    last_second = s
    redraw_manager = true
    if view.state.switching_until then
      if t >= view.state.switching_until then view.state.switching_until = nil end
      redraw_view = true
    end
  end
  if redraw_view and view.is_open() then view.render() end
  if redraw_manager and manager.is_open() then manager.render_all() end
end)
