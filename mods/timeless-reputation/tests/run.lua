-- Timeless Reputation's tests: lua tests/run.lua (from the mod folder).
local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

H.mod_id = "timeless-reputation"
dofile("init.lua")
local W = require("lib.window")
local protocol = require("lib.protocol")

-- Asks once after login, for the menu card
H.fire("login")
H.run(6)
ok(H.take_sent()[1] == "rp", "asks after login")
H.run(40)
ok(#H.take_sent() == 0, "not again while closed")

-- The answer, hidden from chat
local lines = { "RP1|B|n=1,r=6,p=1234", "RP1|A|0|f=130,l=3,lo=125,hi=225", "RP1|A|1|f=700,l=9,lo=613,hi=0",
  "RP1|A|2|f=0,l=1,lo=0,hi=50", "RP1|A|3|f=415,l=5,lo=325,hi=425", "RP1|A|4|f=553,l=8,lo=550,hi=613", "RP1|A|5|f=60,l=2,lo=50,hi=125", "RP1|E" }
for _, l in ipairs(lines) do ok(H.line(l) == false, "hidden: " .. l:sub(1, 8)) end
ok(H.line("Hrodgar : hello") ~= false, "ordinary chat untouched")
ok(W.model.loaded and W.model.nation == 1 and W.model.rank == 6, "nation and rank")
ok(W.model.areas[1].level == 9 and W.model.areas[3].fame == 415, "areas")
ok(protocol.progress(W.model.areas[1]) == 1, "the top level is full")
ok(math.abs(protocol.progress(W.model.areas[0]) - 0.05) < 0.001, "progress through a level")

-- The menu card
local card = H.provided["timeless-reputation"].timeless_menu()
ok(card.title == "Reputation" and card.open == "//reputation", "a card on the Timeless menu")
ok(card.desc:find("Bastok 9", 1, true) and card.desc:find("Norg 2", 1, true), "its summary: " .. card.desc)

-- Opened from the menu
H.command("/reputation")
ok(ui.is_open("reputation"), "opens")
H.run(0.2)
ok(H.take_sent()[1] == "rp", "asks on open")
H.check_draws("reputation")
local texts = {}
for _, d in ipairs(H.windows.reputation.el.rep.draw) do if d.type == "text" then texts[#texts + 1] = d.text end end
local all = table.concat(texts, " | ")
ok(all:find("A citizen of Bastok, rank 6.", 1, true), "nation line")
for _, name in ipairs({ "San d'Oria", "Bastok", "Windurst", "Jeuno", "Selbina & Rabao", "Norg" }) do ok(all:find(name, 1, true), name) end
ok(all:find("Legendary", 1, true) and all:find("As famous as anyone gets.", 1, true), "Bastok at the top")
ok(all:find("95 to Respected", 1, true), "San d'Oria: 95 to the next level")
ok(all:find("Unknown", 1, true), "Windurst unknown")
local icons = {}
for _, d in ipairs(H.windows.reputation.el.rep.draw) do if d.type == "icon" then icons[d.id] = d.kind end end
ok(icons[181] == "item" and icons[182] and icons[183] and icons[184], "the game's own flag icons for the nations and Jeuno")
ok(not icons[1135], "Selbina & Rabao and Norg keep their drawn emblems")
H.run(31)
ok(H.take_sent()[1] == "rp", "asks every 30 s while open")
H.click("reputation", "rep", "refresh")
H.run(0.2)
ok(H.take_sent()[1] == "rp", "Refresh asks")
H.command("/reputation")
ok(not ui.is_open("reputation"), "closes")
H.run(40)
ok(#H.take_sent() == 0, "quiet while closed")

-- Before any answer
local fresh = protocol.new_model()
ok(protocol.feed(fresh, "RP1|A|0|f=1,l=1,lo=0,hi=50") == true and not fresh.loaded, "an area line before B is ignored")
ok(protocol.feed(fresh, "WM1|E") == false, "someone else's line")

io.write(("%d checks passed\n"):format(n))
