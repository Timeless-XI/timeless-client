-- A card found: a small window with the big card (the monster's model turning slowly), where it came from, and the
-- cardbook a click away. It closes by itself after a while.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local cards = require("lib.cards")
local Art = require("lib.art")

local F = {}

F.WIN = "ironhand_found"
F.W = 300
F.SECONDS = 14
F.state = { id = nil, how = nil, at = 0 }
F.open_book = function() end

local HOW = { drop = "Dropped by a monster.", nm = "A notorious monster's own card!", champion = "A Champion's foil!",
  won = "Won at the Steaming Sheep's table.", legend = "Given to you across the table: their own card!", gm = "A gift." }

function F.draw(c)
  local id = F.state.id
  local card = id and cards.get(id)
  if not card then return end
  c:text(F.W / 2, 0, "A new card!", { size = 1.2, bold = true, colour = S.GOLD, align = "center" })
  Art.card(c, (F.W - Art.W * 1.6) / 2, 26, 1.6, id, { portrait = true, detail = "big" })
  local y = 26 + Art.H * 1.6 + 8
  c:text(F.W / 2, y, HOW[F.state.how] or HOW.drop, { size = 0.85, colour = S.TEXT, align = "center", w = F.W })
  c:button((F.W - 140) / 2, y + 24, 140, 28, "Open cardbook", { style = "primary", on = function() ui.close(F.WIN) F.open_book() end, key = "book" })
end

function F.setup()
  ui.window(F.WIN, { title = "Ironhand", width = F.W + 20, elements = { { type = "canvas", id = "found", w = F.W, h = 420, draw = {} } } })
  Canvas.add(F.WIN, "found", F.W, F.draw)
  ui.on(F.WIN, function(win, e)
    if e.kind == "close" then F.state.id = nil return end
    Canvas.event(win, e)
  end)
end

function F.show(id, how, t)
  F.state.id, F.state.how, F.state.at = id, how, t
  ui.open(F.WIN)
  Canvas.render(F.WIN, "found")
end

function F.tick(t)
  if F.state.id and t - F.state.at >= F.SECONDS and ui.is_open(F.WIN) then
    ui.close(F.WIN)
    F.state.id = nil
  end
end

return F
