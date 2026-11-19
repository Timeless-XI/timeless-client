-- Timeless Shop: the Everglass sellers' shops as real windows, dressed for the shopkeeper.
--
-- Talk to Grizznix at the Goblin Bazaar, the Wandering Merchant, an expedition quartermaster's Everglass Shop, or
-- type !shop: instead of pages of small menus, a shop window opens. Sections on the left, the items as cards with
-- their icons, prices and stock, and a preview of the one you pick (the game's description, level, jobs, why you
-- can't buy it yet). Buy asks once more, then the window shows what happened and the stock left.
--
-- The window takes after whoever runs the shop: a goblin's patched sack and "shinies", a moogle's pom-pom and
-- kupos, the House of an Elvaan purveyor, a Tarutaru's starry emporium, a Mithra's Hunt & Haul, a Galka's stone
-- trading post, a Hume's honest counter (and Quadav, Yagudo, Orc and Tonberry, should one ever open a stall). The
-- server names the race, or the client reads it off the shopkeeper. Each keeper has a voice of their own: a remark
-- about whatever you look at, greetings for the time of day and for their regulars (they remember how often you buy,
-- and give you a title), a little burst and a sound when you buy, and a word when you leave.
--
-- The mod tells the server it's here ("sh on", every two minutes, as a packet, not chat); without it the server
-- uses the old menus. Shop lines are taken out of the chat log. Needs the server's grub_shop storefronts.
--
--   /storefront demo [race]   a sample shop in a race's colours (goblin, moogle, hume, elvaan, tarutaru, mithra,
--                             galka, quadav, yagudo, orc, tonberry, guild); again for the next race
--   /storefront close         close the window

local action = require("lib.action")
local protocol = require("lib.protocol")
local window = require("lib.window")
local themes = require("lib.themes")
local flavor = require("lib.flavor")

local BEAT = 120
local st = { next_beat = os.clock() + 1, queue = {} }

local function send(words)
  st.queue[#st.queue + 1] = words
end
window.send = function(words)
  if window.state.demo then
    -- The demo buys from itself
    local shop = window.model.shop
    local s, i = words:match("^buy %d+ (%d+) (%d+)$")
    local item = s and shop.sections[tonumber(s)].items[tonumber(i)]
    if item then
      shop.balance = shop.balance - item.price
      if item.left > 0 then item.left = item.left - 1 end
      window.model.result = { ok = true, msg = ("You obtain %s. (%d %s; %d left)"):format(item.label, item.price, shop.currency, shop.balance), at = os.clock() }
      window.answered()
    end
    return
  end
  send(words)
end

-- The shop's sounds: sounds/<style>_open.wav and _buy.wav, the style by the shopkeeper's race
local function prefs()
  storage.character.shop = storage.character.shop or {}
  local p = storage.character.shop
  if p.sound == nil then p.sound = true end
  return p
end
window.play = function(which)
  if not prefs().sound then return end
  local fl = flavor[themes.for_shop(window.model.shop).key] or flavor.guild
  pcall(audio.play, ("sounds/%s_%s.wav"):format(fl.sound, which), 0.6)
end

window.setup()

filters.chat(function(text)
  if type(text) ~= "string" then return nil end
  local line = protocol.extract(text)
  if not line then return nil end
  local what = protocol.feed(window.model, line, os.clock())
  if what == "open" then
    window.state.demo = false
    window.opened()
  elseif what == "result" then
    window.answered()
  end
  return false
end)

events.on("zone_change", function()
  window.close()
  st.next_beat = math.min(st.next_beat, os.clock() + 2)
end)
events.on("login", function() st.next_beat = math.min(st.next_beat, os.clock() + 2) end)
events.on("unload", function()
  pcall(action.send, "sh off")
  window.close()
end)

-- A sample shop in a race's colours
local DEMO_KEEPERS = { goblin = "Grizznix", moogle = "Mogmog", hume = "Odilo the Wanderer", elvaan = "Gaudierre",
  tarutaru = "Tobiko", mithra = "Jhenna Pashro", galka = "Bargrudd", quadav = "Zo'Gho Hoarder", yagudo = "Xa'Yhu Seller",
  orc = "Grukk the Fence", tonberry = "Tonberry", guild = "Artisans Guild" }
local demo_at = 0

local function demo(race)
  if not themes[race or ""] or not themes[race].colours then
    demo_at = demo_at % #themes.ORDER + 1
    race = themes.ORDER[demo_at]
  else
    for i, k in ipairs(themes.ORDER) do if k == race then demo_at = i end end
  end
  local m = protocol.new_model()
  local lines = {
    ("SH1|B|t=0,k=%s,i=0,r=%s,c=Everglass,b=64,n=3"):format(DEMO_KEEPERS[race], race),
    "SH1|S|1|Wares", "SH1|S|2|Treats", "SH1|S|3|Odds and ends",
    "SH1|I|1.1|id=1130,ic=0,g=,p=12,q=2,a=1,l=Moon Orb",
    "SH1|I|1.2|id=1131,ic=0,g=,p=18,q=1,a=1,l=Star Orb",
    "SH1|I|1.3|id=1875,ic=0,g=,p=15,q=0,a=0,l=Ancient Beastcoin",
    "SH1|W|1.3|Sold out!",
    "SH1|I|1.4|id=3643,ic=0,g=,p=70,q=1,a=1,l=Carillon Vermeil",
    "SH1|I|2.1|id=4539,ic=0,g=,p=3,q=6,a=1,l=Goblin Pie",
    "SH1|I|2.2|id=5608,ic=0,g=,p=15,q=2,a=1,l=Zegham Carrot",
    "SH1|I|3.1|id=0,ic=0,g=sack,p=12,q=3,a=1,l=Mystery sack",
    "SH1|D|3.1|One roll of the sack: goblin food, clusters, orbs... or something rare.",
    "SH1|I|3.2|id=0,ic=0,g=tonic,p=8,q=-1,a=1,l=Rested tonic +5",
    "SH1|D|3.2|Rested synths at once: better skill-up odds on your next 5 synths.",
    "SH1|I|3.3|id=0,ic=0,g=lure,p=12,q=-1,a=1,l=Champion lure 30m",
    "SH1|D|3.3|Champions are far more likely in your fights while it lasts.",
    "SH1|I|3.4|id=0,ic=0,g=gil,p=50,q=-1,a=1,l=50 for 15,000 gil",
    "SH1|E|t=0",
  }
  for _, l in ipairs(lines) do protocol.feed(m, l, os.clock()) end
  window.model = m
  window.state.demo = true
  window.state.token = nil
  window.opened()
  return race
end

local function storefront_command(sub, value)
  sub = (sub or ""):lower()
  if sub == "demo" or sub == "" then
    local race = demo(value and value:lower() or nil)
    print(("Shop: a sample %s shop. /storefront demo again for the next race."):format(themes[race].label))
  elseif sub == "close" then
    window.close(true)
  elseif sub == "sound" then
    local p = prefs()
    p.sound = not p.sound
    pcall(storage.save)
    print(p.sound and "Shop: shop sounds on." or "Shop: shop sounds off.")
  else
    print("Shop: /storefront demo [race] | sound | close")
  end
end
commands.register("storefront", storefront_command)
commands.register("timelessshop", storefront_command)

mods.provide("timeless_menu", function()
  return {
    title = "Shops",
    desc = "The Goblin Bazaar, the Wandering Merchant and the Everglass Shop open as real windows, dressed for the shopkeeper's race.",
    order = 40,
    open = "//storefront demo",
    buttons = {
      { label = "Everglass Shop", command = "/say !shop", tip = "Opens the Everglass Shop (the Artisans Guild's)." },
      { label = "Demo", command = "//storefront demo", tip = "A sample shop; again for the next race." },
    },
  }
end)

local last_frame = 0
timer.every(0.1, function()
  local t = os.clock()
  -- What moves (ornaments, a purchase's burst) redraws about eight times a second while the shop is open
  if ui.is_open(window.WIN) and window.animated() and t - last_frame >= 0.12 then
    last_frame = t
    window.render()
  end
  if t >= st.next_beat then
    st.next_beat = t + BEAT
    send("sh on")
  end
  if #st.queue > 0 then
    local w = table.remove(st.queue, 1)
    pcall(action.send, w:sub(1, 3) == "sh " and w or ("sh " .. w))
  end
end)
