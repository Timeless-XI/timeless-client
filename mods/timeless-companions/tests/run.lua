-- Loads the mod in the stand-in API, feeds it the server's lines, and works both windows.
-- Run from the mod folder: lua tests/run.lua [dump-folder]

local H = dofile("tests/harness.lua")
mod = { id = "timeless-companions", job_icons = true } -- the Timeless fork's client
local OUT = arg and arg[1]
local passed = 0
local function check(cond, what)
  assert(cond, what)
  passed = passed + 1
end

require("init")
local link = require("lib.link")
local view = require("lib.view")
local manager = require("lib.manager")

-- Lines as the server prints them (companions_core.lua sheetLines, companions_manage.lua lines).
local CV = {
  "CV1|B|0|count=3",
  "CV1|M|0|th=1,nuke=0,wardps=0,plddps=0,drktank=0",
  "CV1|H|1801|name=Brokk,mjob=7,mlv=75,sjob=3,slv=37,race=8,tank=1,kept=1,bond=Trusted",
  "CV1|G|1801|main=16594,sub=12345,ranged=0,ammo=0,head=12512,neck=13056,ear1=0,ear2=0",
  "CV1|G|1801|body=12554,hands=0,ring1=13495,ring2=0,back=0,waist=0,legs=12938,feet=0",
  "CV1|S|1801|hp=1180,mhp=1420,mp=210,mmp=380,tp=1250,att=301,def=389,acc=288,eva=241,ratt=0,racc=0",
  "CV1|A|1801|str=72+9,dex=66+4,vit=74-2,agi=58+0,int=50+0,mnd=60+3,chr=62+0",
  "CV1|H|1802|name=Shantotto Junior,mjob=6,mlv=75,sjob=13,slv=37,race=5,tank=0,kept=0,bond=",
  "CV1|G|1802|main=16594,sub=0,ranged=0,ammo=0,head=0,neck=0,ear1=0,ear2=0",
  "CV1|G|1802|body=0,hands=0,ring1=0,ring2=0,back=0,waist=0,legs=0,feet=0",
  "CV1|S|1802|hp=840,mhp=840,mp=0,mmp=0,tp=300,att=320,def=260,acc=310,eva=300,ratt=120,racc=150",
  "CV1|A|1802|str=60+2,dex=70+12,vit=55+0,agi=72+8,int=50+0,mnd=48+0,chr=50+0",
  "CV1|H|1803|name=Mira,mjob=3,mlv=75,sjob=5,slv=37,race=7,tank=0,kept=1,bond=Sworn",
  "CV1|G|1803|main=0,sub=0,ranged=0,ammo=0,head=0,neck=0,ear1=0,ear2=0",
  "CV1|G|1803|body=0,hands=0,ring1=0,ring2=0,back=0,waist=0,legs=0,feet=0",
  "CV1|S|1803|hp=700,mhp=760,mp=900,mmp=1100,tp=0,att=150,def=230,acc=180,eva=200,ratt=0,racc=0",
  "CV1|A|1803|str=50+0,dex=52+0,vit=55+0,agi=50+0,int=60+0,mnd=80+15,chr=70+0",
  "CV1|Z|0|count=3",
}

local CMD = { "war", "mnk", "whm", "blm", "rdm", "thf", "pld", "drk", "bst", "brd", "rng", "sam", "nin", "drg", "smn",
  "blu", "cor", "pup", "dnc", "sch", "geo", "run" }
local ROLE = { [7] = "TANK", [13] = "TANK", [3] = "HEALER", [5] = "HEALER", [20] = "HEALER", [10] = "SUPPORT",
  [15] = "SUPPORT", [17] = "SUPPORT" }

local function cm_lines()
  local l = {
    "CM1|B|0|count=3",
    "CM1|L|0|party=4,partymax=6,keptmax=8,presetmax=5,presetjobs=5,cd=900,downed=300",
    "CM1|C|0|can=1,battle=0,raise=1,reason=",
    "CM1|M|0|th=1,nuke=0,chat=1,wardps=0,plddps=0,drktank=1,engage=attack",
  }
  for j = 1, 22 do
    local wait = (j == 4) and 300 or ((j == 8) and 65 or 0)
    local party = (j == 7 or j == 6 or j == 3) and 1 or 0
    l[#l + 1] = ("CM1|J|%d|cmd=%s,role=%s,wait=%d,sub=%d,party=%d,egp=%d"):format(j, CMD[j], ROLE[j] or "DPS", wait,
      j == 6 and 13 or 0, party, j == 4 and 5 or 0)
  end
  l[#l + 1] = "CM1|X|3|interrupt=on,dispel=off,weakness=on,sleep=on,aoe=never,range=far,mp=20,talk=on"
  l[#l + 1] = "CM1|P|1801|name=Brokk,job=7,lv=75,sjob=3,slv=37,hpp=83,mpp=55,kept=1,bond=Trusted,pe=Stalwart"
  l[#l + 1] = "CM1|P|1802|name=Shantotto Junior,job=6,lv=75,sjob=13,slv=37,hpp=100,mpp=0,kept=0,bond=,pe=Cheeky"
  l[#l + 1] = "CM1|P|1803|name=Mira,job=3,lv=75,sjob=5,slv=37,hpp=92,mpp=81,kept=2,bond=Sworn,pe=Gentle"
  l[#l + 1] = "CM1|K|1|name=Brokk,job=7,race=8,points=320,rank=Trusted,at=300,next=Veteran,nextat=700,where=party,wait=0,pe=Stalwart,kl=212,lg=0"
  l[#l + 1] = "CM1|K|2|name=Mira,job=3,race=7,points=2400,rank=Sworn,at=2000,next=,nextat=0,where=party,wait=0,pe=Gentle,kl=980,lg=0"
  l[#l + 1] = "CM1|K|3|name=Vex,job=4,race=3,points=40,rank=Acquainted,at=0,next=Friendly,nextat=100,where=wait,wait=300,pe=Fiery,kl=31,lg=0"
  l[#l + 1] = "CM1|K|4|name=Talo,job=11,race=1,points=150,rank=Friendly,at=100,next=Trusted,nextat=300,where=errand,wait=1800,pe=Quiet,kl=77,lg=0"
  l[#l + 1] = "CM1|K|5|name=Iroha,job=12,race=2,points=5000,rank=Legend,at=0,next=,nextat=0,where=back,wait=0,pe=Noble,kl=9,lg=1"
  l[#l + 1] = "CM1|R|1|name=Main,jobs=7.3.10.4.12"
  l[#l + 1] = "CM1|R|2|name=Farm,jobs=6.3.11"
  l[#l + 1] = "CM1|D|1|name=Gorm,job=1,left=120"
  l[#l + 1] = "CM1|Z|0|count=3"
  return l
end

-- Nothing is asked at login
H.run(1)
check(#H.take_sent() == 0, "nothing sent at login")

-- /cv opens the view and asks for the feed
H.commands.cv()
check(ui.is_open(view.WIN), "/cv opens Companion View")
H.run(0.2)
local sent = H.take_sent()
check(sent[1] == "data", "asks for the CV feed: " .. tostring(sent[1]))

for _, l in ipairs(CV) do check(H.line(l) == false, "feed line hidden: " .. l) end
check(H.line("Hello there") == "Hello there", "other lines pass")
H.run(0.2)
check(#link.cv.order == 3, "three companions")
check(view.state.selected == nil or link.cv.companions[view.state.selected], "selection valid")
H.run(0.1)
local n = H.check_draws(view.WIN)
check(n > 100, "view drew " .. n .. " commands")
if OUT then H.dump(view.WIN, nil, OUT .. "/view_1.json") end

-- Pick the thief: the gear switch appears; switching sends it quietly
H.click(view.WIN, "body", "card1802")
check(view.state.selected == 1802, "picked the thief")
H.run(0.1)
check(H.region(view.WIN, "body", "th2"), "the thief's Damage switch")
H.click(view.WIN, "body", "th2")
H.run(0.6)
sent = H.take_sent()
check(sent[1] == "-th off", "TH off sent quietly: " .. tostring(sent[1]))
check(view.state.switching_until ~= nil, "switching")
H.check_draws(view.WIN)
if OUT then H.dump(view.WIN, nil, OUT .. "/view_thief.json") end
H.run(9)
sent = H.take_sent()
check(sent[1] == "data", "asks again after the switch: " .. tostring(sent[1]))
check(view.state.switching_until == nil, "switch over")

-- The paladin's role switch
H.click(view.WIN, "body", "card1801")
H.run(0.1)
H.click(view.WIN, "body", "role2")
H.run(0.6)
sent = H.take_sent()
check(sent[1] == "-pld dps", "PLD dps: " .. tostring(sent[1]))
if OUT then H.click(view.WIN, "body", "card1801") H.run(10) H.take_sent() H.dump(view.WIN, nil, OUT .. "/view_pld.json") end

-- Item tooltip
local title, text = view.item_tip("main", 16594)
check(title == "Bastokan Axe" and text:find("DMG:24", 1, true) and text:find("Lv.30 WAR DRK", 1, true), "item tip: " .. text)
title, text = view.item_tip("head", 12512)
check(text:find("All Jobs", 1, true) and text:find("Rare", 1, true), "all jobs, rare")

-- Companion Manager
H.commands.cmgr()
check(ui.is_open(manager.WIN), "/cmgr opens the manager")
H.run(0.2)
sent = H.take_sent()
check(sent[1] == "mdata", "asks for the CM feed: " .. tostring(sent[1]))
check(H.windows[manager.WIN].el.waiting ~= nil, "a waiting note before the feed")
for _, l in ipairs(cm_lines()) do check(H.line(l) == false, "cm line hidden") end
H.run(0.2)
check(link.cm.loaded, "manager loaded")
check(H.windows[manager.WIN].el.tabs ~= nil and ui.is_open(manager.WIN), "tabs after the first feed, still open")

local pages = { "party", "call", "kept", "presets", "tactics", "settings" }
for i, name in ipairs(pages) do
  H.event(manager.WIN, { kind = "set", id = "tabs", index = i })
  H.run(0.1)
  H.check_draws(manager.WIN)
  if OUT then H.dump(manager.WIN, i, OUT .. "/cm_" .. name .. ".json") end
end

-- Party: keep the thief, two-hour, dismiss (asks first)
H.event(manager.WIN, { kind = "set", id = "tabs", index = 1 })
H.click(manager.WIN, "party", "keep1802")
H.run(0.6)
sent = H.take_sent()
check(sent[1] == "keep Shantotto Junior", "keep: " .. tostring(sent[1]))
H.click(manager.WIN, "party", "rel1801")
check(#H.prompts == 1 and H.prompts[1].text:find("Brokk", 1, true), "dismiss asks first")
H.prompts[1].handler(2)
H.run(0.6)
check(#H.take_sent() == 0, "cancel sends nothing")
H.click(manager.WIN, "party", "rel1801")
H.prompts[2].handler(1)
H.run(0.6)
sent = H.take_sent()
check(sent[1] == "release Brokk", "release: " .. tostring(sent[1]))
H.click(manager.WIN, "party", "why1803")
H.run(0.6)
sent = H.take_sent()
check(sent[1] == ">why Mira", "why answers in chat: " .. tostring(sent[1]))
H.click(manager.WIN, "party", "fill")
H.run(0.6)
check(H.take_sent()[1] == "fill", "fill party")
H.click(manager.WIN, "party", "raise1")
H.run(0.6)
check(H.take_sent()[1] == "raise Gorm", "raise")

-- Server answers show at the top while open, in chat while closed
check(H.line("CM1|T|0|text=Brokk has left the party.") == false, "answer hidden while open")
H.run(0.1)
check(link.cm.messages[#link.cm.messages].text == "Brokk has left the party.", "answer kept")
if OUT then H.dump(manager.WIN, 1, OUT .. "/cm_party_msg.json") end

-- Call: a ready job, a job on cooldown with an Everglass price, one on cooldown without
H.event(manager.WIN, { kind = "set", id = "tabs", index = 2 })
H.run(0.1)
H.click(manager.WIN, "call", "call12")
H.run(0.6)
check(H.take_sent()[1] == "sam", "call SAM")
check(H.region(manager.WIN, "call", "call4").enabled, "BLM on cooldown, callable for Everglass")
check(not H.region(manager.WIN, "call", "call8").enabled, "DRK on cooldown, not callable")
H.run(70)
H.take_sent()
H.event(manager.WIN, { kind = "set", id = "tabs", index = 2 })
H.run(0.1)
-- (the countdown uses os.time, so it doesn't move in this stand-in; the region stays as sent)

-- Kept: call, errand menu, forget asks, collect
H.event(manager.WIN, { kind = "set", id = "tabs", index = 3 })
H.run(0.1)
H.click(manager.WIN, "kept", "kcall3")
H.run(0.6)
check(H.take_sent()[1] == "call Vex", "call a kept companion on cooldown for Everglass")
check(not H.region(manager.WIN, "kept", "errand4").enabled, "no errand while away")
H.click(manager.WIN, "kept", "errand3")
check(#H.menus == 1 and #H.menus[1].items == 6, "errand menu")
H.menus[1].handler(5)
H.run(0.6)
check(H.take_sent()[1] == "errand Vex dig", "errand dig")
H.click(manager.WIN, "kept", "collect")
H.run(0.6)
check(H.take_sent()[1] == "collect", "collect")

-- Presets: load, edit into the builder, build and save, save the party with a bad and a good name
H.event(manager.WIN, { kind = "set", id = "tabs", index = 4 })
H.run(0.1)
H.click(manager.WIN, "presets", "load1")
H.run(0.6)
check(H.take_sent()[1] == "load Main", "load preset")
H.click(manager.WIN, "presets", "edit2")
check(#manager.state.build.jobs == 3 and manager.state.build_name == "Farm", "edit fills the builder")
H.run(0.1)
H.click(manager.WIN, "builder", "b14")
check(#manager.state.build.jobs == 4, "add DRG")
H.event(manager.WIN, { kind = "change", id = "build_name", text = "Farm" })
H.windows[manager.WIN].el.build_name.text = "Farm"
H.event(manager.WIN, { kind = "press", id = "build", index = 1 })
H.run(0.6)
sent = H.take_sent()
check(sent[1] == "define thf,whm,rng,drg Farm", "define: " .. tostring(sent[1]))
H.windows[manager.WIN].el.save_name.text = "  "
H.event(manager.WIN, { kind = "press", id = "save", index = 1 })
H.run(0.6)
check(#H.take_sent() == 0 and manager.state.note.bad, "an empty name is refused")
H.windows[manager.WIN].el.save_name.text = "Night|Crew!"
H.event(manager.WIN, { kind = "press", id = "save", index = 1 })
H.run(0.6)
sent = H.take_sent()
check(sent[1] == "save NightCrew", "name cleaned: " .. tostring(sent[1]))
H.run(0.1)
H.check_draws(manager.WIN)
if OUT then H.dump(manager.WIN, 4, OUT .. "/cm_presets_note.json") end

-- Tactics: pick WHM (custom), change a rule, MP, defaults
H.event(manager.WIN, { kind = "set", id = "tabs", index = 5 })
H.run(0.1)
H.click(manager.WIN, "tactics", "tj3")
H.run(0.1)
H.click(manager.WIN, "tactics", "t_dispel1")
H.run(0.6)
check(H.take_sent()[1] == "tactics whm dispel on", "tactics rule")
H.click(manager.WIN, "tactics", "t_mp4")
H.run(0.6)
check(H.take_sent()[1] == "tactics whm mp 30", "tactics mp")
H.click(manager.WIN, "tactics", "t_reset")
H.run(0.6)
check(H.take_sent()[1] == "tactics whm reset", "tactics reset")
if OUT then H.dump(manager.WIN, 5, OUT .. "/cm_tactics.json") end

-- Settings: a switch, a sub job from the menu, an in-chat button
H.event(manager.WIN, { kind = "set", id = "tabs", index = 6 })
H.run(0.1)
H.click(manager.WIN, "settings", "engage2")
H.run(0.6)
check(H.take_sent()[1] == "engage retail", "engage retail")
H.click(manager.WIN, "settings", "drk2")
H.run(0.6)
check(H.take_sent()[1] == "drk dps", "drk dps (drktank was on)")
H.click(manager.WIN, "settings", "sub6")
local menu = H.menus[#H.menus]
check(menu.items[1] == "Default", "sub menu starts with Default")
menu.handler(1)
H.run(0.6)
check(H.take_sent()[1] == "sub thf default", "sub default")
H.click(manager.WIN, "settings", "chat_Status")
H.run(0.6)
check(H.take_sent()[1] == ">status", "status in chat")

-- /cm kept opens on that tab; the window closed: answers go to chat as plain lines
H.commands.cmgr()
check(not ui.is_open(manager.WIN), "/cmgr closes it")
check(H.line("CM1|T|0|text=Vex is on their way.") == "Vex is on their way.", "answer in chat while closed")
H.commands.cmgr("kept")
check(ui.is_open(manager.WIN) and manager.state.tab == 3, "/cmgr kept")

-- A slow action asks both feeds again later
H.take_sent()
link.waiting.cm, link.waiting.cv = nil, nil
link.send("!companion", link.SLOW)
H.run(0.6)
check(H.take_sent()[1] == "fill", "fill")
H.run(9)
sent = H.take_sent()
local asked = table.concat(sent, ",")
check(asked:find("mdata", 1, true) and asked:find("data", 1, true), "asked again after a slow action: " .. asked)

-- No answer: "Updating..." gives up
link.ask("cv")
H.run(6)
check(link.waiting.cv == nil, "gave up waiting")

-- Packets refused (no permission): the status says so
packets.inject_out = function() error("changing or blocking packets needs packets.write (/lua grant)") end
link.ask("cm")
H.run(0.6)
local status, bad = link.status("cm")
check(bad and status:find("packets", 1, true), "permission note: " .. status)


packets.inject_out = H.inject
link.error = nil

-- Live: the party list and entities as the client has them
H.party = {
  { slot = 0, party = 0, id = 100, index = 900, name = "Ryan", hp = 900, hpp = 100, mp = 0, mpp = 0, tp = 0, main_job = 1,
    main_level = 75, sub_job = 0, sub_level = 0, in_zone = true, buffs = {} },
  { slot = 1, party = 0, id = 0x1000701, index = 1801, name = "Brokk", hp = 1000, hpp = 70, mp = 200, mpp = 52, tp = 1800,
    main_job = 7, main_level = 75, sub_job = 3, sub_level = 37, in_zone = true, buffs = { 116, 43, 33 }, hp_max = 1420 },
  { slot = 2, party = 0, id = 0x1000702, index = 1802, name = "Shantotto Junior", hp = 840, hpp = 100, mp = 0, mpp = 0,
    tp = 450, main_job = 6, main_level = 75, sub_job = 13, sub_level = 37, in_zone = true, buffs = {} },
  { slot = 3, party = 0, id = 0x1000703, index = 1803, name = "Mira", hp = 0, hpp = 0, mp = 300, mpp = 27, tp = 0,
    main_job = 3, main_level = 75, sub_job = 5, sub_level = 37, in_zone = true, buffs = {} },
}
H.entities = {
  [0x1000701] = { id = 0x1000701, index = 1801, name = "Brokk", trust = true, status = 1, target_id = 0x2001,
    pos = { x = 10, y = -2, z = 5 }, height = 2.1 },
  [0x1000702] = { id = 0x1000702, index = 1802, name = "Shantotto Junior", trust = true, status = 0,
    pos = { x = 12, y = -1, z = 4 }, height = 1.0 },
  [0x1000703] = { id = 0x1000703, index = 1803, name = "Mira", trust = true, status = 2, pos = { x = 8, y = -1.5, z = 6 },
    height = 1.6 },
  [0x2001] = { id = 0x2001, index = 3001, name = "Goblin Smithy", status = 1, pos = { x = 11, y = -2, z = 6 } },
}
H.fire("action", { actor = 0x1000702, category = 8, param = 24931,
  targets = { { id = 0x1000701, results = { { value = 4 } } } } })
H.fire("action", { actor = 0x1000701, category = 3, param = 42, targets = { { id = 0x2001, results = { { value = 900 } } } } })
local live = require("lib.live")
local a1 = live.activity(0x1000702, H.entities[0x1000702])
check(a1 == "Casting Cure IV → Brokk", "casting line: " .. tostring(a1))
local a2 = live.activity(0x1000701, H.entities[0x1000701])
check(a2 == "Savage Blade! → Goblin Smithy", "weapon skill line: " .. tostring(a2))
H.run(0.5)

-- The world: plates, rings, bubbles
check(H.looks[0x1000701] and H.looks[0x1000701].name_colour, "role name plate")
check(H.draws["ring_" .. 0x1000701], "ring under Brokk")
H.fire("chat_message", "Watch out, it's casting!", "Brokk", 4, false)
H.run(0.2)
check(H.hud["bub_" .. 0x1000701] and H.hud["bubt_" .. 0x1000701].text:find("Watch out", 1, true), "speech bubble")
H.run(12)
check(not H.hud["bub_" .. 0x1000701], "bubble gone")
H.settings.rings = false
H.run(0.2)
check(not H.draws["ring_" .. 0x1000701], "rings off")
H.settings.rings = true

-- Keys from the settings
H.settings.key_view = "ctrl+v"
H.fire("settings_changed")
check(type(H.binds["ctrl+v"]) == "function", "key bound")
H.settings.key_view = ""
H.fire("settings_changed")
check(H.binds["ctrl+v"] == nil, "key unbound")

-- Companion View and the Party tab show the live HP
if not ui.is_open(view.WIN) then H.commands.cv() end
view.state.selected = 1801
H.fire("action", { actor = 0x1000701, category = 3, param = 42, targets = { { id = 0x2001, results = { { value = 900 } } } } })
H.run(0.5)
local found = false
for _, d in ipairs(H.windows[view.WIN].el.body.draw) do
  if d.type == "text" and d.text == "1000 / 1420" then found = true end
end
check(found, "view shows the live HP")
found = false
for _, d in ipairs(H.windows[view.WIN].el.body.draw) do
  if d.type == "text" and d.text:find("Savage Blade", 1, true) then found = true end
end
check(found, "view shows what Brokk is doing")
local jobicon = false
for _, d in ipairs(H.windows[view.WIN].el.body.draw) do
  if d.type == "icon" and d.kind == "job" and d.id == 7 then jobicon = true end
end
check(jobicon, "the view draws the client's PLD job icon")
-- A stock client (no job icons): a lettered badge stands in, never an icon of an unknown kind
local J = require("lib.jobicons")
J.native = false
local fake = { draw = {}, circle = function(self, ...) self.draw[#self.draw + 1] = { type = "circle" } end,
  text = function(self, x, y, t) self.draw[#self.draw + 1] = { type = "text", text = t } end,
  icon = function(self) self.draw[#self.draw + 1] = { type = "icon" } end }
J.draw(fake, 0, 0, 28, 7)
local kinds = {}
for _, d in ipairs(fake.draw) do kinds[#kinds + 1] = d.type .. (d.text and (":" .. d.text) or "") end
check(table.concat(kinds, ",") == "circle,circle,text:PLD", "stock client badge: " .. table.concat(kinds, ","))
J.native = true
H.click(view.WIN, "body", "target")
check(H.targeted == 0x1000701, "Target button")
H.check_draws(view.WIN)
if OUT then H.dump(view.WIN, nil, OUT .. "/view_live.json") end

check(#H.log == 0, "no render errors: " .. table.concat(H.log, " | "))
print(("ok: %d checks"):format(passed))
