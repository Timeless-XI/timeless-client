-- Canvases drawn from a render function, with clickable regions, hover highlights and tooltips.
--
-- Each canvas has a render function(c) that draws into the builder `c` (rects, text, icons, bars, buttons) and marks
-- regions; Canvas.render turns that into the canvas's draw list and height. Window events go through Canvas.event:
-- a click runs the region's `on`, and the pointer moving onto another region redraws the canvas with its hover look.

local S = require("lib.style")

local M = {}

local canvases = {} -- "window/id" -> { win, id, w, render, regions, hover = {x, y}, hot = key }

local function key(win, id) return win .. "/" .. id end

-- Text width estimate in canvas units (the modern UI's small text is about 6.5 units a letter at size 1).
function M.measure(text, size, bold)
  local n = utf8.len(text or "") or #(text or "")
  return n * (bold and 7.1 or 6.5) * (size or 1)
end

-- Text cut to fit w units, with an ellipsis.
function M.fit(text, w, size, bold)
  text = tostring(text or "")
  if M.measure(text, size, bold) <= w then return text end
  local per = (bold and 7.1 or 6.5) * (size or 1)
  local n = math.max(1, math.floor(w / per) - 1)
  local cut = utf8.offset(text, n + 1)
  return (cut and text:sub(1, cut - 1) or text) .. "…"
end

-- Text broken into lines that fit w units (words kept whole unless one alone is too long).
function M.wrap(text, w, size, bold)
  local lines = {}
  for para in (tostring(text or "") .. "\n"):gmatch("(.-)\n") do
    local cur = ""
    for word in para:gmatch("%S+") do
      local try = cur == "" and word or (cur .. " " .. word)
      if M.measure(try, size, bold) <= w then
        cur = try
      else
        if cur ~= "" then lines[#lines + 1] = cur end
        while M.measure(word, size, bold) > w do
          local per = (bold and 7.1 or 6.5) * (size or 1)
          local n = math.max(1, math.floor(w / per))
          local cut = utf8.offset(word, n + 1) or (#word + 1)
          lines[#lines + 1] = word:sub(1, cut - 1)
          word = word:sub(cut)
        end
        cur = word
      end
    end
    lines[#lines + 1] = cur
  end
  return lines
end

-- The height wrapped text takes (16 units a line at size 1, plus 2 between lines).
function M.wrapped_height(text, w, size, bold)
  return #M.wrap(text, w, size, bold) * (18 * (size or 1))
end

local Builder = {}
Builder.__index = Builder

local function new_builder(entry)
  return setmetatable({ draw = {}, regions = {}, w = entry.w, h = 0, hover = entry.hover, entry = entry }, Builder)
end

function Builder:grow(y) if y > self.h then self.h = y end end

function Builder:rect(x, y, w, h, colour, o)
  o = o or {}
  self.draw[#self.draw + 1] = { type = "rect", x = x, y = y, w = w, h = h, colour = colour, round = o.round,
    line = o.line, to = o.to }
  self:grow(y + h)
end

function Builder:line(x1, y1, x2, y2, colour, width)
  self.draw[#self.draw + 1] = { type = "line", x1 = x1, y1 = y1, x2 = x2, y2 = y2, colour = colour, width = width or 1 }
end

function Builder:circle(x, y, r, colour, line)
  self.draw[#self.draw + 1] = { type = "circle", x = x, y = y, r = r, colour = colour, line = line }
end

function Builder:tri(x1, y1, x2, y2, x3, y3, colour)
  self.draw[#self.draw + 1] = { type = "tri", x1 = x1, y1 = y1, x2 = x2, y2 = y2, x3 = x3, y3 = y3, colour = colour }
end

function Builder:text(x, y, text, o)
  o = o or {}
  local size = o.size or 1
  text = tostring(text or "")
  if o.w then text = M.fit(text, o.w, size, o.bold) end
  self.draw[#self.draw + 1] = { type = "text", x = x, y = y, text = text, size = size, align = o.align,
    font = o.bold and "bold" or nil, colour = o.colour or S.TEXT, outline = o.outline }
  self:grow(y + 16 * size)
  return M.measure(text, size, o.bold)
end

function Builder:icon(x, y, size, kind, id, colour)
  if not id or id == 0 then return end
  self.draw[#self.draw + 1] = { type = "icon", x = x, y = y, size = size, kind = kind or "item", id = id, colour = colour }
  self:grow(y + size)
end

function Builder:tip(x, y, w, h, title, text)
  if (title or "") == "" and (text or "") == "" then return end
  self.draw[#self.draw + 1] = { type = "tip", x = x, y = y, w = w, h = h, title = title or "", text = text or "" }
end

-- A live portrait of a model (TimelessXI client with mod.model_portraits): o = { model, spin, turn, bg, frame, shape }.
function Builder:portrait(x, y, w, h, o)
  o = o or {}
  self.draw[#self.draw + 1] = { type = "portrait", x = x, y = y, w = w, h = h, model = o.model, entity = o.entity,
    spin = o.spin, turn = o.turn, bg = o.bg, frame = o.frame, frame_w = o.frame_w, shape = o.shape or "square",
    body = o.body }
  self:grow(y + h)
end

-- Wrapped text from (x, y), w units wide. Returns the y below it.
function Builder:para(x, y, w, text, o)
  o = o or {}
  local size = o.size or 1
  for _, line in ipairs(M.wrap(text, w, size, o.bold)) do
    self:text(x, y, line, { size = size, colour = o.colour, bold = o.bold })
    y = y + 18 * size
  end
  return y
end

-- Whether the pointer is over the box.
function Builder:over(x, y, w, h)
  local p = self.hover
  return p ~= nil and p.x >= x and p.x < x + w and p.y >= y and p.y < y + h
end

-- A clickable region: o.on (run on a click), o.tip / o.title (its tooltip), o.enabled (false: no click).
-- Returns whether the pointer is over it (and it is enabled), for the caller's hover look.
function Builder:region(x, y, w, h, o)
  o = o or {}
  local k = o.key or (#self.regions + 1)
  self.regions[#self.regions + 1] = { x = x, y = y, w = w, h = h, on = o.on, enabled = o.enabled ~= false, key = k }
  if o.tip or o.title then self:tip(x, y, w, h, o.title, o.tip) end
  return o.enabled ~= false and self:over(x, y, w, h)
end

-- A bar filled to frac, with an optional label at its left and text at its right drawn above it.
function Builder:bar(x, y, w, h, frac, colour, o)
  o = o or {}
  frac = math.max(0, math.min(1, frac or 0))
  self:rect(x, y, w, h, S.TRACK, { round = h / 2 })
  if frac > 0 then
    self:rect(x, y, math.max(h, w * frac), h, colour, { round = h / 2 })
  end
end

-- Button styles: primary (gold), danger (red), on (a lit switch), off (an unlit switch), plain.
local STYLES = {
  plain   = { fill = { 255, 255, 255, 18 }, hot = { 255, 255, 255, 40 }, edge = { 255, 255, 255, 46 }, text = S.TEXT },
  primary = { fill = { 170, 128, 72, 120 }, hot = { 200, 152, 86, 170 }, edge = { 228, 192, 119, 210 }, text = { 255, 240, 205, 255 } },
  danger  = { fill = { 150, 52, 46, 120 }, hot = { 190, 64, 56, 170 }, edge = { 236, 108, 96, 200 }, text = { 255, 222, 214, 255 } },
  on      = { fill = { 170, 128, 72, 150 }, hot = { 170, 128, 72, 150 }, edge = { 228, 192, 119, 230 }, text = { 255, 240, 205, 255 } },
  off     = { fill = { 255, 255, 255, 10 }, hot = { 255, 255, 255, 32 }, edge = { 255, 255, 255, 30 }, text = S.MUTED },
}

-- A rounded button. o: style, enabled, tip, title, on, size (text), colour (text), key, sub (a second, smaller line).
function Builder:button(x, y, w, h, label, o)
  o = o or {}
  local st = STYLES[o.style or "plain"]
  local enabled = o.enabled ~= false
  local hot = self:region(x, y, w, h, { on = o.on, tip = o.tip, title = o.title, enabled = enabled and o.on ~= nil,
    key = o.key })
  local k = enabled and 1 or 0.45
  self:rect(x, y, w, h, S.fade(hot and st.hot or st.fill, k), { round = o.round or 6 })
  self:rect(x, y, w, h, S.fade(hot and S.EDGE_HI or st.edge, k), { round = o.round or 6, line = 1 })
  local size = o.size or 0.9
  local colour = S.fade(o.colour or st.text, k)
  if o.sub then
    self:text(x + w / 2, y + h / 2 - 15 * size, label, { size = size, align = "center", colour = colour, bold = true, w = w - 6 })
    self:text(x + w / 2, y + h / 2 + 1, o.sub, { size = size * 0.85, align = "center", colour = S.fade(o.sub_colour or S.MUTED, k), w = w - 6 })
  else
    self:text(x + w / 2, y + (h - 15 * size) / 2, label, { size = size, align = "center", colour = colour, bold = o.bold, w = w - 6 })
  end
  self:grow(y + h)
end

-- A row of switch buttons, one lit: items {label, tip}; returns nothing. o.on(i) runs on a click on an unlit one.
function Builder:switch(x, y, h, items, lit, o)
  o = o or {}
  local cx = x
  for i, it in ipairs(items) do
    local w = it.w or math.max(64, M.measure(it.label, 0.9, true) + 22)
    self:button(cx, y, w, h, it.label, { style = i == lit and "on" or "off", tip = it.tip, enabled = o.enabled,
      on = (i ~= lit and o.on) and function() o.on(i) end or nil, key = (o.key or "sw") .. i })
    cx = cx + w + 4
  end
  return cx - x - 4
end

-- A panel: a faint rounded card.
function Builder:card(x, y, w, h, o)
  o = o or {}
  self:rect(x, y, w, h, o.fill or S.CARD, { round = o.round or 8 })
  self:rect(x, y, w, h, o.edge or S.EDGE, { round = o.round or 8, line = 1 })
end

-- A heading: gold text with a rule after it, and an optional faint note at the right.
function Builder:heading(x, y, w, text, note)
  local tw = self:text(x, y, text, { bold = true, colour = S.GOLD })
  if note then self:text(x + w, y + 1, note, { size = 0.85, colour = S.MUTED, align = "right", w = w - tw - 20 }) end
  self:line(x, y + 19, x + w, y + 19, { 228, 192, 119, 60 }, 1)
  return y + 26
end

----------------------------------------------------------------------------------------------------------------------

-- Registers a canvas element of a window. render(c) draws it.
function M.add(win, id, w, render)
  canvases[key(win, id)] = { win = win, id = id, w = w, render = render, regions = {}, hover = nil, hot = nil }
end

-- Draws a canvas now. Returns its height.
function M.render(win, id)
  local e = canvases[key(win, id)]
  if not e then return 0 end
  local c = new_builder(e)
  local ok, err = pcall(e.render, c)
  if not ok then
    c = new_builder(e)
    c:text(0, 0, "Error: " .. tostring(err), { colour = S.BAD, w = e.w })
    log.warn(tostring(err))
  end
  e.regions = c.regions
  e.hot = M.hot_key(e)
  local h = math.max(1, math.ceil(c.h + 2))
  pcall(ui.update, win, id, { w = e.w, h = h, draw = c.draw })
  return h
end

-- The key of the enabled region under the pointer, or nil.
function M.hot_key(e)
  local p = e.hover
  if not p then return nil end
  for i = #e.regions, 1, -1 do
    local r = e.regions[i]
    if r.enabled and r.on and p.x >= r.x and p.x < r.x + r.w and p.y >= r.y and p.y < r.y + r.h then
      return r.key
    end
  end
  return nil
end

-- Handles a window event for one of its canvases. Returns true when it was one.
function M.event(win, ev)
  local e = canvases[key(win, ev.id or "")]
  if not e then return false end
  if ev.kind == "click" then
    for i = #e.regions, 1, -1 do
      local r = e.regions[i]
      if ev.x >= r.x and ev.x < r.x + r.w and ev.y >= r.y and ev.y < r.y + r.h then
        if r.enabled and r.on then r.on(ev) end
        break
      end
    end
    return true
  elseif ev.kind == "hover" then
    e.hover = { x = ev.x, y = ev.y }
    if M.hot_key(e) ~= e.hot then M.render(win, ev.id) end
    return true
  elseif ev.kind == "leave" then
    e.hover = nil
    if e.hot ~= nil then M.render(win, ev.id) end
    return true
  end
  return false
end

-- For tests: a canvas's entry (its regions and hover).
function M._entry(win, id) return canvases[key(win, id)] end

return M
