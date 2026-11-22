-- Reads the server's Cardian feed (modules/cardian):
--   CD1|K                                   the cardbook follows: forget the old one
--   CD1|C|<id>:<n>,...                      cards you have, and how many
--   CD1|D|<id>,...                          your hand of five
--   CD1|W|<opponent>:<wins>,...             wins against each opponent
--   CD1|L|<town>|<next town>                where Shaali the card shark is today, and tomorrow
--   CD1|V|<binder 1/0>|<binder price>|<Card Mod here 1/0>   your Card Binder, and whether Card Mod works where you are
--   CD1|U|club|<rank, 0 none yet>|<next, beaten, locked>     the Card Club (its words are in lib/cards.lua)
--   CD1|U|masters|<job>,...                 the job cards you have
--   CD1|U|lost|<card id>|<holder, '' hidden, ? away>|<1 yours>   the lost cards
--   CD1|U|job|<your main job>
--   CD1|B                                   the end of the cardbook
--   CD1|M|<text>                            a word from the server (a hand it won't take, and why)
--   CD1|G|<opponent>|<friendly or ante>|<key>|<rules: open,same,plus,elemental>   a game starts (key "pvp": a player)
--   CD1|Y|<day element>|<night 1/0>|<moon text>                 the world at the table
--   CD1|F|<front>|<name>|<what its world does>                   the columns' fronts
--   CD1|X|<square>|<element>                                     a marked square
--   CD1|S|<whose turn: 1 you, 2 them>|<play or over>|<placed>    the table now (then O I J Z lines, E)
--   CD1|O|<square>|<card id>|<owner: 1 you, 2 them>|<t,r,b,l>    a card on the board, its numbers there
--   CD1|I|<n>|<card id>|<t,r,b,l>                                your hand
--   CD1|J|<n>|<card id, or 0 hidden>                             theirs
--   CD1|Z|<square>|<who: 1 you, 2 them>|<square:how,...>         the last move and what it flipped
--   CD1|E                                                        the end of the table
--   CD1|R|<winner 1/2/0>|<mine-theirs-who>,... x3|<text>         the game is over
--   CD1|N|<card id>|<how>                   you found a card (drop, nm, champion, bound, set, won, legend, gm)
--   CD1|Q|<player>|<friendly or ante>|<your card put up, or 0>   another player challenges you
-- No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "CD1"

function P.new_model()
  return { book = { have = {}, hand = {}, wins = {}, version = 0 }, pending_book = nil, game = nil, pending = nil,
    result = nil, found = {}, message = nil, version = 0, shark = nil }
end

function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

local function split(s, sep)
  local out = {}
  for part in ((s or "") .. sep):gmatch("(.-)" .. sep:gsub("%p", "%%%0")) do out[#out + 1] = part end
  return out
end

local function nums(s)
  local out = {}
  for n in (s or ""):gmatch("%d+") do out[#out + 1] = tonumber(n) end
  return out
end

local function new_table()
  return { board = {}, hand = {}, theirs = {}, last = nil }
end

-- Applies one line. Returns what changed: "book", "game", "state", "result", "found", "message", "invite"; true for
-- our other lines, false for anything else.
function P.feed(m, line, now)
  if type(line) ~= "string" then return false end
  local kind, rest = line:match("^CD1|(%u)|?(.*)$")
  if not kind then return false end
  local f = split(rest, "|")
  if kind == "K" then
    m.pending_book = { have = {}, hand = {}, wins = {} }
    return true
  elseif kind == "C" then
    local b = m.pending_book or m.book
    for id, n in (rest or ""):gmatch("(%d+):(%d+)") do b.have[tonumber(id)] = tonumber(n) end
    return true
  elseif kind == "D" then
    local b = m.pending_book or m.book
    b.hand = nums(rest)
    return true
  elseif kind == "W" then
    local b = m.pending_book or m.book
    for k, n in (rest or ""):gmatch("(%a+):(%d+)") do b.wins[k] = tonumber(n) end
    return true
  elseif kind == "L" then
    m.shark = { town = f[1] or "", next = f[2] or "" }
    return true
  elseif kind == "V" then
    local b = m.pending_book or m.book
    b.binder, b.binder_price, b.mod_here = f[1] == "1", tonumber(f[2]) or 0, f[3] == "1"
    return true
  elseif kind == "U" then
    local b = m.pending_book or m.book
    b.club = b.club or {}
    b.lost = b.lost or {}
    if f[1] == "club" then
      local r = tonumber(f[2]) or 0
      if r == 0 then b.club_locked = true else b.club[r] = f[3] end
    elseif f[1] == "masters" then
      b.job_cards = {}
      for j in (f[2] or ""):gmatch("%u+") do b.job_cards[j] = true end
    elseif f[1] == "lost" then
      b.lost[tonumber(f[2]) or 0] = { holder = f[3] or "", mine = f[4] == "1" }
    elseif f[1] == "job" then
      b.job = (f[2] or "") ~= "" and f[2] or nil
    end
    return true
  elseif kind == "B" then
    local b = m.pending_book or m.book
    b.version = (m.book.version or 0) + 1
    m.book, m.pending_book = b, nil
    return "book"
  elseif kind == "M" then
    m.message = { text = rest, at = now }
    return "message"
  elseif kind == "G" then
    local rules = {}
    for r in (f[4] or ""):gmatch("%a+") do rules[r] = true end
    m.game = { opponent = f[1] or "", stakes = f[2] or "friendly", key = f[3] or "", rules = rules, world = {}, fronts = { {}, {}, {} },
      squares = {}, now = new_table() }
    m.result = nil
    return "game"
  elseif kind == "Y" then
    if m.game then
      m.game.world = { day = f[1], night = f[2] == "1", moon = (f[3] or "") ~= "" and f[3] or nil }
    end
    return true
  elseif kind == "F" then
    local fr = m.game and m.game.fronts[tonumber(f[1]) or 0]
    if fr then fr.name, fr.twist = f[2] or "", f[3] or "" end
    return true
  elseif kind == "X" then
    if m.game then m.game.squares[tonumber(f[1]) or 0] = f[2] end
    return true
  elseif kind == "S" then
    if not m.game then return true end
    m.pending = new_table()
    m.pending.mine = f[1] == "1"
    m.pending.over = f[2] == "over"
    m.pending.placed = tonumber(f[3]) or 0
    return true
  elseif kind == "O" then
    if m.pending then
      m.pending.board[tonumber(f[1]) or 0] = { id = tonumber(f[2]), owner = tonumber(f[3]) or 1, sides = nums(f[4]) }
    end
    return true
  elseif kind == "I" then
    if m.pending then m.pending.hand[tonumber(f[1]) or (#m.pending.hand + 1)] = { id = tonumber(f[2]), sides = nums(f[3]) } end
    return true
  elseif kind == "J" then
    if m.pending then m.pending.theirs[tonumber(f[1]) or (#m.pending.theirs + 1)] = tonumber(f[2]) or 0 end
    return true
  elseif kind == "Z" then
    if m.pending then
      local flips = {}
      for cell, how in (f[3] or ""):gmatch("(%d+):(%a+)") do flips[#flips + 1] = { cell = tonumber(cell), how = how } end
      m.pending.last = { cell = tonumber(f[1]) or 0, who = tonumber(f[2]) or 1, flips = flips }
    end
    return true
  elseif kind == "E" then
    if m.game and m.pending then
      m.game.now = m.pending
      m.pending = nil
      m.version = m.version + 1
    end
    return "state"
  elseif kind == "R" then
    local fronts = {}
    for a, b, w in (f[2] or ""):gmatch("(%d+)%-(%d+)%-(%d+)") do fronts[#fronts + 1] = { tonumber(a), tonumber(b), tonumber(w) } end
    m.result = { winner = tonumber(f[1]) or 0, fronts = fronts, text = f[3] or "", at = now }
    if m.game then m.game.over = true end
    return "result"
  elseif kind == "N" then
    m.found[#m.found + 1] = { id = tonumber(f[1]), how = f[2] or "drop", at = now }
    return "found"
  elseif kind == "Q" then
    m.invite = { from = f[1] or "?", stakes = (f[2] == "ante" or f[2] == "lost") and f[2] or "friendly", ante = tonumber(f[3]) or 0, at = now }
    return "invite"
  end
  return true
end

return P
