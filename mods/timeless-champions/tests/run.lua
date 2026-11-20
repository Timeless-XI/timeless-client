-- Timeless Champions' tests: lua tests/run.lua (from the mod folder).

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

H.mod_id = "timeless-champions"
H.player.pos = { x = 0, y = -10, z = 0 }
H.player.heading = 0
H.entities = {
  -- An Elite Champion 6 yalms east, a Legendary Alpha with two rallied wolves, and a far one
  [17000100] = { id = 17000100, index = 100, name = "Elite Crab", kind = "mob", pos = { x = 6, y = -10, z = 0 }, heading = math.pi,
    hpp = 80, hitbox = 1.2, height = 1.5, distance = 6, target_index = 1 },
  [17000101] = { id = 17000101, index = 101, name = "Legendary Wolf", kind = "mob", pos = { x = -8, y = -10, z = 4 }, hpp = 100,
    hitbox = 1, height = 1.8, distance = 9 },
  [17000102] = { id = 17000102, index = 102, name = "Wolf", kind = "mob", pos = { x = -10, y = -10, z = 6 }, hpp = 100, hitbox = 1,
    height = 1.8, distance = 11 },
  [17000103] = { id = 17000103, index = 103, name = "Wolf", kind = "mob", pos = { x = -11, y = -10, z = 2 }, hpp = 100, hitbox = 1,
    height = 1.8, distance = 12 },
  [17000104] = { id = 17000104, index = 104, name = "Far Champion", kind = "mob", pos = { x = 0, y = -10, z = 90 }, hpp = 100,
    hitbox = 1, height = 1.8, distance = 90 },
}
H.player.height, H.player.hitbox, H.player.hpp = 1.6, 0.5, 100

dofile("init.lua")
local st = require("lib.state")
local protocol = require("lib.protocol")
local A = require("lib.affixes")
local scene = require("lib.scene")

ok(H.commands.champs and H.commands.timelesschampions, "commands")
ok(H.provided["timeless-champions"].timeless_menu().title == "Champions", "a TimelessMenu card")

-- Asking: "ch on full" at login, then "ch on" every 20 s
H.run(1.6)
local s = H.take_sent()
ok(#s == 1 and s[1] == "ch on", "ch on after load: " .. table.concat(s, ","))
H.fire("login")
H.run(0.2)
s = H.take_sent()
ok(#s == 1 and s[1] == "ch on full", "ch on full at login")
H.run(20)
ok(#H.take_sent() == 1, "heartbeat every 20 s")

-- The parser
local c = protocol.parse(protocol.extract("\30\1CH1|M|100|r=e,a=elemental.enraged.vampiric,e=fire,f=1,rg=0\127"))
ok(c and c.op == "M" and c.index == 100 and c.rank == "elite" and #c.affixes == 3 and c.has.vampiric and c.element == "fire"
  and c.fighting and c.raged == false and c.undying == nil and c.pack == nil and #c.rallied == 0, "parsed an Elite line")
c = protocol.parse("CH1|M|101|r=l,a=pack.undying,ud=0,rl=102.103")
ok(c.undying == false and c.rallied[1] == 102 and c.rallied[2] == 103 and not c.fighting, "undying spent, rallied list")
ok(protocol.parse("CH1|M|101|r=z,a=pack") == nil, "an unknown rank is refused")
ok(protocol.parse("CH1|M|101|r=c,a=pack,pk=999").pack == 40, "a huge rally range is capped")
ok(protocol.parse("CH1|C").op == "C" and protocol.parse("CH1|X|7").index == 7, "C and X")
ok(protocol.parse("TG1|1|s=c") == nil, "not ours")

-- Badges: names, the element's prefix and icon, states
local b = A.badges(protocol.parse("CH1|M|1|r=e,a=elemental.regen.undying.enraged.blinking,e=ice,rn=0,ud=0,rg=1,sh=1"))
ok(#b == 5, "five badges")
ok(b[1].name == "Frost" and b[1].icon == 95 and b[1].text:find("Ice magic heals it"), "Elemental shows its element")
ok(b[2].dim and b[2].text:find("stopping it"), "Renewing stopped: dim")
ok(b[3].dim and b[3].text:find("spent"), "Undying used: dim")
ok(b[4].alert and b[4].text:find("Enraged now"), "Enraged in its rage: alert")
ok(b[5].alert and b[5].text:find("Shadows up"), "Blinking with shadows: alert")
for key, a in pairs(A.AFFIX) do
  ok(type(a.icon) == "number" and a.icon > 0 and a.text ~= "" and a.name ~= "", "affix " .. key .. " complete")
end
-- Every affix the server knows has a badge
for _, key in ipairs({ "hasted", "shielded", "warded", "frenzied", "vampiric", "venomous", "enraged", "thorned", "regen",
  "undying", "elemental", "blinking", "pack" }) do
  ok(A.AFFIX[key], "a badge for " .. key)
end

-- Lines in chat are taken out (whether on or off) and fill the list
ok(H.line("CH1|C") == false, "C taken out of chat")
ok(H.line("\30\1CH1|M|100|r=e,a=thorned.shielded.vampiric,f=1") == false, "M taken out of chat")
ok(H.line("CH1|M|101|r=l,a=pack.warded.regen,rn=1,pk=15") == false, "an Alpha before its pull")
ok(H.line("CH1|M|104|r=c,a=hasted") == false, "a far one")
ok(H.line("Hello there") == "Hello there", "other chat left alone")
ok(st.champs[100] and st.champs[101] and st.champs[104], "three Champions known")

-- A frame: auras, lights, hazards, badges for those in range
H.target = H.entities[17000100]
H.fire("frame")
ok(H.draws["c100_a_edge"] and H.draws["c100_a_disc"].filled, "the Elite's aura")
ok(H.draws["c100_a_out"] and not H.draws["c100_a_rune1"], "an Elite has the outer ring, not the turning marks")
ok(H.draws["c101_a_rune1"] and H.draws["c101_a_rune6"], "a Legendary has the turning marks")
ok(H.draws["c104_a_edge"] == nil, "nothing for one 90 yalms off")
ok(H.lights["c100_light"] and H.lights["c100_light"].entity == 17000100, "the Elite's light follows it")
ok(H.lights["c100_light"].colour[1] == 255 and H.lights["c100_light"].colour[2] == 196, "an Elite's light is gold")
ok(H.lights["c101_light"].range == 10 and H.lights["c101_light"].halo > 0, "a Legendary's light reaches farther, with a halo")
ok(H.draws["c100_h_th1"] and H.draws["c100_h_th12"], "thorns round the Thorned one")
ok(H.draws["c100_h_sh1"] and H.draws["c100_h_sh3"] and not H.draws["c100_h_wd1"], "a steel dome for Shielded")
ok(H.draws["c101_h_wd1"], "a blue dome for Warded")
ok(H.draws["c100_h_vp"] and H.draws["c100_h_vp1"], "the drain from us to the Vampiric one")
ok(H.draws["c101_h_pk"] and H.draws["c101_h_pk"].radius == 15 and H.draws["c101_h_pk_t"], "the Alpha's rally range, labelled")
ok(H.draws["c101_h_rn1"], "Renewing heals: rising rings")
ok(H.hud["c100_b1"] and H.hud["c100_b2"] and H.hud["c100_b3"], "three badges over the Elite")
ok(H.hud["c100_b1"].id == 34 and H.hud["c100_b1"].tip.title:find("Thorned") and H.hud["c100_b1"].tip.title:find("Elite"),
  "a badge's icon and tooltip")
ok(H.hud["c101_b1"], "badges for a Champion near us too")
ok(H.hud["c100_b1d"].visible == false, "an active badge isn't dimmed")

-- Badges stay put between frames (made once, then moved): the tooltip stays up
local made = H.hud["c100_b1"]
H.fire("frame")
ok(H.hud["c100_b1"] == made, "the badge was changed, not made again")

-- Lights aren't sent again every frame
local calls = H.light_calls
H.run(0.1)
H.fire("frame")
ok(H.light_calls == calls, "a steady light isn't sent again")

-- The pull: the Alpha fights, rallies two, flies into a rage; the rally range goes, the lines come
H.line("CH1|M|101|r=l,a=pack.warded.regen.enraged,f=1,rn=0,rg=1,rl=102.103")
H.fire("frame")
ok(H.draws["c101_h_pk"] == nil and H.draws["c101_h_pk_t"] == nil, "no rally range once it fights")
ok(H.draws["c101_r_l102"] and H.draws["c101_r_l103"] and H.draws["c101_r_r102"], "lines and rings to the rallied wolves")
ok(H.draws["c101_h_rn1"] == nil, "Renewing stopped: no rising rings")
local before = H.lights["c101_light"].intensity
local seen = {}
for _ = 1, 10 do
  H.run(0.1)
  H.fire("frame")
  seen[#seen + 1] = H.lights["c101_light"].intensity
end
local moved = false
for _, v in ipairs(seen) do moved = moved or math.abs(v - before) > 0.4 end
ok(moved, "an enraged Champion's light pulses")
ok(H.draws["c101_a_edge"].colour[2] < 110, "its aura turns toward red")
H.entities[17000103].hpp = 0
H.fire("frame")
ok(H.draws["c101_r_l103"] == nil and H.draws["c101_r_l102"], "a fallen wolf's line goes")

-- Badges for the target only
H.command("/champs badges target")
H.fire("frame")
ok(H.hud["c100_b1"] and H.hud["c101_b1"] == nil, "target only: no badges on the other")
H.command("/champs badges all")

-- X: gone, everything of it removed
H.line("CH1|X|100")
H.fire("frame")
ok(count(H.draws, "c100_") == 0 and count(H.hud, "c100_") == 0 and H.lights["c100_light"] == nil, "a dropped Champion leaves nothing")

-- Parts off
H.command("/champs hazards")
H.fire("frame")
ok(count(H.draws, "c101_h_") == 0 and H.draws["c101_a_edge"], "hazards off, aura stays")
H.command("/champs hazards")
H.command("/champs lights")
H.fire("frame")
ok(H.lights["c101_light"] == nil, "lights off")
H.command("/champs lights")

-- Off altogether: the server is told, everything goes
H.take_sent()
H.command("/champs")
H.run(0.1)
ok(H.take_sent()[1] == "ch off", "off tells the server")
H.fire("frame")
ok(count(H.draws) == 0 and count(H.hud) == 0 and count(H.lights) == 0, "off: nothing drawn")
H.command("/champs")
H.run(0.1)
ok(H.take_sent()[1] == "ch on full", "on asks for everything again")

-- A new zone forgets
H.line("CH1|M|101|r=l,a=pack")
H.fire("zone_change")
ok(next(st.champs) == nil, "a zone change forgets")
H.fire("frame")
ok(count(H.draws) == 0, "and draws nothing")

-- The test: a sample Legendary on the target, raging after 6 s, gone after 20 s
H.target = H.entities[17000102]
H.command("/champs test")
H.fire("frame")
ok(H.draws["t102_a_rune1"] and H.draws["t102_h_pk"] and H.draws["t102_h_th1"] and H.hud["t102_b1"], "the sample is drawn")
H.run(6.2)
H.fire("frame")
ok(st.test.c.raged and H.draws["t102_h_pk"] == nil, "it rages after 6 s")
H.run(15)
H.fire("frame")
ok(st.test == nil and count(H.draws) == 0, "gone after 20 s")
H.target = nil
H.command("/champs test")
H.fire("frame")
ok(H.draws["t1_a_edge"], "with no target, on us")

-- The menu card
local card = H.provided["timeless-champions"].timeless_menu()
ok(#card.buttons == 4 and card.open == "//champs test", "menu card buttons")

-- Unload: off and clean
H.take_sent()
H.fire("unload")
ok(H.take_sent()[1] == "ch off" and count(H.draws) == 0 and count(H.hud) == 0, "unload cleans up")

io.write(("timeless-champions: %d checks passed\n"):format(n))
