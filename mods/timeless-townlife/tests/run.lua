-- Timeless Town Life's tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

H.mod_id = "timeless-townlife"
H.settings = {}
H.player.pos = { x = -190, y = -6, z = -78 }
H.player.zone = 235
H.entities[17000301] = { id = 17000301, index = 301, name = "Ysolde", kind = "npc", pos = { x = -186.5, y = -6, z = -80 } }

dofile("init.lua")
local protocol = require("lib.protocol")
local world = require("lib.world")
local town = require("lib.town")

-- Asks the server ("tl on") soon after loading, and every 20 s
H.run(3)
local sent = H.take_sent()
ok(#sent == 1 and sent[1] == "tl on", "asks with tl on")
H.run(10)
ok(#H.take_sent() == 0, "not again at once")
H.run(11)
ok(H.take_sent()[1] == "tl on", "and again 20 s later")

-- The snapshot, hidden from chat
local snap = {
  "TL1|C|235|Bastok Markets",
  "TL1|L|1|-140.2,-4.0,-107.5|1", "TL1|L|2|-147.6,-4.0,-103.7|1",
  "TL1|S|1|smoke|-190.0,8.0,-147.0|0", "TL1|S|2|hammer|-190.0,2.0,-146.0|28", "TL1|S|3|crowd|-276.1,-13.6,-68.1|35",
  "TL1|T|news|The caravan from Jeuno reached Bastok Markets.",
  "TL1|T|here|Gustaberg Goods: Ysolde until 6:00, then Hrodgar.",
  "TL1|T|caravan|Daily: up the South Gustaberg road from 4:00, in the Markets 7:00-10:00.",
  "TL1|T|market|Market day is every Earthsday, 6:00-20:00.",
  "TL1|T|gossip|Ysolde: Three hundred gil, Hrodgar.",
  "TL1|E",
}
for _, l in ipairs(snap) do ok(H.line(l) == false, "feed line hidden: " .. l:sub(1, 12)) end
ok(H.line("Hrodgar : Have a look.") ~= false, "ordinary chat untouched")
local m = town.model
ok(m.zone == 235 and m.zone_name == "Bastok Markets", "zone")
ok(m.lamps[1] and m.lamps[2] and m.lamps[1].x == -140.2, "lamps lit")
ok(m.scenery[2].kind == "hammer" and m.scenery[2].range == 28, "scenery")
ok(#m.rows == 5, "town rows")

-- No glow sprites floating where no lamp post stands
H.run(0.3)
ok(H.draws.lamp1 == nil and H.draws.lamp1g == nil, "no sprite glows")

-- Sounds by distance: the hammer from 28 yalms is out of range at 68 yalms away; walk closer
ok(#H.sounds == 0, "no sounds far away")
H.player.pos = { x = -190, y = 2, z = -140 }
H.run(0.5)
ok(#H.sounds == 1 and H.sounds[1].path == "sounds/hammer.wav" and H.sounds[1].volume > 0.2, "hammering by the furnaces")
H.run(2)
ok(#H.sounds == 1, "not every frame")
H.run(4)
ok(#H.sounds == 2, "again after the sound's length")
H.settings.sounds = false
H.run(10)
ok(#H.sounds == 2, "sounds setting off")
H.settings.sounds = nil

-- Sounds that come and go: the tavern's music only while the bard plays
H.sounds = {}
H.player.pos = { x = -158, y = -7.5, z = 11 }
H.line("TL1|S|7|music|-160.0,-7.5,11.0|26")
H.line("TL1|O|7|0")
H.run(1)
ok(#H.sounds == 0, "no music while the bard is away")
H.line("TL1|O|7|1")
H.run(0.5)
ok(#H.sounds == 1 and H.sounds[1].path == "sounds/music.wav", "the tavern's lute while he plays")
H.run(13)
ok(#H.sounds == 2, "played again after the tune's length")
H.line("TL1|O|7|0")
H.run(13)
ok(#H.sounds == 2, "and stops when he does")
for _, k in ipairs({ "music", "cards", "docks" }) do
  local f = io.open(("sounds/%s.wav"):format(k), "rb")
  ok(f ~= nil, "the " .. k .. " sound is there")
  if f then f:close() end
end
H.player.pos = { x = -190, y = 2, z = -140 }

-- Smoke near the furnaces
ok(H.draws.smoke1_0 == nil, "no smoke floating where no chimney stands")

-- Speech bubbles
H.player.pos = { x = -190, y = -6, z = -78 }
H.line("TL1|B|301|Go on home, Hrodgar. I've got it from here.")
H.run(0.2)
ok(H.hud.tlb_301 and H.hud.tlb_301.kind == "bubble", "a speech bubble over Ysolde")
ok(H.hud.tlbt_301.text:find("Go on home", 1, true), "with what she said")
H.run(10)
ok(H.hud.tlb_301 == nil, "and it goes")
H.line("TL1|B|999|Nobody here.")
H.run(0.2)
ok(H.hud.tlb_999 == nil, "no bubble for someone out of sight")

-- The Town window
H.take_sent()
H.command("/town")
ok(ui.is_open("town"), "/town opens the window")
H.run(0.2)
ok(H.take_sent()[1] == "tl town", "and asks for fresh rows")
H.check_draws("town")
local texts = {}
for _, d in ipairs(H.windows.town.el.town.draw) do if d.type == "text" then texts[#texts + 1] = d.text end end
local all = table.concat(texts, " | ")
ok(all:find("Bastok Markets", 1, true) and all:find("Headlines", 1, true) and all:find("Gossip", 1, true), "sections")
ok(all:find("Ysolde until 6:00", 1, true), "who has the stall")
H.click("town", "town", "refresh")
H.run(0.2)
ok(H.take_sent()[1] == "tl town", "Refresh asks again")
H.command("/town")
ok(not ui.is_open("town"), "/town closes it")

-- Real lights (TimelessXI's Night Lighting Modern): the lamp posts on his round, the lantern and torches
local L, Z = {}, {}
render.light = function(id, spec) L[id] = spec end
render.lamp_zone = function(id, spec) Z[id] = spec end
local modern = true
render.night = function() return { modern = modern, amount = 1, darkness = 0.6 } end
-- The zone's lamp posts: one 3 yalms from lamp 1, none near lamp 2
render.lamps = function() return { { x = -142.0, y = -7.0, z = -105.0, level = 1 }, { x = -60.0, y = -7.0, z = -60.0, level = 1 } } end
H.line("TL1|L|3|-320.6,-15.0,-89.0|1")
H.line("TL1|A|301|lantern")
H.line("TL1|A|302|torch")
H.run(0.3)
ok(Z.tlz1 and Z.tlz1.x == -142.0 and Z.tlz1.level == 1, "a lit lamp is the lamp post by it, lit")
ok(Z.tlz3 == nil, "a spot with no lamp post near it shows nothing")
ok(next(L) and L.tlc301 and L.tlc301.entity == 301 and L.tlc301.night_only, "the lamplighter's lantern")
ok(L.tlc302 and L.tlc302.range > L.tlc301.range, "a guard's torch")
H.line("TL1|A|303|candle")
H.run(0.3)
ok(L.tlc303 and L.tlc303.range < L.tlc301.range and L.tlc303.night_only, "the card table's candle")
H.line("TL1|A|303|-")
ok(L.tll1 == nil and L.tll3 == nil, "no lights floating at the lamp spots")
H.line("TL1|L|1|-140.2,-4.0,-107.5|0")
H.run(0.3)
ok(Z.tlz1.level == 0, "put out: the lamp post goes dark")
H.line("TL1|L|1|-140.2,-4.0,-107.5|1")
H.run(0.3)
ok(Z.tlz1.level == 1 and Z.tlz1.fade > 0, "lit again: it fades on")
H.line("TL1|A|301|-")
H.run(0.3)
ok(L.tlc301 == nil, "his lantern goes home with him")
modern = false
H.run(0.3)
ok(next(L) == nil and next(Z) == nil, "Original lighting: the game's own lamps")
modern = true
H.run(0.3)

-- A new zone: everything forgotten, a fresh snapshot asked for
H.fire("zone_change")
H.run(0.3)
ok(H.draws.lamp1 == nil and #town.model.rows == 0, "zone change clears the town")
ok(next(L) == nil and next(Z) == nil, "and its lights")
H.run(3)
ok(H.take_sent()[1] == "tl on", "and asks again in the new zone")

-- Menu card
local card = H.provided["timeless-townlife"].timeless_menu()
ok(card.title == "Town" and card.open == "//town", "a card on the Timeless menu")

-- Unload
H.fire("unload")
ok(H.take_sent()[1] == "tl off", "tl off on unload")

-- The protocol on its own
local pm = protocol.new_model()
ok(protocol.feed(pm, "TL1|L|x|bad|1", 0) == true, "a bad lamp line is ignored")
ok(protocol.feed(pm, "WM1|E|x", 0) == false, "someone else's line")
ok(protocol.extract("[3] TL1|E\n") == "TL1|E", "extract")
ok(#world.wrap(string.rep("word ", 60), 34) == 3, "bubbles wrap to 3 lines")

io.write(("%d checks passed\n"):format(n))
