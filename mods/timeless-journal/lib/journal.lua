-- The Journal window (Missions, Quests, Retail) and the tracker, an edge window with your current missions.
--
-- What they show comes from the server's questlog module (lib/protocol.lua reads its QL1 feed). The windows never wait
-- on the server: they draw what the model holds and redraw when a batch comes in.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local protocol = require("lib.protocol")

local J = {}

J.WIN = "journal"
J.TRACKER = "journal_tracker"
J.WIDTH = 580
local CW = 556 -- canvas width
local TW = 270 -- the tracker's canvas width

J.TABS = { "missions", "quests", "retail" }
J.TAB_LABELS = { "Missions", "Quests", "Retail" }

J.model = protocol.new_model()
J.state = {
  tab = 1,
  revealed = {}, -- storyline log -> true: steps shown in full this session
  waiting = false,
  updated = nil, -- os.time() of the last batch
}

-- Asks the server (set by init.lua: it owns the packet queue and the clock).
J.ask = function() end

-- The player's choices, kept per character.
local function prefs()
  storage.character.journal = storage.character.journal or {}
  return storage.character.journal
end
function J.spoilers() return prefs().spoilers == true end
function J.tracker_on() return prefs().tracker == true end

local COLOUR = {
  done = S.GOOD,
  accept = S.WARN,
  locked = { 220, 140, 128, 255 },
}

local STATE_TEXT = {
  active = "In progress",
  accept = "Next mission",
  locked = "Not yet available",
  done = "Complete",
  none = "Not started",
}

-- Where-only text when spoilers are hidden: "Head to <zone>." (a step with no zone shows its own text).
function J.step_text(step, full)
  if full or step.w == nil or step.w == "" then return step.t end
  return ("Head to %s."):format(step.w)
end

local function story_full(story)
  return J.spoilers() or J.state.revealed[story.log] == true or story.state ~= "active"
end

local function status_text()
  if J.state.waiting then return "Updating..." end
  if J.state.updated then return "Updated " .. os.date("%I:%M %p", J.state.updated):gsub("^0", "") end
  return ""
end

----------------------------------------------------------------------------------------------------------------------
-- The head: Refresh, the two switches, and when it last heard from the server.

function J.draw_head(c)
  c:button(0, 0, 92, 28, "Refresh", { style = "primary", enabled = not J.state.waiting, key = "refresh",
    tip = "Ask the server for your latest progress.", on = function() J.ask() end })
  c:button(100, 0, 132, 28, "Show full steps", { style = J.spoilers() and "on" or "off", key = "spoilers",
    tip = "Off: missions only say where to go next, so the story isn't spoiled. \"Show step\" reveals one.",
    on = function() J.set_spoilers(not J.spoilers()) end })
  c:button(240, 0, 92, 28, "Tracker", { style = J.tracker_on() and "on" or "off", key = "tracker",
    tip = "A small panel on the screen's edge with your current missions.",
    on = function() J.set_tracker(not J.tracker_on()) end })
  c:text(CW, 7, status_text(), { size = 0.85, colour = S.FAINT, align = "right", w = CW - 344 })
end

local function waiting_draw(c)
  c:para(0, 4, CW, J.state.waiting and "Asking the server..."
    or "No answer from the server yet. Press Refresh (the server needs the questlog module).", { colour = S.MUTED })
end

----------------------------------------------------------------------------------------------------------------------
-- Missions: one card per storyline.

local PAD = 12

-- The lines a storyline's card shows, and its height.
local function story_layout(story)
  local inner = CW - PAD * 2
  local full = story_full(story)
  local steps = protocol.steps_of(story)
  local texts = {}
  for i, step in ipairs(steps) do
    texts[#texts + 1] = (#steps > 1 and (i .. ". ") or "") .. (J.step_text(step, full) or "")
  end
  local h = PAD + 20 + 20 -- the story name, then its mission and state
  for _, t in ipairs(texts) do h = h + Canvas.wrapped_height(t, inner) end
  local guess = story.guess and story.state == "active"
  if guess then
    h = h + 4 + Canvas.wrapped_height("Best guess: your exact step wasn't recognised, so this is the nearest one.", inner, 0.85)
  end
  local reveal = not full and #steps > 0
  if reveal then h = h + 6 + 26 end
  return { texts = texts, guess = guess, reveal = reveal, h = h + PAD }
end

local function draw_story(c, y, story)
  local L = story_layout(story)
  local x, inner = PAD, CW - PAD * 2
  c:card(0, y, CW, L.h)
  local ty = y + PAD
  local nw = c:text(x, ty, story.story or "", { bold = true, colour = S.GOLD, w = inner - 130 })
  local of = tonumber(story.of) or 0
  if of > 0 then
    c:text(CW - PAD, ty + 1, ("%d / %d missions"):format(tonumber(story.num) or 0, of),
      { size = 0.85, colour = S.FAINT, align = "right" })
  end
  ty = ty + 20
  local heading = (story.code and story.code ~= "") and (story.code .. "  " .. (story.title or "")) or (story.title or "")
  local hx = x
  local state_text = STATE_TEXT[story.state] or ""
  local sw = Canvas.measure(state_text, 0.9)
  if heading ~= "" then
    hx = hx + c:text(x, ty, heading, { w = inner - sw - 12 }) + 10
  end
  c:text(hx, ty + 1, state_text, { size = 0.9, colour = COLOUR[story.state] or S.MUTED })
  ty = ty + 20
  for _, t in ipairs(L.texts) do
    ty = c:para(x, ty, inner, t)
  end
  if L.guess then
    ty = c:para(x, ty + 4, inner, "Best guess: your exact step wasn't recognised, so this is the nearest one.",
      { size = 0.85, colour = S.FAINT })
  end
  if L.reveal then
    c:button(x, ty + 6, 100, 24, "Show step", { key = "reveal" .. tostring(story.log),
      tip = "Shows the full step for this storyline (this session only). Turn on \"Show full steps\" to always see them.",
      on = function()
        J.state.revealed[story.log] = true
        J.render_all()
      end })
  end
  return y + L.h + 8
end

function J.draw_missions(c)
  if #J.model.stories == 0 then
    c:text(0, 4, "No missions yet.", { colour = S.MUTED })
    return
  end
  local y = 0
  for _, story in ipairs(J.model.stories) do
    y = draw_story(c, y, story)
  end
end

----------------------------------------------------------------------------------------------------------------------
-- Quests: the server's custom quests, in progress and those you can start.

local function quest_name(q)
  return (q.job and q.job ~= "") and ("[" .. q.job .. "] " .. (q.name or "")) or (q.name or "")
end

function J.draw_quests(c)
  local active, available = {}, {}
  for _, q in ipairs(J.model.quests) do
    table.insert(q.state == "active" and active or available, q)
  end
  local counts = J.model.counts or {}
  local total = tonumber(counts.qtotal) or 0
  local y = c:heading(0, 0, CW, "In progress",
    total > 0 and ("%d of %d finished"):format(tonumber(counts.qdone) or 0, total) or nil)
  if #active == 0 then
    c:text(0, y, "Nothing in progress.", { colour = S.MUTED })
    y = y + 22
  end
  for _, q in ipairs(active) do
    c:text(0, y, quest_name(q), { bold = true, colour = S.GOLD, w = CW })
    y = c:para(0, y + 20, CW, (q.text and q.text ~= "") and q.text or "Talk to the quest giver.") + 8
  end
  y = c:heading(0, y + 6, CW, "You can start", "on your current job")
  if #available == 0 then
    c:text(0, y, "Nothing new for this job right now.", { colour = S.MUTED })
    return
  end
  for _, q in ipairs(available) do
    c:text(0, y, quest_name(q), { bold = true, w = CW })
    y = c:para(0, y + 20, CW, q.text or "", { colour = S.MUTED }) + 8
  end
end

----------------------------------------------------------------------------------------------------------------------
-- Retail: the retail quests you've accepted, by area.

function J.draw_retail(c)
  local counts = J.model.counts or {}
  local y = c:heading(0, 0, CW, "Accepted", ("%d finished"):format(tonumber(counts.rdone) or 0))
  if #J.model.retail == 0 then
    c:text(0, y, "No retail quests in progress.", { colour = S.MUTED })
    return
  end
  local last = nil
  for _, q in ipairs(J.model.retail) do
    if q.area ~= last then
      if last ~= nil then y = y + 6 end
      c:text(0, y, q.area or "", { bold = true, colour = S.GOLD, w = CW })
      y = y + 20
      last = q.area
    end
    c:circle(8, y + 8, 2.5, S.MUTED)
    c:text(18, y, q.name or "", { w = CW - 18 })
    y = y + 20
  end
  c:para(0, y + 8, CW, "Steps for retail quests aren't tracked yet: the game's own quest log has their descriptions.",
    { size = 0.85, colour = S.FAINT })
end

----------------------------------------------------------------------------------------------------------------------
-- The tracker: one entry per storyline in progress, and how many custom quests are.

function J.draw_tracker(c)
  local y, any = 0, false
  for _, story in ipairs(J.model.stories) do
    if story.state == "active" or story.state == "accept" then
      any = true
      c:text(0, y, ((story.story or "") .. " " .. (story.code or "")):gsub("%s+$", ""), { bold = true, colour = S.GOLD, w = TW })
      y = y + 19
      local first = protocol.steps_of(story)[1]
      if first then
        y = c:para(0, y, TW, J.step_text(first, story_full(story)) or "", { size = 0.9, colour = S.MUTED })
      end
      y = y + 6
    end
  end
  local active = 0
  for _, q in ipairs(J.model.quests) do
    if q.state == "active" then active = active + 1 end
  end
  if active > 0 then
    any = true
    c:text(0, y, ("%d custom quest%s in progress (/ql quests)"):format(active, active == 1 and "" or "s"),
      { size = 0.85, colour = S.FAINT, w = TW })
    y = y + 18
  end
  if not any then
    c:text(0, y, J.model.loaded and "Nothing in progress." or "Asking the server...", { size = 0.9, colour = S.FAINT })
  end
end

----------------------------------------------------------------------------------------------------------------------

local function elements()
  local list = { { type = "canvas", id = "head", w = CW, h = 30, draw = {} } }
  if not J.model.loaded then
    list[#list + 1] = { type = "canvas", id = "waiting", w = CW, h = 40, draw = {} }
    return list
  end
  list[#list + 1] = { type = "tabs", id = "tabs", items = J.TAB_LABELS, at = J.state.tab }
  list[#list + 1] = { type = "canvas", id = "missions", w = CW, h = 100, draw = {}, page = 1 }
  list[#list + 1] = { type = "canvas", id = "quests", w = CW, h = 100, draw = {}, page = 2 }
  list[#list + 1] = { type = "canvas", id = "retail", w = CW, h = 100, draw = {}, page = 3 }
  return list
end

local built_loaded = nil

-- (Re)defines the window: with the waiting note until the first batch, then with the tabs.
function J.define()
  local loaded = J.model.loaded
  if built_loaded == loaded then return end
  built_loaded = loaded
  local was_open = ui.is_open(J.WIN)
  ui.window(J.WIN, {
    title = "Journal",
    width = J.WIDTH,
    help = "Your missions and quests, and what to do next. With \"Show full steps\" off, missions only say where to go.",
    elements = elements(),
  })
  if was_open and not ui.is_open(J.WIN) then ui.show(J.WIN) end
end

function J.define_tracker()
  ui.window(J.TRACKER, {
    title = "Journal",
    heading = "Current missions",
    edge = true,
    elements = { { type = "canvas", id = "list", w = TW, h = 40, draw = {} } },
  })
end

function J.render_all()
  if ui.is_open(J.WIN) then
    Canvas.render(J.WIN, "head")
    if J.model.loaded then
      Canvas.render(J.WIN, J.TABS[J.state.tab])
    else
      Canvas.render(J.WIN, "waiting")
    end
  end
  if ui.is_open(J.TRACKER) then Canvas.render(J.TRACKER, "list") end
end

-- A batch came in (or an ask gave up): switch to the tabs on the first one, then redraw.
function J.refresh()
  J.define()
  J.render_all()
end

local function on_event(win, e)
  if Canvas.event(win, e) then return end
  if e.kind == "set" and e.id == "tabs" then
    J.state.tab = e.index
    J.render_all()
  end
end

function J.setup()
  J.define()
  J.define_tracker()
  Canvas.add(J.WIN, "head", CW, J.draw_head)
  Canvas.add(J.WIN, "waiting", CW, waiting_draw)
  Canvas.add(J.WIN, "missions", CW, J.draw_missions)
  Canvas.add(J.WIN, "quests", CW, J.draw_quests)
  Canvas.add(J.WIN, "retail", CW, J.draw_retail)
  Canvas.add(J.TRACKER, "list", TW, J.draw_tracker)
  ui.on(J.WIN, on_event)
  ui.on(J.TRACKER, function(win, e) Canvas.event(win, e) end)
  if J.tracker_on() then ui.show(J.TRACKER) end
end

function J.is_open() return ui.is_open(J.WIN) end

function J.open(tab)
  if tab then
    J.state.tab = tab
    if J.model.loaded then pcall(ui.update, J.WIN, "tabs", { at = tab }) end
  end
  if not ui.is_open(J.WIN) then
    ui.show(J.WIN)
    J.ask()
  end
  J.render_all()
end

function J.toggle()
  if ui.is_open(J.WIN) then ui.close(J.WIN) else J.open() end
end

function J.set_spoilers(on)
  prefs().spoilers = on
  storage.save()
  J.render_all()
end

function J.set_tracker(on)
  prefs().tracker = on
  storage.save()
  if on then
    ui.show(J.TRACKER)
    if not J.model.loaded then J.ask() end
  else
    ui.close(J.TRACKER)
  end
  J.render_all()
end

-- Whether anything shown wants fresh data (the window or the tracker).
function J.wanted()
  return ui.is_open(J.WIN) or J.tracker_on()
end

function J.tab_index(name)
  for i, t in ipairs(J.TABS) do
    if t == name then return i end
  end
  return nil
end

return J
