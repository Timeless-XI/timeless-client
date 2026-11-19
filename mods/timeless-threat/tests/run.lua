-- Timeless Threat's tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

H.mod_id = "timeless-threat"
H.entities = {
  [17000100] = { id = 17000100, index = 100, name = "Leaping Lizzy", kind = "mob" },
  [17000300] = { id = 17000300, index = 300, name = "Goblin Pathfinder", kind = "mob" },
  [1002] = { id = 1002, index = 1792, name = "Brokk", kind = "player", trust = true, master = 1 },
  [1003] = { id = 1003, index = 1800, name = "Carbuncle", kind = "mob", master = 1 },
}
H.party = { { index = 1, name = "Raven" }, { index = 55, name = "Faraway" } }

dofile("init.lua")
local T = require("lib.threat")
local protocol = require("lib.protocol")

local function texts()
  local out = {}
  for _, d in ipairs(H.windows.threat.el.list.draw or {}) do if d.type == "text" then out[#out + 1] = d.text end end
  return table.concat(out, " | ")
end

ok(H.commands.threat and H.commands.timelessthreat, "commands")
ok(H.windows.threat.spec.edge == true, "an edge window")
ok(not ui.is_open("threat"), "hidden out of combat")
ok(H.provided["timeless-threat"].timeless_menu().open == "//threat", "a TimelessMenu card")

-- Asking: the full ask a second after load, then a heartbeat every 20 s
H.run(1.5)
local s = H.take_sent()
ok(#s == 1 and s[1] == "en on full", "full ask after load: " .. table.concat(s, ","))
H.run(20)
s = H.take_sent()
ok(#s == 1 and s[1] == "en on", "heartbeat")

-- The parser on its own
local m = protocol.new_model()
protocol.feed(m, protocol.extract("\30\1EN1|M|100|h=64,t=1,me=1\127"), 1)
protocol.feed(m, "EN1|L|100|1:1500:2000;1792:900:300;1800:50:0", 1)
ok(m.mobs[100].hpp == 64 and m.mobs[100].holder == 1 and m.mobs[100].mine, "M parsed (colour codes stripped)")
ok(#m.mobs[100].rows == 3 and m.mobs[100].rows[2].index == 1792 and m.mobs[100].rows[2].ve == 300, "L parsed")
ok(protocol.percent(m.mobs[100], m.mobs[100].rows[2]) == 34, "share of the holder: 1200 of 3500 = 34%")
protocol.feed(m, "EN1|X|100", 2)
ok(m.mobs[100] == nil, "X drops it")
protocol.feed(m, "EN1|M|5|h=1,t=0,me=0", 2)
protocol.feed(m, "EN1|C", 3)
ok(next(m.mobs) == nil, "C clears")
ok(protocol.feed(m, "EN1|Q|odd", 3) == true and protocol.feed(m, "GA1|J|1|n=0", 3) == false, "unknown kinds ignored, other feeds not ours")

-- The feed: hidden from chat, shows the window
ok(H.line("EN1|C") == false, "C hidden")
ok(H.line("EN1|M|100|h=64,t=1,me=1") == false, "M hidden")
ok(H.line("EN1|L|100|1:1500:2000;1792:900:300;1800:50:0") == false, "L hidden")
ok(H.line("Hello there") == "Hello there", "chat untouched")
H.run(0.1)
ok(ui.is_open("threat"), "shown in a fight")
local t = texts()
ok(t:find("Leaping Lizzy", 1, true) and t:find("Raven", 1, true) and t:find("Brokk", 1, true) and t:find("Carbuncle", 1, true), "names from the game: " .. t)
ok(t:find("100%", 1, true) and t:find("34%", 1, true) and t:find("64%", 1, true), "shares and HP: " .. t)
ok(t:find("►", 1, true), "the holder is marked")
H.check_draws("threat")
local tips = 0
for _, d in ipairs(H.windows.threat.el.list.draw) do
  if d.type == "tip" and d.text:find("Cumulative 900 + volatile 300 = 1200", 1, true) then tips = tips + 1 end
end
ok(tips == 1, "pointing at a bar shows the exact numbers")
ok(not T.animated() and H.hud.threat_warn == nil, "no warning while you hold it with a lead")

-- Brokk closes in on the tank (you): the header warns, no screen warning (you're not pulling)
H.line("EN1|L|100|1:1500:2000;1792:1500:1800;1800:50:0")
H.run(0.1)
ok(texts():find("Brokk 94%", 1, true), "losing hate: the header names who is close: " .. texts())
ok(H.hud.threat_warn == nil, "no screen warning for the holder")

-- Now Brokk holds it and you're at 95%: your bar pulses, the screen warns
H.line("EN1|M|100|h=60,t=1792,me=1")
H.line("EN1|L|100|1792:2000:2000;1:1800:2000")
H.run(0.1)
ok(T.animated(), "your bar pulses")
ok(H.hud.threat_warn and H.hud.threat_warn.text == "Careful: 95% of Brokk's enmity", "the screen warns: " .. tostring(H.hud.threat_warn and H.hud.threat_warn.text))
H.command("/threat warn")
ok(H.hud.threat_warn == nil, "/threat warn turns it off")
H.command("/threat warn")

-- A second monster, an add attacking a far-off party member; your target stays first
H.line("EN1|M|300|h=100,t=55,me=0")
H.line("EN1|L|300|55:100:50")
H.run(0.1)
t = texts()
ok(t:find("Goblin Pathfinder", 1, true) and t:find("Faraway", 1, true), "names from the party list when out of sight: " .. t)
ok(t:find("Leaping Lizzy", 1, true) < t:find("Goblin Pathfinder", 1, true), "your target first")
H.command("/threat compact")
ok(not texts():find("Goblin", 1, true), "compact: only your target")
H.command("/threat compact")
H.command("/threat rows 2")
ok(T.prefs().rows == 2, "rows")
H.command("/threat rows 99")
ok(T.prefs().rows == 7, "rows clamped")
H.command("/threat width 50")
ok(T.prefs().width == 160 and H.windows.threat.el.list.w == 160, "width clamped and the window rebuilt")
H.check_draws("threat")

-- Long lists and long names stay inside the canvas
H.entities[1004] = { id = 1004, index = 1900, name = "Averyveryverylongcompanionname", kind = "player", trust = true, master = 1 }
H.line("EN1|L|100|1900:30000:30000;1792:29000:30000;1:20000:100;1800:5:0;7:1:1;8:1:1;9:1:1")
H.run(0.1)
H.check_draws("threat")

-- Out of combat: X for both; the window hides
H.line("EN1|X|100")
H.line("EN1|X|300")
H.run(0.1)
ok(not ui.is_open("threat"), "hides when the fight is over")

-- The demo
H.command("/threat demo")
ok(ui.is_open("threat") and texts():find("(demo)", 1, true) and texts():find("Kupipi", 1, true), "the demo shows a sample fight")
ok(H.hud.threat_warn ~= nil, "the demo shows the warning too")
H.line("EN1|M|100|h=50,t=1,me=1")
ok(T.demo.mobs[100] ~= nil and T.model.mobs[100] == nil, "lines during the demo go to the real model")
H.command("/threat demo")
ok(T.demo == nil and T.model.mobs[100] ~= nil, "back from the demo")

-- Zoning forgets the fights and asks again
H.take_sent()
H.fire("zone_change")
ok(next(T.model.mobs) == nil, "zoning forgets")
H.run(3.2)
s = H.take_sent()
ok(#s == 1 and s[1] == "en on full", "and asks for everything after zoning")

-- Unloading tells the server
H.fire("unload")
s = H.take_sent()
ok(#s == 1 and s[1] == "en off", "unload: en off")

io.stdout:write(("ok: %d checks\n"):format(n))
