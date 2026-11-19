-- Timeless NM Atlas's tests: lua tests/run.lua (from the mod folder). tests/sample_feed.txt is real output of the
-- server's nm_atlas module (modules/nm_atlas) for a search, a page and two monsters.
local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

H.mod_id = "timeless-atlas"
local ZONES = { [107] = "South Gustaberg", [127] = "Behemoth's Dominion" }
res.zone = function(id) return ZONES[id] end
local ITEMS = { [926] = { name = "Lizard Tail" }, [15351] = { name = "Bounding Boots" }, [852] = { name = "Lizard Skin" },
  [3341] = { name = "Beastly Shank" }, [3342] = { name = "Savory Shank" } }
res.item = function(id) return ITEMS[id] end
H.player.zone = 107
mod = { model_portraits = true }
dofile("init.lua")
local A = require("lib.atlas")
local P = require("lib.protocol")

local feed = {}
for line in io.lines("tests/sample_feed.txt") do feed[#feed + 1] = line end
local function play(from, to) for i = from, to do ok(H.line(feed[i]) == false, "hidden: " .. feed[i]:sub(1, 10)) end end
local function find(prefix) for i, l in ipairs(feed) do if l:sub(1, #prefix) == prefix then return i end end end
local function texts()
  local out = {}
  for _, d in ipairs(H.windows.atlas.el.atlas.draw or {}) do if d.type == "text" then out[#out + 1] = d.text end end
  return table.concat(out, " | ")
end

-- Opened from the menu: asks for the first page
local card = H.provided["timeless-atlas"].timeless_menu()
ok(card.title == "NM Atlas" and card.open == "//atlas", "a card on the Timeless menu")
H.command("/atlas")
ok(ui.is_open("atlas"), "opens")
H.run(0.2)
local sent = H.take_sent()
ok(sent[1] == "na list 1 0 0 0 0", "asks for page 1: " .. tostring(sent[1]))

-- Typing a search: asked once, a moment after the last key
H.event("atlas", { kind = "edit", id = "q", text = "liz" })
H.run(0.1)
H.event("atlas", { kind = "edit", id = "q", text = "lizzy" })
H.run(0.2)
ok(#H.take_sent() == 0, "not while typing")
H.run(0.4)
sent = H.take_sent()
ok(#sent == 1 and sent[1] == "na list 1 0 0 0 0 lizzy", "then the search: " .. tostring(sent[1]))

-- The answer
local e1 = find("NA1|E")
play(1, e1)
ok(#A.model.list.rows >= 1 and A.model.list.rows[1].name == "Leaping Lizzy", "the list")
H.check_draws("atlas")
local t = texts()
ok(t:find("Leaping Lizzy", 1, true) and t:find("South Gustaberg", 1, true) and t:find("Lv10-11", 1, true), "a row: name, zone, level")
ok(t:find("Can pop", 1, true), "lottery ready")

-- Pick her
H.click("atlas", "atlas", "row17215868")
H.run(0.2)
ok(H.take_sent()[1] == "na info 17215868", "asks for her detail")
local d1 = find("NA1|D|17215868")
local z1 = d1
while feed[z1] ~= "NA1|Z" do z1 = z1 + 1 end
play(d1, z1)
local i = A.model.info
ok(i and i.name == "Leaping Lizzy" and i.model == 328 and i.spawn.chance == 8, "her detail read")
H.check_draws("atlas")
t = texts()
ok(t:find("Lottery: when Rock Lizard is defeated", 1, true), "lottery explained: " .. t:sub(1, 600))
ok(t:find("Ice", 1, true) and t:find("Weak", 1, true), "weak to ice")
ok(t:find("Lizard Tail", 1, true) and t:find("24%", 1, true) and t:find("Bounding Boots", 1, true) and t:find("10%", 1, true), "drops with chances")
ok(t:find("Can pop now", 1, true), "status line")
ok(t:find("steal: ", 1, true), "steal")
local portrait
for _, d in ipairs(H.windows.atlas.el.atlas.draw) do if d.type == "portrait" then portrait = d end end
ok(portrait and portrait.model == 328 and portrait.spin and portrait.w == A.PORTRAIT - 4, "her model, turning, in a portrait")
local icons = 0
for _, d in ipairs(H.windows.atlas.el.atlas.draw) do if d.type == "icon" then icons = icons + 1 end end
ok(icons >= 3, "item icons for the drops")

-- King Behemoth: a timer, a pop, one-of groups, immunities
local d2 = find("NA1|D|17297441")
local z2 = d2
while feed[z2] ~= "NA1|Z" do z2 = z2 + 1 end
play(d2, z2)
H.run(0.1)
t = texts()
ok(t:find("King Behemoth", 1, true) and t:find("80,000 HP", 1, true), "header with HP")
ok(t:find("Can spawn in 20h 00m", 1, true), "the countdown")
ok(t:find("Land King", 1, true), "a Land King")
ok(t:find("Savory Shank", 1, true) and t:find("Trade at the ???", 1, true), "what to trade at its ???")
ok(t:find("One of these (100%):", 1, true), "a one-of group")
ok(t:find("Immune", 1, true), "immunities")
H.check_draws("atlas")
-- the countdown runs
H.run(125)
A.render()
ok(texts():find("Can spawn in 19h 58m", 1, true), "the countdown counts down")

-- Filters
H.take_sent()
H.click("atlas", "atlas", "up")
H.run(0.2)
ok(H.take_sent()[1] == "na list 1 1 0 0 0 lizzy", "up only")
H.click("atlas", "atlas", "here")
H.run(0.2)
ok(H.take_sent()[1] == "na list 1 1 107 0 0 lizzy", "this zone")
H.click("atlas", "atlas", "band5")
H.run(0.2)
ok(H.take_sent()[1] == "na list 1 1 107 71 0 lizzy", "level 71+")

-- Paging
play(find("NA1|H|1|"), find("NA1|E"))
local second = nil
for k = find("NA1|E") + 1, #feed do if feed[k]:match("^NA1|H|") then second = k break end end
local se = second
while feed[se] ~= "NA1|E" do se = se + 1 end
play(second, se)
ok(A.model.list.pages > 1, "many pages")
H.take_sent()
H.click("atlas", "atlas", "next")
H.run(0.2)
ok(H.take_sent()[1]:match("^na list 2 "), "next page")

-- Refreshes while open, quiet when closed
H.take_sent()
H.run(61)
local s = H.take_sent()
ok(#s >= 2 and s[1]:match("^na list") and s[2]:match("^na info "), "fresh statuses every minute: " .. table.concat(s, "; "))
H.command("/atlas")
ok(not ui.is_open("atlas"), "closes")
ok(#A.model.list.rows == 0 and A.model.info == nil, "closed: the answers let go")
ok(A.state.query == "lizzy" and A.state.selected, "but the search and the pick kept")
H.line("NA1|H|1|1|1")
ok(#A.model.list.rows == 0 and H.line("NA1|R|17215868|107|10|11|3|0|Leaping Lizzy") == false and #A.model.list.rows == 0,
  "a late answer after closing: hidden, not kept")
H.run(70)
ok(#H.take_sent() == 0, "quiet while closed")

-- Without the new client: a note instead of the model
mod.model_portraits = nil
H.command("/atlas")
H.run(0.2)
local again = H.take_sent()
local asked = table.concat(again, "; ")
ok(asked:find("na list ", 1, true) and asked:find("na info 17215868", 1, true), "opened again: the list and her detail asked afresh: " .. asked)
play(d1, z1)
A.render()
ok(texts():find("Update your TimelessXI client", 1, true), "an older client: a note in place of the model")

-- A former lottery NM the server now spawns on a timer (modules/timed_nms), and a scripted window (Roc)
mod.model_portraits = true
local function detail(lines)
  for _, l in ipairs(lines) do ok(H.line(l) == false, "hidden: " .. l:sub(1, 10)) end
  A.render()
  return texts()
end
H.line("NA1|H|1|1|1")
H.line("NA1|R|17187111|100|9|10|4|0|Jaggedy-Eared Jack")
H.line("NA1|E")
A.state.selected = 17187111
t = detail({ "NA1|D|17187111|100|9|10|26|200||||Jaggedy-Eared Jack", "NA1|K|beast|rabbit|forest hare|",
  "NA1|S|timed|180||0|0|Forest Hare|2400-2400", "NA1|X|4|0|-1", "NA1|G|100|926", "NA1|Z" })
ok(A.model.info.spawn.timed and A.model.info.spawn.timer[1] == 2400, "timed and its timer read")
ok(t:find("On a timer: it appears on its own about 40m after it was last defeated", 1, true) and t:find("place of Forest Hare", 1, true),
  "how a timed NM spawns: " .. t)
ok(not t:find("Lottery", 1, true), "no lottery text for it")
ok(t:find("Due: its timer is up", 1, true), "due")
ok(t:find("Due", 1, true) and A.model.list.rows[1].state == 4, "the row says due")
H.check_draws("atlas")
t = detail({ "NA1|D|17268851|120|80|82|300|9000|a||wind||Roc", "NA1|K|bird|roc|roc|sight",
  "NA1|S|scripted|0||0|0||75600-86400", "NA1|X|2|600|-1", "NA1|Z" })
ok(t:find("Respawns 21h 00m to 24h 00m after it", 1, true), "Roc: his window: " .. t)
ok(not t:find("by its own rules", 1, true), "not 'by its own rules' when the timer is known")
-- an older server: no timer field
t = detail({ "NA1|D|17215868|107|10|11|328|230|lc|fire||Leaping Lizzy", "NA1|S||300||8|3600|Rock Lizard", "NA1|X|3|0|-1", "NA1|Z" })
ok(t:find("Lottery: when Rock Lizard is defeated", 1, true), "an older server's line still reads")

-- The protocol
ok(P.percent(240) == "24%" and P.percent(5) == "0.5%" and P.percent(1000) == "100%", "percents")
ok(P.span(1200) == "20h 00m" and P.span(45) == "45m" and P.span(3000) == "2d 2h", "spans")
ok(P.feed(P.new_model(), "WM1|E", 0) == false, "someone else's line")

io.write(("%d checks passed\n"):format(n))
