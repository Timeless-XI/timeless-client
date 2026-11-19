-- Timeless Map's tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

H.mod_id = "timeless-map"
local ZONES = { [100] = "West Ronfaure", [103] = "Valkurm Dunes", [107] = "South Gustaberg", [25] = "Misareaux Coast",
  [51] = "Wajaom Woodlands", [126] = "Qufim Island" }
res.zone = function(id) return ZONES[id] end
res.day = function(d) return ({ [0] = "Firesday", [1] = "Earthsday" })[d] end
res.moon = function(m) return ({ [3] = "Full Moon", [7] = "New Moon", [1] = "Waxing Crescent" })[m] end
H.time = { weekday = 0, moon_phase = 1, moon = 30 }
game.time = function() return H.time end
H.player.pos = { x = 0, y = -10, z = 0 }

dofile("init.lua")
local M = require("lib.worldmap")
local protocol = require("lib.protocol")
local beams = require("lib.beams")

local function texts()
  local out = {}
  for _, d in ipairs(H.windows.worldmap.el.map.draw or {}) do if d.type == "text" then out[#out + 1] = d.text end end
  return table.concat(out, " | ")
end
local function tip_with(s)
  for _, d in ipairs(H.windows.worldmap.el.map.draw or {}) do
    if d.type == "tip" and (d.text:find(s, 1, true) or d.title:find(s, 1, true)) then return d end
  end
end

ok(H.commands.worldmap and H.commands.wmap, "commands")
ok(H.windows.worldmap and not H.windows.worldmap.spec.edge, "a normal window")
ok(not ui.is_open("worldmap"), "closed at first")
ok(H.provided["timeless-map"].timeless_menu().open == "//worldmap", "a TimelessMenu card")

-- Asking: full a second after load, "still here" every 30 s
H.run(1.5)
local s = H.take_sent()
ok(#s == 1 and s[1] == "wm on full", "full ask after load: " .. table.concat(s, ","))
H.run(25)
ok(#H.take_sent() == 0, "no heartbeat before 30 s")
H.run(6)
s = H.take_sent()
ok(#s == 1 and s[1] == "wm on", "heartbeat at 30 s")

-- The feed (as the server's own test sends it)
local SNAP = {
  "WM1|B|v=4,n=6,o=0301000000000000003",
  "WM1|E|103|r=1,k=R,c=2,m=30",
  "WM1|E|100|r=0,k=W,l=45",
  "WM1|E|25|r=18,k=F",
  "WM1|E|100|r=0,k=C,l=52,c=3",
  "WM1|E|107|r=3,k=C,l=34,c=2",
  "WM1|E|51|r=28,k=C,l=70,c=1",
  "WM1|D|v=4",
}
for _, line in ipairs(SNAP) do ok(H.line(line) == false, "hidden from chat: " .. line) end
ok(H.line("Just chatting") == "Just chatting", "chat untouched")
ok(M.model.version == 4 and #M.model.events == 6, "the snapshot is in")
ok(M.model.owners[0] == 0 and M.model.owners[1] == 3 and M.model.owners[3] == 1 and M.model.owners[18] == 3, "owners by region")
ok(#H.printed == 0, "the first snapshot after loading raises no alerts")

-- A partial snapshot (B without D) changes nothing
H.line("WM1|B|v=5,n=1,o=0000000000000000000")
H.line("WM1|E|126|r=10,k=R,c=1,m=20")
ok(M.model.version == 4 and #M.model.events == 6, "nothing replaced until D")

-- D: a new rift in Qufim: an alert names it
H.line("WM1|D|v=5")
ok(M.model.version == 5 and #M.model.events == 1, "the new snapshot replaces the old")
ok(H.printed[1] == "World map: Time Rift in Qufim Island (closes in 20 min) (Qufim Island).", "alert: " .. tostring(H.printed[1]))
H.printed = {}
for _, line in ipairs(SNAP) do H.line(line) end
local alerts = table.concat(H.printed, "\n")
ok(alerts:find("Ancient Rift in Valkurm Dunes", 1, true) and alerts:find("warband", 1, true) and alerts:find("Fomor band", 1, true)
  and alerts:find("Legendary Champion (Lv. 52) in West Ronfaure", 1, true), "alerts for rifts, warbands, Fomors, Legendaries")
ok(not alerts:find("Elite Champion", 1, true) and not alerts:find("(Lv. 70)", 1, true), "no alert for ordinary Champions")
H.printed = {}

-- Counting a rift down
local rift = M.model.events[1]
ok(protocol.minutes_left(rift, H.clock) == 30, "30 minutes left")
ok(protocol.minutes_left(rift, H.clock + 61) == 29, "a minute later, 29")

-- The window
H.command("/worldmap")
ok(ui.is_open("worldmap"), "/worldmap opens it")
local t = texts()
ok(t:find("Vana'diel now", 1, true) and t:find("Waxing Crescent 30%", 1, true) and t:find("Firesday", 1, true), "moon and day: " .. t)
ok(t:find("Everywhere", 1, true) and t:find("Ancient Rift", 1, true) and t:find("Valkurm Dunes", 1, true) and t:find("30m", 1, true), "the list")
ok(t:find("Zulkheim", 1, true) and t:find("Tavnazia", 1, true) and t:find("San d'Oria", 1, true), "regions and cities on the map")
ok(t:find("Aht Urhgan (1)", 1, true), "an off-map group with its count")
local zt = tip_with("Zulkheim")
ok(zt and zt.text:find("Held by the beastmen", 1, true) and zt.text:find("Ancient Rift in Valkurm Dunes", 1, true), "a region's tip: owner and events")
H.check_draws("worldmap")

-- Clicking Ronfaure lists only its events
H.click("worldmap", "map", "r0")
t = texts()
ok(M.selected == 0, "Ronfaure selected")
ok(t:find("Held by San d'Oria", 1, true) and t:find("Warband", 1, true) and t:find("Legendary", 1, true), "Ronfaure's events: " .. t)
ok(not t:find("Valkurm Dunes", 1, true), "not other regions' (in the panel)")
H.click("worldmap", "map", "all")
ok(M.selected == nil, "All goes back to everywhere")
H.click("worldmap", "map", "r1")
H.click("worldmap", "map", "r1")
ok(M.selected == nil, "clicking a selected region again clears it")
H.click("worldmap", "map", "g1")
ok(M.selected == "g1" and texts():find("Wajaom Woodlands", 1, true), "a group lists its events")
H.click("worldmap", "map", "g1")
ok(M.selected == nil, "clicking it again goes back")

-- Full and new moon notes
H.time = { weekday = 1, moon_phase = 3, moon = 96 }
M.render()
ok(texts():find("Full moon: the beastmen rally.", 1, true), "full moon note")
H.time = { weekday = 1, moon_phase = 7, moon = 4 }
M.render()
ok(texts():find("New moon: Fomors stir", 1, true), "new moon note")

-- Many events: the list says how many more
local lines = { "WM1|B|v=9,n=30,o=0000000000000000000" }
for i = 1, 30 do lines[#lines + 1] = ("WM1|E|%d|r=0,k=C,l=%d,c=1"):format(100, i) end
lines[#lines + 1] = "WM1|D|v=9"
for _, l in ipairs(lines) do H.line(l) end
H.run(0.1)
ok(texts():find("more.", 1, true), "a long list ends with how many more")
H.check_draws("worldmap")
for _, line in ipairs(SNAP) do H.line(line) end

-- Your zone: pillars of light
H.line("WM1|H|n=2")
H.line("WM1|P|k=R,i=901,x=40,y=-10,z=30,l=0,c=2")
H.line("WM1|P|k=C,i=500,x=5,y=-10,z=2,l=34,c=3")
H.run(0.3)
ok(H.draws.beam1 and H.draws.beam1.kind == "line" and H.draws.beam1.to.y < H.draws.beam1.from.y, "a pillar rises over the rift (y down)")
ok(H.draws.beam1t and H.draws.beam1t.text == "Ancient Rift  50y", "labelled with what and how far: " .. tostring(H.draws.beam1t and H.draws.beam1t.text))
ok(H.draws.beam2 == nil, "no pillar over something right next to you")
-- The rift's entity comes into sight: its live position is used
H.entities[17000901] = { id = 17000901, index = 901, name = "Ancient Rift", kind = "npc", pos = { x = 60, y = -10, z = 0 } }
H.run(0.3)
ok(H.draws.beam1.from.x == 60 and H.draws.beam1t.text == "Ancient Rift  60y", "the live position, once in sight")
-- Walk away from the Champion: it gets a pillar too
H.player.pos = { x = -30, y = -10, z = 0 }
H.run(0.3)
ok(H.draws.beam2 and H.draws.beam2t.text:find("Legendary Champion (Lv. 34)", 1, true), "a pillar over the Legendary")
H.command("/worldmap beams")
H.run(0.1)
ok(H.draws.beam1 == nil and H.draws.beam2t == nil, "beams off: all gone")
H.command("/worldmap beams")
H.run(0.3)
ok(H.draws.beam1 ~= nil, "back on")
H.line("WM1|H|n=0")
H.run(0.1)
ok(H.draws.beam1 == nil and H.draws.beam2 == nil, "an empty list takes them down")

-- Alerts off
H.command("/worldmap alerts")
H.printed = {}
H.line("WM1|B|v=12,n=1,o=0000000000000000000")
H.line("WM1|E|126|r=10,k=W,l=40")
H.line("WM1|D|v=12")
ok(#H.printed == 0, "alerts off: no chat line")
H.command("/worldmap alerts")

-- Closing; zoning; unloading
H.command("/worldmap")
ok(not ui.is_open("worldmap"), "/worldmap closes it")
H.take_sent()
H.line("WM1|H|n=1")
H.line("WM1|P|k=R,i=901,x=40,y=-10,z=30,l=0,c=1")
H.run(0.3)
H.fire("zone_change")
ok(#M.model.here == 0 and H.draws.beam1 == nil, "zoning takes the pillars down")
H.run(3.2)
ok(H.take_sent()[1] == "wm on full", "and asks again after zoning")
H.fire("unload")
ok(H.take_sent()[1] == "wm off", "unload: wm off")

io.stdout:write(("ok: %d checks\n"):format(n))
