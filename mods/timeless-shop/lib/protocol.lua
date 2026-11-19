-- Reads the server's storefront lines (modules/grub_shop, grub_shop_storefront.lua):
--   SH1|B|t=<token>,k=<keeper>,i=<keeper index>,r=<race>,c=<currency>,b=<balance>,n=<sections>
--   SH1|T|<title>          SH1|G|<greeting>
--   SH1|S|<s>|<section name>
--   SH1|I|<s>.<i>|id=,ic=,g=,p=,q=,a=<1 can buy>,l=<label>   (the label is last and may hold commas)
--   SH1|W|<s>.<i>|<why not>          SH1|D|<s>.<i>|<description>
--   SH1|E|t=<token>        the shop is complete: open (or refresh) the window
--   SH1|R|<1 ok, 0 not>|<message>    what a purchase did
-- No game or UI code here, so it can be tested on its own.

local P = {}

P.VERSION = "SH1"

function P.new_model()
  return { shop = nil, pending = nil, result = nil }
end

function P.extract(message)
  if type(message) ~= "string" then return nil end
  local start = message:find(P.VERSION .. "|", 1, true)
  if not start then return nil end
  return (message:sub(start):gsub("[%c].*$", ""))
end

local function fields(body)
  local out = {}
  body = body or ""
  local label = body:match(",l=(.*)$") or body:match("^l=(.*)$")
  if label then
    out.l = label
    body = body:sub(1, #body - #label - 2)
  end
  for k, v in body:gmatch("([%w_]+)=([^,]*)") do out[k] = v end
  return out
end

local function item_at(shop, key)
  local s, i = key:match("^(%d+)%.(%d+)$")
  s, i = tonumber(s), tonumber(i)
  local sec = s and shop.sections[s]
  return sec and sec.items[i], s, i
end

-- Applies one line. Returns "open" when a shop was completed, "result" for a purchase's answer, true for other
-- lines of ours, false for anything else.
function P.feed(model, line, now)
  if type(line) ~= "string" then return false end
  local kind, rest = line:match("^SH1|(%u)|?(.*)$")
  if not kind then return false end
  local shop = model.pending
  if kind == "B" then
    local f = fields(rest)
    model.pending = {
      token = tonumber(f.t) or 0, keeper = f.k or "Merchant", index = tonumber(f.i) or 0, race = f.r or "",
      currency = f.c or "Everglass", balance = tonumber(f.b) or 0, sections = {}, title = nil, greeting = nil,
    }
  elseif kind == "T" and shop then
    shop.title = rest
  elseif kind == "G" and shop then
    shop.greeting = rest
  elseif kind == "S" and shop then
    local s, name = rest:match("^(%d+)|(.*)$")
    if s then shop.sections[tonumber(s)] = { name = name, items = {} } end
  elseif kind == "I" and shop then
    local key, body = rest:match("^(%d+%.%d+)|(.*)$")
    local s, i = (key or ""):match("^(%d+)%.(%d+)$")
    local sec = s and shop.sections[tonumber(s)]
    if sec then
      local f = fields(body)
      sec.items[tonumber(i)] = {
        s = tonumber(s), i = tonumber(i), id = tonumber(f.id) or 0, icon = tonumber(f.ic) or 0, glyph = f.g or "",
        price = tonumber(f.p) or 0, left = tonumber(f.q) or -1, can = f.a == "1", label = f.l or "?",
      }
    end
  elseif (kind == "W" or kind == "D") and shop then
    local key, text = rest:match("^(%d+%.%d+)|(.*)$")
    local item = key and item_at(shop, key)
    if item then
      if kind == "W" then item.why = text else item.desc = text end
    end
  elseif kind == "E" then
    if not shop then return true end
    model.pending = nil
    model.shop = shop
    shop.at = now
    return "open"
  elseif kind == "R" then
    local ok, msg = rest:match("^([01])|(.*)$")
    model.result = { ok = ok == "1", msg = msg or "", at = now }
    return "result"
  end
  return true
end

-- Whether the player can buy it now: the server says yes and they have the Everglass.
function P.affordable(shop, item)
  return item ~= nil and item.can and shop.balance >= item.price
end

return P
