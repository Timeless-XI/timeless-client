-- Timeless Cast Bars: see what monsters are casting.
--
-- A bar over each casting monster's head with the spell's (or move's) name, the seconds until it lands and a fill in
-- its element's colour; moves being readied show orange. A red frame means it is aimed at you or hits an area
-- ("AREA"). Interrupt it and the bar turns red ("Interrupted!").
--
-- The bars come from the game's own action packets, so they work anywhere; with the TimelessXI server (combat_feed,
-- "cb on", asked with a packet and renewed every 20 s, taken out of the chat log) their times are exact: the real
-- cast time (fast cast counted) and how long a move takes to ready.
--
--   /castbars          on or off
--   /castbars target   bars only for your target (again for every monster near you)
--   /castbars moves    bars for moves being readied on or off
--   /castbars size <0.6-2>
--   /castbars test     a sample cast on your target

local action = require("lib.action")
local protocol = require("lib.protocol")
local casts = require("lib.casts")
local bars = require("lib.bars")

local BEAT = 20
local RANGE = 45 -- yalms
local MAX = 6

local st = { queue = {}, next_beat = os.clock() + 1.5 }

local function prefs()
  storage.character.castbars = storage.character.castbars or {}
  local p = storage.character.castbars
  if p.enabled == nil then p.enabled = true end
  if p.target_only == nil then p.target_only = false end
  if p.moves == nil then p.moves = true end
  p.size = math.max(0.6, math.min(2, tonumber(p.size) or 1))
  return p
end

local function ask(words)
  st.queue[#st.queue + 1] = words
  st.next_beat = os.clock() + BEAT
end

local function msg(s) print("Cast bars: " .. s) end

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  local l = protocol.parse(line)
  if l and prefs().enabled then casts.on_line(l, os.clock()) end
  return false
end)

local function mob(id)
  local ok, e = pcall(game.entity, id)
  -- Monsters only: not our pet, a trust or anyone in the party (a party member's pet is a "mob" too)
  if ok and e and e.kind == "mob" and not e.pet and not e.trust and not e.in_party and not e.in_alliance then
    local master = (e.master or 0) ~= 0 and game.entity(e.master) or nil
    if not (master and (master.kind == "player" or master.me)) then return e end
  end
  return nil
end

local function base(id)
  local s = res.spell(id)
  return s and tonumber(s.cast_time) or nil
end

events.on("action", function(a)
  if not prefs().enabled then return end
  casts.on_action(a, os.clock(), mob, base)
end)

events.on("frame", function()
  local p = prefs()
  local now = os.clock()
  casts.expire(now)
  if not p.enabled or next(casts.casts) == nil then
    if next(bars.huds) then bars.clear() end
    return
  end
  local me = game.player()
  local target = game.target()
  local list = {}
  for index, c in pairs(casts.casts) do
    if p.moves or c.kind ~= "skill" then
      local ok, e = pcall(game.entity, index)
      if ok and e and (e.hpp or 1) > 0 and (e.distance or 0) <= RANGE
        and (not p.target_only or (target and target.index == index)) then
        list[#list + 1] = { index = index, c = c, e = e }
      end
    end
  end
  table.sort(list, function(x, y) return (x.e.distance or 0) < (y.e.distance or 0) end)
  local o = { scale = p.size * ((hud.screen().ui_scale) or 1), me = me and me.index, now = now,
    progress = function(c) return casts.progress(c, now) end }
  bars.begin()
  for i = 1, math.min(#list, MAX) do
    local it = list[i]
    bars.draw("cb" .. it.index .. "_", it.c, it.e, o)
  end
  bars.finish()
end)

local function clear_all()
  casts.clear()
  bars.clear()
end

events.on("zone_change", clear_all)
events.on("login", function() ask("cb on") end)
events.on("unload", function()
  pcall(action.send, "cb off")
  clear_all()
end)

-- A sample: a 4-second area spell aimed at you on your target (or on you), interrupted if you test again at once
local function test()
  local me = game.player()
  if not me then return end
  local target = game.target()
  local index = (target and not target.me) and target.index or me.index
  local now = os.clock()
  local c = casts.casts[index]
  if c and c.state == "casting" and c.test then
    casts.finish(index, now, "cut")
    msg("interrupted.")
    return
  end
  c = casts.begin(index, "spell", 176, now, 4, true, me.index)
  c.test = true
  timer.after(4, function()
    if casts.casts[index] == c then casts.finish(index, os.clock(), "done") end
  end)
  msg("a sample cast for 4 seconds (/castbars test again to interrupt it).")
end

local function command(sub, value)
  local p = prefs()
  sub = (sub or ""):lower()
  if sub == "" or sub == "on" or sub == "off" then
    if sub == "" then p.enabled = not p.enabled else p.enabled = sub == "on" end
    if p.enabled then ask("cb on") else ask("cb off") clear_all() end
    msg(p.enabled and "on." or "off (/castbars to turn them back on).")
  elseif sub == "target" or sub == "all" then
    p.target_only = sub == "target"
    msg(p.target_only and "your target only." or "every monster near you.")
  elseif sub == "moves" then
    p.moves = not p.moves
    msg(p.moves and "moves being readied shown." or "spells only.")
  elseif sub == "size" and tonumber(value) then
    p.size = tonumber(value)
  elseif sub == "test" then
    test()
  else
    msg("/castbars | target | all | moves | size <n> | test")
    return
  end
  storage.save()
  prefs()
end
commands.register("castbars", command)

mods.provide("timeless_menu", function()
  local p = prefs()
  return {
    title = "Cast Bars",
    desc = "Bars over casting monsters' heads: the spell, the seconds until it lands, red when it's aimed at you.",
    order = 38,
    open = "//castbars test",
    buttons = {
      { label = p.enabled and "Turn off" or "Turn on", command = "//castbars", tip = "Cast bars on or off." },
      { label = "Test", command = "//castbars test", tip = "A sample cast on your target (again to interrupt it)." },
      { label = p.target_only and "Target only" or "All nearby", command = p.target_only and "//castbars all" or "//castbars target",
        tip = "Bars for your target only, or for every monster near you." },
      { label = p.moves and "Moves: on" or "Moves: off", command = "//castbars moves", tip = "Bars for moves being readied." },
    },
  }
end)

timer.every(0.1, function()
  if prefs().enabled and os.clock() >= st.next_beat then ask("cb on") end
  if #st.queue > 0 then pcall(action.send, table.remove(st.queue, 1)) end
end)
