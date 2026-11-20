-- What each monster is casting or readying, from two sources:
--   the game's own action packets (the `action` event): category 8 starts a spell and 7 readies a move ("ca" in the
--   param's low half; anything else there is an interrupt), and 3, 4 and 11 finish them. These give the spell and a
--   base cast time (the spell's record), but no time at all for a move.
--   the server's CB1 lines: the real cast time (fast cast and the like counted) and a move's ready time.
-- Whichever comes first starts the bar; the other, within a second, only makes its time exact.
-- No drawing here, so it can be tested on its own.

local C = {}

C.casts = {}       -- mob index -> {kind, id, start, secs, exact, target, state = "casting" | "done" | "cut", at}
C.MATCH = 1.0      -- seconds: a packet and a line this close together are the same cast
C.LINGER = 1.0     -- seconds an interrupted or finished bar stays (its last look)
C.GUESS = 4        -- seconds a move with no known ready time shows for, at most
C.START = 0x6163   -- "ca"

local FINISH = { [3] = true, [4] = true, [11] = true }

function C.clear() C.casts = {} end

local function same(c, kind, id, now)
  return c and c.state == "casting" and c.kind == kind and c.id == id and now - c.start <= C.MATCH
end

-- A cast starts or its time becomes known. secs nil: not known (a move from a packet).
function C.begin(index, kind, id, now, secs, exact, target)
  local c = C.casts[index]
  if same(c, kind, id, now) then
    if exact and not c.exact then c.secs, c.exact = secs, true end
    if target and not c.target then c.target = target end
    return c
  end
  c = { kind = kind, id = id, start = now, secs = secs, exact = exact or false, target = target, state = "casting" }
  C.casts[index] = c
  return c
end

function C.finish(index, now, how)
  local c = C.casts[index]
  if c and c.state == "casting" then
    c.state, c.at = how, now
  end
end

-- A CB1 line, parsed.
function C.on_line(l, now)
  if l.kind == "cut" then
    C.finish(l.index, now, "cut")
  else
    C.begin(l.index, l.kind, l.id, now, l.secs, true, l.target)
  end
end

-- An `action` event. mob(id) gives the entity for a server id when it is a monster (nil otherwise); base(id) a
-- spell's base cast time in seconds.
function C.on_action(a, now, mob, base)
  if type(a) ~= "table" then return end
  local cat = a.category
  if cat ~= 7 and cat ~= 8 and not FINISH[cat] then return end
  local e = mob(a.actor)
  if not e then return end
  if FINISH[cat] then
    C.finish(e.index, now, "done")
    return
  end
  local first = a.targets and a.targets[1]
  local result = first and first.results and first.results[1]
  if ((a.param or 0) & 0xffff) ~= C.START then
    C.finish(e.index, now, "cut")
    return
  end
  local id = result and result.value
  if not id or id <= 0 then return end
  local kind = cat == 8 and "spell" or "skill"
  local target
  if first and first.id and first.id ~= a.actor then
    local t = game.entity(first.id)
    target = t and t.index or nil
  end
  local secs = kind == "spell" and base(id) or nil
  if secs and secs <= 0 then return end -- an instant cast
  C.begin(e.index, kind, id, now, secs, false, target)
end

-- How far along a cast is: fraction 0..1 (nil when its time isn't known) and the seconds left (nil likewise).
function C.progress(c, now)
  if not c.secs then return nil, nil end
  local t = now - c.start
  return math.max(0, math.min(1, t / c.secs)), math.max(0, c.secs - t)
end

-- Drops what is over: lingered long enough after finishing, or long past when it should have landed.
function C.expire(now)
  for index, c in pairs(C.casts) do
    if c.state ~= "casting" then
      if now - c.at > C.LINGER then C.casts[index] = nil end
    elseif now - c.start > (c.secs or C.GUESS) + 2 then
      C.casts[index] = nil
    end
  end
end

return C
