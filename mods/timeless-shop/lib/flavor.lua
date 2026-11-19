-- More of each shopkeeper's character, on top of lib/themes.lua:
--   quips      what they say about the item you're looking at, by what kind of item it is
--   regulars   how they greet you once you've bought from them 3, 10 and 25 times, and the title they give you
--   night / morning   greetings by the Vana'diel hour
--   stamps     the little tags on item cards: a bargain, the last one, sold out
--   bye        what they say when you leave (in chat)
--   burst      the little celebration when you buy something (coins, sparkles, feathers...)
--   sound      the sounds the shop makes when it opens and when you buy (sounds/<sound>_open.wav, _buy.wav)
--   anim(c, w, h, t)   what moves in the header (t: seconds)
-- {item} in a quip is the item's name.

local F = {}

local function tri(c, x1, y1, x2, y2, x3, y3, col)
  c.draw[#c.draw + 1] = { type = "tri", x1 = x1, y1 = y1, x2 = x2, y2 = y2, x3 = x3, y3 = y3, colour = col }
end

local function star(c, x, y, r, col)
  tri(c, x, y - r, x - r * 0.3, y, x + r * 0.3, y, col)
  tri(c, x, y + r, x - r * 0.3, y, x + r * 0.3, y, col)
  tri(c, x - r, y, x, y - r * 0.3, x, y + r * 0.3, col)
  tri(c, x + r, y, x, y - r * 0.3, x, y + r * 0.3, col)
end

local function a(v) return math.max(0, math.min(255, math.floor(v))) end

F.goblin = {
  sound = "coins", burst = "coins",
  quips = {
    orb = { "Shiny orb! Glows in dark. Grizznix found it. Nobody saw. Hee hee!", "{item}! Big fight inside. Grizznix no go in. You go in!" },
    food = { "{item}! Goblin recipe. Only little bit poison. Joke! Mostly joke.", "Yum yum {item}. Grizznix eat three today, still alive!" },
    weapon = { "{item}! Pointy end go in monster. Very simple.", "Good stabber, this {item}. Previous owner no need any more." },
    armor = { "{item}! Fits you. Fits everybody. Grizznix make it fit!", "Smell little bit. Smell fades. Probably." },
    coin = { "Old shiny coin! Beastmen pay big for this.", "{item}! Grizznix keep some, sell some. Is business!" },
    crystal = { "Sparkle rock! Make things go boom. Or synthesis. Same thing." },
    sack = { "Mystery sack! Grizznix no know what inside either. Is mystery!", "Shake it! Hear that? Is treasure. Or pie. Hee hee!" },
    thing = { "{item}! Rare rare. Only one in Vana'diel. This week.", "No ask where {item} come from. Grizznix no ask either." },
  },
  regulars = {
    { 3, "Good customer", { "You come back! Grizznix knew you would. Shinies ready?" } },
    { 10, "Best friend customer", { "Best friend customer! Grizznix save good stuff for you. Maybe." } },
    { 25, "Grizznix's favourite", { "Favourite customer! Grizznix almost give discount. Almost!" } },
  },
  night = { "Moon high, prices fair! Goblin prices always fair at night.", "Shhh! Night time trading. Very secret. Very good." },
  morning = { "Too bright! Grizznix sleepy. You buy, Grizznix go nap." },
  stamps = { bargain = "Cheap-cheap!", last = "Grab fast!", sold = "All gone!" },
  bye = { "Come back with more shinies!", "Bye bye! No tell guards about Grizznix!", "Hee hee! Good trade, good trade." },
  anim = function(c, w, h, t)
    -- a coin that catches the light now and then
    local glint = math.max(0, math.sin(t * 2.2)) ^ 6
    local x, y = w - 196, 24
    c:circle(x, y, 8, { 214, 172, 70, 255 })
    c:circle(x, y, 8, { 120, 86, 30, 255 }, 1.2)
    c:text(x, y - 6, "G", { size = 0.62, bold = true, align = "center", colour = { 110, 76, 20, 255 } })
    if glint > 0.05 then star(c, x + 4, y - 4, 7 * glint, { 255, 250, 220, a(255 * glint) }) end
  end,
}

F.moogle = {
  sound = "bell", burst = "hearts",
  quips = {
    orb = { "Ooh, an orb, kupo! It goes to a scary place. Be careful, kupo!" },
    food = { "{item}! Mog tried one, kupo. Okay, Mog tried four." },
    weapon = { "{item}, kupo! Very pointy. Mog doesn't touch the pointy ones." },
    armor = { "That would look so nice on you, kupo!" },
    sack = { "A surprise, kupo! Mog loves surprises!" },
    thing = { "{item}! A fine choice, kupo!", "Mog polished that one specially, kupo!" },
  },
  regulars = {
    { 3, "Friend of Mog", { "Kupo! You came back! Mog is so happy!" } },
    { 10, "Kupo pal", { "Mog's favourite pal, kupo! Look what's new!" } },
    { 25, "Honorary moogle", { "You're practically a moogle now, kupo! Here, look-look!" } },
  },
  night = { "Kupo... Mog was napping. Welcome anyway, kupo!" },
  morning = { "Good morning, kupo! Fresh stock, kupo!" },
  stamps = { bargain = "Kupo deal!", last = "Last one, kupo!", sold = "Gone, kupo!" },
  bye = { "Come back soon, kupo!", "Kupopo! Take care out there!" },
  anim = function(c, w, h, t)
    -- the pom-pom bobs
    local cx, bob = w - 70, math.sin(t * 3) * 2.5
    c:circle(cx + 9, 10 + bob, 7, { 226, 48, 80, 255 })
    c:circle(cx + 7, 8 + bob, 2, { 255, 200, 210, 255 })
  end,
}

F.hume = {
  sound = "wood", burst = "coins",
  quips = {
    orb = { "Folks pay a fortune for those in Jeuno. Fair price here." },
    food = { "{item}. Good road food. Keeps you going." },
    weapon = { "{item}. Solid. I've seen worse come off a Bastok forge." },
    armor = { "{item}. It'll take a hit or two for you." },
    thing = { "{item}. Picked it up on my travels.", "That one's a good find, if I say so myself." },
  },
  regulars = {
    { 3, "Regular", { "Back again? Good to see you, friend." } },
    { 10, "Old friend", { "There's my favourite customer. What'll it be?" } },
    { 25, "Partner in trade", { "I ought to make you a partner, the business you bring me." } },
  },
  night = { "Late one, isn't it? I keep the lamp lit for travellers." },
  morning = { "Morning! Coffee's on, prices are fair." },
  stamps = { bargain = "Good price", last = "Last one", sold = "Sold out" },
  bye = { "Safe travels, friend.", "Come back any time." },
  anim = function(c, w, h, t)
    -- the lamp in the corner flickers
    local f = 0.75 + 0.25 * math.sin(t * 9) * math.sin(t * 5.3)
    local x, y = w - 40, 24
    for r = 16, 6, -5 do c:circle(x, y, r, { 255, 210, 120, a(28 * f) }) end
    c:rect(x - 4, y - 6, 8, 12, { 255, 220, 140, a(220 * f) }, { round = 2 })
  end,
}

F.elvaan = {
  sound = "harp", burst = "sparks",
  quips = {
    orb = { "An orb of the old battlefields. Only the worthy should carry it." },
    food = { "{item}. Even a knight must eat." },
    weapon = { "{item}. A blade with honour in its steel.", "Wield {item} in the Kingdom's name, and it will not fail you." },
    armor = { "{item}. Fit for an audience at the Chateau." },
    thing = { "{item}. Of the finest quality, I assure you.", "A worthy choice. The house approves." },
  },
  regulars = {
    { 3, "Valued patron", { "Ah, you return. The house is honoured." } },
    { 10, "Friend of the House", { "A friend of the House is always welcome here." } },
    { 25, "Noble patron", { "Your patronage would make any noble house proud." } },
  },
  night = { "The Cathedral bells have long rung. Still, the house serves you." },
  morning = { "Good morning. The finest wares are set out for the day." },
  stamps = { bargain = "Fair price", last = "The last", sold = "Unavailable" },
  bye = { "May the Goddess watch over you.", "The house thanks you. Go with honour." },
  anim = function(c, w, h, t)
    -- a slow glint travels along the gilt line
    local x = 6 + ((t * 90) % (w - 12))
    c:rect(x - 14, 4, 28, 2, { 255, 240, 190, 140 }, { round = 1 })
  end,
}

F.tarutaru = {
  sound = "chime", burst = "stars",
  quips = {
    orb = { "Ooh, an orb-orb! It hums when you hold it. Try-try!" },
    food = { "{item}! Yummy-yum! I had two for breakfast-wakfast." },
    weapon = { "{item}! Swishy-swish! Not for me, I prefer fire-fire." },
    armor = { "{item}! You'd look so dashing-wishing!" },
    crystal = { "Sparkly crystals! Perfect for a little magic-wagic." },
    thing = { "{item}! It's magical-ish, I promise-womise!", "Ooh, good eye! That one's my favourite-wavourite!" },
  },
  regulars = {
    { 3, "Repeat-peat customer", { "You're back-back! I knew-knew you'd come!" } },
    { 10, "Best buddy-wuddy", { "My best buddy-wuddy! I saved the sparkliest ones!" } },
    { 25, "Honorary Taru", { "You're practically a Tarutaru now! Just a bit tall-tall." } },
  },
  night = { "Ooh, stargazing time! Look-look, the stars match my shop!" },
  morning = { "Good morning-worning! Fresh sparkles today!" },
  stamps = { bargain = "Cheap-cheap!", last = "Last-last!", sold = "Gone-gone!" },
  bye = { "Bye-bye! Come back soon-soon!", "Toodle-oo! Mind the sparkles!" },
  anim = function(c, w, h, t)
    -- the stars twinkle
    local stars = { { w - 150, 12, 4, 0 }, { w - 120, 30, 3, 1.3 }, { w - 96, 10, 2.5, 2.1 }, { w - 40, 40, 3, 0.7 }, { 200, 8, 2, 2.9 } }
    for _, s in ipairs(stars) do
      local k = 0.5 + 0.5 * math.sin(t * 2.6 + s[4] * 2)
      star(c, s[1], s[2], s[3] * (1.4 + 0.8 * k), { 255, 250, 210, a(120 + 135 * k) })
    end
  end,
}

F.mithra = {
  sound = "bamboo", burst = "leaves",
  quips = {
    orb = { "Ooh, shiny! Like the eyes of a big jungle cat, meow." },
    food = { "{item}! Mmm, smells better than fish. Almost." },
    weapon = { "{item}! Sharp as a claw. Good for the hunt." },
    armor = { "{item}! Light enough to pounce in." },
    thing = { "{item}! Fresh from the hunt, meow.", "Good catch! Ehehe." },
  },
  regulars = {
    { 3, "Hunting partner", { "You again! The hunt goes well for you, no?" } },
    { 10, "Pack friend", { "Pack friend! I kept the best of the hunt for you." } },
    { 25, "Honorary huntress", { "You hunt like a Mithra now. Ehehe!" } },
  },
  night = { "Night is the best time to hunt, meow. And to shop!" },
  morning = { "Mmm, sleepy... the sun is too warm. Buy something, meow!" },
  stamps = { bargain = "Good catch!", last = "Last one!", sold = "Too slow!" },
  bye = { "Happy hunting!", "Ehehe, come back soon!" },
  anim = function(c, w, h, t)
    -- the tail flicks
    local x, y = 26, h - 18
    local s = math.sin(t * 2.4)
    c:line(x, y, x + 10 + 5 * s, y - 10, { 60, 30, 14, 200 }, 3)
    c:line(x + 10 + 5 * s, y - 10, x + 8 + 10 * s, y - 20, { 60, 30, 14, 200 }, 3)
  end,
}

F.galka = {
  sound = "anvil", burst = "sparks",
  quips = {
    orb = { "An orb. Heavy fight behind it." },
    food = { "{item}. Fills you up." },
    weapon = { "{item}. Bastok steel. It won't break.", "Good weight on that one." },
    armor = { "{item}. It'll hold." },
    thing = { "{item}. Fair price.", "Hm. Good choice." },
  },
  regulars = {
    { 3, "Regular", { "Hm. You again. Good." } },
    { 10, "Trusted customer", { "Trusted customer. Look all you like." } },
    { 25, "Old friend", { "Old friend. I put the good ones at the front." } },
  },
  night = { "Late. The forges are cooling. Still open." },
  morning = { "Morning. Mines are loud today." },
  stamps = { bargain = "Fair", last = "Last", sold = "None" },
  bye = { "Hm. Take care.", "Come back." },
  anim = function(c, w, h, t)
    -- an ember drifts up from the forge
    for i = 0, 2 do
      local p = (t * 0.5 + i / 3) % 1
      c:circle(w - 90 + 12 * math.sin(p * 6 + i), h - 4 - p * (h - 8), 1.8, { 255, 160, 70, a(220 * (1 - p)) })
    end
  end,
}

F.quadav = {
  sound = "shell", burst = "coins",
  quips = { orb = { "Hrrn. Glowing stone. Shiny. Expensive." }, thing = { "Hrrn. {item}. Trade.", "Shiny thing. Good shiny." } },
  regulars = { { 3, "Shiny-giver", { "Hrrn! Shiny-giver back." } }, { 10, "Shell-friend", { "Shell-friend. Trade, trade." } },
    { 25, "Hoard-friend", { "Hoard-friend! Hrrn, hrrn!" } } },
  night = { "Hrrn. Dark. Good for hoarding." }, morning = { "Hrrn. Light. Bad. Buy fast." },
  stamps = { bargain = "Cheap shiny", last = "Last shiny", sold = "No shiny" },
  bye = { "Hrrn. Go.", "Bring more shiny." },
  anim = function() end,
}

F.yagudo = {
  sound = "gong", burst = "feathers",
  quips = { orb = { "A holy orb! Blessed by the Divine Kindred, yes-ah!" }, thing = { "{item}! Sacred, sacred!", "Praise! A worthy offering!" } },
  regulars = { { 3, "Faithful", { "The faithful return! Praise!" } }, { 10, "Devoted", { "Devoted one! The Kindred smile on you!" } },
    { 25, "Blessed", { "Blessed one! Take, take, with the Kindred's favour!" } } },
  night = { "Night prayers! Buy, and be blessed!" }, morning = { "Morning prayers! The Kindred provide!" },
  stamps = { bargain = "Blessed price", last = "Last relic", sold = "Taken" },
  bye = { "Go with the Kindred!", "Praise be! Return, return!" },
  anim = function(c, w, h, t)
    local p = (t * 0.35) % 1
    tri(c, w - 160 + 20 * math.sin(p * 7), 6 + p * (h - 12), w - 166 + 20 * math.sin(p * 7), 14 + p * (h - 12), w - 152 + 20 * math.sin(p * 7), 12 + p * (h - 12), { 230, 190, 80, a(200 * (1 - p)) })
  end,
}

F.orc = {
  sound = "drum", burst = "embers",
  quips = { weapon = { "{item}! Good for smashing. Orc approve." }, thing = { "{item}! Stolen fair and square. Hah!", "Loot! Pay! Go!" } },
  regulars = { { 3, "Not-enemy", { "Hah! You again. Not enemy today." } }, { 10, "War-friend", { "War-friend! Orc give best loot." } },
    { 25, "Honorary orc", { "You fight like orc now! Hah! Take loot!" } } },
  night = { "Night raid loot! Fresh! Hah!" }, morning = { "Morning. Orc tired from raid. Pay fast." },
  stamps = { bargain = "Cheap loot!", last = "Last loot!", sold = "Gone! Hah!" },
  bye = { "Go! Before orc change mind!", "Hah! Come back with more shinies!" },
  anim = function(c, w, h, t)
    local k = 0.5 + 0.5 * math.sin(t * 4)
    for r = 18, 6, -6 do c:circle(w - 56, h - 10, r, { 255, 120, 40, a(26 * k) }) end
  end,
}

F.tonberry = {
  sound = "hush", burst = "embers",
  quips = { thing = { "...", "......" } },
  regulars = { { 3, "Noticed", { "..." } }, { 10, "Remembered", { "...!" } }, { 25, "Grudge-free", { "...♪" } } },
  night = { "..." }, morning = { "..." },
  stamps = { bargain = "...", last = "...!", sold = "......" },
  bye = { "...", "......" },
  anim = function(c, w, h, t)
    local f = 0.7 + 0.3 * math.sin(t * 7) * math.sin(t * 3.1)
    local x, y = w - 60, h / 2
    for r = 26, 8, -6 do c:circle(x, y, r, { 255, 210, 110, a(30 * f) }) end
  end,
}

F.guild = {
  sound = "guild", burst = "sparks",
  quips = {
    sack = { "One roll of the sack. Most of it is food; now and then, something rare." },
    tonic = { "Rested synths straight away: better odds of a skill-up." },
    lure = { "Champions turn up far more often in your fights while it lasts." },
    focus = { "Double gathering experience while it lasts." },
    feast = { "Everyone online gets the bonus. Very popular on weekends." },
    gil = { "A poor rate, on purpose: Everglass is for spending." },
    slot = { "One more companion you can keep for good." },
    recall = { "Clears that job's wait, so its companions can be called at once." },
    hourglass = { "Turns back your last knockout." },
    thing = { "{item}. Point at anything for what it does." },
  },
  regulars = { { 3, "Guild member", { "Welcome back. The guild appreciates it." } }, { 10, "Patron", { "A patron of the guild. Thank you for your support." } },
    { 25, "Pillar of the guild", { "One of the guild's pillars. Everything you need is here." } } },
  night = { "The guild hall never closes." }, morning = { "Good morning. The guild's stock is fresh." },
  stamps = { bargain = "Best value", last = "Last one", sold = "Sold out" },
  bye = { "The guild thanks you.", "Good luck out there." },
  anim = function(c, w, h, t)
    -- sand trickles through the hourglass
    local x, y = w - 50, h / 2
    local p = (t * 0.8) % 1
    c:circle(x, y - 2 + p * 12, 1.2, { 228, 192, 119, 220 })
  end,
}

-- What kind of item it is, for the quips: a glyph's name, weapon, armor, orb, crystal, coin, food, or thing.
local FOOD = { "pie", "taco", "glace", "carrot", "mushpot", "soup", "meat", "bread", "cookie", "fish", "egg", "stew", "omelet", "rice", "juice", "milk", "cake", "sushi" }
function F.kind(item, info)
  if item.glyph and item.glyph ~= "" then return item.glyph end
  local k = info and info.kind
  if k == "weapon" then return "weapon" end
  if k == "armor" then return "armor" end
  local name = ((info and info.name) or item.label or ""):lower()
  if name:find("orb") then return "orb" end
  if name:find("crystal") or name:find("cluster") then return "crystal" end
  if name:find("beastcoin") or name:find("coin") then return "coin" end
  for _, f in ipairs(FOOD) do if name:find(f, 1, true) then return "food" end end
  return "thing"
end

-- A quip about an item, picked by a number so it stays put while you look at it
function F.quip(race, item, info, n)
  local fl = F[race] or F.guild
  local kind = F.kind(item, info)
  local list = fl.quips[kind] or fl.quips.thing or F.guild.quips.thing
  local line = list[((n or 1) - 1) % #list + 1]
  local name = (info and info.name) or item.label or "that"
  return (line:gsub("{item}", function() return name end))
end

-- The regulars' tier for a number of purchases: { at, title, lines } or nil
function F.regular(race, count)
  local fl = F[race] or F.guild
  local best
  for _, r in ipairs(fl.regulars or {}) do if count >= r[1] then best = r end end
  return best
end

-- "night" (20:00-04:59), "morning" (05:00-09:59), or nil
function F.time_of_day()
  local ok, t = pcall(game.time)
  local h = ok and type(t) == "table" and t.hour or nil
  if not h then return nil end
  if h >= 20 or h < 5 then return "night" end
  if h < 10 then return "morning" end
  return nil
end

return F
