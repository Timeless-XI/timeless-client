-- How to play: Cardian's rules in a window of four pages (the game, house rules and the world, collecting, where to
-- play), opened from the table and the cardbook. What it says is what the server's rules do
-- (modules/cardian/lua/cardian_engine.lua and cardian.lua).

local S = require("lib.style")
local Canvas = require("lib.canvas")
local Art = require("lib.art")

local H = {}

H.WIN = "cardian_rules"
H.W = 640
H.H = 600
H.state = { page = 1 }

local PAGES = {
  { label = "The game" },
  { label = "Rules & world" },
  { label = "Collecting" },
  { label = "Where to play" },
}

-- Each page: a list of { heading, text } blocks
local TEXT = {
  {
    { "Take two of three fronts",
      "The board is 3 squares by 3. Each column is a front, a real place in Vana'diel, named at its head. When the board is full, owning two of a column's three squares takes that front; take two fronts and you win." },
    { "Taking turns",
      "Each of you brings a hand of five. On your turn, pick a card from your hand, then an empty square. Whoever goes first plays five cards, the other four." },
    { "Capturing",
      "A card has four numbers: top, right, bottom and left (A is 10). When you place it next to an enemy card, compare the numbers that touch: if yours is higher, their card flips to your colour (blue). Captured cards can be taken back the same way." },
    { "Same, Plus and Combo",
      "Same: if your card's numbers equal the touching numbers of two or more neighbours, the enemy ones among them flip. Plus: if two or more neighbours' touching numbers add up to the same total with yours, the enemy ones flip. Combo: a card flipped by Same or Plus then flips its own weaker enemy neighbours, and so on." },
  },
  {
    { "House rules",
      "Each table has its own, shown at the top of the board. The Steaming Sheep plays Open (both hands shown) and Same. Shaali plays Open, Plus and Elemental (two more marked squares). Two players play Open, Same and Plus. Combo comes with Same and Plus." },
    { "The world moves the numbers",
      "The line under your opponent's name says what is in play. Numbers never go below 1 or above A." },
    { "The day", "Cards of the day's element get +1 on every side (Firesday: fire cards, and so on)." },
    { "Weather: marked squares", "A front with weather marks its middle square with that element: a card of the element gets +1 there, any other card -1." },
    { "Conquest", "Where the beastmen hold a front's region, Beastman cards get +1 in that column." },
    { "The moon", "At full moon, Beastman cards get +1. At new moon, Undead cards get +1." },
  },
  {
    { "Your cardbook and hand",
      "Every card you own is in your cardbook, with the ones you haven't found as silhouettes. Click a card to see it big, with the monster turning in it. Pick the five cards you bring to the table (one legendary at most) and press Save hand." },
    { "Levels",
      "Cards go from level 1 to 10: field monsters 1 to 5, notorious monsters 6 and 7, legends 8 to 10. Higher levels have bigger numbers." },
    { "Monsters drop them",
      "Monsters of a card's family drop it now and then, when they are within 10 levels of you. Rares come from notorious monsters, and a notorious monster can drop its own card." },
    { "Foils",
      "Champions drop foils of their family's card: Gilded (its lowest side +1), Nightborne (every side +1 from 20:00 to 6:00), Rift-touched (the world's numbers don't touch it) and Juggernaut (its highest side +2)." },
    { "Legendaries", "Beat a card player three times and they give you their own card." },
  },
  {
    { "Where to play",
      "The card players sit in the Steaming Sheep in Port Bastok after dark, from 19:00 to 4:00: Hrodgar is the gentlest, then Fenwick and Kipiko; Dagny plays to win. Shaali Nine-Lives, the travelling card shark, spends a day in each town; the cardbook says where she is." },
    { "Friendly or for a card",
      "A friendly game costs nothing. For a card, you put up the best card in your hand you have a spare of (never a legendary): win, and you take your opponent's best card that isn't a legendary; lose, and they take yours." },
    { "Against another player",
      "Target a player standing close by, open your cardbook and challenge them. They accept or decline. Each turn waits 60 seconds, then a card is played for whoever is slow. Folding, leaving the zone or logging out loses the game." },
    { "Leaving", "Closing the table doesn't end the game: Back to the game in the cardbook opens it again. Fold gives up the game (and the card you put up)." },
  },
}

function H.draw(c)
  local st = H.state
  c:text(0, 2, "How to play Cardian", { size = 1.25, bold = true, colour = S.GOLD })
  c:switch(H.W - 430, 0, 26, PAGES, st.page, { on = function(i) st.page = i H.render() end, key = "page" })
  local y = 44
  for _, block in ipairs(TEXT[st.page] or {}) do
    c:text(0, y, block[1], { size = 0.95, bold = true, colour = S.TEXT })
    y = c:para(0, y + 20, H.W - 4, block[2], { size = 0.85, colour = S.MUTED }) + 8
  end
  -- An example card on the last page
  if st.page == 1 then
    Art.card(c, H.W - Art.W * 0.9 - 4, H.H - Art.H * 0.9 - 44, 0.9, 1, { detail = "small" })
  end
  c:button(H.W - 110, H.H - 30, 110, 28, st.page < #PAGES and "Next" or "Done", { style = "primary", key = "next",
    on = function() if st.page < #PAGES then st.page = st.page + 1 H.render() else ui.close(H.WIN) end end })
end

function H.setup()
  ui.window(H.WIN, { title = "Cardian", width = H.W + 20, help = "Cardian's rules: the game, house rules and the world's numbers, collecting cards, and where to play.",
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
