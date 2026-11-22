-- A challenge from another player: who, for what (a friendly game, or for a card: the one you'd put up), and
-- Accept / Decline. It closes by itself when the challenge runs out (the server's config.pvp.inviteSeconds).

local S = require("lib.style")
local Canvas = require("lib.canvas")
local cards = require("lib.cards")
local Art = require("lib.art")

local I = {}

I.WIN = "cardian_invite"
I.W = 340
I.SECONDS = 60
I.state = { invite = nil, at = 0 }
I.send = function() end

function I.draw(c)
  local inv = I.state.invite
  if not inv then return end
  c:text(I.W / 2, 0, "A challenge!", { size = 1.2, bold = true, colour = S.GOLD, align = "center" })
  local y = c:para(0, 28, I.W, ("%s challenges you to a game of Cardian%s."):format(inv.from,
    inv.stakes == "ante" and ", for a card" or ""), { size = 0.9, colour = S.TEXT })
  if inv.stakes == "ante" and cards.get(inv.ante) then
    y = c:para(0, y + 4, I.W, "You'd put up this card. The winner takes the loser's:", { size = 0.8, colour = S.MUTED })
    Art.strip(c, 2, y + 4, I.W - 4, inv.ante, {})
    y = y + 4 + Art.STRIP + 6
  else
    y = c:para(0, y + 4, I.W, "A friendly game: nothing changes hands.", { size = 0.8, colour = S.MUTED })
  end
  c:button(0, y + 10, 160, 30, "Accept", { style = "primary", key = "accept", on = function() I.answer(true) end })
  c:button(I.W - 160, y + 10, 160, 30, "Decline", { key = "decline", on = function() I.answer(false) end })
end

function I.answer(yes)
  I.state.invite = nil
  ui.close(I.WIN)
  I.send(yes and "cd accept" or "cd decline")
end

function I.setup()
  ui.window(I.WIN, { title = "Cardian", width = I.W + 20, elements = { { type = "canvas", id = "invite", w = I.W, h = 190, draw = {} } } })
  Canvas.add(I.WIN, "invite", I.W, I.draw)
  ui.on(I.WIN, function(win, e)
    if e.kind == "close" then
      if I.state.invite then I.state.invite = nil I.send("cd decline") end
      return
    end
    Canvas.event(win, e)
  end)
end

function I.show(invite, t)
  I.state.invite, I.state.at = invite, t
  ui.open(I.WIN)
  Canvas.render(I.WIN, "invite")
end

function I.tick(t)
  if I.state.invite and t - I.state.at >= I.SECONDS then
    I.state.invite = nil
    if ui.is_open(I.WIN) then ui.close(I.WIN) end
  end
end

return I
