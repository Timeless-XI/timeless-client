-- Timeless Gauges' tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

H.mod_id = "timeless-gauges"
dofile("init.lua")
local G = require("lib.gauges")
local protocol = require("lib.protocol")

local function texts()
  local out = {}
  for _, d in ipairs(H.windows.gauges.el.bar.draw or {}) do if d.type == "text" then out[#out + 1] = d.text end end
  return table.concat(out, " | ")
end
local function count(kind)
  local k = 0
  for _, d in ipairs(H.windows.gauges.el.bar.draw or {}) do if d.type == kind then k = k + 1 end end
  return k
end

ok(H.commands.gauges and H.commands.timelessgauges, "commands")
ok(H.windows.gauges.spec.edge == true, "an edge window")
ok(not ui.is_open("gauges"), "hidden until there are gauges")

-- Load: the full ask a second later, then the heartbeat every 20 s
H.run(0.5)
ok(#H.take_sent() == 0, "nothing in the first half second")
H.run(1)
local s = H.take_sent()
ok(#s == 1 and s[1] == "ga on full", "full ask after load: " .. table.concat(s, ","))
H.run(19)
ok(#H.take_sent() == 0, "no heartbeat before 20 s")
H.run(1.5)
s = H.take_sent()
ok(#s == 1 and s[1] == "ga on", "heartbeat at 20 s")

-- The feed: hidden from chat, fills the model and shows the bar
local now = H.clock
ok(H.line("GA1|J|3|n=1") == false, "J hidden")
ok(H.line("\30\1GA1|G|1|k=faith,l=Faith,t=bar,v=40,m=100,h=0,s=,c=gold,tot=1,left=0\127") == false, "G hidden (colour codes)")
ok(H.line("Hello") == "Hello", "chat untouched")
H.run(0.1)
ok(ui.is_open("gauges"), "bar shown with gauges")
ok(texts():find("White Mage", 1, true) and texts():find("Faith", 1, true) and texts():find("40 / 100", 1, true), "bar draws: " .. texts())
H.check_draws("gauges")
ok(not G.animated(), "a bar not full doesn't animate")

-- Full: glows (redraws often), with a tooltip
H.line("GA1|G|1|k=faith,l=Faith,t=bar,v=100,m=100,h=1,s=Miracle!,c=gold,tot=1,left=0")
H.run(0.1)
ok(G.animated() and texts():find("Miracle!", 1, true), "full gauge glows")
local tips = 0
for _, d in ipairs(H.windows.gauges.el.bar.draw) do if d.type == "tip" and d.text:find("Miracle", 1, true) then tips = tips + 1 end end
ok(tips == 1, "tooltip explains it")
local first = H.windows.gauges.el.bar.draw
H.run(0.3)
ok(H.windows.gauges.el.bar.draw ~= first, "redrawn while glowing")

-- A job with timers, pips and a cooldown
H.line("GA1|J|16|n=3")
H.line("GA1|G|1|k=ess,l=Essence,t=bar,v=10,m=100,h=0,s=,c=purple,tot=1,left=0")
H.line("GA1|G|2|k=weave,l=Weave,t=pips,v=2,m=3,h=0,s=,c=teal,tot=30,left=24")
H.line("GA1|G|3|k=azure,l=Azure Chain,t=timer,v=0,m=1,h=1,s=Burst!,c=blue,tot=8,left=6")
H.run(0.1)
ok(texts():find("Blue Mage", 1, true) and texts():find("24s", 1, true) and texts():find("Burst!  6s", 1, true), "pips and timer: " .. texts())
H.check_draws("gauges")
H.run(3)
ok(texts():find("Burst!  3s", 1, true) and texts():find("21s", 1, true), "timers count down here")
H.run(4)
ok(not texts():find("Burst!", 1, true), "timer ran out")

-- A job without gauges hides the bar
H.line("GA1|J|7|n=0")
H.run(0.1)
ok(not ui.is_open("gauges"), "no gauges: hidden")

-- Commands
H.line("GA1|J|1|n=1")
H.line("GA1|G|1|k=never,l=Never Yield,t=cd,v=0,m=1,h=0,s=,c=red,tot=300,left=65")
H.run(0.1)
ok(texts():find("1:05", 1, true), "cooldown clock")
H.command("/gauges labels")
ok(storage.character.gauges.labels == false and H.windows.gauges.el.bar.w == 170, "labels off: narrower canvas")
ok(not texts():find("Never Yield", 1, true), "no label drawn")
H.command("/gauges width 260")
ok(H.windows.gauges.el.bar.w == 260, "width")
H.check_draws("gauges")
H.command("/gauges width 999")
ok(storage.character.gauges.width == 400, "width clamped")
H.command("/gauges labels")
H.command("/gauges job")
ok(not texts():find("Warrior", 1, true), "job name hidden")
H.command("/gauges opacity 0.5")
ok(H.windows.gauges.opacity == 0.5, "opacity")
H.command("/gauges")
ok(not ui.is_open("gauges") and storage.character.gauges.visible == false, "/gauges hides it")
H.take_sent()
H.command("/gauges")
H.run(0.1)
ok(ui.is_open("gauges"), "/gauges shows it again")
ok(H.take_sent()[1] == "ga on full", "showing asks for everything")

-- Demo: sample gauges, real feed kept underneath
H.command("/gauges job")
H.command("/gauges demo")
H.run(0.1)
ok(G.demo and texts():find("(demo)", 1, true) and texts():find("Essence", 1, true), "demo shown")
H.line("GA1|G|1|k=never,l=Never Yield,t=cd,v=0,m=1,h=0,s=,c=red,tot=300,left=0")
ok(G.demo.gauges[1].left == 0, "the feed goes into the real model under the demo")
H.command("/gauges demo")
H.run(0.1)
ok(not G.demo and texts():find("Ready", 1, true), "back to own gauges")

-- Zoning asks for everything 3 s later
H.take_sent()
H.fire("zone_change", 1, 2)
H.run(3.5)
ok(H.take_sent()[1] == "ga on full", "zoning asks again")

-- Unloading tells the server to stop
H.fire("unload")
ok(H.take_sent()[1] == "ga off", "unload says ga off")

-- TimelessMenu card
local card = mods.call("timeless-gauges", "timeless_menu")
ok(card.title == "Job Gauges" and card.open == "//gauges" and #card.buttons == 3, "menu card")

io.stdout:write(("ok: %d checks\n"):format(n))
