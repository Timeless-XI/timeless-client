-- Timeless Artisan: recipes with your chances, an account-wide crafting storage and a craft queue (/artisan).
--
-- Recipes (by craft and level, or found by name or material) show your success, high-quality and skill-up chances
-- the way the server works them out, and what you have of each material in your bags and in storage. Craft queues
-- up to 12 synths, which the server runs one every 15 seconds from your inventory and storage. Favorites keep
-- recipes at hand; Guilds shows each guild's item for the day and crafts the number still needed; Storage deposits
-- and withdraws crafting materials (crystals and clusters too), shared by every character on the account, with
-- auto-deposit that leaves what you withdrew alone. A small HUD window keeps the queue, Repeat, your favorites and
-- storage on screen.
--
-- It talks to the server's timeless_artisan module with packets (\x7F"TA|" requests, "TA1|" answers, kept out of
-- chat). Nothing it does is automatic beyond the 12-synth queue the server allows.
--
--   /artisan                       open or close the window
--   /artisan recipes|favorites|guilds|storage|queue
--   /artisan hud                   show or hide the HUD window
--   /artisan repeat | stop | deposit

local C = require("lib.core")
local W = require("lib.window")
local K = require("lib.card")
local P = require("lib.amount")
local H = require("lib.hud")

local TABS = { recipes = 1, favorites = 2, favourites = 2, guilds = 3, guild = 3, storage = 4, queue = 5 }

W.setup()
K.setup()
P.setup()
H.setup()
W.open_card = K.open
W.ask_amount = P.open
H.open_window = function(tab) W.open(tab) end
K.on_change = function() W.render() H.render() end

-- The server's lines: taken out of the chat log here, read in the mod phase (no game state in a chat filter)
local inbox = {}
filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  if text:find("TA1|", 1, true) then
    inbox[#inbox + 1] = text
    return false
  end
  if text:find("skill rises", 1, true) then inbox[#inbox + 1] = { chat = text } end
  return nil
end)

local function in_world()
  local ok, p = pcall(game.player)
  return ok and p ~= nil
end

local world = false
local drawn = 0
local function redraw()
  if drawn == C.rev then return end
  drawn = C.rev
  W.render()
  K.render()
  H.render()
end

timer.every(0.25, function()
  if not in_world() then
    world = false
    return
  end
  if not world then
    world = true
    C.hello()
    H.sync()
  end
  while #inbox > 0 do
    local line = table.remove(inbox, 1)
    if type(line) == "table" then C.on_chat(line.chat) else C.on_line(line) end
  end
  if C.scan() then C.inventory_changed() end
  C.flush()
  C.auto_tick()
  -- the Withdraw all confirmation runs out
  if C.wa_confirm and C.now() >= C.wa_confirm then C.wa_confirm = nil C.changed() end
  if C.waiting_too_long() and not C.net.told then C.net.told = true C.changed() end
  redraw()
end)

-- Slower: kept items shrink as you use them; ask again if the server never answered
timer.every(5, function()
  if not world then return end
  C.kept_tick()
  if not C.ST.ready and C.net.asked_at and C.now() - C.net.asked_at > 20 then C.hello() end
end)

events.on("login", function() world = false end)
events.on("zone_change", function() C.net.ki = nil end)
events.on("synth_result", function() C.request("md") end)

local function artisan(sub)
  sub = (sub or ""):lower()
  if sub == "" then
    W.toggle()
  elseif TABS[sub] then
    W.open(TABS[sub])
  elseif sub == "hud" then
    if ui.is_open(H.WIN) then ui.close(H.WIN) else ui.show(H.WIN) H.render() end
  elseif sub == "repeat" then
    local last, n = C.last()
    local why = last and C.block(last)
    if not last then C.note("nothing crafted yet to repeat.")
    elseif why then C.note(why)
    else C.craft(last, n) end
  elseif sub == "stop" then
    C.stop()
  elseif sub == "deposit" then
    C.deposit_all()
  else
    print("Artisan: /artisan [recipes | favorites | guilds | storage | queue | hud | repeat | stop | deposit]")
  end
end
commands.register("artisan", artisan)
commands.register("timelessartisan", artisan)

-- The Timeless menu's card
mods.provide("timeless_menu", function()
  local v = C.queue_view()
  local _, total = W.storage_totals()
  return {
    title = "Artisan",
    category = "Crafting & Shops",
    order = 44,
    open = "//artisan",
    command = "/artisan",
    desc = v and ("Crafting: %d synths to go. Recipes, favorites, guild items and your crafting storage."):format(v.left)
      or (total > 0 and ("Recipes with your chances, guild items, a craft queue and your crafting storage (%s stored)."):format(C.plural(total, "item"))
      or "Recipes with your chances, favorites, today's guild items, a craft queue and your account-wide crafting storage."),
    buttons = {
      { label = "Recipes", command = "//artisan recipes" },
      { label = "Favorites", command = "//artisan favorites" },
      { label = "Guilds", command = "//artisan guilds" },
      { label = "Storage", command = "//artisan storage" },
      { label = "Queue", command = "//artisan queue" },
      { label = "HUD", command = "//artisan hud" },
    },
  }
end)

-- Settings: the HUD window and the hotkey
local bound
local function apply_settings()
  if bound then pcall(binds.unset, bound) bound = nil end
  local ok, key = pcall(settings.get, "key_artisan")
  if ok and type(key) == "string" and key ~= "" and pcall(binds.set, key, W.toggle) then bound = key end
  if world then H.sync() end
  C.changed()
end
apply_settings()
events.on("settings_changed", apply_settings)
