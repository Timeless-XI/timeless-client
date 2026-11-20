-- Timeless Champions: Champions you can see coming.
--
-- The TimelessXI server tells the client about the Champions near you (rank, affixes, and what those affixes are doing
-- right now). This mod draws:
--   Auras      a ring on the ground under each Champion and a light that follows it: pale blue for a Champion, gold
--              for an Elite, orange-red for a Legendary. It pulses red once an Enraged one flies into its rage. The
--              lights show with Settings > Night Lighting on Modern.
--   Badges     the game's icons for its affixes over its head, each with a tooltip saying what it does. A dark badge
--              is spent or stopped (Undying used, Renewing held off by poison); a red frame means it is acting now.
--   Hazards    an Alpha's rally range before you pull it, thorns round a Thorned one, a dome over a Shielded (steel) or
--              Warded (blue) one, a Venomous one's haze, a Vampiric one's drain, a Renewing one healing, shadows up.
--   Rally      lines from an Alpha to the monsters it rallied.
--   Names      a Champion's whole name on its plate and in the target window: the game's own is cut at 24 letters
--              ("Legendary Goblin Pathfi"), so the server sends the rest.
--   Variants   a Gilded (treasure) Champion glitters gold, with sparkles and the seconds before it vanishes; a
--              Nightborne one turns violet by night; a Rift-touched one has a turning rift round it until it has
--              rewound; a Juggernaut is drawn half again as big.
--
-- The server sends these only while this mod asks (a packet, not chat, renewed every 20 s); they are taken out of
-- the chat log. Needs the server's champions module.
--
--   /champs                  everything on or off
--   /champs aura|lights|badges|hazards|rally|labels|names   one part on or off
--   /champs badges target    badges for your target only (again for every Champion near you)
--   /champs opacity <0.2-1>
--   /champs test             a sample Legendary Champion on your target (or on you) for 20 seconds

local action = require("lib.action")
local protocol = require("lib.protocol")
local scene = require("lib.scene")
local A = require("lib.affixes")

local BEAT = 20
local RANGE = 60       -- yalms: Champions farther than this aren't drawn
local BADGE_RANGE = 30 -- yalms: badges for Champions other than your target

local st = require("lib.state")
st.next_beat = os.clock() + 1.5

local function prefs()
  storage.character.champions = storage.character.champions or {}
  local p = storage.character.champions
  for _, k in ipairs({ "enabled", "aura", "lights", "badges", "hazards", "rally", "labels", "names" }) do
    if p[k] == nil then p[k] = true end
  end
  if p.badges_target_only == nil then p.badges_target_only = false end
  p.opacity = math.max(0.2, math.min(1, tonumber(p.opacity) or 0.9))
  return p
end

local function ask(words)
  st.queue[#st.queue + 1] = words
  st.next_beat = os.clock() + BEAT
end

-- How Champions draw (render.entity): the whole name of one the game had to cut, a Juggernaut's size. want: id -> look
local function look_key(l) return (l.name or "") .. "|" .. tostring(l.scale or 1) end
local function rename(want)
  for id, l in pairs(st.renamed) do
    if not want[id] or look_key(want[id]) ~= look_key(l) then
      render.entity(id, nil)
      st.renamed[id] = nil
    end
  end
  for id, l in pairs(want) do
    if not st.renamed[id] then
      render.entity(id, l)
      st.renamed[id] = l
    end
  end
end

local function clear_all()
  st.champs = {}
  st.test = nil
  scene.clear()
  rename({})
end

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  local c = protocol.parse(line)
  if c then
    if c.op == "C" then
      st.champs = {}
    elseif c.op == "X" then
      st.champs[c.index] = nil
    elseif c.op == "M" then
      c.at = os.clock()
      st.champs[c.index] = c
    end
  end
  return false
end)

local function find(index)
  local me = game.player()
  if me and index == me.index then return me end
  local ok, e = pcall(game.entity, index)
  return ok and e or nil
end

local function frame()
  local p = prefs()
  local test = st.test
  if test and os.clock() > test.ends then
    st.test, test = nil, nil
  end
  if not p.enabled or (next(st.champs) == nil and not test) then
    if next(scene.drawn) or next(scene.huds) or next(scene.lights) then scene.clear() end
    if next(st.renamed) then rename({}) end
    return
  end
  local names = {}
  local t = os.clock()
  local target = game.target()
  local o = { opacity = p.opacity, lights = p.aura and p.lights, labels = p.labels, clock = t }
  scene.begin()
  local list = {}
  for index, c in pairs(st.champs) do list[#list + 1] = { index = index, c = c } end
  if test then list[#list + 1] = { index = test.c.index, c = test.c, test = true } end
  for _, item in ipairs(list) do
    local e = find(item.index)
    if e and not e.me then
      local l = {}
      if p.names and item.c.name then l.name = item.c.name end
      for k in pairs(item.c.has or {}) do
        if A.SCALE[k] and p.aura then l.scale = A.SCALE[k] end
      end
      if next(l) then names[e.id] = l end
    end
    if item.c.gilded and item.c.at then item.c.gilded_left = item.c.gilded - (t - item.c.at) end
    if e and (e.hpp or 1) > 0 and (e.distance or 0) <= RANGE then
      local key = (item.test and "t" or "c") .. item.index .. "_"
      if p.aura then scene.aura(key, item.c, e, o, t) end
      if p.hazards then scene.hazards(key, item.c, e, o, t, find) end
      if p.rally then scene.rally(key, item.c, e, o, t, find) end
      if p.aura then scene.variants(key, item.c, e, o, t) end
      local targeted = target ~= nil and target.index == e.index
      if p.badges and (targeted or (not p.badges_target_only and (e.distance or 0) <= BADGE_RANGE)) then
        scene.badges(key, item.c, e, o)
      end
    end
  end
  scene.finish()
  rename(names)
end
events.on("frame", frame)

events.on("zone_change", clear_all)
events.on("login", function() ask("ch on full") end)
events.on("unload", function()
  pcall(action.send, "ch off")
  clear_all()
end)

local function msg(s) print("Champions: " .. s) end

-- A sample Legendary on your target (or on you): fire Elemental, Thorned, Shielded, Venomous, an Alpha before its
-- pull, in a rage after 6 seconds.
local function test()
  local me = game.player()
  if not me then return end
  local target = game.target()
  local index = (target and not target.me) and target.index or me.index
  local c = protocol.parse("CH1|M|" .. index .. "|r=l,a=elemental.thorned.shielded.venomous.pack.enraged.regen,e=fire,rg=0,rn=1,pk=12"
    .. ",n=Legendary Sample Champion of the Long Name")
  st.test = { c = c, ends = os.clock() + 20 }
  timer.after(6, function()
    if st.test and st.test.c == c then
      c.raged, c.fighting, c.pack, c.regen = true, true, nil, false
    end
  end)
  msg("a sample Legendary Champion for 20 seconds" .. (index == me.index and " (on you: target a monster to put it there)." or "."))
end

local PARTS = { aura = "auras", lights = "lights", badges = "badges", hazards = "hazards", rally = "rally lines",
  labels = "labels", names = "whole names" }

local function champs_command(sub, value)
  local p = prefs()
  sub = (sub or ""):lower()
  value = (value or ""):lower()
  if sub == "" or sub == "on" or sub == "off" then
    if sub == "" then p.enabled = not p.enabled else p.enabled = sub == "on" end
    if p.enabled then
      ask("ch on full")
      msg("on: Champions' auras, badges and hazards are shown.")
    else
      ask("ch off")
      clear_all()
      msg("off (/champs to turn it back on).")
    end
  elseif sub == "badges" and (value == "target" or value == "all") then
    p.badges = true
    p.badges_target_only = value == "target"
    msg(p.badges_target_only and "badges for your target only." or "badges for every Champion near you.")
  elseif PARTS[sub] then
    p[sub] = not p[sub]
    msg(PARTS[sub] .. (p[sub] and " on." or " off."))
  elseif sub == "opacity" and tonumber(value) then
    p.opacity = tonumber(value)
  elseif sub == "test" then
    test()
  else
    msg("/champs | aura | lights | badges [target|all] | hazards | rally | labels | names | opacity <n> | test")
    return
  end
  storage.save()
  prefs()
end
commands.register("champs", champs_command)
commands.register("timelesschampions", champs_command)

mods.provide("timeless_menu", function()
  local p = prefs()
  local n = 0
  for _ in pairs(st.champs) do n = n + 1 end
  return {
    title = "Champions",
    desc = "Champions' auras, affix badges (hover for what they do), hazards and rally lines."
      .. (n > 0 and (" %d near you now."):format(n) or ""),
    order = 37,
    open = "//champs test",
    buttons = {
      { label = p.enabled and "Turn off" or "Turn on", command = "//champs", tip = "Everything on or off." },
      { label = "Test", command = "//champs test", tip = "A sample Legendary Champion on your target for 20 seconds." },
      { label = p.badges_target_only and "Badges: target" or "Badges: all",
        command = p.badges_target_only and "//champs badges all" or "//champs badges target",
        tip = "Badges for your target only, or for every Champion near you." },
      { label = p.lights and "Lights on" or "Lights off", command = "//champs lights",
        tip = "The lights that follow Champions (Night Lighting Modern)." },
    },
  }
end)

-- Every tenth of a second: send what is queued and the heartbeat while on.
timer.every(0.1, function()
  local t = os.clock()
  if prefs().enabled and t >= st.next_beat then ask("ch on") end
  if #st.queue > 0 then pcall(action.send, table.remove(st.queue, 1)) end
end)
