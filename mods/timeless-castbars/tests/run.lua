-- Timeless Cast Bars' tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end
local function count(t, prefix)
  local k = 0
  for id in pairs(t) do if not prefix or id:find(prefix, 1, true) == 1 then k = k + 1 end end
  return k
end

H.mod_id = "timeless-castbars"
H.player.pos = { x = 0, y = -10, z = 0 }
H.player.height, H.player.hitbox, H.player.hpp = 1.6, 0.5, 100
H.entities = {
  [17000100] = { id = 17000100, index = 100, name = "Goblin Mage", kind = "mob", pos = { x = 6, y = -10, z = 0 }, hpp = 80,
    height = 1.5, distance = 6, master = 0 },
  [17000101] = { id = 17000101, index = 101, name = "Orc Warlord", kind = "mob", pos = { x = -8, y = -10, z = 4 }, hpp = 100,
    height = 2.2, distance = 9, master = 0 },
  [17000102] = { id = 17000102, index = 102, name = "Far Mage", kind = "mob", pos = { x = 0, y = -10, z = 90 }, hpp = 100,
    height = 1.5, distance = 90, master = 0 },
  [17000103] = { id = 17000103, index = 103, name = "Carbuncle", kind = "mob", pos = { x = 1, y = -10, z = 1 }, hpp = 100,
    height = 1, distance = 1, master = 1, pet = true },
  [17000104] = { id = 17000104, index = 104, name = "Shantotto", kind = "mob", pos = { x = 2, y = -10, z = 1 }, hpp = 100,
    height = 1, distance = 2, master = 0, trust = true, in_party = true },
}
local SPELLS = {
  [144] = { id = 144, name = "Fire", element = 0, cast_time = 0.5, area = false },
  [176] = { id = 176, name = "Firaga III", element = 0, cast_time = 5, area = true },
  [149] = { id = 149, name = "Blizzard", element = 1, cast_time = 1, area = false },
  [57] = { id = 57, name = "Haste", element = 2, cast_time = 0, area = false },
}
res.spell = function(id) return SPELLS[id] end
res.mob_skill = function(id) return ({ [628] = "Frenzied Rage", [629] = "Bomb Toss" })[id] end

dofile("init.lua")
local casts = require("lib.casts")
local protocol = require("lib.protocol")

ok(H.commands.castbars, "the command")
ok(H.provided["timeless-castbars"].timeless_menu().title == "Cast Bars", "a TimelessMenu card")
H.run(1.6)
ok(H.take_sent()[1] == "cb on", "asks the server")
H.fire("login")
H.run(0.2)
ok(H.take_sent()[1] == "cb on", "and at login")

-- The parser
local l = protocol.parse(protocol.extract("\30\1CB1|100|k=m,i=176,ms=3750,t=1\127"))
ok(l and l.index == 100 and l.kind == "spell" and l.id == 176 and l.secs == 3.75 and l.target == 1, "a cast line")
l = protocol.parse("CB1|100|k=s,i=628,ms=2500,t=0")
ok(l and l.kind == "skill" and l.secs == 2.5 and l.target == nil, "a move line (no target)")
ok(protocol.parse("CB1|100|k=x").kind == "cut", "an interrupt line")
ok(protocol.parse("CB1|100|k=m,i=176,ms=0") == nil and protocol.parse("CB1|100|k=q") == nil and protocol.parse("TG1|1|s=c") == nil,
  "bad lines refused")

local function act(actor, category, param, value, target)
  H.fire("action", { actor = actor, category = category, param = param, info = 0,
    targets = { { id = target or actor, results = { { value = value or 0 } } } } })
end
local CA = 0x6163

-- A spell from the packet alone: base time
act(17000100, 8, CA, 176, H.player.id)
local c = casts.casts[100]
ok(c and c.kind == "spell" and c.id == 176 and c.secs == 5 and not c.exact and c.target == H.player.index,
  "a packet starts a bar with the spell's base time, aimed at us")
H.fire("frame")
ok(H.hud["cb100_fill"] and H.hud["cb100_name"].text == "Firaga III  > you", "the bar and its name: " .. tostring(H.hud["cb100_name"].text))
ok(H.hud["cb100_fill"].colour[1] == 255 and H.hud["cb100_fill"].colour[2] == 110, "fire's colour")
ok(H.hud["cb100_warn"].visible == true and H.hud["cb100_area"] and H.hud["cb100_area"].text == "AREA", "aimed at us, an area spell: framed red, AREA")
ok(H.hud["cb100_left"].text == "5.0", "seconds left: " .. tostring(H.hud["cb100_left"].text))

-- The server's line a moment later: the real time
H.run(0.3)
ok(H.line("CB1|100|k=m,i=176,ms=3750,t=1") == false, "the line is taken out of chat")
ok(c.exact and c.secs == 3.75 and casts.casts[100] == c, "the line makes the same bar's time exact")
H.run(1.0)
H.fire("frame")
ok(H.hud["cb100_left"].text == "2.4", "counting down by the real time: " .. H.hud["cb100_left"].text)
local w = H.hud["cb100_fill"].w
H.run(1.0)
H.fire("frame")
ok(H.hud["cb100_fill"].w > w, "the fill grows")
local made = H.hud["cb100_fill"]
H.fire("frame")
ok(H.hud["cb100_fill"] == made, "changed, not made again")

-- It lands: a flash, then gone
act(17000100, 4, 176, 0, H.player.id)
ok(c.state == "done", "category 4 finishes it")
H.fire("frame")
ok(H.hud["cb100_fill"].w == H.hud["cb100_bg"].w - 2 and H.hud["cb100_left"].text == "", "landed: full")
H.run(1.2)
H.fire("frame")
ok(casts.casts[100] == nil and count(H.hud, "cb100_") == 0, "and gone a moment later")

-- The line first, then the packet: still one bar with the exact time
H.line("CB1|101|k=m,i=149,ms=800,t=0")
act(17000101, 8, CA, 149, 17000101)
c = casts.casts[101]
ok(c and c.exact and c.secs == 0.8, "line then packet: the exact time stays")
H.fire("frame")
ok(H.hud["cb101_warn"].visible == false and H.hud["cb101_area"] == nil, "not at us, not an area: no frame")
ok(H.hud["cb101_fill"].colour[1] == 140, "ice's colour")

-- Interrupted (the packet's other param)
act(17000101, 8, 0x7073, 0, 17000101)
ok(c.state == "cut", "an interrupt from the packet")
H.fire("frame")
ok(H.hud["cb101_name"].text == "Interrupted!" and H.hud["cb101_fill"].colour[1] == 255 and H.hud["cb101_fill"].colour[2] == 70,
  "shown red")
H.run(1.2)
H.fire("frame")
ok(count(H.hud, "cb101_") == 0, "then gone")

-- Interrupted (the server's line)
H.line("CB1|101|k=m,i=149,ms=800,t=0")
H.line("CB1|101|k=x")
ok(casts.casts[101].state == "cut", "an interrupt from the server")
casts.clear()

-- A move: no time from the packet (it pulses), then the server's ready time
act(17000101, 7, CA, 628, H.player.id)
c = casts.casts[101]
ok(c and c.kind == "skill" and c.secs == nil, "a readied move from the packet: time not known")
H.fire("frame")
ok(H.hud["cb101_name"].text == "Frenzied Rage  > you" and H.hud["cb101_left"].text == "" and H.hud["cb101_fill"].colour[1] == 255
  and H.hud["cb101_fill"].colour[2] == 160, "orange, named, no seconds")
H.line("CB1|101|k=s,i=628,ms=2500,t=1")
ok(c.secs == 2.5 and c.exact, "the server gives its ready time")
act(17000101, 11, 628, 0, H.player.id)
ok(c.state == "done", "category 11 finishes it")
casts.clear()

-- Instant casts, pets, trusts and the far one get no bar
act(17000100, 8, CA, 57, H.player.id)
ok(casts.casts[100] == nil, "an instant cast: no bar")
act(17000103, 8, CA, 144, 17000100)
act(17000104, 8, CA, 144, 17000100)
ok(casts.casts[103] == nil and casts.casts[104] == nil, "not our pet, not a trust")
act(17000102, 8, CA, 144, H.player.id)
ok(casts.casts[102], "the far one is tracked")
H.fire("frame")
ok(count(H.hud, "cb102_") == 0, "but not drawn (90 yalms)")
act(1, 8, CA, 144, 17000100)
ok(casts.casts[1] == nil, "our own casts: not a monster")
casts.clear()

-- At most six, the closest
for i = 0, 7 do
  local id = 17000200 + i
  H.entities[id] = { id = id, index = 200 + i, name = "Imp", kind = "mob", pos = { x = i, y = -10, z = 3 }, hpp = 100, height = 1,
    distance = 3 + i, master = 0 }
  act(id, 8, CA, 144, H.player.id)
end
H.fire("frame")
ok(count(H.hud, "cb200_") > 0 and count(H.hud, "cb205_") > 0 and count(H.hud, "cb206_") == 0, "six bars at most, the closest")

-- Target only
H.target = H.entities[17000203]
H.command("/castbars target")
H.fire("frame")
ok(count(H.hud, "cb203_") > 0 and count(H.hud, "cb200_") == 0, "target only")
H.command("/castbars all")
H.fire("frame")
ok(count(H.hud, "cb200_") > 0, "all again")
casts.clear()

-- Moves off
act(17000101, 7, CA, 628, H.player.id)
H.command("/castbars moves")
H.fire("frame")
ok(count(H.hud, "cb101_") == 0, "moves off: no bar for a move")
H.command("/castbars moves")
H.fire("frame")
ok(count(H.hud, "cb101_") > 0, "moves on")

-- Expired: long past when it should have landed
casts.clear()
act(17000100, 8, CA, 149, H.player.id)
H.run(3.2)
H.fire("frame")
ok(casts.casts[100] == nil and count(H.hud, "cb100_") == 0, "a bar whose end never came goes by itself")

-- Off
H.take_sent()
H.command("/castbars")
H.run(0.1)
ok(H.take_sent()[1] == "cb off", "off tells the server")
act(17000100, 8, CA, 176, H.player.id)
ok(casts.casts[100] == nil, "off: nothing tracked")
H.command("/castbars")
H.run(0.1)
ok(H.take_sent()[1] == "cb on", "on asks again")

-- The test: on the target, interrupted by a second test
H.target = H.entities[17000100]
H.command("/castbars test")
H.fire("frame")
ok(H.hud["cb100_name"] and H.hud["cb100_name"].text:find("Firaga III"), "the sample cast")
H.command("/castbars test")
ok(casts.casts[100].state == "cut", "a second test interrupts it")
casts.clear()
H.target = nil
H.command("/castbars test")
ok(casts.casts[H.player.index] and casts.casts[H.player.index].test, "with no target, on us")

-- Zone change and unload
H.fire("zone_change")
ok(next(casts.casts) == nil, "a zone change forgets")
H.take_sent()
H.fire("unload")
ok(H.take_sent()[1] == "cb off" and count(H.hud) == 0, "unload cleans up")

io.write(("timeless-castbars: %d checks passed\n"):format(n))
