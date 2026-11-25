-- Withdraw: a small pop-up asking how many, "[x] / N" where N is how many are stored. Enter or Withdraw takes them out.

local C = require("lib.core")

local P = {}

P.WIN = "artisan_amount"
P.id = nil

local function stored() return P.id and C.stored(P.id) or 0 end

function P.label()
  return ("%s: how many?  (of %s stored)"):format(C.name(P.id), C.commas(stored()))
end

-- Takes the typed number out (clamped to what's stored); returns whether it asked the server
function P.submit(text)
  local L = C.L
  local n = L.parse_amount(text, stored())
  if not n then return false end
  C.withdraw(P.id, n)
  ui.close(P.WIN)
  return true
end

function P.setup()
  ui.window(P.WIN, {
    title = "Withdraw",
    popup = true,
    width = 300,
    elements = {
      { type = "input", id = "n", label = "", text = "", max = 5, tip = "Type a number, then Enter or Withdraw." },
      { type = "buttons", id = "go", items = { "Withdraw", "All", "Cancel" } },
    },
  })
  ui.on(P.WIN, function(win, e)
    if e.kind == "close" then return end
    if e.id == "n" and e.kind == "change" then
      P.submit(e.text)
    elseif e.id == "go" and e.kind == "press" then
      if e.index == 1 then
        local ok, v = pcall(ui.get, P.WIN, "n")
        P.submit(ok and v and v.text or "")
      elseif e.index == 2 then
        P.submit(tostring(stored()))
      else
        ui.close(P.WIN)
      end
    end
  end)
end

function P.open(id)
  P.id = id
  pcall(ui.update, P.WIN, "n", { label = P.label(), text = "" })
  ui.open(P.WIN)
end

return P
