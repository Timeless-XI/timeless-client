-- The world map window: Vana'diel's conquest regions as a map, each in its owner's colour, with markers for what is
-- happening there now (open rifts, beastman warbands, Fomor bands, roaming Champions), and a panel listing them.
-- Click a region to list only its events; point at one for the details.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local protocol = require("lib.protocol")

local M = {}

M.WIN = "worldmap"
M.W, M.H = 620, 352
M.MAP_W = 410
M.PANEL_X = 424

M.model = protocol.new_model()
M.selected = nil -- a region id, a group key ("g1".."g6"), or nil for everywhere

function M.prefs()
  storage.character.worldmap = storage.character.worldmap or {}
  local p = storage.character.worldmap
  if p.beams == nil then p.beams = true end
  if p.alerts == nil then p.alerts = true end
  return p
end

-- Where each conquest region sits on the map (a sketch of Vana'diel: Quon in the north-west, Mindartia in the south,
-- the sky over Tu'Lia at the top), with a short name for its label.
M.REGIONS = {
  [0] = { 62, 146, "Ronfaure" }, [1] = { 130, 168, "Zulkheim" }, [2] = { 120, 116, "Norvallen" },
  [3] = { 268, 168, "Gustaberg" }, [4] = { 190, 124, "Derfland" }, [5] = { 166, 278, "Sarutabaruta" },
  [6] = { 226, 242, "Kolshushu" }, [7] = { 160, 222, "Aragoneu" }, [8] = { 128, 72, "Fauregandi" },
  [9] = { 156, 34, "Valdeaunia" }, [10] = { 214, 66, "Qufim" }, [11] = { 286, 82, "Li'Telor" },
  [12] = { 330, 206, "Kuzotz" }, [13] = { 350, 256, "Vollbow" }, [14] = { 254, 292, "Elshimo Low." },
  [15] = { 312, 306, "Elshimo Up." }, [16] = { 232, 22, "Tu'Lia" }, [17] = { 302, 128, "Movalpolos" },
  [18] = { 26, 200, "Tavnazia" },
}
M.FULL_NAMES = {
  [0] = "Ronfaure", [1] = "Zulkheim", [2] = "Norvallen", [3] = "Gustaberg", [4] = "Derfland", [5] = "Sarutabaruta",
  [6] = "Kolshushu", [7] = "Aragoneu", [8] = "Fauregandi", [9] = "Valdeaunia", [10] = "Qufim Island",
  [11] = "Li'Telor", [12] = "Kuzotz", [13] = "Vollbow", [14] = "Elshimo Lowlands", [15] = "Elshimo Uplands",
  [16] = "Tu'Lia", [17] = "Movalpolos", [18] = "Tavnazian Archipelago",
}
M.LINKS = { { 0, 1 }, { 0, 2 }, { 1, 2 }, { 1, 4 }, { 2, 4 }, { 2, 8 }, { 8, 9 }, { 8, 10 }, { 4, 10 }, { 10, 11 },
  { 4, 3 }, { 1, 3 }, { 3, 17 }, { 3, 12 }, { 12, 13 }, { 4, 7 }, { 7, 5 }, { 7, 6 }, { 5, 6 }, { 6, 14 }, { 14, 15 },
  { 11, 16 } }
M.SEA = { { 1, 18 }, { 1, 6 }, { 3, 6 } } -- ferries and sea routes, drawn dashed
M.CITIES = { { 46, 128, "San d'Oria" }, { 206, 100, "Jeuno" }, { 286, 188, "Bastok" }, { 160, 302, "Windurst" } }

-- Regions off the sketch, grouped
M.GROUPS = {
  { key = "g1", name = "Aht Urhgan", regions = { 28, 29, 30, 31, 32 } },
  { key = "g2", name = "The past", regions = { 33, 34, 35, 36, 37, 38, 39, 40 } },
  { key = "g3", name = "Abyssea", regions = { 41 } },
  { key = "g4", name = "Adoulin", regions = { 44, 45 } },
  { key = "g5", name = "Beyond", regions = { 23, 24, 25, 26, 27, 42, 43, 255 } },
  { key = "g6", name = "Cities", regions = { 19, 20, 21, 22 } },
}
local GROUP_OF = {}
for _, g in ipairs(M.GROUPS) do for _, r in ipairs(g.regions) do GROUP_OF[r] = g end end

M.OWNERS = {
  [0] = { "San d'Oria", { 196, 72, 72, 230 } }, [1] = { "Bastok", { 82, 132, 222, 230 } },
  [2] = { "Windurst", { 96, 192, 112, 230 } }, [3] = { "the beastmen", { 104, 86, 116, 240 } },
}
local UNKNOWN = { 90, 96, 110, 220 }

M.KINDS = {
  R = { colour = { 82, 219, 204, 255 }, letter = "R" },
  W = { colour = { 236, 108, 96, 255 }, letter = "W" },
  F = { colour = { 184, 122, 250, 255 }, letter = "F" },
  C = { colour = { 247, 204, 97, 255 }, letter = "C" },
}
local LEGENDARY = { 255, 140, 60, 255 }
local CHAMP = { "Champion", "Elite Champion", "Legendary Champion" }
local RIFT = { "Time Rift", "Ancient Rift", "Hero's Rift" }

function M.zone_name(id)
  local ok, name = pcall(res.zone, id)
  return ok and type(name) == "string" and name ~= "" and name or ("Zone " .. tostring(id))
end

function M.kind_colour(e)
  if e.k == "C" and e.c == 3 then return LEGENDARY end
  return M.KINDS[e.k].colour
end

-- "Ancient Rift", "Elite Champion (Lv. 34)", ...
function M.title(e)
  if e.k == "C" then return ("%s (Lv. %d)"):format(CHAMP[e.c or 1] or "Champion", e.l or 0) end
  if e.k == "W" then return ("Beastman warband (Lv. %d)"):format(e.l or 0) end
  if e.k == "F" then return "Fomor band" end
  return RIFT[e.c or 1] or "Time Rift"
end

-- The panel's bold line: "Elite", "Ancient Rift", "Warband", "Fomor band"
local SHORT_CHAMP = { "Champion", "Elite", "Legendary" }
function M.short(e)
  if e.k == "C" then return SHORT_CHAMP[e.c or 1] or "Champion" end
  if e.k == "R" then return RIFT[e.c or 1] or "Time Rift" end
  return e.k == "W" and "Warband" or "Fomor band"
end

function M.describe(e, now)
  local text = M.title(e) .. " in " .. M.zone_name(e.zone)
  local left = protocol.minutes_left(e, now)
  if left then text = text .. (" (closes in %d min)"):format(left) end
  return text
end

function M.region_name(r)
  if M.FULL_NAMES[r] then return M.FULL_NAMES[r] end
  local g = GROUP_OF[r]
  return g and g.name or "Elsewhere"
end

-- Whether event e belongs to the current selection.
function M.matches(e, sel)
  if sel == nil then return true end
  if type(sel) == "number" then return e.region == sel end
  local g = GROUP_OF[e.region]
  return g ~= nil and g.key == sel
end

function M.selection_name(sel)
  if sel == nil then return "Everywhere" end
  if type(sel) == "number" then return M.FULL_NAMES[sel] or "Region" end
  for _, g in ipairs(M.GROUPS) do if g.key == sel then return g.name end end
  return "Everywhere"
end

local function badge(c, x, y, colour, letter, ring)
  if ring then c:circle(x, y, 8, S.fade(colour, 0.45)) end
  c:circle(x, y, 6, colour)
  c:circle(x, y, 6, { 20, 16, 12, 220 }, 1)
  c:text(x, y - 6, letter, { size = 0.62, align = "center", bold = true, colour = { 24, 18, 12, 255 } })
end

function M.draw_map(c, now)
  c:rect(0, 0, M.MAP_W, M.H, { 22, 36, 54, 250 }, { to = { 12, 20, 32, 250 } })
  c:rect(0, 0, M.MAP_W, M.H, { 228, 192, 119, 90 }, { round = 6, line = 1 })
  local R = M.REGIONS
  for _, l in ipairs(M.LINKS) do
    local a, b = R[l[1]], R[l[2]]
    c:line(a[1], a[2], b[1], b[2], { 228, 192, 119, 46 }, 1.5)
  end
  for _, l in ipairs(M.SEA) do
    local a, b = R[l[1]], R[l[2]]
    for i = 0, 9, 2 do
      local t0, t1 = i / 10, (i + 1) / 10
      c:line(a[1] + (b[1] - a[1]) * t0, a[2] + (b[2] - a[2]) * t0, a[1] + (b[1] - a[1]) * t1, a[2] + (b[2] - a[2]) * t1,
        { 140, 180, 230, 60 }, 1)
    end
  end
  for _, city in ipairs(M.CITIES) do
    c:rect(city[1] - 3, city[2] - 3, 6, 6, { 240, 228, 200, 200 }, { round = 1 })
    c:text(city[1] + 6, city[2] - 7, city[3], { size = 0.62, colour = { 200, 196, 180, 170 } })
  end

  -- Events by region
  local by = {}
  for _, e in ipairs(M.model.events) do
    by[e.region] = by[e.region] or {}
    table.insert(by[e.region], e)
  end

  for id, r in pairs(R) do
    local x, y = r[1], r[2]
    local owner = M.model.owners[id]
    local info = M.OWNERS[owner]
    local sel = M.selected == id
    local hot = c:region(x - 16, y - 16, 32, 32, { key = "r" .. id, on = function()
      if M.selected == id then M.selected = nil else M.selected = id end
      M.render()
    end })
    local lines = { (info and ("Held by " .. info[1]) or "Owner unknown") .. "." }
    for _, e in ipairs(by[id] or {}) do lines[#lines + 1] = M.describe(e, now) end
    c:tip(x - 16, y - 16, 32, 32, M.FULL_NAMES[id], table.concat(lines, "\n"))
    c:circle(x, y, 11, info and info[2] or UNKNOWN)
    local edge = (sel or hot) and S.GOLD or (owner == 3 and { 236, 108, 96, 220 } or { 20, 18, 16, 200 })
    c:circle(x, y, 11, edge, (sel or hot) and 2 or 1.2)
    if owner == 3 then c:text(x, y - 7, "!", { size = 0.8, align = "center", bold = true, colour = { 255, 210, 200, 255 } }) end
    c:text(x, y + 12, r[3], { size = 0.64, align = "center", colour = sel and S.GOLD or { 214, 210, 198, 220 } })

    -- One badge per kind present, round the top right of the node
    local kinds, seen, legendary = {}, {}, false
    for _, e in ipairs(by[id] or {}) do
      if not seen[e.k] then seen[e.k] = true kinds[#kinds + 1] = e.k end
      if e.k == "C" and e.c == 3 then legendary = true end
    end
    table.sort(kinds, function(a, b) return ("RWFC"):find(a) < ("RWFC"):find(b) end)
    local angles = { -0.9, -0.1, 0.7, 1.5 }
    for i, k in ipairs(kinds) do
      local ang = angles[i]
      local bx, by2 = x + math.cos(ang) * 15, y + math.sin(ang) * 15
      local colour = (k == "C" and legendary) and LEGENDARY or M.KINDS[k].colour
      badge(c, bx, by2, colour, M.KINDS[k].letter, k == "R" or (k == "C" and legendary))
    end
  end

  -- Off the map: groups with something going on
  local gx = 8
  for _, g in ipairs(M.GROUPS) do
    local count = 0
    for _, e in ipairs(M.model.events) do if GROUP_OF[e.region] == g then count = count + 1 end end
    if count > 0 then
      local label = ("%s (%d)"):format(g.name, count)
      local w = Canvas.measure(label, 0.72, true) + 16
      c:button(gx, M.H - 24, w, 18, label, { size = 0.72, bold = true, style = M.selected == g.key and "on" or "plain",
        key = g.key, on = function()
          if M.selected == g.key then M.selected = nil else M.selected = g.key end
          M.render()
        end })
      gx = gx + w + 4
    end
  end
end

function M.moon_line()
  local ok, t = pcall(game.time)
  if not ok or type(t) ~= "table" then return nil end
  local day = t.weekday and select(2, pcall(res.day, t.weekday))
  local moon = t.moon_phase and select(2, pcall(res.moon, t.moon_phase))
  local parts = {}
  if type(moon) == "string" then parts[#parts + 1] = ("%s %d%%"):format(moon, t.moon or 0) end
  if type(day) == "string" then parts[#parts + 1] = day end
  local note
  if (t.moon or 50) >= 90 then
    note = "Full moon: the beastmen rally."
  elseif (t.moon or 50) <= 10 then
    note = "New moon: Fomors stir on dark nights."
  end
  return table.concat(parts, " · "), note
end

function M.draw_panel(c, now)
  local x, w = M.PANEL_X, M.W - M.PANEL_X
  local y = c:heading(x, 0, w, "Vana'diel now")
  local line, note = M.moon_line()
  if line then
    c:text(x, y, line, { size = 0.85, colour = S.MUTED, w = w })
    y = y + 16
  end
  if note then
    c:text(x, y, note, { size = 0.85, colour = S.WARN, w = w })
    y = y + 16
  end
  y = y + 6
  local name = M.selection_name(M.selected)
  c:text(x, y, name, { bold = true, colour = S.TEXT, w = w - 50 })
  if M.selected ~= nil then
    c:button(x + w - 44, y - 1, 44, 18, "All", { size = 0.78, key = "all", on = function()
      M.selected = nil
      M.render()
    end, tip = "List what is happening everywhere." })
  end
  y = y + 20
  if type(M.selected) == "number" then
    local info = M.OWNERS[M.model.owners[M.selected]]
    c:text(x, y, info and ("Held by " .. info[1]) or "Owner unknown", { size = 0.8, colour = info and info[2] or S.FAINT })
    y = y + 16
  end

  local list = {}
  for _, e in ipairs(M.model.events) do if M.matches(e, M.selected) then list[#list + 1] = e end end
  local bottom = M.H - 44
  if #list == 0 then
    c:text(x, y, M.model.version == 0 and "Waiting for the server..." or "Nothing stirs here right now.",
      { size = 0.85, colour = S.FAINT, w = w })
  end
  for i, e in ipairs(list) do
    if y > bottom - 20 then
      c:text(x, y, ("...and %d more."):format(#list - i + 1), { size = 0.8, colour = S.FAINT })
      break
    end
    local colour = M.kind_colour(e)
    badge(c, x + 6, y + 8, colour, M.KINDS[e.k].letter)
    local left = protocol.minutes_left(e, now)
    local right = left and (left .. "m") or (e.l and ("Lv." .. e.l) or "")
    local short = M.short(e)
    c:text(x + 17, y, short, { size = 0.82, bold = true, colour = colour, w = w - 56 })
    c:text(x + w, y, right, { size = 0.78, align = "right", colour = S.MUTED })
    c:text(x + 17, y + 13, M.zone_name(e.zone), { size = 0.76, colour = S.TEXT, w = w - 20 })
    c:tip(x, y, w, 28, M.title(e), M.describe(e, now) .. "\n" .. M.region_name(e.region))
    y = y + 32
  end

  -- Switches at the bottom
  local p = M.prefs()
  c:button(x, M.H - 20, 64, 18, p.beams and "Beams: on" or "Beams: off", { size = 0.74, style = p.beams and "on" or "off",
    key = "beamsbtn", tip = "Pillars of light over what is happening in your own zone.", on = function()
      p.beams = not p.beams storage.save() M.render()
    end })
  c:button(x + 68, M.H - 20, 64, 18, p.alerts and "Alerts: on" or "Alerts: off", { size = 0.74,
    style = p.alerts and "on" or "off", key = "alerts", tip = "A line in chat when a rift, warband, Fomor band or Legendary Champion appears.",
    on = function() p.alerts = not p.alerts storage.save() M.render() end })
  local ago = math.max(0, math.floor(now - (M.model.received or now)))
  if M.model.version > 0 then
    c:text(x + w, M.H - 18, ago < 5 and "live" or (ago .. "s ago"), { size = 0.7, align = "right", colour = S.FAINT })
  end
end

function M.draw(c)
  local now = os.clock()
  M.draw_map(c, now)
  M.draw_panel(c, now)
end

function M.render()
  if ui.is_open(M.WIN) then Canvas.render(M.WIN, "map") end
end

function M.setup()
  ui.window(M.WIN, {
    title = "World Map",
    width = M.W + 20,
    help = "What is happening across Vana'diel: conquest owners, open rifts, warbands, Fomor bands and roaming Champions. Click a region to list its events.",
    elements = { { type = "canvas", id = "map", w = M.W, h = M.H, draw = {} } },
  })
  Canvas.add(M.WIN, "map", M.W, M.draw)
  ui.on(M.WIN, function(win, e)
    if e.kind == "close" then return end
    Canvas.event(win, e)
  end)
end

function M.toggle()
  if ui.is_open(M.WIN) then
    ui.close(M.WIN)
  else
    ui.open(M.WIN)
    M.render()
  end
end

return M
