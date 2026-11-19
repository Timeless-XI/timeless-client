-- The shop window dresses for whoever is behind the counter. Each race has its colours, its ornaments (drawn on the
-- canvas: a goblin's patched leather and rivets, a moogle's pom-pom, Elvaan gilt, Tarutaru stars, Mithran claw
-- marks, Galkan stone...), a name for the shop and a voice: what the shopkeeper says when you come in, buy, can't
-- afford something or find it sold out.
--
-- A theme: key, label (what kind of shopkeeper), shop(name) -> the shop's name, money(n, currency) -> the balance
-- as they'd say it, colours (bg, bg2 the header's gradient; body, card, card_hi, edge, accent, title, text, muted,
-- price, buy, buy_edge, bad), voice { greet, buy, poor, sold, browse }, and deco(c, w, h): ornaments over the header
-- band (h tall) of a w-wide canvas; card(c, x, y, w, h, selected): extra touches on an item card.

local T = {}

local function rgba(r, g, b, a) return { r, g, b, a or 255 } end

-- Small drawing helpers
local function stitches(c, x1, y, x2, colour, step)
  step = step or 7
  local x = x1
  while x + 4 <= x2 do
    c:line(x, y, x + 4, y, colour, 1.2)
    x = x + step
  end
end

local function vstitches(c, x, y1, y2, colour, step)
  step = step or 7
  local y = y1
  while y + 4 <= y2 do
    c:line(x, y, x, y + 4, colour, 1.2)
    y = y + step
  end
end

local function rivet(c, x, y, colour)
  c:circle(x, y, 3, colour)
  c:circle(x - 1, y - 1, 1.2, rgba(255, 255, 255, 110))
end

local function star(c, x, y, r, colour)
  c.draw[#c.draw + 1] = { type = "tri", x1 = x, y1 = y - r, x2 = x - r * 0.35, y2 = y, x3 = x + r * 0.35, y3 = y, colour = colour }
  c.draw[#c.draw + 1] = { type = "tri", x1 = x, y1 = y + r, x2 = x - r * 0.35, y2 = y, x3 = x + r * 0.35, y3 = y, colour = colour }
  c.draw[#c.draw + 1] = { type = "tri", x1 = x - r, y1 = y, x2 = x, y2 = y - r * 0.35, x3 = x, y3 = y + r * 0.35, colour = colour }
  c.draw[#c.draw + 1] = { type = "tri", x1 = x + r, y1 = y, x2 = x, y2 = y - r * 0.35, x3 = x, y3 = y + r * 0.35, colour = colour }
end

local function diamond(c, x, y, r, colour)
  c.draw[#c.draw + 1] = { type = "tri", x1 = x - r, y1 = y, x2 = x, y2 = y - r, x3 = x + r, y3 = y, colour = colour }
  c.draw[#c.draw + 1] = { type = "tri", x1 = x - r, y1 = y, x2 = x, y2 = y + r, x3 = x + r, y3 = y, colour = colour }
end

local function tri(c, x1, y1, x2, y2, x3, y3, colour)
  c.draw[#c.draw + 1] = { type = "tri", x1 = x1, y1 = y1, x2 = x2, y2 = y2, x3 = x3, y3 = y3, colour = colour }
end

T.helpers = { stitches = stitches, rivet = rivet, star = star, diamond = diamond, tri = tri }

local function poss(name)
  name = name or "Merchant"
  return name:sub(-1) == "s" and (name .. "'") or (name .. "'s")
end

----------------------------------------------------------------------------------------------------------------------
-- Goblin: a patched leather sack of a stall, rivets, crooked stitching, "shinies"
----------------------------------------------------------------------------------------------------------------------
T.goblin = {
  key = "goblin", label = "Goblin merchant",
  shop = function(n) return poss(n) .. " Sack o' Stuff" end,
  money = function(n) return ("%d shinies"):format(n) end,
  colours = {
    bg = rgba(92, 64, 38), bg2 = rgba(58, 40, 24), body = rgba(40, 30, 20, 250), card = rgba(120, 96, 62, 70),
    card_hi = rgba(170, 132, 80, 120), edge = rgba(196, 150, 84, 150), accent = rgba(140, 196, 70), title = rgba(232, 214, 150),
    text = rgba(240, 228, 200), muted = rgba(190, 170, 130), price = rgba(214, 232, 120), buy = rgba(96, 132, 48, 200),
    buy_edge = rgba(170, 220, 90), bad = rgba(236, 120, 90),
  },
  voice = {
    greet = { "Hee hee! Shiny shiny, you buy?", "Grizznix no sell junk. Mostly.", "No ask where stuff come from!" },
    buy = { "Good trade! No refunds, hee hee!", "Yesss, shiny for Grizznix!", "You got good nose for bargain!" },
    poor = { "No shinies, no stuff! Go dig, go dig.", "Pockets empty? Grizznix sad." },
    sold = { "All gone! Too popular, Grizznix is.", "Sold out! You too slow, hee hee." },
    browse = { "Look, look! Touch with eyes only.", "Everything fair price. Goblin fair!" },
  },
  deco = function(c, w, h)
    local patch = { rgba(118, 84, 50), rgba(80, 58, 34), rgba(104, 92, 54) }
    c:rect(w - 150, 6, 58, 26, patch[1], { round = 3 })
    stitches(c, w - 147, 9, w - 95, rgba(232, 214, 150, 170))
    stitches(c, w - 147, 29, w - 95, rgba(232, 214, 150, 170))
    c:rect(w - 86, 18, 40, 30, patch[3], { round = 3 })
    vstitches(c, w - 83, 21, 46, rgba(232, 214, 150, 160))
    vstitches(c, w - 49, 21, 46, rgba(232, 214, 150, 160))
    c:rect(10, h - 16, 70, 12, patch[2], { round = 2 })
    stitches(c, 13, h - 10, 78, rgba(232, 214, 150, 140), 6)
    for _, p in ipairs({ { 8, 8 }, { w - 8, 8 }, { 8, h - 8 }, { w - 8, h - 8 } }) do rivet(c, p[1], p[2], rgba(170, 150, 110)) end
    stitches(c, 6, h - 2, w - 6, rgba(232, 214, 150, 110), 9)
  end,
  card = function(c, x, y, w, h)
    stitches(c, x + 4, y + 3, x + w - 4, rgba(232, 214, 150, 70), 8)
  end,
}

----------------------------------------------------------------------------------------------------------------------
-- Moogle: soft cream and pink, a red pom-pom, little wings, kupo
----------------------------------------------------------------------------------------------------------------------
T.moogle = {
  key = "moogle", label = "Moogle merchant",
  shop = function(n) return poss(n) .. " Kupo Shoppe" end,
  money = function(n, cur, short) return short and ("%d %s"):format(n, cur) or ("%d %s, kupo"):format(n, cur) end,
  colours = {
    bg = rgba(246, 226, 230), bg2 = rgba(226, 196, 210), body = rgba(70, 52, 64, 245), card = rgba(255, 236, 244, 40),
    card_hi = rgba(255, 190, 214, 90), edge = rgba(240, 170, 196, 170), accent = rgba(226, 64, 96), title = rgba(120, 50, 74),
    text = rgba(255, 240, 246), muted = rgba(226, 196, 210), price = rgba(255, 196, 120), buy = rgba(204, 70, 104, 210),
    buy_edge = rgba(255, 170, 196), bad = rgba(255, 140, 140),
  },
  title_on_light = true,
  voice = {
    greet = { "Kupo! Welcome, welcome, kupo!", "Mog has the finest things, kupo!" },
    buy = { "Thank you kupo! Come again, kupo!", "Kupopo! Good choice!" },
    poor = { "Not enough Everglass, kupo...", "Mog can't give discounts, kupo. Sorry!" },
    sold = { "Sold out, kupo! Mog will find more!" },
    browse = { "Take your time, kupo!" },
  },
  deco = function(c, w, h)
    local cx = w - 70
    c:line(cx, 30, cx + 8, 12, rgba(120, 60, 70), 2)
    c:circle(cx + 9, 10, 7, rgba(226, 48, 80))
    c:circle(cx + 7, 8, 2, rgba(255, 200, 210))
    tri(c, cx - 30, 30, cx - 10, 20, cx - 12, 38, rgba(170, 120, 200, 200))
    tri(c, cx + 30, 30, cx + 10, 20, cx + 12, 38, rgba(170, 120, 200, 200))
    for i = 0, math.floor(w / 18) do
      c:circle(9 + i * 18, h - 4, 2.2, (i % 2 == 0) and rgba(240, 150, 180) or rgba(255, 255, 255, 200))
    end
  end,
}

----------------------------------------------------------------------------------------------------------------------
-- Hume: an honest wooden counter, planks and brass nails
----------------------------------------------------------------------------------------------------------------------
T.hume = {
  key = "hume", label = "Hume merchant",
  shop = function(n) return poss(n) .. " General Goods" end,
  money = function(n, cur) return ("%d %s"):format(n, cur) end,
  colours = {
    bg = rgba(132, 92, 58), bg2 = rgba(92, 62, 38), body = rgba(34, 30, 28, 250), card = rgba(255, 240, 220, 22),
    card_hi = rgba(222, 170, 100, 80), edge = rgba(200, 160, 100, 140), accent = rgba(222, 180, 90), title = rgba(250, 232, 196),
    text = rgba(240, 232, 218), muted = rgba(190, 176, 156), price = rgba(236, 200, 110), buy = rgba(150, 104, 54, 210),
    buy_edge = rgba(226, 182, 110), bad = rgba(236, 120, 96),
  },
  voice = {
    greet = { "Take your time, friend.", "Good to see a new face. Have a look around." },
    buy = { "Pleasure doing business.", "That'll serve you well. Safe travels." },
    poor = { "Can't do it for that, I'm afraid." },
    sold = { "Fresh out of those, sorry." },
    browse = { "Everything here's been on the road with me." },
  },
  deco = function(c, w, h)
    for y = 14, h - 4, 14 do c:line(0, y, w, y, rgba(60, 38, 20, 120), 1) end
    for row = 0, math.floor(h / 14) do
      local y = 7 + row * 14
      local off = (row % 2) * 60
      for x = 40 + off, w - 20, 120 do c:circle(x, y, 1.6, rgba(226, 190, 120, 180)) end
    end
  end,
}

----------------------------------------------------------------------------------------------------------------------
-- Elvaan: San d'Oria's navy and silver, gilt filigree, the formal voice of a royal purveyor
----------------------------------------------------------------------------------------------------------------------
T.elvaan = {
  key = "elvaan", label = "Elvaan purveyor",
  shop = function(n) return "The House of " .. (n or "the Kingdom") end,
  money = function(n, cur) return ("%d %s"):format(n, cur) end,
  colours = {
    bg = rgba(30, 46, 92), bg2 = rgba(16, 24, 54), body = rgba(16, 20, 36, 250), card = rgba(200, 210, 240, 20),
    card_hi = rgba(120, 140, 210, 80), edge = rgba(200, 186, 130, 150), accent = rgba(220, 196, 120), title = rgba(236, 226, 196),
    text = rgba(232, 234, 244), muted = rgba(170, 178, 204), price = rgba(230, 210, 140), buy = rgba(44, 64, 130, 220),
    buy_edge = rgba(220, 196, 120), bad = rgba(232, 120, 120),
  },
  voice = {
    greet = { "Welcome. Only the finest wares are kept here.", "You honour this house. Choose with care." },
    buy = { "A fine choice. May it serve you with honour.", "The house thanks you." },
    poor = { "Regrettably, you lack the means." },
    sold = { "That is no longer available. My apologies." },
    browse = { "Every piece is worthy of the Kingdom." },
  },
  deco = function(c, w, h)
    local gold = rgba(220, 196, 120, 200)
    c:line(6, 5, w - 6, 5, gold, 1)
    c:line(6, 8, w - 6, 8, rgba(220, 196, 120, 90), 1)
    c:line(6, h - 5, w - 6, h - 5, gold, 1)
    c:line(6, h - 8, w - 6, h - 8, rgba(220, 196, 120, 90), 1)
    for _, x in ipairs({ 10, w - 10 }) do diamond(c, x, 6.5, 5, gold) diamond(c, x, h - 6.5, 5, gold) end
    local cx = w - 60
    c:circle(cx, 22, 4, gold)
    c:circle(cx - 8, 30, 3, gold)
    c:circle(cx + 8, 30, 3, gold)
    c:line(cx, 26, cx, 44, gold, 1.5)
  end,
}

----------------------------------------------------------------------------------------------------------------------
-- Tarutaru: Windurst's night sky, stars and a crescent, a bubbly sing-song voice
----------------------------------------------------------------------------------------------------------------------
T.tarutaru = {
  key = "tarutaru", label = "Tarutaru shopkeep",
  shop = function(n) return poss(n) .. " Wondrous Emporium" end,
  money = function(n, cur) return ("%d %s"):format(n, cur) end,
  colours = {
    bg = rgba(62, 42, 112), bg2 = rgba(30, 20, 66), body = rgba(24, 18, 44, 250), card = rgba(220, 200, 255, 22),
    card_hi = rgba(160, 120, 240, 90), edge = rgba(170, 150, 240, 150), accent = rgba(120, 220, 160), title = rgba(250, 236, 160),
    text = rgba(240, 236, 255), muted = rgba(190, 180, 220), price = rgba(250, 220, 120), buy = rgba(90, 64, 170, 220),
    buy_edge = rgba(170, 230, 180), bad = rgba(255, 140, 150),
  },
  voice = {
    greet = { "Ooh, a customer! Look-look, everything's magical-ish!", "Welcome-welcome! Mind the sparkles!" },
    buy = { "Yaaay! Pleasure doing business-wusiness!", "Ooh, that one's my favourite-wavourite!" },
    poor = { "Hmm-hmm, not enough Everglass, I think-think." },
    sold = { "All gone-gone! So sorry-worry!" },
    browse = { "Point at things! I'll tell you all about them!" },
  },
  deco = function(c, w, h)
    local stars = { { w - 150, 12, 4 }, { w - 120, 30, 3 }, { w - 96, 10, 2.5 }, { w - 40, 40, 3 }, { 90, h - 10, 2.5 },
      { 200, 8, 2 }, { w - 200, 44, 2 }, { w - 20, 14, 2.5 } }
    for _, s in ipairs(stars) do star(c, s[1], s[2], s[3] * 1.6, rgba(255, 240, 170, 220)) end
    c:circle(w - 66, 24, 12, rgba(255, 238, 170, 230))
    c:circle(w - 60, 20, 11, T.tarutaru.colours.bg)
    local rainbow = { rgba(240, 90, 90), rgba(240, 170, 80), rgba(240, 230, 100), rgba(110, 210, 120), rgba(100, 160, 240), rgba(170, 110, 230) }
    local seg = (w - 12) / #rainbow
    for i, col in ipairs(rainbow) do c:rect(6 + (i - 1) * seg, h - 3, seg, 2, col) end
  end,
}

----------------------------------------------------------------------------------------------------------------------
-- Mithra: sun-warm sand and jungle green, claw marks and a paw print
----------------------------------------------------------------------------------------------------------------------
T.mithra = {
  key = "mithra", label = "Mithran trader",
  shop = function(n) return poss(n) .. " Hunt & Haul" end,
  money = function(n, cur) return ("%d %s"):format(n, cur) end,
  colours = {
    bg = rgba(186, 140, 72), bg2 = rgba(56, 96, 70), body = rgba(26, 34, 28, 250), card = rgba(240, 230, 190, 22),
    card_hi = rgba(110, 170, 110, 90), edge = rgba(214, 180, 110, 150), accent = rgba(120, 200, 120), title = rgba(255, 244, 210),
    text = rgba(246, 240, 222), muted = rgba(196, 190, 160), price = rgba(250, 210, 110), buy = rgba(70, 120, 70, 220),
    buy_edge = rgba(150, 220, 140), bad = rgba(240, 120, 100),
  },
  voice = {
    greet = { "Welcome, welcome! Fresh from the hunt, meow.", "Ooh, you have the look of a hunter!" },
    buy = { "Good catch! Ehehe.", "A fine prize for a fine hunter!" },
    poor = { "Not enough Everglass, no? Hunt some more!" },
    sold = { "Gone already! Too slow, too slow." },
    browse = { "Sniff around, take your time." },
  },
  deco = function(c, w, h)
    local claw = rgba(60, 30, 14, 170)
    for i = 0, 2 do c:line(w - 120 + i * 10, 8, w - 104 + i * 10, h - 10, claw, 2.4) end
    local px, py = w - 50, h / 2 + 4
    local paw = rgba(255, 240, 210, 150)
    c:circle(px, py + 4, 7, paw)
    for _, d in ipairs({ { -8, -6 }, { -3, -10 }, { 3, -10 }, { 8, -6 } }) do c:circle(px + d[1], py + d[2], 2.6, paw) end
  end,
}

----------------------------------------------------------------------------------------------------------------------
-- Galka: grey stone blocks and bronze bolts, few words
----------------------------------------------------------------------------------------------------------------------
T.galka = {
  key = "galka", label = "Galkan trader",
  shop = function(n) return poss(n) .. " Trading Post" end,
  money = function(n, cur) return ("%d %s"):format(n, cur) end,
  colours = {
    bg = rgba(104, 100, 96), bg2 = rgba(62, 58, 56), body = rgba(30, 28, 28, 250), card = rgba(230, 220, 210, 20),
    card_hi = rgba(200, 140, 80, 80), edge = rgba(176, 130, 80, 150), accent = rgba(214, 130, 60), title = rgba(240, 226, 206),
    text = rgba(236, 230, 222), muted = rgba(180, 172, 162), price = rgba(230, 170, 90), buy = rgba(140, 80, 40, 220),
    buy_edge = rgba(220, 150, 80), bad = rgba(236, 110, 90),
  },
  voice = {
    greet = { "Hm. Look all you like.", "Bastok-made. Built to last." },
    buy = { "A fair trade. Good.", "Hm. Take care of it." },
    poor = { "You haven't the Everglass." },
    sold = { "None left. Come back later." },
    browse = { "Ask if you need something." },
  },
  deco = function(c, w, h)
    local line = rgba(40, 36, 34, 150)
    for row = 0, 3 do
      local y = row * 15
      c:line(0, y, w, y, line, 1)
      local off = (row % 2) * 30
      for x = off, w, 60 do c:line(x, y, x, y + 15, line, 1) end
    end
    for _, x in ipairs({ 12, w - 12 }) do rivet(c, x, 12, rgba(196, 140, 80)) rivet(c, x, h - 12, rgba(196, 140, 80)) end
  end,
}

----------------------------------------------------------------------------------------------------------------------
-- Quadav, Yagudo, Orc, Tonberry: the beastmen and stranger folk who sometimes trade
----------------------------------------------------------------------------------------------------------------------
T.quadav = {
  key = "quadav", label = "Quadav hoarder",
  shop = function(n) return poss(n) .. " Shell Hoard" end,
  money = function(n) return ("%d shiny-shinies"):format(n) end,
  colours = {
    bg = rgba(150, 110, 50), bg2 = rgba(80, 56, 26), body = rgba(34, 26, 16, 250), card = rgba(240, 210, 150, 22),
    card_hi = rgba(220, 170, 80, 90), edge = rgba(220, 170, 80, 150), accent = rgba(240, 190, 80), title = rgba(255, 236, 190),
    text = rgba(244, 232, 210), muted = rgba(200, 180, 140), price = rgba(250, 210, 100), buy = rgba(150, 100, 30, 220),
    buy_edge = rgba(240, 190, 80), bad = rgba(236, 120, 90),
  },
  voice = {
    greet = { "Hrrn. Shiny for shiny. Trade, trade." }, buy = { "Hrrn! Good shiny." }, poor = { "No shiny, no trade. Hrrn." },
    sold = { "Hoard empty. Go." }, browse = { "Look. No touch shell." },
  },
  deco = function(c, w, h)
    for row = 0, 2 do
      for i = 0, math.floor(w / 26) do
        diamond(c, 13 + i * 26 + (row % 2) * 13, 10 + row * 16, 7, rgba(60, 40, 16, 70))
      end
    end
  end,
}

T.yagudo = {
  key = "yagudo", label = "Yagudo vendor",
  shop = function(n) return poss(n) .. " Sacred Offerings" end,
  money = function(n, cur) return ("%d %s"):format(n, cur) end,
  colours = {
    bg = rgba(84, 40, 96), bg2 = rgba(44, 20, 54), body = rgba(28, 16, 32, 250), card = rgba(240, 210, 255, 20),
    card_hi = rgba(220, 180, 80, 80), edge = rgba(220, 180, 80, 150), accent = rgba(230, 190, 80), title = rgba(250, 232, 180),
    text = rgba(244, 234, 248), muted = rgba(196, 176, 206), price = rgba(240, 200, 100), buy = rgba(110, 50, 120, 220),
    buy_edge = rgba(230, 190, 80), bad = rgba(240, 120, 120),
  },
  voice = {
    greet = { "The Divine Kindred sells. You buy. Praise!" }, buy = { "Praise be! Blessed trade!" },
    poor = { "Unworthy purse! Begone, begone." }, sold = { "Offering taken. None remain." }, browse = { "Behold. Holy wares." },
  },
  deco = function(c, w, h)
    for i = 0, 6 do
      local x = w - 30 - i * 16
      tri(c, x, h - 4, x - 6, 8 + (i % 2) * 6, x + 6, 8 + (i % 2) * 6, rgba(230, 190, 80, 90 + i * 10))
    end
  end,
}

T.orc = {
  key = "orc", label = "Orcish dealer",
  shop = function(n) return poss(n) .. " War Loot" end,
  money = function(n) return ("%d shinies"):format(n) end,
  colours = {
    bg = rgba(110, 30, 26), bg2 = rgba(40, 14, 12), body = rgba(28, 16, 14, 250), card = rgba(255, 220, 200, 18),
    card_hi = rgba(200, 80, 60, 90), edge = rgba(200, 90, 60, 150), accent = rgba(230, 120, 60), title = rgba(250, 220, 190),
    text = rgba(244, 228, 220), muted = rgba(200, 170, 160), price = rgba(240, 180, 100), buy = rgba(140, 40, 30, 220),
    buy_edge = rgba(230, 120, 60), bad = rgba(255, 150, 120),
  },
  voice = {
    greet = { "Hah! You want loot? You pay!" }, buy = { "Hah! Good. Now go." }, poor = { "Weak purse! No loot for you!" },
    sold = { "Gone! Somebody else take it. Hah!" }, browse = { "Look fast. No haggle." },
  },
  deco = function(c, w, h)
    for x = 0, w - 16, 16 do tri(c, x, h, x + 8, h - 9, x + 16, h, rgba(230, 220, 200, 150)) end
    for x = 8, w - 16, 16 do tri(c, x, 0, x + 8, 7, x + 16, 0, rgba(30, 10, 8, 160)) end
  end,
}

T.tonberry = {
  key = "tonberry", label = "Tonberry",
  shop = function(n) return poss(n) .. " Grudge Shop" end,
  money = function(n, cur) return ("%d %s"):format(n, cur) end,
  colours = {
    bg = rgba(34, 58, 44), bg2 = rgba(12, 22, 16), body = rgba(12, 18, 14, 250), card = rgba(200, 255, 220, 16),
    card_hi = rgba(240, 200, 100, 70), edge = rgba(160, 200, 140, 120), accent = rgba(255, 210, 110), title = rgba(230, 240, 210),
    text = rgba(226, 236, 226), muted = rgba(150, 176, 160), price = rgba(250, 210, 120), buy = rgba(40, 80, 50, 220),
    buy_edge = rgba(255, 210, 110), bad = rgba(240, 120, 110),
  },
  voice = {
    greet = { "..." }, buy = { "......" }, poor = { "... ..." }, sold = { "..." }, browse = { "..." },
  },
  deco = function(c, w, h)
    local x, y = w - 60, h / 2
    for r = 22, 8, -7 do c:circle(x, y, r, rgba(255, 210, 110, 22)) end
    c:rect(x - 5, y - 8, 10, 14, rgba(255, 220, 140, 230), { round = 2 })
    c:line(x, y - 14, x, y - 8, rgba(200, 200, 180), 1.5)
    c:line(x - 30, y + 10, x - 18, y - 2, rgba(230, 236, 240, 200), 2)
  end,
}

----------------------------------------------------------------------------------------------------------------------
-- The Artisans Guild (the Everglass Shop from !shop): TimelessXI's charcoal and gold, an hourglass
----------------------------------------------------------------------------------------------------------------------
T.guild = {
  key = "guild", label = "Artisans Guild",
  shop = function() return "Everglass Shop" end,
  money = function(n, cur) return ("%d %s"):format(n, cur) end,
  colours = {
    bg = rgba(44, 46, 54), bg2 = rgba(24, 26, 32), body = rgba(20, 22, 28, 250), card = rgba(255, 255, 255, 14),
    card_hi = rgba(228, 192, 119, 50), edge = rgba(228, 192, 119, 120), accent = rgba(228, 192, 119), title = rgba(228, 192, 119),
    text = rgba(232, 226, 212), muted = rgba(160, 168, 176), price = rgba(228, 192, 119), buy = rgba(170, 128, 72, 170),
    buy_edge = rgba(228, 192, 119), bad = rgba(236, 108, 96),
  },
  voice = {
    greet = { "Everything the Artisans Guild sells for Everglass." },
    buy = { "Done. The guild thanks you." }, poor = { "Not enough Everglass." }, sold = { "None left." },
    browse = { "Point at anything to see what it does." },
  },
  deco = function(c, w, h)
    local gold = rgba(228, 192, 119, 200)
    local x, y = w - 50, h / 2
    tri(c, x - 9, y - 14, x + 9, y - 14, x, y, gold)
    tri(c, x - 9, y + 14, x + 9, y + 14, x, y, rgba(228, 192, 119, 120))
    c:line(x - 11, y - 15, x + 11, y - 15, gold, 2)
    c:line(x - 11, y + 15, x + 11, y + 15, gold, 2)
    c:line(6, h - 2, w - 6, h - 2, rgba(228, 192, 119, 90), 1)
  end,
}

T.ORDER = { "goblin", "moogle", "hume", "elvaan", "tarutaru", "mithra", "galka", "quadav", "yagudo", "orc", "tonberry", "guild" }

-- Race numbers as entities give them, to theme keys
local RACE_NUMBERS = { [1] = "hume", [2] = "hume", [3] = "elvaan", [4] = "elvaan", [5] = "tarutaru", [6] = "tarutaru",
  [7] = "mithra", [8] = "galka" }

-- The theme for a shop: the race the server names; else the keeper's own, as the client sees them; else the guild.
function T.for_shop(shop)
  local key = shop and shop.race or ""
  if T[key] and T[key].colours then return T[key] end
  if shop and (shop.index or 0) > 0 then
    local ok, e = pcall(game.entity, shop.index)
    if ok and e and e.race and RACE_NUMBERS[e.race] then return T[RACE_NUMBERS[e.race]] end
  end
  return T.guild
end

-- One of a theme's lines of a kind, picked by a number (so it stays put between redraws).
function T.say(theme, kind, n)
  local list = theme.voice[kind] or theme.voice.greet
  return list[((n or 1) - 1) % #list + 1]
end

return T
