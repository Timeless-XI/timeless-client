-- The Journal's Stories tab: each kept companion's story as a book. A row of book spines picks the companion; its
-- page has the story's title, its chapters (I, II, III) and where each stands (done, under way and where to go,
-- ready to be told, or the bond it opens at), and the words of every chapter you've lived: how it began, how it
-- ended, and what the companion still says of it. Nothing of a chapter you haven't reached is shown.
--
-- From the server's companions module (companions_stories.lua), asked with "st data":
--   ST1|B|n=<count>
--   ST1|K|<slot>|n=<name>,k=<key>,lg=<1 legend>,ch=<chapters>,d=<done>,a=<under way>,b=<bond>,t=<title>
--   ST1|P|<slot>|<perk name>: <perk text>
--   ST1|C|<slot>.<ch>|st=<done|active|ready|locked>,need=<bond rank>,np=<points>,z=<zone>,t=<title>
--   ST1|X|<slot>.<ch>|<i|v|m><line>.<piece>|<text>   (pieces of one line join with nothing between)
--   ST1|E

local S = require("lib.style")
local Canvas = require("lib.canvas")

local B = {}

B.model = { list = nil, pending = nil, received = nil }
B.state = { slot = nil }

function B.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find("ST1|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

local function fields(body)
  local out = {}
  local t = (body or ""):match(",t=(.*)$")
  if t then out.t = t body = body:sub(1, #body - #t - 3) end
  for k, v in (body or ""):gmatch("([%w_]+)=([^,]*)") do out[k] = v end
  return out
end

-- Applies one line. Returns "book" when a book is complete, true for other lines of ours, false otherwise.
function B.feed(model, line, now)
  if type(line) ~= "string" then return false end
  local kind, rest = line:match("^ST1|(%u)|?(.*)$")
  if not kind then return false end
  local p = model.pending
  if kind == "B" then
    model.pending = { list = {}, by = {} }
  elseif kind == "E" then
    if p then
      model.list, model.pending, model.received = p.list, nil, now
      return "book"
    end
  elseif p and kind == "K" then
    local slot, body = rest:match("^(%d+)|(.*)$")
    if slot then
      local f = fields(body)
      local c = { slot = tonumber(slot), name = f.n or "?", key = f.k or "", legend = f.lg == "1", count = tonumber(f.ch) or 0,
        done = tonumber(f.d) or 0, active = tonumber(f.a) or 0, bond = tonumber(f.b) or 0, title = f.t or "", chapters = {} }
      p.list[#p.list + 1] = c
      p.by[c.slot] = c
    end
  elseif p and kind == "P" then
    local slot, text = rest:match("^(%d+)|(.*)$")
    local c = slot and p.by[tonumber(slot)]
    if c then c.perk = text end
  elseif p and kind == "C" then
    local slot, ch, body = rest:match("^(%d+)%.(%d+)|(.*)$")
    local c = slot and p.by[tonumber(slot)]
    if c then
      local f = fields(body)
      c.chapters[tonumber(ch)] = { state = f.st or "locked", need = f.need or "", points = tonumber(f.np) or 0, zone = f.z or "",
        title = f.t or "", intro = {}, victory = {}, memory = {} }
    end
  elseif p and kind == "X" then
    local slot, ch, part, idx, piece, text = rest:match("^(%d+)%.(%d+)|(%a)(%d+)%.(%d+)|(.*)$")
    local c = slot and p.by[tonumber(slot)]
    local chap = c and c.chapters[tonumber(ch)]
    if chap then
      local list = part == "i" and chap.intro or (part == "v" and chap.victory or chap.memory)
      idx, piece = tonumber(idx), tonumber(piece)
      list[idx] = list[idx] or {}
      list[idx][piece] = text
    end
  end
  return true
end

-- A line's pieces put back together
function B.line(pieces)
  local out = {}
  for i = 1, #pieces do out[#out + 1] = pieces[i] or "" end
  return table.concat(out)
end

local ROMAN = { "I", "II", "III", "IV", "V" }
local PAGE = { 238, 226, 196, 255 }
local INK = { 64, 46, 30, 255 }
local INK_SOFT = { 112, 88, 62, 255 }
local GILT = { 176, 128, 56, 255 }
local RIBBON = { 150, 46, 46, 255 }
local SPINES = { { 120, 40, 40, 255 }, { 40, 70, 120, 255 }, { 40, 100, 70, 255 }, { 110, 76, 30, 255 }, { 90, 50, 110, 255 },
  { 60, 60, 70, 255 } }

local STATE = {
  done = { "Complete", { 60, 120, 60, 255 } },
  active = { "Under way", { 176, 110, 30, 255 } },
  ready = { "Ready to be told", { 40, 100, 150, 255 } },
  locked = { "Not yet", { 140, 120, 100, 255 } },
}

function B.selected()
  local list = B.model.list or {}
  for _, c in ipairs(list) do if c.slot == B.state.slot then return c end end
  B.state.slot = list[1] and list[1].slot or nil
  return list[1]
end

local function draw_spines(c, w, render)
  local list = B.model.list
  local x = 0
  for i, comp in ipairs(list) do
    local sel = comp.slot == B.state.slot
    local tw = math.max(70, Canvas.measure(comp.name, 0.85, true) + 26)
    if x + tw > w then break end
    local col = SPINES[(i - 1) % #SPINES + 1]
    c:region(x, 0, tw, 34, { key = "spine" .. comp.slot, on = function()
      B.state.slot = comp.slot
      render()
    end })
    c:rect(x, sel and 0 or 4, tw, sel and 34 or 30, col, { round = 3 })
    c:line(x + 4, (sel and 0 or 4) + 5, x + tw - 4, (sel and 0 or 4) + 5, GILT, 1)
    c:line(x + 4, 29, x + tw - 4, 29, GILT, 1)
    c:text(x + tw / 2, (sel and 0 or 4) + 9, comp.name, { size = 0.85, bold = true, align = "center", colour = { 246, 232, 200, 255 }, w = tw - 8 })
    if comp.done >= comp.count and comp.count > 0 then c:circle(x + tw - 8, 30, 3, GILT) end
    x = x + tw + 4
  end
  return 40
end

local function quote_block(c, x, y, w, lines, colour, size)
  for _, pieces in ipairs(lines) do
    local text = B.line(pieces)
    if text ~= "" then
      y = c:para(x, y, w, "\u{201C}" .. text .. "\u{201D}", { size = size or 0.86, colour = colour }) + 4
    end
  end
  return y
end

function B.draw(c, w, render)
  local list = B.model.list
  if list == nil then
    c:text(0, 4, "Opening the book...", { colour = S.MUTED })
    return
  end
  if #list == 0 then
    c:para(0, 4, w, "No stories yet. Keep a companion (!companion keep) and grow your bond: once it trusts you, it tells you about its past, and you'll find the book here.", { size = 0.9, colour = S.MUTED })
    return
  end
  local comp = B.selected()
  local y = draw_spines(c, w, render)

  -- The page
  local top = y
  c:rect(0, top, w, 8, PAGE, { round = 4 })
  local px, pw = 18, w - 36
  y = top + 14
  c:rect(w - 46, top, 14, 34, RIBBON)
  c.draw[#c.draw + 1] = { type = "tri", x1 = w - 46, y1 = top + 34, x2 = w - 32, y2 = top + 34, x3 = w - 39, y3 = top + 28, colour = PAGE }
  c:text(px, y, comp.title, { size = 1.3, bold = true, colour = INK, w = pw - 50 })
  y = y + 24
  local sub = ("%s's story  ·  chapter %d of %d"):format(comp.name, math.min(comp.done + (comp.active > 0 and 1 or 0), comp.count), comp.count)
  if comp.legend then sub = sub .. "  ·  Legend" end
  c:text(px, y, sub, { size = 0.8, colour = INK_SOFT, w = pw - 50 })
  y = y + 18
  if comp.perk then
    local complete = comp.done >= comp.count
    y = c:para(px, y, pw, (complete and "Earned: " or "At the end: ") .. comp.perk, { size = 0.78, colour = complete and GILT or INK_SOFT })
  end
  c:line(px, y + 4, px + pw, y + 4, GILT, 1)
  c:line(px + pw / 2 - 30, y + 7, px + pw / 2 + 30, y + 7, GILT, 1)
  y = y + 14

  for ch = 1, comp.count do
    local chap = comp.chapters[ch]
    if chap then
      local st = STATE[chap.state] or STATE.locked
      c:circle(px + 12, y + 10, 12, chap.state == "done" and GILT or { 0, 0, 0, 0 })
      c:circle(px + 12, y + 10, 12, GILT, 1.5)
      c:text(px + 12, y + 3, ROMAN[ch] or tostring(ch), { size = 0.82, bold = true, align = "center", colour = chap.state == "done" and PAGE or INK })
      local known = chap.state == "done" or chap.state == "active"
      c:text(px + 32, y + 1, known and chap.title or "?", { size = 1.0, bold = true, colour = known and INK or INK_SOFT, w = pw - 160 })
      c:text(px + pw, y + 3, st[1], { size = 0.76, bold = true, align = "right", colour = st[2] })
      y = y + 26
      local note
      if chap.state == "active" then
        note = ("Bring %s to %s and face what waits there."):format(comp.name, chap.zone ~= "" and chap.zone or "the place they spoke of")
      elseif chap.state == "ready" then
        note = ("Have %s in your party, out of a fight, and listen."):format(comp.name)
      elseif chap.state == "locked" then
        note = chap.need ~= "" and ("Opens at %s bond (%d / %d)."):format(chap.need, comp.bond, chap.points) or "Opens after the chapter before."
        if ch > comp.done + 1 then note = "Opens after the chapter before." end
      end
      if note then y = c:para(px + 32, y, pw - 32, note, { size = 0.8, colour = st[2] }) + 2 end
      if known then
        y = quote_block(c, px + 32, y, pw - 32, chap.intro, INK)
      end
      if chap.state == "done" then
        c:text(px + 32 + (pw - 32) / 2, y, "~  how it ended  ~", { size = 0.72, align = "center", colour = GILT })
        y = y + 16
        y = quote_block(c, px + 32, y, pw - 32, chap.victory, INK)
        if #chap.memory > 0 then
          y = c:para(px + 32, y, pw - 32, ("%s still says: \u{201C}%s\u{201D}"):format(comp.name, B.line(chap.memory[1] or {})),
            { size = 0.76, colour = INK_SOFT }) + 2
        end
      end
      y = y + 10
    end
  end

  -- The page behind it all
  local h = y - top
  table.insert(c.draw, 1, { type = "rect", x = 0, y = top, w = w, h = h, colour = PAGE, round = 6 })
  table.insert(c.draw, 2, { type = "rect", x = 4, y = top + 4, w = w - 8, h = h - 8, colour = { 176, 128, 56, 90 }, line = 1, round = 4 })
  c:grow(y + 4)
end

return B
