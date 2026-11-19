-- A stand-in for vana-client's mod API, enough to load the mod, feed it lines, click its windows and dump what it
-- draws. Run from the mod folder: lua tests/run.lua

local H = { windows = {}, handlers = {}, sent = {}, filters = {}, commands = {}, timers = {}, prompts = {}, menus = {},
  clock = 1000, settings = { auto_refresh = 0 }, log = {} }

package.path = "./?.lua;./?/init.lua;" .. package.path

local real_clock = os.clock
os.clock = function() return H.clock end

local function copy(t)
  if type(t) ~= "table" then return t end
  local o = {}
  for k, v in pairs(t) do o[k] = copy(v) end
  return o
end

ui = {}
function ui.window(id, spec)
  local w = H.windows[id] or { open = false }
  w.spec = copy(spec)
  w.el = {}
  for _, e in ipairs(w.spec.elements or {}) do
    assert(e.type, "element without a type")
    if e.type ~= "text" and e.type ~= "separator" and e.type ~= "gauge" then assert(e.id, e.type .. " needs an id") end
    if e.id then
      assert(not w.el[e.id], "two elements with id " .. e.id)
      w.el[e.id] = e
    end
  end
  H.windows[id] = w
end
function ui.update(id, el, fields)
  local w = assert(H.windows[id], 'no window "' .. id .. '"')
  local e = assert(w.el[el], 'no element "' .. el .. '" in "' .. id .. '"')
  for k, v in pairs(fields) do e[k] = copy(v) end
end
function ui.get(id, el)
  local e = H.windows[id].el[el]
  if e.type == "input" or e.type == "text" then return { text = e.text or "" } end
  return { at = e.at }
end
function ui.open(id) assert(H.windows[id], "no window " .. id).open = true end
ui.show = ui.open
function ui.close(id) if H.windows[id] then H.windows[id].open = false end end
function ui.is_open(id) return H.windows[id] ~= nil and H.windows[id].open end
function ui.on(id, f) H.handlers[id] = f end
function ui.prompt(text, buttons, handler)
  if type(buttons) == "function" then handler, buttons = buttons, nil end
  H.prompts[#H.prompts + 1] = { text = text, buttons = buttons, handler = handler }
end
function ui.menu(items, handler) H.menus[#H.menus + 1] = { items = items, handler = handler } end
ui.native = { rect = function(name) if name == "party" then return { name = "party", x = 1640, y = 560, w = 260, h = 300, shown = true } end end }

packets = {}
function packets.inject_out(id, body)
  assert(id == 0x0B5, "packet id")
  assert(#body <= 504, "body too long")
  H.sent[#H.sent + 1] = body
end
H.inject = packets.inject_out

filters = {}
function filters.chat(f) H.filters[#H.filters + 1] = f return #H.filters end

-- The game's own commands (from FFXiMain.dll's table): the client refuses a mod command of the same name
H.game_commands = {}
for line in io.lines("tests/game_commands.txt") do H.game_commands[line:sub(2)] = true end
commands = {}
function commands.register(name, f)
  name = name:gsub("^/", ""):lower()
  assert(name ~= "lua" and not H.game_commands[name], "/" .. name .. " is a game command: give yours another name")
  H.commands[name] = f
end
for line in io.lines("mod.toml") do
  local n = line:match("^%[commands%.([%w_]+)%]")
  assert(not n or not H.game_commands[n:lower()], "[commands." .. tostring(n) .. "]: /" .. tostring(n) .. " is a game command")
end

timer = {}
function timer.every(s, f) H.timers[#H.timers + 1] = f end

settings = {}
function settings.get(k) return H.settings[k] end

-- Other mods: H.installed (mods.installed), H.provided[mod][name] = function (mods.provide / mods.call)
H.provided, H.installed, H.ran, H.printed = {}, {}, {}, {}
H.mod_id = "this"
mods = {
  provide = function(name, f) H.provided[H.mod_id] = H.provided[H.mod_id] or {} H.provided[H.mod_id][name] = f end,
  call = function(id, name, ...)
    local p = H.provided[id]
    if not p then error("mods.call: no mod " .. id .. " loaded") end
    return copy(p[name](...))
  end,
  installed = function() return copy(H.installed) end,
  list = function()
    local out = {}
    for _, m in ipairs(H.installed) do
      if m.loaded then
        local provides = {}
        for n in pairs(H.provided[m.id] or {}) do provides[#provides + 1] = n end
        out[#out + 1] = { id = m.id, name = m.name, version = m.version, provides = provides }
      end
    end
    return out
  end,
  emit = function() end,
}
commands.run = function(line) H.ran[#H.ran + 1] = line end
print = function(...)
  local t = {}
  for i = 1, select("#", ...) do t[#t + 1] = tostring((select(i, ...))) end
  H.printed[#H.printed + 1] = table.concat(t, " ")
end

log = { warn = function(m) H.log[#H.log + 1] = m end, info = function() end }

-- Events
H.events = {}
events = { on = function(name, f) H.events[name] = H.events[name] or {} table.insert(H.events[name], f) end }
function H.fire(name, ...)
  for _, f in ipairs(H.events[name] or {}) do f(...) end
end

-- The live game: H.party (Member list) and H.entities (server id -> Entity)
H.party, H.entities = {}, {}
game = {
  party = function() return H.party end,
  entity = function(id)
    if H.entities[id] then return H.entities[id] end
    for _, e in pairs(H.entities) do if e.index == id then return e end end
    return nil
  end,
}

-- HUD: elements by id; checks the fields it is given
H.hud, H.hud_handlers = {}, {}
local function hud_make(kind, required)
  return function(id, spec)
    for _, k in ipairs(required or {}) do assert(spec[k] ~= nil, kind .. " " .. id .. " needs " .. k) end
    for k, v in pairs(spec) do
      if (k == "x" or k == "y" or k == "w" or k == "h" or k == "size") then
        assert(type(v) == "number" and v == v, kind .. " " .. id .. "." .. k .. " is not a number")
      end
    end
    local e = copy(spec)
    e.kind = kind
    H.hud[id] = e
  end
end
hud = {
  text = hud_make("text"), rect = hud_make("rect"), bar = hud_make("bar"), icon = hud_make("icon", { "id" }),
  portrait = hud_make("portrait", { "entity" }), bubble = hud_make("bubble"),
  image = function(id, spec)
    assert(spec.path, "image " .. id .. " needs path")
    local f = io.open(spec.path, "rb")
    assert(f, "no image file " .. spec.path)
    f:close()
    hud_make("image")(id, spec)
  end,
  set = function(id, fields)
    local e = assert(H.hud[id], "hud.set on a missing element " .. id)
    for k, v in pairs(fields) do e[k] = copy(v) end
  end,
  remove = function(id) H.hud[id] = nil H.hud_handlers[id] = nil end,
  on = function(id, f) H.hud_handlers[id] = f end,
  screen = function() return { width = 1920, height = 1080, ui_scale = 1.25 } end,
}

-- World drawing and entity looks
H.draws, H.looks = {}, {}
draw = {
  ring = function(id, spec) assert(spec.at and spec.at.x, "ring at") H.draws[id] = copy(spec) end,
  remove = function(id) H.draws[id] = nil end,
  to_screen = function(p) return { x = math.floor(800 + p.x * 10), y = math.floor(400 + p.y * 10), on_screen = true } end,
}
render = { entity = function(id, look) H.looks[id] = copy(look) end }

H.binds = {}
binds = { set = function(k, f) H.binds[k] = f end, unset = function(k) H.binds[k] = nil end }
storage = { character = {}, account = {}, save = function() H.saved = (H.saved or 0) + 1 end }
H.targeted = nil
actions = { target = function(id) H.targeted = id end }
text = { plain = function(s) return s end }

local ITEMS = {
  [16594] = { name = "Bastokan Axe", description = "DMG:24 Delay:276\nSTR+2", damage = 24, delay = 276, level = 30,
    jobs = (1 << 1) | (1 << 8), flags = 0 },
  [12512] = { name = "Emperor Hairpin", description = "DEX+3 AGI+3\nEvasion+10", level = 24, jobs = 0x7FFFFE, flags = 0x8000 },
}
local SPELLS = { [4] = "Cure IV", [146] = "Fire IV", [57] = "Haste" }
local ABILITIES = { [42] = "Savage Blade", [512 + 16] = "Provoke", [512 + 30] = "Sentinel" }
res = {
  item = function(id) return ITEMS[id] end,
  spell = function(id) return SPELLS[id] and { id = id, name = SPELLS[id] } or nil end,
  ability = function(id) return ABILITIES[id] and { id = id, name = ABILITIES[id] } or nil end,
}

-- Feeds a chat line through the mod's filters; returns what became of it (false: dropped, a string: rewritten).
function H.line(text)
  local cur = text
  for _, f in ipairs(H.filters) do
    local r = f(cur, 3, false)
    if r == false then return false end
    if type(r) == "string" then cur = r end
  end
  return cur
end

-- Runs the timers for `seconds` of clock.
function H.run(seconds)
  local steps = math.floor((seconds or 0.1) / 0.1 + 0.5)
  for _ = 1, steps do
    H.clock = H.clock + 0.1
    for _, f in ipairs(H.timers) do f() end
  end
end

-- The actions sent so far, as text (marker stripped), and clears them.
function H.take_sent()
  local out = {}
  for _, b in ipairs(H.sent) do
    assert(b:sub(1, 2) == "\0\0", "kind and spare byte")
    assert(b:sub(3, 6) == "\127CM|", "marker")
    assert(b:sub(-1) == "\0", "NUL at the end")
    out[#out + 1] = b:sub(7, -2)
  end
  H.sent = {}
  return out
end

-- Runs a command as typed: "/ql quests" -> the handler with its words.
function H.command(line)
  local words = {}
  for w in line:gmatch("%S+") do words[#words + 1] = w end
  local name = table.remove(words, 1):gsub("^/+", ""):lower()
  local f = assert(H.commands[name], "no command " .. name)
  return f(table.unpack(words))
end

-- Clicks the region with this key in a canvas.
function H.click(win, canvas, key)
  local Canvas = require("lib.canvas")
  local e = Canvas._entry(win, canvas)
  assert(e, "no canvas " .. win .. "/" .. canvas)
  for _, r in ipairs(e.regions) do
    if r.key == key then
      H.handlers[win](win, { kind = "click", id = canvas, x = r.x + r.w / 2, y = r.y + r.h / 2, button = "left" })
      return r
    end
  end
  error("no region " .. tostring(key) .. " in " .. win .. "/" .. canvas)
end

function H.region(win, canvas, key)
  local e = require("lib.canvas")._entry(win, canvas)
  for _, r in ipairs(e.regions) do
    if r.key == key then return r end
  end
  return nil
end

function H.event(win, ev) H.handlers[win](win, ev) end

-- Checks every draw command of a window's canvases: known types, numbers where numbers go, inside the canvas.
function H.check_draws(win)
  local w = H.windows[win]
  local n = 0
  for id, e in pairs(w.el) do
    if e.type == "canvas" and e.draw then
      for _, d in ipairs(e.draw) do
        n = n + 1
        assert(({ rect = 1, line = 1, circle = 1, tri = 1, text = 1, tip = 1, icon = 1 })[d.type], "draw type " .. tostring(d.type))
        if d.type == "rect" or d.type == "tip" then
          for _, k in ipairs({ "x", "y", "w", "h" }) do assert(type(d[k]) == "number", id .. " " .. d.type .. " " .. k) end
          assert(d.x >= -0.5 and d.x + d.w <= e.w + 0.5, ("%s: %s at x %.1f w %.1f past the canvas (%d)"):format(id, d.type, d.x, d.w, e.w))
        elseif d.type == "text" then
          assert(type(d.text) == "string", id .. " text")
          local Canvas = require("lib.canvas")
          local tw = Canvas.measure(d.text, d.size, d.font == "bold")
          local left = d.align == "center" and d.x - tw / 2 or (d.align == "right" and d.x - tw or d.x)
          assert(left >= -2 and left + tw <= e.w + 2, ("%s: text %q runs past the canvas (%.0f..%.0f of %d)"):format(id, d.text, left, left + tw, e.w))
        elseif d.type == "icon" then
          assert(type(d.id) == "number" and d.id > 0, "icon id")
        end
      end
    end
  end
  return n
end

-- Writes a window's canvases as JSON (for the preview renderer).
local function json(v)
  local t = type(v)
  if t == "table" then
    if #v > 0 or next(v) == nil then
      local out = {}
      for _, x in ipairs(v) do out[#out + 1] = json(x) end
      return "[" .. table.concat(out, ",") .. "]"
    end
    local out = {}
    for k, x in pairs(v) do out[#out + 1] = string.format("%q", tostring(k)) .. ":" .. json(x) end
    return "{" .. table.concat(out, ",") .. "}"
  elseif t == "string" then
    return '"' .. v:gsub('[%c"\\]', function(c)
      if c == "\n" then return "\\n" end
      return string.format("\\u%04x", c:byte())
    end) .. '"'
  elseif t == "boolean" then return tostring(v)
  elseif v == nil then return "null"
  else return tostring(v) end
end

H.json = json

function H.dump(win, page, path)
  local w = H.windows[win]
  local items = {}
  for _, e in ipairs(w.spec.elements) do
    local live = w.el[e.id] or e
    if live.page == nil or live.page == page then
      items[#items + 1] = { type = live.type, id = live.id, w = live.w, h = live.h, draw = live.draw, label = live.label,
        items = live.items, text = live.text, at = live.at }
    end
  end
  local f = assert(io.open(path, "w"))
  f:write(json({ title = w.spec.title, width = w.spec.width, elements = items }))
  f:close()
end

return H
