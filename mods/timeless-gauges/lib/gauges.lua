-- The gauge bar: an edge window with your main job's name and one row per gauge.
--   bar    a meter that fills (Faith, Ki, Beast Essence)
--   pips   stacks (Spell Weaving, Wild Bond, Hot Streak); a window running out shows its seconds
--   timer  something active that runs out (Unleash, Blood Scent)
--   cd     a cooldown that fills up to Ready (Never Yield, Pact Weaving)
--   flag   on or off (Last Ember, Code of Bushido)
-- A gauge that is full or ready glows. Pointing at one explains it.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local protocol = require("lib.protocol")

local G = {}

G.WIN = "gauges"
G.ROW_H = 16   -- a gauge's height
G.ROW_GAP = 6
G.LABEL_W = 100 -- room for labels
G.PIP_GAP = 3

G.model = protocol.new_model()
G.demo = nil -- the real model while the demo shows

-- The player's choices, kept per character.
function G.prefs()
  storage.character.gauges = storage.character.gauges or {}
  local p = storage.character.gauges
  if p.visible == nil then p.visible = true end
  if p.labels == nil then p.labels = true end
  if p.job == nil then p.job = true end
  p.width = math.max(100, math.min(400, tonumber(p.width) or 170))
  return p
end

-- Colour names the server uses.
G.COLOURS = {
  gold = { 247, 204, 97, 255 }, purple = { 184, 122, 250, 255 }, teal = { 82, 219, 204, 255 },
  blue = { 107, 168, 255, 255 }, green = { 122, 224, 122, 255 }, orange = { 255, 158, 71, 255 },
  red = { 245, 97, 97, 255 }, white = { 235, 240, 250, 255 },
  elem1 = { 255, 107, 64, 255 },  -- Fire
  elem2 = { 140, 217, 255, 255 }, -- Ice
  elem3 = { 115, 235, 140, 255 }, -- Wind
  elem4 = { 217, 173, 97, 255 },  -- Earth
  elem5 = { 204, 140, 255, 255 }, -- Thunder
  elem6 = { 89, 140, 255, 255 },  -- Water
  elem7 = { 255, 250, 204, 255 }, -- Light
  elem8 = { 140, 102, 179, 255 }, -- Dark
}

local BACK = { 26, 28, 38, 242 }
local EDGE = { 77, 82, 105, 180 }
local EMPTY = { 43, 46, 61, 255 }
local TEXT = { 247, 247, 255, 255 }
local SHADOW = { 0, 0, 0, 220 }

-- What each gauge does, for its tooltip: [job id .. ":" .. key]
G.TIPS = {
  ["3:faith"] = "Faith builds as you heal and cleanse others. At 100 your next Cure, Raise or Banish is a Miracle.",
  ["16:ess"] = "Beast Essence builds from blue magic, hits and weapon skills. At 100 the beast is Unleashed for 30 seconds.",
  ["16:weave"] = "Alternate physical and magical blue magic: +5% a stack, up to 3.",
  ["16:azure"] = "A physical spell under Chain Affinity opened a window: your next magical blue spell bursts.",
  ["4:attune"] = "Nuking the same element stacks Attunement (+5% a stack). A different element starts over.",
  ["4:ember"] = "Below a quarter of your MP: nukes +15% and black magic costs half.",
  ["10:encore"] = "Every few songs in a fight, the next one is an Encore: cast twice as fast and one Crescendo stack higher.",
  ["9:bond"] = "Wild Bond: grows with every worthy kill your pet makes with you.",
  ["9:scent"] = "Blood Scent on your target: you and your pet deal more damage to it.",
  ["17:streak"] = "Hot Streak: lucky rolls in a row. Three in two minutes and Fortune smiles on you.",
  ["14:bond"] = "Bonded Pair: you and your wyvern grow stronger the longer you fight together.",
  ["2:ki"] = "Ki builds from hits, kicks and counters. Chakra and Chi Blast spend it; full Ki makes your next weapon skill hit harder.",
  ["2:pp"] = "Pressure Points on your target: when it fills, your next weapon skill strikes it.",
  ["13:step"] = "Shadow Step: Innin from behind builds it. At 5 your next weapon skill strikes unseen.",
  ["13:wheel"] = "Cast elemental ninjutsu around the wheel (Fire, Ice, Wind, Earth, Thunder, Water). The sixth in a row completes it.",
  ["5:exploit"] = "Exploit Weakness: every enfeeble on your target adds to your damage, up to +15%.",
  ["5:surge"] = "Crimson Surge: spells left that are free and fast after Convert.",
  ["11:focus"] = "Focus: sweet-spot shots build it. At 5 your next shot is a sure critical hit.",
  ["11:prey"] = "Mark Prey: your damage to your prey grows the longer you hunt it.",
  ["11:ambush"] = "Camouflage ambush: your next shot deals +50% and adds no enmity.",
  ["12:hasso"] = "Hasso: your attack grows every round you stay in it.",
  ["12:draw"] = "Meditate's draw: your next weapon skill is an Iaijutsu.",
  ["12:bushido"] = "Code of Bushido: fighting one on one gives accuracy and critical hit rate.",
  ["15:bond"] = "Pact Bond with your avatar: grows with every worthy kill it shares with you.",
  ["15:weave"] = "Pact Weaving: release your avatar soon after its Blood Pact for a lingering spirit.",
  ["6:shadows"] = "From the Shadows: Hide readied a free Sneak Attack, and your first round adds no enmity.",
  ["6:crew"] = "Honor Amongst Thieves: more Thieves in the alliance mean more gil and Treasure Hunter.",
  ["1:never"] = "Never Yield: once every 5 minutes a killing blow leaves you at 1 HP.",
  ["1:duel"] = "Champion's Challenge: you called out a notorious monster.",
  ["1:unyield"] = "Unyielding: at 25% HP or less, Double Attack and attack go up.",
  ["8:chain"] = "Chain Reaper: worthy kills in quick succession stack attack.",
}

local function mix(a, b, t)
  return { math.floor(a[1] + (b[1] - a[1]) * t), math.floor(a[2] + (b[2] - a[2]) * t),
    math.floor(a[3] + (b[3] - a[3]) * t), math.floor((a[4] or 255) + ((b[4] or 255) - (a[4] or 255)) * t) }
end

local function dim(c, f)
  return { math.floor(c[1] * f), math.floor(c[2] * f), math.floor(c[3] * f), c[4] or 255 }
end

-- A slow pulse (0..1) for things that are ready.
local function pulse(now)
  return 0.5 + 0.5 * math.sin((now or 0) * 5.0)
end

-- The canvas's width for the current choices.
function G.width()
  local p = G.prefs()
  return (p.labels and G.LABEL_W or 0) + p.width
end

-- One gauge at (x, y), w wide.
function G.draw_gauge(c, x, y, w, g, now, tip)
  local h = G.ROW_H
  local colour = G.COLOURS[g.c or ""] or G.COLOURS.gold
  local ready = protocol.ready(g, now)
  local glow = ready and pulse(now) or 0
  local edge = ready and mix(EDGE, colour, 0.45 + 0.55 * glow) or EDGE
  local text = protocol.text(g, now)
  if g.t == "pips" then
    local m = math.max(1, g.m or 1)
    local pw = (w - G.PIP_GAP * (m - 1)) / m
    local v = g.v or 0
    for i = 1, m do
      local px = x + (i - 1) * (pw + G.PIP_GAP)
      local fill = i <= v and (ready and mix(colour, TEXT, 0.25 * glow) or colour) or EMPTY
      c:rect(px, y, pw, h, fill, { round = 3 })
      c:rect(px, y, pw, h, edge, { round = 3, line = ready and 1.5 or 1 })
    end
  else
    local frac = protocol.fill(g, now)
    c:rect(x, y, w, h, BACK, { round = 3 })
    if frac > 0 then
      local fill = ready and mix(colour, TEXT, 0.25 * glow) or (g.t == "cd" and dim(colour, 0.6) or colour)
      c:rect(x, y, math.max(4, w * frac), h, fill, { round = 3 })
    end
    c:rect(x, y, w, h, edge, { round = 3, line = ready and 1.5 or 1 })
  end
  if text ~= "" then
    c:text(x + w / 2, y + (h - 14) / 2, text, { size = 0.82, align = "center", colour = TEXT, outline = SHADOW,
      bold = true, w = w - 4 })
  end
  if tip then c:tip(x, y, w, h, g.l or "", tip) end
end

-- The canvas.
function G.draw(c)
  local p = G.prefs()
  local now = os.clock()
  local list = protocol.list(G.model)
  local y = 0
  if p.job then
    local tw = c:text(0, 0, protocol.job_name(G.model), { bold = true, colour = S.GOLD })
    if G.demo then c:text(tw + 8, 1, "(demo)", { size = 0.85, colour = S.FAINT }) end
    y = 22
  end
  local gx = p.labels and G.LABEL_W or 0
  for i, g in ipairs(list) do
    if p.labels then
      c:text(0, y, g.l or "", { size = 0.9, colour = S.MUTED, w = G.LABEL_W - 8 })
    end
    G.draw_gauge(c, gx, y, p.width, g, now, G.TIPS[tostring(G.model.job) .. ":" .. (g.k or "")] or "")
    y = y + G.ROW_H + (i < #list and G.ROW_GAP or 0)
  end
end

-- Whether anything on the bar moves on its own (a glow or a countdown): then it redraws often.
function G.animated()
  local now = os.clock()
  for _, g in ipairs(protocol.list(G.model)) do
    if protocol.ready(g, now) or protocol.left(g, now) > 0 then return true end
  end
  return false
end

local built_w = nil

function G.define()
  local w = G.width()
  if built_w == w then return end
  built_w = w
  local was_open = ui.is_open(G.WIN)
  ui.window(G.WIN, {
    title = "Job Gauges",
    heading = "Job Gauges",
    edge = true,
    help = "Your main job's own meters from the TimelessXI job perks. Full or ready gauges glow; point at one to see what it does.",
    elements = { { type = "canvas", id = "bar", w = w, h = 40, draw = {} } },
  })
  Canvas.add(G.WIN, "bar", w, G.draw)
  if was_open then ui.show(G.WIN) end
end

-- Shows the bar when it is wanted and has gauges, hides it otherwise; redraws it.
function G.update()
  local want = G.prefs().visible and #protocol.list(G.model) > 0
  if want and not ui.is_open(G.WIN) then
    ui.show(G.WIN)
  elseif not want and ui.is_open(G.WIN) then
    ui.close(G.WIN)
  end
  if ui.is_open(G.WIN) then Canvas.render(G.WIN, "bar") end
end

function G.setup()
  G.define()
  ui.on(G.WIN, function(win, e) Canvas.event(win, e) end)
end

-- Sample gauges (Blue Mage), to place the bar without a fight.
function G.demo_model()
  local m = protocol.new_model()
  local now = os.clock()
  protocol.feed(m, "GA1|J|16|n=3", now)
  protocol.feed(m, "GA1|G|1|k=ess,l=Essence,t=bar,v=72,m=100,h=0,s=,c=purple,tot=1,left=0", now)
  protocol.feed(m, "GA1|G|2|k=weave,l=Weave,t=pips,v=2,m=3,h=0,s=,c=teal,tot=30,left=24", now)
  protocol.feed(m, "GA1|G|3|k=azure,l=Azure Chain,t=timer,v=0,m=1,h=1,s=Burst!,c=blue,tot=8,left=6", now)
  return m
end

function G.toggle_demo()
  if G.demo == nil then
    G.demo = G.model
    G.model = G.demo_model()
  else
    G.model = G.demo
    G.demo = nil
  end
  G.update()
end

return G
