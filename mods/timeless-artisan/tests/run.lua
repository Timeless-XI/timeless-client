-- Timeless Artisan's tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

local L = require("lib.logic")
local IDX = L.index(require("lib.data"))

----------------------------------------------------------------------------------------------------------------------
-- The maths (the server's synthutils rules)
----------------------------------------------------------------------------------------------------------------------
do
  local st = L.new_state()
  L.parse(st, "TA1|V|ver=2,max=12,modern=1,cm=1,hm=1")
  L.parse(st, "TA1|K|0:0.105:1.0:0.0:0.0:0.0:0.0:0.0:0")
  local o = L.odds(st, IDX.by_id[10002])
  ok(math.abs(o.success - 0.95) < 1e-9, "success 95%")
  ok(math.abs(o.hq - 0.95 * 0.015625) < 1e-9 and o.hq_tier == 1 and o.hq2 == 0, "tier 1 HQ 1/64")
  local k = 3 - math.log(1.2 + 105 / 100)
  ok(math.abs(o.crafts[2].on_success - k / 14) < 1e-9, "skill-up formula")
  L.parse(st, "TA1|K|0:0.10:1.0:0.0:0.0:0.0:0.0:0.0:0")
  o = L.odds(st, IDX.by_id[10015])
  ok(math.abs(o.success - 0.60) < 1e-9 and o.hq == 0, "d=5: 60%, no HQ")
  k = math.min(1, 5 * (3 - math.log(1.2 + 10 / 100)) / 5)
  ok(math.abs(o.crafts[2].skillup - (0.6 * k + 0.4 * k / 2)) < 1e-9, "skill-up with breaks")
  L.parse(st, "TA1|V|ver=2,max=12,modern=0,cm=1,hm=1")
  o = L.odds(st, IDX.by_id[10015])
  ok(math.abs(o.crafts[2].on_success - 0.6) < 1e-9, "era 60%")
  L.parse(st, "TA1|K|0:0.100:0.0:0.0:0.0:0.0:0.0:0.0:0")
  o = L.odds(st, IDX.by_id[10015])
  ok(o.crafts[2].skillup == 0 and o.crafts[2].why == "cap", "rank cap")
  L.parse(st, "TA1|K|0:0.1000:9.0:0.0:0.0:0.0:0.0:0.0:0")
  L.parse(st, "TA1|M|sr=0,hq=51,sg=0,ml=0,c=0.0.0.0.0.0.0.0,ah=0.0.0.0.0.0.0.0,cl=0.0.0.0.0.0.0.0")
  o = L.odds(st, IDX.by_id[10015])
  ok(o.hq_tier == 4 and math.abs(o.hq_given - (50 + 100 * 51 / 512) / 100) < 1e-9, "tier 4 + HQ rate")
  L.parse(st, "TA1|M|sr=0,hq=0,sg=0,ml=0,c=0.0.0.0.0.0.0.0,ah=0.1.0.0.0.0.0.0,cl=0.0.0.0.0.0.0.0")
  o = L.odds(st, IDX.by_id[10015])
  ok(o.hq == 0 and math.abs(o.success - 0.96) < 1e-9, "craft ring")
  -- materials
  local c = L.check(st, IDX.by_id[2], function() return 0 end, function(id) return id == 4098 and 1 or 0 end)
  ok(not c.ok and #c.missing == 1 and c.missing[1].id == 688, "missing log")
  c = L.check(st, IDX.by_id[2], function(id) return id == 688 and 3 or 0 end, function(id) return id == 4106 and 1 or 0 end)
  ok(c.ok and c.max == 3 and c.crystals == 12, "a cluster counts as 12 crystals")
  c = L.check(st, IDX.by_id[3], function() return 10 end, function() return 0 end, function() return false end)
  ok(not c.ok and c.no_ki, "key item recipe")
  -- guild plan: 1500 points left at 20 each = 75; 5 owned; 12 at a time
  L.parse(st, "TA1|B|gd") L.parse(st, "TA1|G|2|649|1500|20|2000|1") L.parse(st, "TA1|Z|gd")
  local p = L.guild_plan(st, IDX, 2, function() return 0 end, function(id) return id == 649 and 5 or 0 end)
  ok(p.items == 75 and p.own == 5 and p.to_make == 70 and p.crafts == 70 and p.queue == 12, "guild plan")
  -- kept items and auto-deposit
  local kept, carried = { [641] = 7, [640] = 3 }, { [641] = 10, [640] = 0 }
  L.kept_clamp(kept, function(id) return carried[id] or 0 end)
  ok(kept[641] == 7 and kept[640] == nil, "kept: none carried = released")
  local plan = L.auto_plan({ 641, 656 }, { [641] = 4 }, function(id) return id == 641 and 9 or 2 end, function(id) return id == 641 and 6 or 2 end)
  ok(#plan == 2 and plan[1].qty == 5 and plan[2].qty == 2, "auto plan: extras only")
  ok(L.parse_amount("7", 20) == 7 and L.parse_amount("999", 20) == 20 and L.parse_amount("", 20) == nil
    and L.parse_amount("1,000", 5000) == 1000, "typed amounts")
  ok(IDX.materials[1997] and L.group_of(IDX, 1997) == 4, "Sailcloth is a Clothcraft material")
  ok(IDX.materials[1998] and L.group_of(IDX, 1998) == 5, "Sheep Chammy is a Leathercraft material")
  -- a line as the chat log hands it over (colour codes, trailing space)
  local st2 = L.new_state()
  ok(L.parse(st2, "\30\1TA1|V|ver=2,max=12,modern=1,cm=1,hm=1 ") == "V" and st2.ready, "line with colour codes")
  local chunks = L.chunk_requests("dp 1", (function() local t = {} for i = 1, 40 do t[i] = "1234:12" end return t end)())
  for _, ch in ipairs(chunks) do ok(#ch <= L.MAX_REQUEST, "chunk fits a packet") end
end

----------------------------------------------------------------------------------------------------------------------
-- The mod
----------------------------------------------------------------------------------------------------------------------
H.mod_id = "timeless-artisan"
dofile("init.lua")
local C = require("lib.core")
local W = require("lib.window")

ok(H.commands.artisan and H.commands.timelessartisan, "commands")
ok(H.provided["timeless-artisan"].timeless_menu, "gives a Timeless menu card")
local card = H.provided["timeless-artisan"].timeless_menu()
ok(card.title == "Artisan" and card.category == "Crafting & Shops" and card.open == "//artisan" and #card.buttons == 6, "the card")

-- In the world: it says hello, and the HUD shows
H.bags[0] = { { id = 640, count = 12 }, { id = 640, count = 5 }, { id = 4096, count = 3 }, { id = 12345, count = 1 } }
H.run(0.5)
local sent = H.take_sent()
ok(sent[1] == "hi", "asks the server: " .. tostring(sent[1]))
ok(ui.is_open("artisan_hud"), "HUD shows")

-- The server answers; its lines stay out of chat; other lines pass
local function server(line) return H.line("TA1|" .. line) end
local function find_sent(list, pat) for _, s in ipairs(list) do if s:find(pat) then return s end end end
ok(server("V|ver=2,max=12,modern=1,cm=1,hm=1,cap=0,spec=0") == false, "server line hidden")
server("K|0:0.105:1.0:0.0:0.0:0.0:0.0:0.0:0")
server("M|sr=0,hq=0,sg=0,ml=0,c=0.0.0.0.0.0.0.0,ah=0.0.0.0.0.0.0.0,cl=0.0.0.0.0.0.0.0")
server("B|st") server("S|0|641:20.4104:2") server("S|1|1998:3") server("Z|st")
server("Q|run=0,n=0,cur=0,list=")
ok(H.line("Tester's Smithing skill rises 0.1 points.") ~= false, "other lines pass")
H.run(0.5)
ok(C.ST.ready and C.ST.store[641] == 20 and C.ST.store_own[1998] == 3, "server state read")
ok(C.have(640) == 17, "bags read")
ok(C.session.skill.Smithing and math.abs(C.session.skill.Smithing - 0.1) < 1e-9, "skill-up counted")
H.take_sent()

-- Open the window on every tab: nothing draws an error
local function canvas_texts(win, id)
  local out = {}
  for _, d in ipairs(H.windows[win].el[id].draw or {}) do
    if d.type == "text" then out[#out + 1] = d.text end
  end
  return table.concat(out, "\n")
end
H.command("/artisan")
ok(ui.is_open("artisan"), "window opens")
for t = 1, 5 do
  H.handlers.artisan("artisan", { kind = "set", id = "tabs", index = t })
  H.run(0.3)
  local txt = canvas_texts("artisan", "main")
  ok(not txt:find("Error:", 1, true), "tab " .. t .. " draws: " .. txt:sub(1, 300))
end
local asked = H.take_sent()
local joined = table.concat(asked, ";")
ok(joined:find("gd", 1, true) and joined:find("st", 1, true) and joined:find("qs", 1, true), "tabs ask the server: " .. joined)

-- Recipes: Smithing, the 1-10 band; Bronze Ingot from tin ore (640 x3 + 641 x1? no: 640 copper ore x3 + 641 tin ore)
W.state.craft, W.state.band = 2, 2
W.go(1)
H.run(0.3)
local txt = canvas_texts("artisan", "main")
ok(txt:find("Bronze Ingot", 1, true), "Smithing recipes listed")
H.click("artisan", "main", "craft10002")
H.run(0.3)
sent = H.take_sent()
ok(sent[1] == "cq 10002 1 1", "Craft queues one: " .. tostring(sent[1]))

-- A favorite, then the Favorites tab and the HUD list it
H.click("artisan", "main", "fav10002")
ok(C.is_fav(10002), "favorite added")
W.go(2)
ok(canvas_texts("artisan", "main"):find("Bronze Ingot", 1, true), "Favorites tab")
H.run(0.3)
ok(canvas_texts("artisan_hud", "hud"):find("Bronze Ingot", 1, true), "HUD lists favorites")

-- Find: by material name across crafts
H.handlers.artisan("artisan", { kind = "edit", id = "q", text = "tin ore" })
W.go(1)
ok(canvas_texts("artisan", "main"):find("Every craft", 1, true), "Find searches every craft")
H.handlers.artisan("artisan", { kind = "edit", id = "q", text = "" })

-- The recipe card: Craft 3 with the stepper
H.click("artisan", "main", "row10002")
ok(ui.is_open("artisan_recipe"), "recipe card opens")
ok(canvas_texts("artisan_recipe", "card"):find("Your chances", 1, true), "card shows chances")
H.click("artisan_recipe", "card", "plus")
H.click("artisan_recipe", "card", "plus")
H.click("artisan_recipe", "card", "craft")
H.run(0.3)
sent = H.take_sent()
ok(sent[1] == "cq 10002 3 1", "card queues 3: " .. tostring(sent[1]))

-- Queue lines: progress; Repeat comes back when it's empty
server("Q|run=1,n=2,cur=10002,list=10002*2")
H.run(0.3)
local v = C.queue_view()
ok(v and v.total == 3 and v.at == 1 and v.recipe.id == 10002, "queue view")
server("R|recipe=10002,res=2,item=649,qty=1")
server("Q|run=1,n=1,cur=10002,list=10002*1")
H.run(0.3)
v = C.queue_view()
ok(v.at == 2 and v.total == 3 and C.session.hq == 1, "second synth, an HQ counted")
ok(canvas_texts("artisan_hud", "hud"):find("2 / 3", 1, true), "HUD shows progress")
server("Q|run=0,n=0,cur=0,list=")
H.run(0.3)
ok(C.queue_view() == nil and C.qpeak == 0, "queue empty")
H.click("artisan_hud", "hud", "repeat")
H.run(0.3)
sent = H.take_sent()
ok(sent[1] == "cq 10002 3 1", "Repeat queues the last craft again: " .. tostring(sent[1]))

-- Storage: Withdraw asks how many (of N); the hand withdrawal is kept from auto-deposit
W.go(4)
W.state.open[1] = true -- Woodworking... tin ore's group
local g = L.group_of(C.IDX, 641)
W.state.open[g] = true
W.render()
H.click("artisan", "main", "wd641")
ok(ui.is_open("artisan_amount"), "amount popup")
ok(H.windows.artisan_amount.el.n.label:find("of 20 stored", 1, true), "label says of N: " .. H.windows.artisan_amount.el.n.label)
H.handlers.artisan_amount("artisan_amount", { kind = "change", id = "n", text = "5" })
H.run(0.3)
sent = H.take_sent()
ok(find_sent(sent, "^wd") == "wd 641 5", "withdraw 5: " .. table.concat(sent, ";"))
ok(not ui.is_open("artisan_amount"), "popup closes")
-- auto-deposit everything; the withdrawn 5 land and stay
C.set_auto_mode(1)
server("D|act=wd,i=641,q=5,why=")
server("B|st") server("S|0|641:15.4104:2") server("S|1|1998:3") server("Z|st")
H.bags[0][#H.bags[0] + 1] = { id = 641, count = 5 }
H.run(6)
sent = H.take_sent()
local dp = nil
for _, s in ipairs(sent) do if s:find("^dp") then dp = s end end
ok(dp ~= nil and not dp:find("641:", 1, true) and dp:find("640:17", 1, true) and dp:find("4096:3", 1, true), "auto-deposit skips kept tin ore: " .. tostring(dp))
ok(not dp:find("12345", 1, true), "gear stays")
ok(C.kept_of(641) == 5, "5 kept")
-- picking up 3 more tin ore: only those 3 go
H.bags[0][#H.bags[0] + 1] = { id = 641, count = 3 }
H.run(6)
sent = H.take_sent()
dp = nil
for _, s in ipairs(sent) do if s:find("^dp") then dp = s end end
ok(dp and dp:find("641:3", 1, true), "only the extra 3 go: " .. tostring(dp))
-- Deposit by hand releases the kept ones
H.click("artisan", "main", "dp641")
H.run(0.3)
sent = H.take_sent()
ok(sent[#sent] and sent[#sent]:match("^dp %d+ 641:0$") and C.kept_of(641) == 0, "Deposit stores all and releases: " .. tostring(sent[#sent]))
C.set_auto_mode(0)

-- Withdraw all needs a second click
W.render()
H.click("artisan", "main", "wa")
H.run(0.2)
ok(#H.take_sent() == 0, "first click only asks")
H.click("artisan", "main", "wa")
H.run(0.3)
sent = H.take_sent()
ok(find_sent(sent, "^wa") == "wa", "second click withdraws all")

-- Deposit all from the command
H.command("/artisan deposit")
H.run(0.3)
sent = H.take_sent()
ok(find_sent(sent, "^da %d+ "), "deposit all: " .. table.concat(sent, ";"))

-- All characters view asks the server
H.click("artisan", "main", "view2")
H.run(0.3)
sent = H.take_sent()
ok(find_sent(sent, "^ac$"), "all characters asks: " .. table.concat(sent, ";"))
server("B|ac") server("A|Tester|1|641:3") server("A|Alt|0|4096:12") server("Z|ac")
H.run(0.3)
ok(canvas_texts("artisan", "main"):find("Alt", 1, true), "other characters listed")

-- Guilds tab: Craft N
server("B|gd") server("G|2|649|1500|20|2000|1") server("Z|gd")
H.bags[0][#H.bags[0] + 1] = { id = 656, count = 48 } -- the easiest recipe's material
W.go(3)
H.run(0.3)
ok(canvas_texts("artisan", "main"):find("Craft 12", 1, true), "guild Craft 12")
H.take_sent()
H.click("artisan", "main", "gcraft2")
H.run(0.3)
sent = H.take_sent()
ok(find_sent(sent, "^cq %d+ 12 1$"), "guild craft: " .. table.concat(sent, ";"))

-- Settings: HUD off hides it
H.settings.hud = false
H.fire("settings_changed", "hud", false)
ok(not ui.is_open("artisan_hud"), "HUD hidden by its setting")

-- The lobby: nothing breaks, nothing is sent
H.in_world = false
H.run(1)
ok(#H.take_sent() == 0, "quiet in the lobby")

ok(#H.log == 0, "no warnings: " .. table.concat(H.log, "; "))
io.write(("Timeless Artisan: %d checks passed\n"):format(n))
