-- Reads the server's Ironhand feed (modules/ironhand):
--   IH1|K                                   the cardbook follows: forget the old one
--   IH1|C|<id>:<n>,...                      cards you have, and how many
--   IH1|D|<id>,...                          your deck
--   IH1|W|<regular>:<wins>,...              wins against each of the Steaming Sheep's regulars
--   IH1|B                                   the end of the cardbook
--   IH1|M|<text>                            a word from the server (a deck it won't take, and why)
--   IH1|G|<opponent>|<friendly or ante>|<key>   a game starts
--   IH1|Y|<day element>|<night 1/0>|<night till 6:00 1/0>|<moon text>   the world at the table
--   IH1|S|<round>|<crystals>|<their hand>|<your deck left>|<play or over>   the table now (then H F P T lines, E)
--   IH1|H|<n>|<card id>                     your hand
--   IH1|F|<front>|<name>|<what its world does>
--   IH1|P|<front>|<side 1 you, 2 them>|<n>|<card id>|<power>|<bonus>|<silenced 1/0>
--   IH1|T|<front>|<your total>|<their total>
--   IH1|E                                   the end of the table
--   IH1|R|<winner 1/2/0>|<you-them-who>,... x3|<text>   the game is over
--   IH1|N|<card id>|<how>                   you found a card (drop, nm, champion, won, legend, gm)
--   IH1|Q|<player>|<friendly or ante>|<your card put up, or 0>   another player challenges you
--   IH1|A|<player>                          your opponent (a player) has revealed: they're ready
-- A game against a player has the key "pvp" (G line).
-- No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "IH1"

function P.new_model()
  return { book = { have = {}, deck = {}, wins = {}, version = 0 }, pending_book = nil, game = nil, pending = nil,
    result = nil, found = {}, message = nil, version = 0 }
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

local function new_table()
  return { hand = {}, fronts = { {}, {}, {} } }
end

-- Applies one line. Returns what changed: "book", "game", "state", "result", "found", "message"; true for our other
-- lines, false for anything else.
function P.feed(m, line, now)
  if type(line) ~= "string" then return false end
  local kind, rest = line:match("^IH1|(%u)|?(.*)$")
  if not kind then return false end
  local f = split(rest, "|")
  if kind == "K" then
    m.pending_book = { have = {}, deck = {}, wins = {} }
    return true
  elseif kind == "C" then
    local b = m.pending_book or m.book
    for id, n in (rest or ""):gmatch("(%d+):(%d+)") do b.have[tonumber(id)] = tonumber(n) end
    return true
  elseif kind == "D" then
    local b = m.pending_book or m.book
    b.deck = {}
    for id in (rest or ""):gmatch("%d+") do b.deck[#b.deck + 1] = tonumber(id) end
    return true
  elseif kind == "W" then
    local b = m.pending_book or m.book
    for k, n in (rest or ""):gmatch("(%a+):(%d+)") do b.wins[k] = tonumber(n) end
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
    m.game = { opponent = f[1] or "", stakes = f[2] or "friendly", key = f[3] or "", world = {}, now = new_table() }
    m.result = nil
    return "game"
  elseif kind == "Y" then
    if m.game then
      m.game.world = { day = f[1], night = f[2] == "1", night_late = f[3] == "1", moon = f[4] ~= "" and f[4] or nil }
    end
    return true
  elseif kind == "S" then
    if not m.game then return true end
    m.pending = new_table()
    m.pending.round = tonumber(f[1]) or 0
    m.pending.crystals = tonumber(f[2]) or 0
    m.pending.their_hand = tonumber(f[3]) or 0
    m.pending.deck_left = tonumber(f[4]) or 0
    m.pending.over = f[5] == "over"
    return true
  elseif kind == "H" then
    if m.pending then m.pending.hand[tonumber(f[1]) or (#m.pending.hand + 1)] = tonumber(f[2]) end
    return true
  elseif kind == "F" then
    local fr = m.pending and m.pending.fronts[tonumber(f[1]) or 0]
    if fr then fr.name, fr.twist, fr.cards = f[2], f[3], { {}, {} } end
    return true
  elseif kind == "P" then
    local fr = m.pending and m.pending.fronts[tonumber(f[1]) or 0]
    local side = tonumber(f[2])
    if fr and fr.cards and (side == 1 or side == 2) then
      fr.cards[side][tonumber(f[3]) or (#fr.cards[side] + 1)] = { id = tonumber(f[4]), power = tonumber(f[5]) or 0,
        bonus = tonumber(f[6]) or 0, silenced = f[7] == "1" }
    end
    return true
  elseif kind == "T" then
    local fr = m.pending and m.pending.fronts[tonumber(f[1]) or 0]
    if fr then fr.mine, fr.theirs = tonumber(f[2]) or 0, tonumber(f[3]) or 0 end
    return true
  elseif kind == "E" then
    if m.game and m.pending then
      m.game.ready = nil
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
  elseif kind == "Q" then
    m.invite = { from = f[1] or "?", stakes = f[2] == "ante" and "ante" or "friendly", ante = tonumber(f[3]) or 0, at = now }
    return "invite"
  elseif kind == "A" then
    if m.game then m.game.ready = f[1] or true end
    return "ready"
  elseif kind == "N" then
    m.found[#m.found + 1] = { id = tonumber(f[1]), how = f[2] or "drop", at = now }
    return "found"
  end
  return true
end

return P
