-- The small HUD window: what's crafting (with Stop), Repeat, your first favorites with Craft, what's stored with
-- Deposit all, and the auto-deposit switch. It never takes the keys.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local C = require("lib.core")

local H = {}

H.WIN = "artisan_hud"
H.CW = 240
H.FAVS = 4

H.open_window = function() end -- set by init.lua: (tab)

function H.draw(c)
  local v = C.queue_view()
  local y = 0
  if not C.ST.ready then
    c:text(0, y, C.waiting_too_long() and "The server isn't answering." or "Connecting…", { size = 0.8, colour = S.WARN, w = H.CW })
    y = y + 20
  elseif v then
    c:text(0, y, v.recipe and v.recipe.name or "Crafting…", { bold = true, colour = S.GOLD, size = 0.85, w = H.CW - 70 })
    c:text(H.CW, y + 1, ("%d / %d"):format(v.at, v.total), { align = "right", size = 0.8, colour = S.MUTED })
    c:bar(0, y + 20, H.CW - 62, 6, v.frac, S.GOLD)
    c:button(H.CW - 56, y + 14, 56, 20, "Stop", { style = "danger", size = 0.75, key = "stop", on = function() C.stop() end,
      tip = "Stops after the synth in progress." })
    y = y + 38
  else
    local last, n = C.last()
    local why = last and C.block(last)
    c:button(0, y, H.CW, 26, last and ("Repeat %s x%d"):format(last.name, n) or "Nothing to repeat yet", { style = "primary", size = 0.8, key = "repeat",
      enabled = last ~= nil and why == nil, tip = last and (why or "Queues your last craft again.") or "Craft something first.",
      on = last and function() C.craft(last, n) end or nil })
    y = y + 32
  end
  -- favorites
  local favs = C.favorites()
  for i = 1, math.min(H.FAVS, #favs) do
    local r = favs[i]
    local why = C.block(r)
    c:icon(0, y + 1, 18, "item", r.result)
    c:text(22, y + 2, r.name, { size = 0.8, w = H.CW - 86, colour = why and S.MUTED or S.TEXT })
    local o = C.ST.ready and C.odds(r)
    if o and not o.too_hard then
      c:region(22, y, H.CW - 86, 20, { title = r.name, tip = ("Success %s · HQ %s · skill-up %s"):format(C.L.pct(o.success), C.L.pct(o.hq), C.L.pct(o.skillup)) })
    end
    c:button(H.CW - 58, y, 58, 20, "Craft", { style = "plain", size = 0.75, key = "fav" .. r.id, enabled = why == nil,
      tip = why or "Crafts one.", on = function() C.craft(r, 1) end })
    y = y + 24
  end
  if #favs == 0 and C.ST.ready then
    c:text(0, y, "Your favorites show here (the star on a recipe).", { size = 0.75, colour = S.FAINT, w = H.CW })
    y = y + 20
  end
  -- storage
  c:line(0, y + 2, H.CW, y + 2, S.EDGE, 1)
  y = y + 8
  local total = 0
  for _, t in ipairs({ C.ST.store, C.ST.store_own }) do for _, n in pairs(t) do total = total + n end end
  c:text(0, y + 3, ("%s stored"):format(C.commas(total)), { size = 0.8, colour = S.MUTED })
  c:region(0, y, 90, 20, { key = "sto", on = function() H.open_window(4) end, tip = "Open the Storage tab." })
  c:button(H.CW - 102, y, 102, 20, "Deposit all", { size = 0.75, key = "da", enabled = C.ST.ready, on = function() C.deposit_all() end,
    tip = "Stores every crafting material in your bags." })
  y = y + 26
  local m = C.auto_mode()
  c:text(0, y + 3, "Auto-deposit", { size = 0.8, colour = S.MUTED })
  c:button(H.CW - 102, y, 102, 20, C.AUTO[m], { style = m == 0 and "off" or "on", size = 0.75, key = "auto",
    tip = "Click to change: Off, All materials, or Chosen items (ticked on the Storage tab).",
    on = function() C.set_auto_mode(m + 1) end })
  y = y + 26
  c:button(0, y, H.CW, 22, "Open Timeless Artisan", { size = 0.75, key = "open", on = function() H.open_window() end })
  c:grow(y + 22)
end

function H.render()
  if ui.is_open(H.WIN) then Canvas.render(H.WIN, "hud") end
end

function H.setup()
  ui.window(H.WIN, {
    title = "Artisan",
    hud = true,
    width = H.CW + 20,
    help = "Timeless Artisan at a glance: the queue, Repeat, your favorites, storage and auto-deposit. /artisan opens the full window; Settings > Mods > Timeless Artisan hides this.",
    elements = { { type = "canvas", id = "hud", w = H.CW, h = 200, draw = {} } },
  })
  Canvas.add(H.WIN, "hud", H.CW, H.draw)
  ui.on(H.WIN, function(win, e)
    if e.kind == "close" then return end
    Canvas.event(win, e)
  end)
end

function H.sync()
  local want = C.setting("hud", true) == true
  if want and not ui.is_open(H.WIN) then ui.show(H.WIN) H.render()
  elseif not want and ui.is_open(H.WIN) then ui.close(H.WIN) end
end

return H
