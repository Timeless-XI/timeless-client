-- Timeless Masterworks' tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

H.mod_id = "timeless-masterworks"
H.synth = { busy = false }
game.synth = function() return H.synth end
dofile("init.lua")
local L = require("lib.ledger")
local protocol = require("lib.protocol")

local function texts()
  local out = {}
  for _, d in ipairs(H.windows.masterworks.el.ledger.draw or {}) do if d.type == "text" then out[#out + 1] = d.text end end
  return table.concat(out, " | ")
end
local function tip_with(s)
  for _, d in ipairs(H.windows.masterworks.el.ledger.draw or {}) do
    if d.type == "tip" and ((d.text or ""):find(s, 1, true) or (d.title or ""):find(s, 1, true)) then return d end
  end
end

-- As the server's craft_book.lua sends it (craft_plus_test checks the same lines)
local STATUS = {
  "CB1|B|0|kind=status",
  "CB1|S|0|ww=0:0,sm=652:6,gs=310:3,cl=0:0,lt=0:0,bn=0:0,al=0:0,ck=455:5",
  "CB1|P|0|rested=4,max=10,gain=50,insight=3,ready=0,lesson=5,lmax=3,l=0.2.0.0.0.0.0.1,loss=10,t=25.50.100",
  "CB1|W|0|n=3,on=1,c=0.2.4,tn=Practiced.Mastered.Grandmaster,hq=10.20.30",
  "CB1|A|2|n=Brokk,c=2,s=412,r=Journeyman,nx=Artisan,ns=55,as=3,al=4,ws=10,away=0,job=0",
  "CB1|O|1|r=100,n=Iron Sword,c=2,l=40,q=1,d=0,who=Brannoc,gil=1200",
  "CB1|O|2|r=101,n=Crawler Omelet,c=8,l=35,q=1,d=2,who=Seraphine,gil=900",
  "CB1|M|100|n=Iron Sword,made=112,hq=31",
  "CB1|M|102|n=Mythril Dagger,made=61,hq=9",
  "CB1|M|101|n=Crawler Omelet,made=30,hq=4",
  "CB1|M|103|n=Bronze Sword,made=7,hq=0",
  "CB1|Z|0|kind=status",
}

ok(H.commands.masterworks and H.commands.timelessmasterworks, "commands")
ok(H.provided["timeless-masterworks"].timeless_menu().open == "//masterworks", "a TimelessMenu card")
H.run(5)
local first = H.take_sent()
ok(#first == 1 and first[1] == "cb status 1", "only the Masterwork count is asked while closed: " .. table.concat(first, ","))

-- Opening asks, and shows a waiting line
H.command("/masterworks")
ok(ui.is_open("masterworks"), "opens")
ok(texts():find("Opening your ledger", 1, true), "waiting line")
H.run(0.2)
local s = H.take_sent()
ok(#s == 1 and s[1] == "cb status 20", "asks: " .. table.concat(s, ","))

-- The answer: hidden from chat, drawn
for _, l in ipairs(STATUS) do ok(H.line(l) == false, "hidden: " .. l) end
ok(H.line("Hello") == "Hello", "chat untouched")
local t = texts()
ok(t:find("3", 1, true) and t:find("Masterworks made", 1, true), "Masterworks made: " .. t)
ok(t:find("SM", 1, true) and t:find("65.2", 1, true) and t:find("Craftsman", 1, true), "Smithing medallion: skill and rank")
ok(t:find("Iron Sword", 1, true) and t:find("Grandmaster", 1, true) and t:find("4% MW", 1, true), "Iron Sword: Grandmaster, 4% Masterwork")
ok(t:find("Mythril Dagger", 1, true) and t:find("2% MW", 1, true), "Mythril Dagger: Mastered, 2%")
ok(t:find("Practiced", 1, true) and t:find("20 to go", 1, true), "Crawler Omelet: Practiced, 20 to Mastered")
ok(t:find("Learning", 1, true) and t:find("18 to go", 1, true), "Bronze Sword: still learning")
ok(t:find("Rested  4 / 10", 1, true) and t:find("Flash of Insight: 3%", 1, true) and t:find("SM 2, CK 1", 1, true), "workbench")
ok(t:find("Guild rank: 10% fewer materials lost", 1, true), "guild rank")
ok(t:find("Iron Sword", 1, true) and t:find("1200 gil", 1, true) and t:find("★", 1, true), "orders, the HQ one starred")
ok(t:find("Brokk", 1, true) and t:find("Journeyman Smithing · 41.2", 1, true) and t:find("with you", 1, true), "apprentice")
ok(t:find("2%, or 4% at Grandmaster", 1, true), "how Masterworks happen")
local tp = tip_with("Iron Sword")
ok(tp and tp.text:find("4% chance to come out a Masterwork", 1, true), "a recipe's tip")
ok(tip_with("Brokk").text:find("Artisan at 55", 1, true), "the apprentice's tip")
H.check_draws("masterworks")

-- Parser details
local st = L.model.status
ok(#st.crafts == 8 and st.crafts[2].skill == 65.2 and st.crafts[8].rank == 5, "crafts")
local at, name, nxt = protocol.tier(st, 61)
ok(at == 2 and name == "Mastered" and nxt == 100 and protocol.chance(st, at) == 2, "tiers")
ok(protocol.feed(L.model, "CB1|R|5|n=x", 0) == true and protocol.feed(L.model, "GA1|J|1|n=0", 0) == false, "other CB1 kinds ignored")

-- Every 30 s while open; just after a synth
H.run(29)
ok(#H.take_sent() == 0, "not before 30 s")
H.run(1.5)
ok(H.take_sent()[1] == "cb status 20", "every 30 s")
H.synth.busy = true
H.run(0.5)
H.synth.busy = false
H.run(1.0)
ok(#H.take_sent() == 0, "a moment after the synth...")
H.run(1.0)
ok(H.take_sent()[1] == "cb status 20", "...it asks again")

-- An empty ledger and Masterworks off
local EMPTY = { "CB1|B|0|kind=status", "CB1|S|0|ww=0:0,sm=0:0,gs=0:0,cl=0:0,lt=0:0,bn=0:0,al=0:0,ck=0:0",
  "CB1|P|0|rested=0,max=10,gain=50,insight=3,ready=1,lesson=5,lmax=3,l=0.0.0.0.0.0.0.0,loss=0,t=25.50.100",
  "CB1|W|0|n=0,on=0,c=0.2.4,tn=Practiced.Mastered.Grandmaster,hq=10.20.30", "CB1|Z|0|kind=status" }
for _, l in ipairs(EMPTY) do H.line(l) end
t = texts()
ok(t:find("Nothing mastered yet", 1, true) and t:find("No orders today", 1, true) and t:find("None.", 1, true), "an empty ledger explains")
ok(t:find("Masterworks are off", 1, true) and t:find("Flash of Insight: ready!", 1, true), "off; insight ready")
H.check_draws("masterworks")

-- A long list ends with "more"
local long = { STATUS[1], STATUS[2], STATUS[3], STATUS[4] }
for i = 1, 20 do long[#long + 1] = ("CB1|M|%d|n=Recipe number %d,made=%d,hq=1"):format(200 + i, i, 120 - i) end
long[#long + 1] = STATUS[#STATUS]
for _, l in ipairs(long) do H.line(l) end
ok(texts():find("more.", 1, true), "a long list says how many more")
H.check_draws("masterworks")

-- Closing stops asking
H.command("/masterworks")
ok(not ui.is_open("masterworks"), "closes")
H.take_sent()
H.run(40)
ok(#H.take_sent() == 0, "nothing asked while closed")

-- High-quality and Masterwork moments
local moment = require("lib.moment")
local shaders, params, lights, cams, shots, sounds = {}, {}, {}, {}, {}, {}
render.shader = function(id, src) shaders[id] = src end
render.shader_params = function(id, p) params[id] = p end
render.light = function(id, spec) lights[id] = spec end
local camd = 1.4
render.camera_distance = function() return camd end
render.set_camera_distance = function(d) camd = d cams[#cams + 1] = d end
audio = { play = function(path, vol) sounds[#sounds + 1] = path end }
client = { screenshot = function(name) shots[#shots + 1] = name return name .. ".png" end }
H.player.index = 1
local function count_status(made)
  return { "CB1|B|0|kind=status", "CB1|W|0|n=" .. made .. ",on=1,c=0.2.4,tn=Practiced.Mastered.Grandmaster,hq=10.20.30", "CB1|Z|0|kind=status" }
end
for _, l in ipairs(count_status(3)) do H.line(l) end
ok(moment.state.base == 3, "the Masterwork count to compare with")

-- A high-quality synth that is no Masterwork: flash, chime, a check, no burst
H.take_sent()
H.fire("synth_result", 0, 16594, 1, 1)
ok(shaders.mwflash and sounds[#sounds] == "sounds/chime.wav", "a high-quality synth flashes and chimes")
H.run(0.2)
ok(params.mwflash and params.mwflash[1][3] > 0, "the flash is on, round you")
H.run(0.6)
ok(H.take_sent()[1] == "cb status 1", "and asks whether it was a Masterwork")
for _, l in ipairs(count_status(3)) do H.line(l) end
H.run(1)
ok(shaders.mwflash == nil and shaders.mwgold == nil, "the flash fades; no Masterwork, no burst")
H.fire("synth_result", 0, 16594, 1, 0)
ok(shaders.mwflash == nil, "a normal synth: nothing")
H.fire("synth_result", 1, 16594, 0, 0)
ok(shaders.mwflash == nil, "a failed one: nothing")

-- A Masterwork
H.fire("synth_result", 0, 16594, 1, 1)
H.run(0.8)
for _, l in ipairs(count_status(4)) do H.line(l) end
ok(shaders.mwgold and sounds[#sounds] == "sounds/fanfare.wav", "one more Masterwork: the golden burst and the fanfare")
ok(lights.mwglow and lights.mwglow.entity == 1, "a golden light on you")
H.run(0.5)
ok(params.mwgold and params.mwgold[1][3] > 0.9, "the burst at full")
ok(#cams > 0 and camd < 1.4 and camd >= 0.5, "the camera eases in")
H.run(1.0)
ok(#shots == 1 and shots[1]:find("^masterworks/") and shots[1]:find("Bastokan Axe", 1, true), "a screenshot: " .. tostring(shots[1]))
ok(storage.character.moments and storage.character.moments[1].item == "Bastokan Axe", "remembered for the ledger")
H.run(4)
ok(shaders.mwgold == nil and lights.mwglow == nil, "the burst fades")
ok(math.abs(camd - 1.4) < 1e-6, "the camera back where it was")
H.command("/masterworks")
H.run(0.2)
for _, l in ipairs(STATUS) do H.line(l) end
ok(texts():find("Latest Masterwork: Bastokan Axe", 1, true), "the ledger shows the latest")
ok(tip_with("screenshots/masterworks/"), "and where its screenshot is")
H.check_draws("masterworks")
H.command("/masterworks")

-- Settings off
H.settings.hq_moment, H.settings.masterwork_moment, H.settings.screenshot = false, false, false
for _, l in ipairs(count_status(4)) do H.line(l) end
local nshots = #shots
H.fire("synth_result", 0, 16594, 1, 2)
ok(shaders.mwflash == nil, "flash off")
H.run(0.8)
for _, l in ipairs(count_status(5)) do H.line(l) end
H.run(3)
ok(shaders.mwgold == nil and #shots == nshots, "burst and screenshot off")
ok(moment.safe_name("Mythril Sword +1 / x:y") == "Mythril Sword 1 xy", "a file name from the item")
for _, k in ipairs({ "chime", "fanfare" }) do
  local f = io.open(("sounds/%s.wav"):format(k), "rb")
  ok(f ~= nil, "the " .. k .. " sound is there")
  if f then f:close() end
end

io.stdout:write(("ok: %d checks\n"):format(n))
