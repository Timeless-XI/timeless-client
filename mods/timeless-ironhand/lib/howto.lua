-- How to play: Ironhand's rules in a window of four pages (the game, the world, collecting, the tables), opened from
-- the table, the cardbook and the Timeless menu card. What it says is what the server's rules do
-- (modules/ironhand/lua/ironhand_engine.lua and ironhand.lua).

local S = require("lib.style")
local Canvas = require("lib.canvas")
local Art = require("lib.art")

local H = {}

H.WIN = "ironhand_rules"
H.W = 640
H.H = 560
H.state = { page = 1 }

local PAGES = {
  { label = "The game" },
  { label = "The world" },
  { label = "Collecting" },
  { label = "At the table" },
}

-- Each page: a list of { heading, text } blocks
local TEXT = {
  {
    { "Take two of three fronts",
      "A game is fought over three fronts, real places in Vana'diel. After six rounds, whoever has more power on a front takes it. Take two fronts and you win. If you each take one (or none), the higher power over all three fronts wins." },
    { "Each round",
      "You get as many crystals as the round's number: 1 in round 1, up to 6 in round 6. Crystals you don't spend are lost. A card costs the number in its top corner. Pick a card from your hand, then pick a front to put it on. Click a card on a front to take it back before you reveal." },
    { "Reveal",
      "When you press Reveal, both of you turn your cards over at once. Then each card's ability happens: yours first, then theirs, in the order they were played." },
    { "Your hand and deck",
      "You start with 4 cards and draw one more each round, up to 8 in hand. Each front holds 4 of your cards (a few cards can squeeze in a fifth)." },
    { "Power",
      "A card's power is the number at its bottom, plus bonuses (the green number beside it on the table) and minus anything that weakened it (red). A front's power is all your cards there added up. If a front is tied, a Gilded card wins it; otherwise nobody takes it." },
  },
  {
    { "The world plays too",
      "The line under your opponent's name lists what is in play. A game played on Firesday, at night, in the rain, or while the beastmen hold a region plays differently from one at noon." },
    { "The day", "Cards of the day's element get +1 (Firesday: fire cards +1, and so on)." },
    { "Weather", "Each front has the weather it has in the world right now. Cards of that weather's element get +1 there." },
    { "Conquest", "Where the beastmen hold a front's region, Beastman cards get +1 on that front." },
    { "The moon", "At full moon, Beastman cards get +1. At new moon, Undead cards get +1." },
    { "Night", "From 20:00 to 4:00, night cards are awake: their night abilities work. A Lamplighter on the other side keeps them asleep." },
    { "Element edge",
      "A card gets +1 while an enemy card on the same front has the element it beats: fire beats ice, ice beats wind, wind beats earth, earth beats thunder, thunder beats water, water beats fire; light and dark beat each other." },
    { "Round 5", "At the end of round 5, Self-Destruct and Venom go off." },
  },
  {
    { "Your cardbook",
      "Every card you own is in your cardbook, with the ones you haven't found as silhouettes. Click a card to see it big, with the monster turning in it. Your deck is 15 cards: at most 2 of any card, and only one legendary. Change it in the cardbook and press Save deck." },
    { "Monsters drop them",
      "Monsters of a card's family drop it now and then, when they are within 10 levels of you. Rares come from notorious monsters, and a notorious monster can drop its own legendary card." },
    { "Foils",
      "Champions drop foils of their family's card: Gilded (+1, and wins ties), Nightborne (+1 at night, and its night lasts until 6:00), Rift-touched (+1, and weather and conquest don't touch it) and Juggernaut (+2, but it costs 1 more)." },
    { "Legendaries",
      "Beat a card player at the Steaming Sheep three times and they slide their own card across the table." },
  },
  {
    { "The Steaming Sheep",
      "The card players sit in the Steaming Sheep in Port Bastok after dark, from 19:00 to 4:00. Talk to one to sit down: Hrodgar is the gentlest, then Fenwick and Kipiko; Dagny plays to win." },
    { "Friendly or for a card",
      "A friendly game costs nothing. A game for a card needs a common you have two of: win, and you take a common from their deck; lose, and they take your spare." },
    { "Leaving", "Closing the table doesn't end the game: Back to the game in the cardbook opens it again. Fold gives up the game (and the card you put up). Leaving the zone ends it." },
    { "Reading a card",
      "Top left: its element crystal. Top right: its cost. Under the name: its family and element. Under the picture: what it is, what it does, and its power. On the table, a strip shows its name and its power now." },
  },
}

function H.draw(c)
  local st = H.state
  c:text(0, 2, "How to play Ironhand", { size = 1.25, bold = true, colour = S.GOLD })
  c:switch(H.W - 430, 0, 26, PAGES, st.page, { on = function(i) st.page = i H.render() end, key = "page" })
  local y = 44
  for _, block in ipairs(TEXT[st.page] or {}) do
    c:text(0, y, block[1], { size = 0.95, bold = true, colour = S.TEXT })
    y = c:para(0, y + 20, H.W - 4, block[2], { size = 0.85, colour = S.MUTED }) + 8
  end
  -- An example card on the last page
  if st.page == 4 then
    Art.card(c, H.W - Art.W * 0.9 - 4, H.H - Art.H * 0.9 - 44, 0.9, 1, { detail = "small" })
  end
  c:button(H.W - 110, H.H - 30, 110, 28, st.page < #PAGES and "Next" or "Done", { style = "primary", key = "next",
    on = function() if st.page < #PAGES then st.page = st.page + 1 H.render() else ui.close(H.WIN) end end })
end

function H.setup()
  ui.window(H.WIN, { title = "Ironhand", width = H.W + 20, help = "Ironhand's rules: the game, how the world changes it, collecting cards, and the Steaming Sheep's tables.",
    elements = { { type = "canvas", id = "rules", w = H.W, h = H.H, draw = {} } } })
  Canvas.add(H.WIN, "rules", H.W, H.draw)
  ui.on(H.WIN, function(win, e)
    if e.kind == "close" then return end
    Canvas.event(win, e)
  end)
end

function H.open(page)
  H.state.page = page or H.state.page or 1
  ui.open(H.WIN)
  H.render()
end

function H.render()
  if ui.is_open(H.WIN) then Canvas.render(H.WIN, "rules") end
end

H.TEXT, H.PAGES = TEXT, PAGES
return H
