-- Timeless Telegraphs: when a monster readies an area move, its reach is marked on the ground.
--
-- The TimelessXI server tells the client the moment a monster you're near starts readying a move that hits an area:
-- a circle around it, a circle around its target, or a cone ahead of or behind it, with how long it takes. This mod
-- draws that area where it is, filling up until the move goes off, and follows the monster as it turns and moves.
-- The server picks who is hit when the move goes off, so stepping out of the marker in time avoids it. Notorious
-- monsters, Elite and Legendary Champions and rift echoes take longer to ready their area moves, so there is time.
-- Standing inside one, a "Move out!" line counts down on screen.
--
-- The server sends these only while this mod asks (a packet, not chat, renewed every 20 s); they are taken out of
-- the chat log. Needs the server's combat_feed module.
--
--   /telegraphs          turn the markers on or off
--   /telegraphs alert    the on-screen "Move out!" on or off
--   /telegraphs labels   the move's name over the marker on or off
--   /telegraphs opacity <0.2-1>
--   /telegraphs test     sample markers around you, to see how they look

local action = require("lib.action")
local protocol = require("lib.protocol")
local shapes = require("lib.shapes")

local BEAT = 20
local FLASH = 0.3 -- seconds the marker flashes when the move goes off
local COLOUR = { 255, 84, 40 }

local tg = require("lib.state")
tg.next_beat = os.clock() + 1.5

local function prefs()
  storage.character.telegraphs = storage.character.telegraphs or {}
  local p = storage.character.telegraphs
  if p.enabled == nil then p.enabled = true end
  if p.alert == nil then p.alert = true end
  if p.labels == nil then p.labels = true end
  p.opacity = math.max(0.2, math.min(1, tonumber(p.opacity) or 0.9))
  return p
end

local function ask(words)
  tg.queue[#tg.queue + 1] = words
  tg.next_beat = os.clock() + BEAT
end

local function key_of(t) return "tg" .. tostring(t.mob) end

local function clear_all()
  for _, t in pairs(tg.active) do shapes.remove(key_of(t)) end
  tg.active = {}
  pcall(hud.remove, "tg_alert")
end

-- A new telegraph replaces the monster's old one.
local function add(t)
  local old = tg.active[t.mob]
  if old then shapes.remove(key_of(old)) end
  t.cache = {}
  tg.active[t.mob] = t
end

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  if prefs().enabled then
    local t = protocol.parse(line, os.clock())
    if t then add(t) end
  end
  return false
end)

local function entity(index)
  local me = game.player()
  if me and index == me.index then return { pos = me.pos, heading = me.heading, hpp = me.hpp or 100 } end
  local ok, e = pcall(game.entity, index)
  return ok and e or nil
end

-- Where telegraph t lies now: { centre = point, heading }, or nil when its monster is gone.
local function where(t)
  if t.fixed then return t.fixed end
  local mob = entity(t.mob)
  if mob and (mob.hpp or 1) > 0 then
    t.last = { pos = mob.pos, heading = mob.heading }
  elseif mob then
    return nil -- it died: the move won't go off
  end
  local last = t.last
  if not last then return nil end
  if t.shape == "t" then
    local target = entity(t.target)
    if target then t.last_target = target.pos end
    local centre = t.last_target
    return centre and { centre = centre, heading = last.heading } or nil
  end
  local heading = t.shape == "b" and (last.heading + math.pi) or last.heading
  return { centre = last.pos, heading = heading % (2 * math.pi) }
end

local function frame()
  if next(tg.active) == nil then return end
  local p = prefs()
  local now = os.clock()
  local me = game.player()
  local alert
  for mob, t in pairs(tg.active) do
    local w = where(t)
    if not w or now >= t.ends + FLASH then
      shapes.remove(key_of(t))
      tg.active[mob] = nil
    else
      local total = math.max(0.05, t.ends - t.start)
      local frac = math.max(0, math.min(1, (now - t.start) / total))
      local flash = now >= t.ends
      shapes.draw(key_of(t), t, w, frac, { colour = COLOUR, opacity = p.opacity, labels = p.labels, flash = flash,
        cache = t.cache })
      if not flash and me and shapes.inside(t, w, me.pos) then
        local left = t.ends - now
        if not alert or left < alert then alert = left end
      end
    end
  end
  if alert and p.alert then
    local s = hud.screen()
    hud.text("tg_alert", { x = s.width / 2, y = math.floor(s.height * 0.3), text = ("Move out!  %.1f"):format(alert),
      size = 1.6, align = "center", colour = { 255, 236, 200, 255 }, outline = { 90, 20, 0, 240 }, outline_width = 2,
      font = "bold" })
  else
    pcall(hud.remove, "tg_alert")
  end
end
events.on("frame", frame)

events.on("zone_change", clear_all)
events.on("login", function() ask("tg on") end)
events.on("unload", function()
  pcall(action.send, "tg off")
  clear_all()
end)

local function msg(text) print("Telegraphs: " .. text) end

-- Sample markers around you: a circle under you, and a cone from a spot ahead of you, pointing at you.
local function test()
  local me = game.player()
  if not me then return end
  local now = os.clock()
  local hx, hz = shapes.dir(me.heading or 0)
  local ahead = { x = me.pos.x + hx * 8, y = me.pos.y, z = me.pos.z + hz * 8 }
  add({ mob = -1, shape = "c", radius = 6, angle = 45, target = 0, name = "Sample circle", start = now, ends = now + 3,
    fixed = { centre = me.pos, heading = me.heading or 0 } })
  add({ mob = -2, shape = "f", radius = 12, angle = 45, target = 0, name = "Sample cone", start = now, ends = now + 3.5,
    fixed = { centre = ahead, heading = (me.heading or 0) + math.pi } })
end

local function telegraphs_command(sub, value)
  local p = prefs()
  sub = (sub or ""):lower()
  if sub == "" or sub == "on" or sub == "off" then
    if sub == "" then p.enabled = not p.enabled else p.enabled = sub == "on" end
    if p.enabled then
      ask("tg on")
      msg("on: monsters' area moves are marked on the ground.")
    else
      ask("tg off")
      clear_all()
      msg("off (/telegraphs to turn them back on).")
    end
  elseif sub == "alert" then
    p.alert = not p.alert
    msg(p.alert and "the on-screen \"Move out!\" is on." or "the on-screen \"Move out!\" is off.")
  elseif sub == "labels" then
    p.labels = not p.labels
  elseif sub == "opacity" and tonumber(value) then
    p.opacity = tonumber(value)
  elseif sub == "test" then
    test()
  else
    msg("/telegraphs | alert | labels | opacity <n> | test")
    return
  end
  storage.save()
  prefs()
end
commands.register("telegraphs", telegraphs_command)
commands.register("timelesstelegraphs", telegraphs_command)

mods.provide("timeless_menu", function()
  local p = prefs()
  return {
    title = "Telegraphs",
    desc = "Monsters' area moves are marked on the ground as they ready them. Step out before they go off.",
    order = 36,
    open = "//telegraphs test",
    buttons = {
      { label = p.enabled and "Turn off" or "Turn on", command = "//telegraphs", tip = "Markers on or off." },
      { label = "Test", command = "//telegraphs test", tip = "Sample markers around you." },
      { label = "Alert", command = "//telegraphs alert", tip = "The on-screen \"Move out!\" on or off." },
    },
  }
end)

-- Every tenth of a second: send what is queued and the heartbeat while on.
timer.every(0.1, function()
  local t = os.clock()
  if prefs().enabled and t >= tg.next_beat then ask("tg on") end
  if #tg.queue > 0 then pcall(action.send, table.remove(tg.queue, 1)) end
end)

