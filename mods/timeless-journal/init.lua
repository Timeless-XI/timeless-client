-- Timeless Journal: your missions and quests, and what to do next (replaces the questlog Ashita addon).
--
--   /ql        the Journal: Missions (your nation, Zilart, Promathia, Aht Urhgan: where to go and the next step),
--              Quests (the server's custom quests in progress and those you can start) and Retail (retail quests
--              you've accepted). /journal and /questlog work too.
--   Tracker    an edge window with your current missions, kept on screen (the Tracker switch, or /ql tracker).
--   Stories    your kept companions' stories as a book: each chapter's state, and the words of every chapter you've
--              lived (from the server's companions module, asked with "st data" when you open the tab; /ql stories).
--
-- Spoilers: with "Show full steps" off (the default), missions only say where to go next; each storyline's "Show step"
-- reveals its step for the session. The Journal asks the server's questlog module for your progress when it opens,
-- after you zone and after a cutscene or conversation ends (while it or the tracker is shown), with a packet, not chat,
-- and the answer lines are taken out of the chat log.

local action = require("lib.action")
local protocol = require("lib.protocol")
local journal = require("lib.journal")

local TIMEOUT = 5 -- seconds before "Updating..." gives up
local ql = { asked_at = -100, ask_at = nil, queued = false }

local function now() return os.clock() end

local function ask()
  if journal.state.waiting and now() - ql.asked_at < TIMEOUT then return end
  ql.asked_at = now()
  journal.state.waiting = true
  ql.queued = true
  journal.render_all()
end
journal.ask = ask

-- The Stories tab asks the companions module ("st data"); at most every few seconds
local stories_asked = -100
journal.ask_stories = function()
  if now() - stories_asked < 3 then return end
  stories_asked = now()
  pcall(action.send, "st data")
end

-- Ask in a few seconds (several events in a row only ask once).
local function ask_soon(seconds)
  if not journal.wanted() then return end
  local at = now() + seconds
  if ql.ask_at == nil or at < ql.ask_at then ql.ask_at = at end
end

journal.setup()

-- The feed: read it and take it out of the chat log.
filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  -- The Stories tab's book (companions_stories.lua)
  local sline = journal.stories.extract(text)
  if sline then
    if journal.stories.feed(journal.stories.model, sline, os.time()) == "book" then journal.render_all() end
    return false
  end
  local line = protocol.extract(text)
  if not line then return nil end
  if protocol.feed(journal.model, line, os.time()) then
    journal.state.waiting = false
    journal.state.updated = os.time()
    journal.refresh()
  end
  return false
end)

events.on("zone_change", function() ask_soon(4) end)
events.on("login", function() ask_soon(6) end)
-- A cutscene or conversation ended (status 4 is "in an event"): progress may have changed.
events.on("status_change", function(status, previous)
  if previous == 4 and status ~= 4 then ask_soon(2) end
end)

local function ql_command(sub)
  sub = (sub or ""):lower():gsub("^%s+", ""):gsub("%s+$", "")
  local tab = journal.tab_index(sub)
  if sub == "" then
    journal.toggle()
  elseif tab then
    journal.open(tab)
  elseif sub == "tracker" then
    journal.set_tracker(not journal.tracker_on())
    print(journal.tracker_on() and "Journal tracker on." or "Journal tracker off.")
  elseif sub == "spoilers" then
    journal.set_spoilers(not journal.spoilers())
    print(journal.spoilers() and "Journal: full mission steps shown." or "Journal: missions only say where to go next.")
  elseif sub == "update" or sub == "refresh" then
    ask()
  else
    print("/ql | /ql missions|quests|retail | /ql tracker | /ql spoilers | /ql update")
  end
end
commands.register("ql", ql_command)
commands.register("journal", ql_command)
commands.register("questlog", ql_command)

-- A key from the settings.
local bound
local function apply_key()
  if bound then pcall(binds.unset, bound) bound = nil end
  local ok, key = pcall(settings.get, "key_journal")
  if ok and type(key) == "string" and key ~= "" and pcall(binds.set, key, journal.toggle) then bound = key end
end
apply_key()
events.on("settings_changed", apply_key)

-- TimelessMenu's card.
mods.provide("timeless_menu", function()
  return {
    title = "Journal",
    desc = "Your missions and quests, and what to do next. Missions only say where to go unless you show full steps.",
    order = 20,
    open = "//ql",
    buttons = {
      { label = "Missions", command = "//ql missions" },
      { label = "Quests", command = "//ql quests" },
      { label = "Retail", command = "//ql retail" },
      { label = "Tracker", command = "//ql tracker", tip = "Show or hide your current missions on the screen's edge." },
    },
  }
end)

-- Every tenth of a second: send an ask when one is due, and give up on asks with no answer (a server without the
-- questlog module).
timer.every(0.1, function()
  local t = now()
  if ql.queued then
    ql.queued = false
    local ok = pcall(action.send, "ql data")
    if not ok then journal.state.waiting = false end
  end
  if ql.ask_at and t >= ql.ask_at then
    ql.ask_at = nil
    if journal.wanted() then ask() end
  end
  if journal.tracker_on() and not journal.model.loaded and not journal.state.waiting and t - ql.asked_at > 15 then
    ask()
  end
  if journal.state.waiting and t - ql.asked_at > TIMEOUT then
    journal.state.waiting = false
    journal.refresh()
  end
end)
