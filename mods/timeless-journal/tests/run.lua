-- Timeless Journal's tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end
local function has(list, text)
  for _, v in ipairs(list) do if v == text then return true end end
  return false
end

H.mod_id = "timeless-journal"
dofile("init.lua")
local J = require("lib.journal")
local protocol = require("lib.protocol")

ok(H.commands.ql and H.commands.journal and H.commands.questlog, "commands registered")
ok(H.windows.journal and H.windows.journal_tracker, "both windows defined")
ok(H.windows.journal_tracker.spec.edge == true, "the tracker is an edge window")
ok(not ui.is_open("journal_tracker"), "tracker off by default")

-- Opening asks the server once, with the packet (not chat)
H.command("/ql")
ok(ui.is_open("journal"), "/ql opens")
H.run(0.2)
local sent = H.take_sent()
ok(#sent == 1 and sent[1] == "ql data", "asks 'ql data' once: " .. table.concat(sent, ","))
ok(J.state.waiting, "waiting after the ask")
ok(H.windows.journal.el.waiting ~= nil and H.windows.journal.el.tabs == nil, "waiting note before the first batch")
H.check_draws("journal")

-- A second ask while waiting is not sent
H.click("journal", "head", "refresh")
H.run(0.2)
ok(#H.take_sent() == 0, "refresh while waiting does nothing")

-- A batch comes in: lines are taken out of chat, the tabs appear
local feed = {
  "QL1|B|0|",
  "QL1|S|1|story=Bastok,state=active,code=5-1,title=The Final Image,num=12,of=20,guess=1",
  "QL1|W|1|k=1,t=Zeruhn Mines",
  "QL1|H|1|k=1,t=Speak with the Mythril Musketeer at the mines' far end, then report to Iron Eat",
  "QL1|H|1|k=1,t=er in the President's Office.",
  "QL1|S|2|story=Rise of the Zilart,state=accept,code=ZM3,title=Kazham's Chieftainess,num=2,of=17,guess=0",
  "QL1|W|2|k=1,t=Kazham",
  "QL1|H|2|k=1,t=Talk to Jakoh Wahcondalo in Kazham.",
  "QL1|S|3|story=Chains of Promathia,state=locked,code=,title=,num=0,of=18",
  "QL1|Q|1|state=active,job=PLD,name=Oath of the Paladin",
  "QL1|QH|1|t=Bring the blessed shield to the Cathedral.",
  "QL1|Q|2|state=available,job=PLD,name=Shield of Faith",
  "QL1|QH|2|t=Talk to Curilla at the Chateau d'Oraguille.",
  "QL1|R|0|area=Bastok,name=The Siren's Tear",
  "QL1|R|0|area=Bastok,name=Beauty and the Galka",
  "QL1|R|0|area=Windurst,name=Making Amens",
  "QL1|C|0|qdone=3,qtotal=12,rdone=40,ractive=3",
}
for _, l in ipairs(feed) do ok(H.line(l) == false, "feed line hidden: " .. l) end
ok(not J.model.loaded, "not loaded before Z")
ok(H.line("\30\1QL1|Z|0|\127\49") == false, "Z hidden (with colour codes around it)")
ok(J.model.loaded and not J.state.waiting and J.state.updated, "loaded after Z")
ok(H.line("Hello there") == "Hello there", "ordinary chat untouched")
ok(H.windows.journal.el.tabs ~= nil, "tabs after the first batch")
ok(#J.model.stories == 3 and #J.model.quests == 2 and #J.model.retail == 3, "model filled")
ok(protocol.steps_of(J.model.stories[1])[1].t:find("Iron Eater in the President", 1, true), "pieces joined")
ok(ui.is_open("journal"), "still open after redefining")

-- Missions: spoilers off by default, so the active story says where to go; "Show step" reveals it
local function texts(win, id)
  local out = {}
  for _, d in ipairs(H.windows[win].el[id].draw) do if d.type == "text" then out[#out + 1] = d.text end end
  return table.concat(out, " | ")
end
local m = texts("journal", "missions")
ok(m:find("Head to Zeruhn Mines.", 1, true), "active story: where only")
ok(not m:find("Mythril Musketeer", 1, true), "active story's step hidden")
ok(m:find("Talk to Jakoh", 1, true), "next mission (accept) shows its step")
ok(m:find("12 / 20 missions", 1, true), "counts")
ok(m:find("Best guess", 1, true), "guess note")
ok(m:find("Not yet available", 1, true), "locked state")
H.check_draws("journal")
H.click("journal", "missions", "reveal1")
ok(texts("journal", "missions"):find("Mythril Musketeer", 1, true), "Show step reveals it")
ok(H.region("journal", "missions", "reveal1") == nil, "Show step gone once revealed")

-- Full steps switch is saved per character
J.state.revealed = {}
H.click("journal", "head", "spoilers")
ok(storage.character.journal.spoilers == true and H.saved, "spoilers saved")
ok(texts("journal", "missions"):find("Mythril Musketeer", 1, true), "full steps shown")
H.command("/ql spoilers")
ok(storage.character.journal.spoilers == false, "/ql spoilers toggles back")

-- Quests and Retail tabs
H.event("journal", { kind = "set", id = "tabs", index = 2 })
local q = texts("journal", "quests")
ok(q:find("[PLD] Oath of the Paladin", 1, true) and q:find("Bring the blessed shield", 1, true), "quest in progress")
ok(q:find("3 of 12 finished", 1, true) and q:find("Shield of Faith", 1, true), "quests to start")
H.check_draws("journal")
H.command("/ql retail")
ok(J.state.tab == 3 and H.windows.journal.el.tabs.at == 3, "/ql retail switches tab")
local r = texts("journal", "retail")
ok(r:find("Windurst", 1, true) and r:find("Making Amens", 1, true) and r:find("40 finished", 1, true), "retail")
H.check_draws("journal")

-- Tracker: an edge window, saved, with the current missions
H.click("journal", "head", "tracker")
ok(ui.is_open("journal_tracker") and storage.character.journal.tracker == true, "tracker on")
local t = texts("journal_tracker", "list")
ok(t:find("Bastok 5-1", 1, true) and t:find("Head to Zeruhn Mines.", 1, true), "tracker: active story")
ok(t:find("Rise of the Zilart ZM3", 1, true), "tracker: next mission")
ok(not t:find("Promathia", 1, true), "tracker: locked story left out")
ok(t:find("1 custom quest in progress", 1, true), "tracker: quest count")
H.check_draws("journal_tracker")

-- Zoning asks again a few seconds later (wanted: the tracker is on), once
H.take_sent()
H.fire("zone_change", 230, 231)
H.run(2)
ok(#H.take_sent() == 0, "not yet after 2 s")
H.run(2.5)
local s2 = H.take_sent()
ok(#s2 == 1 and s2[1] == "ql data", "asks after zoning")
-- No answer: "Updating..." gives up after 5 s
H.run(5.5)
ok(not J.state.waiting, "gives up waiting")

-- A cutscene ending asks; other status changes don't
H.fire("status_change", 0, 1)
H.run(3)
ok(#H.take_sent() == 0, "engaging doesn't ask")
H.fire("status_change", 0, 4)
H.run(3)
ok(#H.take_sent() == 1, "an event ending asks")

-- Closed and no tracker: zoning doesn't ask
H.run(6)
ui.close("journal")
H.command("/ql tracker")
ok(not ui.is_open("journal_tracker"), "/ql tracker off")
H.take_sent()
H.fire("zone_change", 231, 232)
H.run(6)
ok(#H.take_sent() == 0, "nothing shown: no ask")

-- TimelessMenu card
local card = mods.call("timeless-journal", "timeless_menu")
ok(card.title == "Journal" and card.open == "//ql" and #card.buttons == 4, "menu card")

-- An empty journal draws its notes
J.model.stories, J.model.quests, J.model.retail = {}, {}, {}
J.open(1)
ok(texts("journal", "missions"):find("No missions yet.", 1, true), "empty missions")
H.check_draws("journal")

-- Wrapping keeps every line inside the canvas
local Canvas = require("lib.canvas")
local lines = Canvas.wrap("A very long sentence that keeps going past the edge of a narrow canvas, with Supercalifragilisticexpialidocious in it.", 120)
for _, l in ipairs(lines) do ok(Canvas.measure(l) <= 120, "wrapped line fits: " .. l) end
ok(#lines > 3, "wrapped into lines")

print(("ok: %d checks"):format(n))
io.stdout:write(("ok: %d checks\n"):format(n))
