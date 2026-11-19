-- The Town window (/town): headlines, what's happening here (who has the stall, your companions in town), the
-- caravan, market day and the latest gossip, from the server's TL1|T rows.

local Canvas = require("lib.canvas")
local S = require("lib.style")

local T = {}

T.WIN = "town"
T.W = 440

T.SECTIONS = {
  { key = "news", title = "Headlines" },
  { key = "here", title = "Around Town" },
  { key = "caravan", title = "The Caravan" },
  { key = "market", title = "Market Day" },
  { key = "gossip", title = "Gossip" },
}

T.model = nil
T.ask = nil -- set by init.lua: asks the server for fresh rows

function T.draw(c)
  local m = T.model
  local y = 0
  local name = (m and m.zone_name ~= "" and m.zone_name) or "Town"
  c:text(0, y, name, { bold = true, size = 1.15, colour = S.GOLD })
  c:button(T.W - 90, y - 2, 90, 24, "Refresh", { key = "refresh", on = function() if T.ask then T.ask() end end, tip = "Ask the town crier for the latest." })
  y = y + 34
  local rows = m and m.rows or {}
  if #rows == 0 then
    c:para(0, y, T.W, "Nothing to report. Town Life runs in Bastok Markets and Port Bastok (and on the South Gustaberg road).",
      { colour = S.MUTED })
    return
  end
  for _, sec in ipairs(T.SECTIONS) do
    local mine = {}
    for _, r in ipairs(rows) do if r.section == sec.key then mine[#mine + 1] = r.text end end
    if #mine > 0 then
      y = c:heading(0, y, T.W, sec.title)
      for _, text in ipairs(mine) do
        c:text(4, y, "•", { colour = S.BRONZE })
        y = c:para(18, y, T.W - 18, text, { size = 0.92, colour = sec.key == "gossip" and S.MUTED or S.TEXT }) + 2
      end
      y = y + 8
    end
  end
  c:grow(y)
end

function T.setup(model)
  T.model = model
  ui.window(T.WIN, {
    title = "Town",
    width = T.W + 20,
    help = "What's going on in town: headlines, who's minding the stall, the caravan, market day and the gossip.",
    elements = { { type = "canvas", id = "town", w = T.W, h = 200, draw = {} } },
  })
  Canvas.add(T.WIN, "town", T.W, T.draw)
  ui.on(T.WIN, function(win, e)
    if e.kind == "close" then return end
    Canvas.event(win, e)
  end)
end

function T.render()
  if ui.is_open(T.WIN) then Canvas.render(T.WIN, "town") end
end

function T.toggle()
  if ui.is_open(T.WIN) then
    ui.close(T.WIN)
  else
    ui.open(T.WIN)
    Canvas.render(T.WIN, "town")
    if T.ask then T.ask() end
  end
end

return T
