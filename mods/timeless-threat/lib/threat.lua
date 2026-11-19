-- The enmity meter: an edge window with one block per monster you are fighting (your target first). Each block has
-- the monster's name and HP, then one bar per name on its enmity list, longest first, measured against whoever the
-- monster is attacking (that one's bar is full and marked with ►). Over 90% of the holder, a bar turns orange; your
-- own turns red and a warning shows on screen, because the next hit may turn the monster on you. Pointing at a bar
-- shows the exact cumulative and volatile enmity.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local protocol = require("lib.protocol")

local T = {}

T.WIN = "threat"
T.ROW_H = 14
T.ROW_GAP = 3
T.NAME_W = 82
T.DANGER = 90 -- percent of the holder's enmity at which a bar warns

T.model = protocol.new_model()
T.demo = nil -- the real model while the demo shows

function T.prefs()
  storage.character.threat = storage.character.threat or {}
  local p = storage.character.threat
  if p.visible == nil then p.visible = true end
  if p.warn == nil then p.warn = true end
  if p.compact == nil then p.compact = false end
  p.width = math.max(160, math.min(360, tonumber(p.width) or 220))
  p.rows = math.max(2, math.min(7, tonumber(p.rows) or 5))
  return p
end

local ME = { 228, 192, 119, 255 }        -- you: gold
local COMPANION = { 110, 166, 255, 255 } -- companions and trusts
local PET = { 82, 219, 204, 255 }
local OTHER = { 196, 200, 210, 255 }     -- other players
local HOLDER = { 236, 108, 96, 255 }
local BACK = { 26, 28, 38, 242 }
local EDGE = { 77, 82, 105, 180 }
local SHADOW = { 0, 0, 0, 220 }

local function pulse(now) return 0.5 + 0.5 * math.sin((now or 0) * 6.0) end

local function mix(a, b, t)
  return { math.floor(a[1] + (b[1] - a[1]) * t), math.floor(a[2] + (b[2] - a[2]) * t),
    math.floor(a[3] + (b[3] - a[3]) * t), math.floor((a[4] or 255) + ((b[4] or 255) - (a[4] or 255)) * t) }
end

local function my_index()
  if T.demo then return 1 end -- "You" in the sample fight
  local ok, p = pcall(game.player)
  return ok and p and p.index or -1
end

-- Who an index is: name and kind ("me", "companion", "pet", "player", "mob", "?").
function T.who(index)
  if T.demo and T.DEMO_NAMES[index] then return T.DEMO_NAMES[index][1], T.DEMO_NAMES[index][2] end
  if index == my_index() then
    local ok, p = pcall(game.player)
    return ok and p and p.name or "You", "me"
  end
  local ok, e = pcall(game.entity, index)
  if ok and e then
    if e.kind == "mob" then return e.name, "mob" end
    if e.trust then return e.name, "companion" end
    if (e.master or 0) ~= 0 then return e.name, "pet" end
    return e.name, "player"
  end
  -- A party member out of sight still has a name in the party list
  local okp, party = pcall(game.party)
  for _, m in ipairs(okp and party or {}) do
    if m.index == index then return m.name, "player" end
  end
  return "#" .. tostring(index), "?"
end

local KIND_COLOUR = { me = ME, companion = COMPANION, pet = PET, player = OTHER, mob = OTHER, ["?"] = S.FAINT }

-- Whether you are about to pull a monster: not its holder, and at DANGER% of the holder or more.
function T.danger(m)
  local me = my_index()
  if m.holder == me then return nil end
  for _, r in ipairs(m.rows) do
    if r.index == me then
      local pct = protocol.percent(m, r)
      if pct >= T.DANGER then return pct end
    end
  end
  return nil
end

-- Whether someone is about to take a monster from you (you hold it): their name and percent.
function T.losing(m)
  local me = my_index()
  if m.holder ~= me then return nil end
  for _, r in ipairs(m.rows) do
    if r.index ~= me then
      local pct = protocol.percent(m, r)
      if pct >= T.DANGER then return (T.who(r.index)), pct end
    end
  end
  return nil
end

function T.list()
  local list = protocol.list(T.model)
  if T.prefs().compact then
    for _, m in ipairs(list) do
      if m.mine then return { m } end
    end
    return { list[1] }
  end
  return list
end

-- One monster's block from y; returns the y below it.
function T.draw_mob(c, m, y, w, now)
  local p = T.prefs()
  local name = T.demo and (T.DEMO_NAMES[m.index] or {})[1] or nil
  if not name then
    local ok, e = pcall(game.entity, m.index)
    name = ok and e and e.name or "Monster"
  end
  local tw = c:text(0, y, name, { bold = true, colour = m.mine and S.GOLD or S.TEXT, w = w - 44 })
  c:text(w, y + 1, m.hpp .. "%", { size = 0.85, align = "right", colour = S.MUTED })
  local who, pct = T.losing(m)
  if who then
    c:text(math.min(tw + 8, w - 120), y + 1, who .. " " .. pct .. "%", { size = 0.85, colour = S.WARN, w = w - tw - 52 })
  end
  y = y + 18
  c:rect(0, y, w, 3, S.TRACK, { round = 1 })
  c:rect(0, y, math.max(1, w * m.hpp / 100), 3, S.HP, { round = 1 })
  y = y + 7

  local bx, bw = T.NAME_W, w - T.NAME_W
  for i, r in ipairs(m.rows) do
    if i > p.rows then break end
    local rname, kind = T.who(r.index)
    local holder = r.index == m.holder
    local share = protocol.percent(m, r)
    local colour = KIND_COLOUR[kind] or OTHER
    if not holder and share >= T.DANGER then
      colour = kind == "me" and mix(HOLDER, { 255, 255, 255, 255 }, 0.35 * pulse(now)) or S.WARN
    end
    if holder then c:text(0, y - 1, "►", { size = 0.8, colour = HOLDER }) end
    c:text(10, y - 1, rname, { size = 0.85, colour = kind == "me" and ME or S.MUTED, w = T.NAME_W - 14, bold = kind == "me" })
    c:rect(bx, y, bw, T.ROW_H, BACK, { round = 3 })
    c:rect(bx, y, math.max(3, bw * math.min(1, share / 100)), T.ROW_H, colour, { round = 3 })
    c:rect(bx, y, bw, T.ROW_H, holder and HOLDER or EDGE, { round = 3, line = 1 })
    c:text(bx + bw - 4, y, share .. "%", { size = 0.78, align = "right", colour = S.TEXT, outline = SHADOW, bold = true })
    c:tip(bx, y, bw, T.ROW_H, rname, string.format("Cumulative %d + volatile %d = %d%s", r.ce, r.ve, protocol.total(r),
      holder and "\nIt is attacking them." or ("\n" .. share .. "% of what the one it attacks has.")))
    y = y + T.ROW_H + T.ROW_GAP
  end
  if #m.rows == 0 then
    c:text(0, y, "No enmity yet.", { size = 0.85, colour = S.FAINT })
    y = y + 16
  end
  return y
end

function T.draw(c)
  local p = T.prefs()
  local now = os.clock()
  local y = 0
  if T.demo then
    c:text(0, 0, "(demo)", { size = 0.85, colour = S.FAINT })
    y = 16
  end
  for i, m in ipairs(T.list()) do
    if i > 1 then
      c:line(0, y + 2, p.width, y + 2, { 228, 192, 119, 50 }, 1)
      y = y + 8
    end
    y = T.draw_mob(c, m, y, p.width, now)
  end
end

-- Whether anything pulses (a warning): then it redraws often.
function T.animated()
  for _, m in ipairs(T.list()) do
    if T.danger(m) then return true end
  end
  return false
end

-- The on-screen warning when you are about to pull your target (or any monster, in the demo).
function T.warning()
  if not T.prefs().warn then
    pcall(hud.remove, "threat_warn")
    return
  end
  local text
  for _, m in ipairs(T.list()) do
    local pct = T.danger(m)
    if pct then
      local holder = protocol.holder_row(m)
      text = string.format("Careful: %d%% of %s's enmity", pct, holder and (T.who(holder.index)) or "the holder")
      break
    end
  end
  if text then
    local s = hud.screen()
    hud.text("threat_warn", { x = s.width / 2, y = math.floor(s.height * 0.22), text = text, size = 1.4, align = "center",
      colour = { 255, 196, 170, 255 }, outline = { 60, 0, 0, 230 }, outline_width = 2, font = "bold" })
  else
    pcall(hud.remove, "threat_warn")
  end
end

local built_w

function T.define()
  local w = T.prefs().width
  if built_w == w then return end
  built_w = w
  local was_open = ui.is_open(T.WIN)
  ui.window(T.WIN, {
    title = "Enmity",
    heading = "Enmity",
    edge = true,
    help = "Exact enmity from the server for the monsters you are fighting. ► is the one each monster attacks; over 90% of theirs, the next hit may turn it.",
    elements = { { type = "canvas", id = "list", w = w, h = 40, draw = {} } },
  })
  Canvas.add(T.WIN, "list", w, T.draw)
  if was_open then ui.show(T.WIN) end
end

function T.update()
  local want = T.prefs().visible and next(T.model.mobs) ~= nil
  if want and not ui.is_open(T.WIN) then
    ui.show(T.WIN)
  elseif not want and ui.is_open(T.WIN) then
    ui.close(T.WIN)
  end
  if ui.is_open(T.WIN) then Canvas.render(T.WIN, "list") end
  T.warning()
end

function T.setup()
  T.define()
  ui.on(T.WIN, function(win, e) Canvas.event(win, e) end)
end

-- A sample fight, to place the window without one.
T.DEMO_NAMES = { [901] = { "Leaping Lizzy", "mob" }, [902] = { "Goblin Pathfinder", "mob" }, [1] = { "You", "me" },
  [2] = { "Brokk", "companion" }, [3] = { "Kupipi", "companion" }, [4] = { "Carbuncle", "pet" } }

function T.demo_model()
  local m = protocol.new_model()
  protocol.feed(m, "EN1|M|901|h=58,t=2,me=1", 0)
  protocol.feed(m, "EN1|L|901|2:5200:3100;1:4100:3500;3:1800:900;4:300:0", 0)
  protocol.feed(m, "EN1|M|902|h=91,t=1,me=0", 0)
  protocol.feed(m, "EN1|L|902|1:900:600;2:200:100", 0)
  return m
end

function T.toggle_demo()
  if T.demo == nil then
    T.demo = T.model
    T.model = T.demo_model()
  else
    T.model = T.demo
    T.demo = nil
  end
  T.update()
end

return T
