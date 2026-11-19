-- TimelessMenu: one menu for every TimelessXI mod (replaces the timelessmenu Ashita addon).
--
--   The hourglass button (drag it anywhere; right-click to hide it) or /tm (also /timeless) opens it.
--   Each TimelessXI mod has a card: what it does, Open, and buttons for its tabs. A mod installed but not running has
--   Load; one not installed yet says so. The Server row says the server's commands (!welcome, !companion status,
--   !guild, ...); the answers come in chat.
--
-- The cards come from the mods (their "timeless_menu" offer), so a new TimelessXI mod shows up without a menu update.

local menu = require("lib.menu")

local BUTTON = "button"
local SIZE = 44 -- the button's size in UI units

local function prefs()
  storage.account.menu = storage.account.menu or {}
  return storage.account.menu
end

menu.run = function(line)
  local ok, err = pcall(commands.run, line)
  if line:sub(1, 5) == "/say " then
    menu.state.note = ok and ("Sent " .. line:sub(6) .. ": the answer is in chat.") or ("Couldn't send it: " .. tostring(err))
    menu.state.note_at = os.clock()
  elseif not ok then
    menu.state.note = "Couldn't run " .. line .. ": " .. tostring(err)
    menu.state.note_at = os.clock()
  end
  menu.state.refresh_at = os.clock() + 0.3
end

menu.setup()

-- The hourglass button.
local drag = nil
local function place_button()
  local p = prefs()
  if p.hidden then
    hud.remove(BUTTON)
    return
  end
  local s = hud.screen()
  local size = math.floor(SIZE * (s.ui_scale or 1))
  local x = tonumber(p.x) or (s.width - size - 24)
  local y = tonumber(p.y) or math.floor(s.height * 0.18)
  x = math.max(0, math.min(s.width - size, x))
  y = math.max(0, math.min(s.height - size, y))
  hud.image(BUTTON, { x = x, y = y, w = size, h = size, path = "img/hourglass.png", layer = 5,
    tip = { title = "TimelessMenu", text = "Click: every TimelessXI mod and the server's commands (/tm).\nDrag to move it; right-click to hide it." } })
  hud.on(BUTTON, function(_, e)
    if e.kind == "click" and e.button == "left" then
      menu.toggle()
    elseif e.kind == "click" and e.button == "right" then
      ui.menu({ "Open TimelessMenu", "Hide this button" }, function(i)
        if i == 1 then menu.open()
        elseif i == 2 then
          prefs().hidden = true
          storage.save()
          place_button()
          print("TimelessMenu: button hidden. /tm button brings it back; /tm opens the menu.")
        end
      end)
    elseif e.kind == "drag_start" then
      drag = { x = x, y = y }
    elseif e.kind == "drag" and drag then
      hud.set(BUTTON, { x = math.max(0, math.min(s.width - size, drag.x + e.dx)),
        y = math.max(0, math.min(s.height - size, drag.y + e.dy)) })
    elseif e.kind == "drag_end" and drag then
      p.x = math.max(0, math.min(s.width - size, drag.x + e.dx))
      p.y = math.max(0, math.min(s.height - size, drag.y + e.dy))
      drag = nil
      storage.save()
      place_button()
    end
  end)
end
place_button()

local function tm_command(sub)
  sub = (sub or ""):lower()
  if sub == "button" then
    prefs().hidden = not prefs().hidden
    storage.save()
    place_button()
    print(prefs().hidden and "TimelessMenu: button hidden (/tm button shows it)." or "TimelessMenu: button shown.")
  elseif sub == "reset" then
    prefs().x, prefs().y, prefs().hidden = nil, nil, false
    storage.save()
    place_button()
  else
    menu.toggle()
  end
end
commands.register("tm", tm_command)
commands.register("timeless", tm_command)

-- A key from the settings.
local bound
local function apply_key()
  if bound then pcall(binds.unset, bound) bound = nil end
  local ok, key = pcall(settings.get, "key_menu")
  if ok and type(key) == "string" and key ~= "" and pcall(binds.set, key, menu.toggle) then bound = key end
end
apply_key()
events.on("settings_changed", apply_key)

-- A new screen size (or UI scale) moves the button back on screen.
events.on("zone_change", place_button)

-- Once, the first time: where the menu is.
if not prefs().hinted then
  prefs().hinted = true
  storage.save()
  print("TimelessMenu: click the hourglass button or type /tm for every TimelessXI mod.")
end

-- While the menu is open: redraw after a button (a window opened, a mod loaded) and every 2 s (mods coming and going).
local last = 0
timer.every(0.25, function()
  if not menu.is_open() then return end
  local t = os.clock()
  if (menu.state.refresh_at and t >= menu.state.refresh_at) or t - last >= 2 then
    menu.state.refresh_at = nil
    last = t
    menu.render()
  end
end)
