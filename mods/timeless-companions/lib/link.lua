-- The link to the server: both feeds' models, the queue of actions sent, and when to ask for fresh data.
--
-- The server prints the feeds as chat lines (CV1|... for Companion View, CM1|... for the manager); the chat filter
-- reads them into the models and hides them. Windows never block on the server: they draw what the models hold and
-- redraw when `dirty` says a batch came in.

local cv_feed = require("lib.feed_cv")
local cm_feed = require("lib.feed_cm")
local action = require("lib.action")

local L = {}

L.SEND_GAP = 0.5   -- seconds between actions sent, so busy clicks never flood the server
L.TIMEOUT = 5      -- seconds before "Updating..." gives up
L.SLOW = 8         -- actions that swap companions out and back in finish later: ask again after this

L.cv = cv_feed.new_model()
L.cm = cm_feed.new_model()
L.queue = {}
L.next_send = 0
L.dirty = { cv = false, cm = false, messages = false }
L.waiting = { cv = nil, cm = nil } -- os.clock() of the ask, while waiting
L.updated = { cv = nil, cm = nil } -- os.time() of the last batch
L.later = { cv = nil, cm = nil }   -- os.clock() when to ask again
L.is_open = { cv = function() return false end, cm = function() return false end }

local function now() return os.clock() end

-- Asks the server for a feed now (kind "cv" or "cm"), unless an ask is already out.
function L.ask(kind)
  if L.waiting[kind] and now() - L.waiting[kind] < L.TIMEOUT then return end
  L.waiting[kind] = now()
  table.insert(L.queue, 1, { text = kind == "cv" and "data" or "mdata", ask = kind })
  L.dirty[kind] = true
end

-- Asks for a feed again after `seconds`.
function L.ask_later(kind, seconds)
  local at = now() + seconds
  if not L.later[kind] or at > L.later[kind] then L.later[kind] = at end
end

-- Sends a !companion command as a window action. delay: ask both feeds again after that many seconds (the manager's
-- feed comes back at once by itself). in_chat: the server answers in chat. quiet: no answer at all.
function L.send(cmd, delay, in_chat, quiet)
  local prefix = in_chat and ">" or (quiet and "-" or "")
  table.insert(L.queue, { text = action.from_command(cmd, prefix) })
  if not in_chat and not quiet then L.waiting.cm = now() end
  if (delay or 0) > 0 then
    L.ask_later("cm", delay)
    L.ask_later("cv", delay)
  end
end

-- The chat filter: feed lines go into the models and are hidden. A manager answer (T) shows in chat as a plain line
-- while the manager window is closed.
function L.on_line(text)
  if type(text) ~= "string" then return nil end
  local line = cv_feed.extract(text)
  if line then
    if cv_feed.feed(L.cv, line) then
      L.waiting.cv = nil
      L.updated.cv = os.time()
    end
    L.dirty.cv = true
    return false
  end
  line = cm_feed.extract(text)
  if line then
    local msg = cm_feed.message_text(line)
    if msg and not L.is_open.cm() then
      return msg
    end
    if cm_feed.feed(L.cm, line, os.time()) then
      L.waiting.cm = nil
      L.updated.cm = os.time()
    end
    if msg then L.dirty.messages = true end
    L.dirty.cm = true
    return false
  end
  return nil
end

-- Runs often (a timer): sends the next queued action, asks again when due, gives up on asks with no answer.
function L.tick(auto_refresh)
  local t = now()
  if #L.queue > 0 and t >= L.next_send then
    local item = table.remove(L.queue, 1)
    local ok, err = pcall(action.send, item.text)
    if not ok then
      L.error = tostring(err)
      if item.ask then L.waiting[item.ask] = nil end
    else
      L.error = nil
    end
    L.next_send = t + L.SEND_GAP
  end
  for _, kind in ipairs({ "cv", "cm" }) do
    if L.waiting[kind] and t - L.waiting[kind] > L.TIMEOUT then
      L.waiting[kind] = nil
      L.dirty[kind] = true
    end
    if L.is_open[kind]() then
      if L.later[kind] and t >= L.later[kind] and #L.queue == 0 then
        L.later[kind] = nil
        L.ask(kind)
      elseif (auto_refresh or 0) > 0 and not L.waiting[kind] and L.updated[kind]
          and os.time() - L.updated[kind] >= auto_refresh then
        L.ask(kind)
      end
    else
      L.later[kind] = nil
    end
  end
end

-- "Updated 3:41:07 PM", "Updating...", or a note when the packet could not be sent.
function L.status(kind)
  if L.error then
    return "Can't reach the server: allow Send packets for Timeless Companions (Config > Extras).", true
  end
  if L.waiting[kind] then return "Updating...", false end
  if L.updated[kind] then return "Updated " .. os.date("%I:%M:%S %p", L.updated[kind]):gsub("^0", ""), false end
  return "", false
end

return L
