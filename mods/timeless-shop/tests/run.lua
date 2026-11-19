-- Timeless Shop's tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

H.mod_id = "timeless-shop"
local ITEMS = {
  [1130] = { name = "Moon Orb", description = "A mysterious orb used to enter certain battlefields." },
  [1131] = { name = "Star Orb", description = "Another orb." },
  [16594] = { name = "Bastokan Axe", kind = "weapon", description = "DMG:24 Delay:276\nSTR+2", level = 30, jobs = (1 << 1) | (1 << 8) },
}
res.item = function(id) return ITEMS[id] end
H.entities[17000300] = { id = 17000300, index = 300, name = "Gaudierre", kind = "npc", race = 3 }

dofile("init.lua")
local W = require("lib.window")
local themes = require("lib.themes")
local protocol = require("lib.protocol")

local function texts()
  local out = {}
  for _, d in ipairs(H.windows.shop.el.shop.draw or {}) do if d.type == "text" then out[#out + 1] = d.text end end
  return table.concat(out, " | ")
end
local function has_tip(s)
  for _, d in ipairs(H.windows.shop.el.shop.draw or {}) do
    if d.type == "tip" and (d.text or ""):find(s, 1, true) then return true end
  end
end
local function feed(lines) for _, l in ipairs(lines) do ok(H.line(l) == false, "hidden from chat: " .. l) end end

ok(H.commands.storefront and H.commands.timelessshop, "commands")
ok(not ui.is_open("shop"), "closed at first")

-- It tells the server it's here, and again every two minutes
H.run(1.2)
local s = H.take_sent()
ok(#s == 1 and s[1] == "sh on", "sh on after load: " .. table.concat(s, ","))
H.run(110)
ok(#H.take_sent() == 0, "not again before two minutes")
H.run(11)
ok(H.take_sent()[1] == "sh on", "again at two minutes")

-- The parser: labels with commas, unknown lines, partial shops
local m = protocol.new_model()
protocol.feed(m, "SH1|B|t=5,k=Odilo,i=0,r=hume,c=Everglass,b=10,n=1", 0)
protocol.feed(m, "SH1|S|1|Rare finds", 0)
protocol.feed(m, "SH1|I|1.1|id=844,ic=0,g=,p=30,q=1,a=1,l=50 for 15,000 gil, today", 0)
ok(m.shop == nil, "nothing until E")
ok(protocol.feed(m, "SH1|E|t=5", 0) == "open" and m.shop.sections[1].items[1].label == "50 for 15,000 gil, today", "a label keeps its commas")
ok(protocol.feed(m, "GA1|J|1|n=0", 0) == false and protocol.feed(m, "SH1|Q|x", 0) == true, "other feeds aren't ours; unknown kinds ignored")

-- Grizznix's stall: the goblin window
local GOBLIN = {
  "SH1|B|t=7,k=Grizznix,i=0,r=goblin,c=Everglass,b=50,n=2",
  "SH1|G|Hee hee! Youuu find Grizznix camp! Good nose. Want buy?",
  "SH1|S|1|Wares", "SH1|S|2|Mystery sack",
  "SH1|I|1.1|id=1130,ic=0,g=,p=12,q=2,a=1,l=Moon Orb",
  "SH1|I|1.2|id=1131,ic=0,g=,p=18,q=0,a=0,l=Star Orb",
  "SH1|W|1.2|Sold out! Grizznix sells fast, yes yes.",
  "SH1|I|1.3|id=16594,ic=0,g=,p=30,q=1,a=1,l=Bastokan Axe",
  "SH1|I|2.1|id=0,ic=0,g=sack,p=12,q=3,a=1,l=Mystery sack",
  "SH1|D|2.1|One roll of Grizznix's sack.",
  "SH1|E|t=7",
}
feed(GOBLIN)
ok(ui.is_open("shop"), "the window opens")
local t = texts()
ok(t:find("Grizznix's Sack o' Stuff", 1, true) and t:find("Goblin merchant", 1, true), "a goblin's shop name: " .. t)
ok(t:find("Hee hee! Youuu find Grizznix camp!", 1, true), "what he said, in the bubble")
ok(t:find("50 shinies", 1, true), "money as a goblin says it")
ok(t:find("Moon Orb", 1, true) and t:find("2 left", 1, true) and t:find("sold out", 1, true), "cards with stock")
ok(not t:find("Grab fast!", 1, true) and not t:find("Cheap-cheap!", 1, true), "no stamps on the cards")
ok(H.sounds[#H.sounds] == "sounds/coins_open.wav", "the goblin's stall jingles as it opens: " .. tostring(H.sounds[#H.sounds]))
ok(t:find("A mysterious orb", 1, true), "the first item's preview: the game's description")
ok(has_tip("Sold out! Grizznix sells fast"), "a sold-out card says why")
H.check_draws("shop")
local stitched = 0
for _, d in ipairs(H.windows.shop.el.shop.draw) do if d.type == "line" then stitched = stitched + 1 end end
ok(stitched > 30, "patched and stitched (" .. stitched .. " lines)")

-- Pick the axe: level and jobs in the preview
H.click("shop", "shop", "item3")
t = texts()
ok(t:find("Lv. 30", 1, true) and t:find("WAR DRK", 1, true) and t:find("DMG:24", 1, true), "level, jobs, description: " .. t)

-- Buying: asks once more, then sends
H.take_sent()
H.click("shop", "shop", "buy")
ok(texts():find("Buy for 30?", 1, true), "Buy asks once more")
ok(#H.take_sent() == 0, "nothing sent yet")
H.click("shop", "shop", "cancel")
ok(not texts():find("Buy for 30?", 1, true), "Cancel")
H.click("shop", "shop", "buy")
H.click("shop", "shop", "buy")
H.run(0.1)
s = H.take_sent()
ok(#s == 1 and s[1] == "sh buy 7 1 3", "sent: " .. table.concat(s, ","))
ok(texts():find("...", 1, true), "waiting")

-- The answer and the refreshed shop: the goblin's voice, the stock and balance, still on the axe
feed({ "SH1|R|1|You obtain Bastokan Axe. (30 Everglass; 0 left)" })
local said = W.speech()
local okvoice = false
for _, v in ipairs(themes.goblin.voice.buy) do if v == said then okvoice = true end end
ok(okvoice, "the goblin says something about the trade: " .. said)
local again = {}
for _, l in ipairs(GOBLIN) do again[#again + 1] = l end
again[1] = "SH1|B|t=8,k=Grizznix,i=0,r=goblin,c=Everglass,b=0,n=2"
again[8] = "SH1|I|1.3|id=16594,ic=0,g=,p=30,q=0,a=0,l=Bastokan Axe"
again[#again] = "SH1|E|t=8"
feed(again)
t = texts()
ok(t:find("0 shinies", 1, true) and t:find("You obtain Bastokan Axe", 1, true), "balance and the message: " .. t)
ok(W.state.item == 3 and W.state.section == 1, "still on the axe")
ok(t:find("Bastokan Axe", 1, true), "the axe is still shown")

-- Too poor: the goblin grumbles
H.click("shop", "shop", "item1")
t = texts()
ok(t:find("No shinies, no stuff", 1, true) or t:find("Pockets empty", 1, true), "can't afford: the goblin says so: " .. t)
feed({ "SH1|R|0|That costs 12 Everglass; you have 0." })
local poor = false
for _, v in ipairs(themes.goblin.voice.poor) do if v == W.speech() then poor = true end end
ok(poor, "a refused buy for money: the poor line")

-- The sack section: drawn as a sack, with its description
H.click("shop", "shop", "sec2")
t = texts()
ok(t:find("Mystery sack", 1, true) and t:find("One roll of Grizznix's sack.", 1, true) and t:find("?", 1, true), "the sack: a drawn sack and its line")
H.check_draws("shop")

-- Old menu and Close
H.take_sent()
H.click("shop", "shop", "classic")
H.run(0.1)
ok(H.take_sent()[1] == "sh classic 8" and not ui.is_open("shop"), "Old menu: asks for the classic menu, closes")
feed(again)
H.take_sent()
H.click("shop", "shop", "close")
H.run(0.1)
ok(H.take_sent()[1] == "sh close" and not ui.is_open("shop"), "Close tells the server")

-- A quartermaster: no race from the server; the client reads the keeper's (an Elvaan)
feed({ "SH1|B|t=9,k=Gaudierre,i=300,r=,c=Everglass,b=120,n=1", "SH1|T|Everglass Shop", "SH1|S|1|Hunting",
  "SH1|I|1.1|id=0,ic=0,g=lure,p=12,q=-1,a=1,l=Champion lure 30m", "SH1|D|1.1|Champions are far more likely.", "SH1|E|t=9" })
t = texts()
ok(t:find("The House of Gaudierre", 1, true) and t:find("Elvaan purveyor", 1, true), "an Elvaan's house: " .. t)
ok(W.state.voice and W.speech() ~= "", "an Elvaan greeting")
H.check_draws("shop")

-- !shop with no keeper: the guild
feed({ "SH1|B|t=10,k=Artisans Guild,i=0,r=guild,c=Everglass,b=120,n=1", "SH1|T|Everglass Shop", "SH1|S|1|Hunting",
  "SH1|I|1.1|id=0,ic=0,g=lure,p=12,q=-1,a=1,l=Champion lure 30m", "SH1|E|t=10" })
ok(texts():find("Everglass Shop", 1, true) and texts():find("Artisans Guild", 1, true), "the guild's shop")

-- Many items: pages
local many = { "SH1|B|t=11,k=Odilo,i=0,r=hume,c=Everglass,b=500,n=1", "SH1|S|1|Gear" }
for i = 1, 25 do many[#many + 1] = ("SH1|I|1.%d|id=1130,ic=0,g=,p=%d,q=1,a=1,l=Item number %d"):format(i, i, i) end
many[#many + 1] = "SH1|E|t=11"
feed(many)
ok(texts():find("1 / ", 1, true), "pages")
H.click("shop", "shop", "next")
ok(texts():find("2 / ", 1, true), "next page")
H.check_draws("shop")

-- Every race's demo draws inside the window
for _, race in ipairs(themes.ORDER) do
  H.command("/storefront demo " .. race)
  ok(ui.is_open("shop") and W.state.demo, race .. ": the demo opens")
  local th = themes[race]
  ok(texts():find(th.label, 1, true), race .. ": its label")
  H.check_draws("shop")
  for _, k in ipairs({ "greet", "buy", "poor", "sold", "browse" }) do ok(#th.voice[k] >= 1, race .. " says something to " .. k) end
end
-- The demo buys from itself, and sends nothing
H.take_sent()
H.command("/storefront demo goblin")
H.click("shop", "shop", "item1")
H.click("shop", "shop", "buy")
H.click("shop", "shop", "buy")
H.run(0.1)
ok(#H.take_sent() == 0 and texts():find("52 shinies", 1, true), "the demo buys from itself")
H.command("/storefront close")
ok(not ui.is_open("shop"), "/storefront close")

-- More flavour: remarks, regulars, the time of day, the burst, goodbyes, sounds
do
  local flavor = require("lib.flavor")
  -- Every race has its sound files, a burst, a goodbye and regulars' tiers
  for _, race in ipairs(themes.ORDER) do
    local fl = flavor[race]
    ok(fl and fl.sound and fl.burst and #fl.bye > 0, race .. ": flavour")
    ok(#fl.regulars == 3 and fl.regulars[1][1] == 3 and fl.regulars[3][1] == 25, race .. ": three regulars' tiers")
    for _, which in ipairs({ "open", "buy" }) do
      audio.play(("sounds/%s_%s.wav"):format(fl.sound, which))
    end
    ok(fl.quips.thing ~= nil or race ~= "guild", race .. ": a remark for anything")
  end
  -- Kinds of items
  ok(flavor.kind({ label = "Moon Orb" }, { name = "Moon Orb", kind = "general" }) == "orb", "an orb")
  ok(flavor.kind({ label = "x" }, { name = "Bastokan Axe", kind = "weapon" }) == "weapon", "a weapon")
  ok(flavor.kind({ label = "Goblin Pie" }, nil) == "food", "food")
  ok(flavor.kind({ label = "Mystery sack", glyph = "sack" }, nil) == "sack", "a glyph is its own kind")
  ok(flavor.quip("goblin", { label = "Moon Orb" }, { name = "Moon Orb" }, 2):find("Moon Orb", 1, true), "a remark names the item")
  ok(flavor.quip("tonberry", { label = "x" }, nil, 1) == "...", "a Tonberry says nothing much")

  -- Picking an item: the keeper remarks on it
  feed(GOBLIN)
  H.click("shop", "shop", "item1")
  local said = W.speech()
  local okq = false
  for _, q in ipairs(flavor.goblin.quips.orb) do if said == q:gsub("{item}", "Moon Orb") then okq = true end end
  ok(okq, "a goblin's remark about the orb: " .. said)
  H.click("shop", "shop", "item3")
  ok(W.speech():find("Pointy", 1, true) or W.speech():find("stabber", 1, true), "about the axe: " .. W.speech())

  -- Regulars: buying counts; at 3 the goblin greets you as a good customer and gives you a title
  storage.character.shop_regulars = { Grizznix = 2 }
  W.close(true)
  ok(H.printed[#H.printed]:find("^Grizznix: ") ~= nil, "a goodbye in chat: " .. tostring(H.printed[#H.printed]))
  feed(GOBLIN)
  ok(not texts():find("Good customer", 1, true), "no title yet")
  feed({ "SH1|R|1|You obtain Moon Orb. (12 Everglass; 38 left)" })
  ok(storage.character.shop_regulars.Grizznix == 3, "a purchase counts")
  ok(W.state.burst ~= nil, "a burst of coins")
  ok(H.sounds[#H.sounds] == "sounds/coins_buy.wav", "and the coins jingle")
  H.check_draws("shop")
  W.close(true)
  feed(GOBLIN)
  local t = texts()
  ok(t:find("★ Good customer", 1, true), "the title in the header: " .. t)
  ok(t:find("You come back! Grizznix knew you would.", 1, true), "a regular's welcome beats the server greeting")
  storage.character.shop_regulars.Grizznix = 30
  W.close(true)
  feed(GOBLIN)
  ok(texts():find("★ Grizznix's favourite", 1, true), "25 and up: the favourite")

  -- The burst fades; the window keeps moving only while something does
  H.run(1.5)
  ok(W.state.burst == nil, "the burst is over after a moment")
  ok(W.animated(), "the goblin's coin keeps glinting")

  -- The time of day, for a keeper without a server greeting
  game.time = function() return { hour = 22, weekday = 0, moon_phase = 1, moon = 30 } end
  W.close(true)
  feed({ "SH1|B|t=40,k=Bargrudd,i=0,r=galka,c=Everglass,b=50,n=1", "SH1|S|1|Gear", "SH1|I|1.1|id=1130,ic=0,g=,p=12,q=2,a=1,l=Moon Orb", "SH1|E|t=40" })
  ok(W.speech() == flavor.galka.night[1], "a Galka at night: " .. W.speech())
  game.time = nil

  -- Sounds off
  H.command("/storefront sound")
  local n0 = #H.sounds
  W.close(true)
  feed(GOBLIN)
  ok(#H.sounds == n0, "sounds off: silent")
  H.command("/storefront sound")
  W.close(true)
end

-- Zoning closes it; unloading tells the server
feed(GOBLIN)
H.fire("zone_change")
ok(not ui.is_open("shop"), "zoning closes the shop")
H.take_sent()
H.fire("unload")
ok(H.take_sent()[1] == "sh off", "unload: sh off")

io.stdout:write(("ok: %d checks\n"):format(n))
