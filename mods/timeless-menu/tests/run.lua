-- TimelessMenu's tests: lua tests/run.lua (from the mod folder).

local H = dofile("tests/harness.lua")
local n = 0
local function ok(cond, what)
  n = n + 1
  if not cond then error("FAIL " .. what, 2) end
end

-- The other mods: Journal running and giving its own card, Gauges installed but off, Companions running (an older
-- version that gives no card), and someone else's mod giving a card.
H.installed = {
  { id = "timeless-menu", name = "TimelessMenu", version = "1.0.0", loaded = true, enabled = true },
  { id = "timeless-companions", name = "Timeless Companions", version = "1.4.0", loaded = true, enabled = true },
  { id = "timeless-journal", name = "Timeless Journal", version = "1.0.0", loaded = true, enabled = true },
  { id = "timeless-gauges", name = "Timeless Gauges", version = "1.0.0", loaded = false, enabled = false },
  { id = "fishtracker", name = "Fish Tracker", version = "0.3", loaded = true, enabled = true },
}
H.provided["timeless-journal"] = { timeless_menu = function()
  return { title = "Journal", desc = "Your missions and quests.", order = 20, open = "//ql",
    buttons = { { label = "Missions", command = "//ql missions" }, { label = "Tracker", command = "//ql tracker" } } }
end }
H.provided["fishtracker"] = { timeless_menu = function()
  return { cards = { { title = "Fish Tracker", desc = "What bites where.", order = 90, open = "//fish" } } }
end }

H.mod_id = "timeless-menu"
dofile("init.lua")
local M = require("lib.menu")

ok(H.commands.tm and H.commands.timeless, "commands")
ok(H.hud.button and H.hud.button.kind == "image" and H.hud.button.path == "img/hourglass.png", "hourglass button")
ok(H.hud_handlers.button ~= nil, "button takes clicks")
ok(H.printed[1] and H.printed[1]:find("/tm", 1, true), "first-time hint")
ok(storage.account.menu.hinted, "hint only once")

-- Click the button: the menu opens with a card per mod, in order
H.hud_handlers.button("button", { kind = "click", button = "left", x = 1, y = 1, dx = 0, dy = 0 })
ok(ui.is_open("timeless_menu"), "click opens the menu")
local function texts()
  local out = {}
  for _, d in ipairs(H.windows.timeless_menu.el.cards.draw) do if d.type == "text" then out[#out + 1] = d.text end end
  return table.concat(out, " | ")
end
local t = texts()
local cm, cv, jo, ga, fi = t:find("Companion Manager", 1, true), t:find("Companion View", 1, true),
  t:find("Journal", 1, true), t:find("Job Gauges", 1, true), t:find("Fish Tracker", 1, true)
ok(cm and cv and jo and ga and fi, "all cards: " .. t)
ok(cm < cv and cv < jo and jo < ga and ga < fi, "in order")
ok(t:find("installed, not running", 1, true), "gauges: off")
ok(t:find("Server", 1, true) and t:find("Welcome", 1, true), "server row")
H.check_draws("timeless_menu")

-- Buttons run their commands
local Canvas = require("lib.canvas")
local function click_label(label)
  local e = Canvas._entry("timeless_menu", "cards")
  for _, d in ipairs(H.windows.timeless_menu.el.cards.draw) do
    if d.type == "text" and d.text == label then
      for _, r in ipairs(e.regions) do
        if d.x >= r.x and d.x <= r.x + r.w and d.y >= r.y and d.y <= r.y + r.h then
          H.event("timeless_menu", { kind = "click", id = "cards", x = r.x + r.w / 2, y = r.y + r.h / 2, button = "left" })
          return true
        end
      end
    end
  end
  return false
end
H.ran = {}
ok(click_label("Missions"), "Missions button there")
ok(H.ran[1] == "//ql missions", "runs //ql missions: " .. tostring(H.ran[1]))
ok(click_label("Settings") and H.ran[2] == "//cmgr settings", "companions' known tabs work without its card")
ok(click_label("Load") and H.ran[3] == "/lua load timeless-gauges", "Load starts the mod")
ok(click_label("Welcome") and H.ran[4] == "/say !welcome", "server command said")
H.run(0.5)
ok(texts():find("Sent !welcome: the answer is in chat.", 1, true), "note after a server command")

-- Every Open button runs its mod's command
local opens = 0
for _, d in ipairs(H.windows.timeless_menu.el.cards.draw) do if d.type == "text" and d.text == "Open" then opens = opens + 1 end end
ok(opens == 4, "Open on the running cards (Manager, View, Journal, Fish): " .. opens)

-- Gauges loads and gives its card: the menu picks it up within 2 s
H.installed[4].loaded = true
H.provided["timeless-gauges"] = { timeless_menu = function()
  return { title = "Job Gauges", desc = "Meters.", order = 30, open = "//gauges", buttons = { { label = "Demo", command = "//gauges demo" } } }
end }
H.run(2.5)
ok(not texts():find("installed, not running", 1, true) and texts():find("Demo", 1, true), "loaded mod shows its card")

-- A mod not installed says so, without buttons
table.remove(H.installed, 3) -- journal gone
H.provided["timeless-journal"] = nil
H.run(2.5)
ok(texts():find("not installed yet", 1, true), "missing mod noted")
H.check_draws("timeless_menu")

-- A card that errors doesn't break the menu
H.provided["fishtracker"].timeless_menu = function() error("boom") end
H.run(2.5)
ok(texts():find("Companion Manager", 1, true) and not texts():find("Fish Tracker", 1, true), "a broken card is left out")

-- Drag the button: it moves and remembers where
local x0 = H.hud.button.x
H.hud_handlers.button("button", { kind = "drag_start", button = "left", x = 0, y = 0, dx = 0, dy = 0 })
H.hud_handlers.button("button", { kind = "drag", button = "left", x = 0, y = 0, dx = -300, dy = 120 })
ok(H.hud.button.x == x0 - 300, "follows the drag")
H.hud_handlers.button("button", { kind = "drag_end", button = "left", x = 0, y = 0, dx = -300, dy = 120 })
ok(storage.account.menu.x == x0 - 300 and storage.account.menu.y, "place saved")
H.hud_handlers.button("button", { kind = "drag_start", button = "left", x = 0, y = 0, dx = 0, dy = 0 })
H.hud_handlers.button("button", { kind = "drag_end", button = "left", x = 0, y = 0, dx = -99999, dy = 99999 })
ok(H.hud.button.x == 0 and H.hud.button.y + H.hud.button.h <= 1080, "kept on screen")

-- Right-click: hide it; /tm button brings it back; /tm toggles the menu
H.hud_handlers.button("button", { kind = "click", button = "right", x = 0, y = 0, dx = 0, dy = 0 })
local m = H.menus[#H.menus]
ok(m and m.items[2] == "Hide this button", "right-click menu")
m.handler(2)
ok(H.hud.button == nil and storage.account.menu.hidden, "hidden")
H.command("/tm button")
ok(H.hud.button ~= nil and not storage.account.menu.hidden, "/tm button shows it")
H.command("/tm")
ok(not ui.is_open("timeless_menu"), "/tm closes")
H.command("/timeless")
ok(ui.is_open("timeless_menu"), "/timeless opens")
H.command("/tm reset")
ok(storage.account.menu.x == nil, "reset")

io.stdout:write(("ok: %d checks\n"):format(n))
