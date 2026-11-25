-- The crafting window (/artisan): Recipes, Favorites, Guilds, Storage and Queue tabs, a Find line that filters
-- whichever tab is showing, and one canvas the chosen tab draws into. A recipe's row opens its recipe card
-- (lib/card.lua); Withdraw asks how many (lib/amount.lua).

local S = require("lib.style")
local Canvas = require("lib.canvas")
local C = require("lib.core")
local L = C.L

local W = {}

W.WIN = "artisan"
W.CW = 660
W.TABS = { "Recipes", "Favorites", "Guilds", "Storage", "Queue" }
W.ROW = 30
W.PER_PAGE = 12
W.BANDS = { { "All", 0, 999 } }
for b = 0, 10 do
  local lo, hi = b * 10 + (b == 0 and 0 or 1), (b + 1) * 10
  W.BANDS[#W.BANDS + 1] = b == 10 and { "101+", 101, 999 } or { ("%d-%d"):format(lo == 0 and 1 or lo, hi), lo, hi }
end

W.state = {
  tab = 1, query = "", craft = 1, band = 1, page = 1, reach = false, ready = false,
  open = {}, sto_view = 1, drawn_rev = 0,
}

-- Set by init.lua
W.open_card = function() end    -- (recipe)
W.ask_amount = function() end   -- (item id)

local function set_page(p) W.state.page = math.max(1, p) W.render() end

----------------------------------------------------------------------------------------------------------------------
-- Finding recipes
----------------------------------------------------------------------------------------------------------------------
local ing_names = {}
local function ingredient_text(r)
  local t = ing_names[r.id]
  if not t then
    local parts = {}
    for _, g in ipairs(r.ing) do parts[#parts + 1] = C.name(g.id):lower() end
    t = table.concat(parts, "|")
    ing_names[r.id] = t
  end
  return t
end

-- A recipe matches the Find text by its name, or (3+ letters) by one of its materials
function W.matches(r, q)
  if q == "" then return true end
  if r.name:lower():find(q, 1, true) then return true end
  return #q >= 3 and ingredient_text(r):find(q, 1, true) ~= nil
end

function W.query() return (W.state.query or ""):lower():gsub("^%s+", ""):gsub("%s+$", "") end

function W.recipe_list()
  local st, q = W.state, W.query()
  local band = W.BANDS[st.band] or W.BANDS[1]
  local src = q ~= "" and C.IDX.list or C.IDX.by_craft[st.craft]
  local out = {}
  for _, r in ipairs(src) do
    if (q ~= "" or (r.level >= band[2] and r.level <= band[3])) and W.matches(r, q) then
      if not st.reach or not C.odds(r).too_hard then
        if not st.ready or C.check(r).ok then out[#out + 1] = r end
      end
    end
  end
  if q ~= "" then
    table.sort(out, function(a, b)
      if a.craft ~= b.craft then return a.craft < b.craft end
      if a.level ~= b.level then return a.level < b.level end
      return a.name < b.name
    end)
  end
  return out
end

----------------------------------------------------------------------------------------------------------------------
-- Pieces
----------------------------------------------------------------------------------------------------------------------
local function pct_colour(x)
  if x >= 0.9 then return S.GOOD elseif x >= 0.6 then return S.WARN end
  return S.BAD
end

local function star(c, x, y, on, r)
  local s = 16
  local hot = c:region(x, y, s, s, { on = function() C.toggle_fav(r.id) end, key = "fav" .. r.id,
    tip = on and "In your Favorites: click to take it out." or "Add to Favorites." })
  c:text(x + s / 2, y - 1, "★", { align = "center", colour = (on or hot) and S.GOLD or S.FAINT, size = 1.05 })
end
W.star = star

-- Pager under a list: "Page 2 of 5" with arrows
local function pager(c, y, page, pages)
  if pages <= 1 then return y end
  c:button(W.CW / 2 - 110, y, 60, 22, "‹ Prev", { enabled = page > 1, on = function() set_page(page - 1) end, key = "prev" })
  c:text(W.CW / 2, y + 4, ("Page %d of %d"):format(page, pages), { align = "center", colour = S.MUTED, size = 0.85 })
  c:button(W.CW / 2 + 50, y, 60, 22, "Next ›", { enabled = page < pages, on = function() set_page(page + 1) end, key = "next" })
  return y + 28
end

-- One recipe row: star, result icon, name and level, chances, what you have, Craft
function W.recipe_row(c, x, y, w, r, show_craft)
  local o, chk = C.odds(r), C.check(r)
  c:tip(x + 270, y, 140, W.ROW - 2, r.name, W.odds_tip(o))
  local hot = c:region(x, y, w - 84, W.ROW - 2, { on = function() W.open_card(r) end, key = "row" .. r.id })
  c:rect(x, y, w, W.ROW - 2, hot and S.CARD_HI or S.CARD, { round = 5 })
  star(c, x + 6, y + 6, C.is_fav(r.id), r)
  c:icon(x + 26, y + 3, 22, "item", r.result)
  local name = r.name .. (r.qty > 1 and (" x" .. r.qty) or "")
  c:text(x + 54, y + 2, name, { w = 210, bold = true, colour = o.too_hard and S.MUTED or S.TEXT, size = 0.9 })
  local sub = {}
  if show_craft then sub[#sub + 1] = L.CRAFTS[r.craft].name end
  sub[#sub + 1] = "Lv " .. r.level
  for _, s in ipairs(L.subcrafts(r)) do sub[#sub + 1] = L.CRAFTS[s.craft].short .. " " .. s.level end
  c:text(x + 54, y + 15, table.concat(sub, "  ·  "), { w = 210, size = 0.75, colour = S.MUTED })
  -- chances
  local cx = x + 272
  if not C.ST.ready then
    c:text(cx, y + 8, "-", { colour = S.FAINT, size = 0.85 })
  elseif o.too_hard then
    c:text(cx, y + 8, "Too difficult", { colour = S.BAD, size = 0.85 })
  else
    c:text(cx, y + 3, L.pct(o.success), { colour = pct_colour(o.success), bold = true, size = 0.85 })
    c:text(cx, y + 16, "success", { colour = S.FAINT, size = 0.65 })
    c:text(cx + 50, y + 3, o.hq > 0 and L.pct(o.hq) or "-", { colour = o.hq > 0 and S.GOLD or S.FAINT, bold = true, size = 0.85 })
    c:text(cx + 50, y + 16, "HQ", { colour = S.FAINT, size = 0.65 })
    c:text(cx + 96, y + 3, o.skillup > 0 and L.pct(o.skillup) or "-", { colour = o.skillup > 0 and S.FOCUS or S.FAINT, bold = true, size = 0.85 })
    c:text(cx + 96, y + 16, "skill-up", { colour = S.FAINT, size = 0.65 })
  end
  -- what you have
  local hx = x + w - 172
  if chk.ok then
    c:text(hx, y + 8, ("Enough for %d"):format(chk.max), { colour = S.GOOD, size = 0.8, w = 84 })
  elseif chk.no_ki then
    c:text(hx, y + 8, "Key item", { colour = S.WARN, size = 0.8 })
  else
    c:text(hx, y + 8, ("Missing %d"):format(#chk.missing), { colour = S.MUTED, size = 0.8 })
  end
  local why = C.block(r)
  c:button(x + w - 80, y + 3, 74, W.ROW - 8, "Craft", { style = "primary", enabled = why == nil, size = 0.85,
    tip = why or ("Craft one now from your bags and storage. Open the recipe for more than one."),
    on = function() C.craft(r, 1) end, key = "craft" .. r.id })
end

function W.odds_tip(o)
  if not C.ST.ready then return "Chances show once the server's Timeless Artisan module answers." end
  if o.too_hard then return "More than 15 levels above your skill: it would almost always break." end
  local t = { ("Success %s per synth."):format(L.pct(o.success)) }
  if o.hq > 0 then
    t[#t + 1] = ("High quality %s (HQ1 %s, HQ2 %s, HQ3 %s)."):format(L.pct(o.hq), L.pct(o.hq1), L.pct(o.hq2), L.pct(o.hq3))
  else
    t[#t + 1] = "No high quality: a craft is above your skill, or a craft ring stops it."
  end
  t[#t + 1] = o.skillup > 0 and ("Skill-up %s per synth (best craft)."):format(L.pct(o.skillup)) or "No skill-up: too easy, or at your rank's cap."
  t[#t + 1] = ("A break loses each material with a %s chance."):format(L.pct(o.loss))
  return table.concat(t, "\n")
end

-- A fold arrow: pointing down when open, right when folded
function W.chevron(c, x, y, open, colour)
  if open then
    c:line(x - 4, y - 2, x, y + 2, colour, 2) c:line(x, y + 2, x + 4, y - 2, colour, 2)
  else
    c:line(x - 2, y - 4, x + 2, y, colour, 2) c:line(x + 2, y, x - 2, y + 4, colour, 2)
  end
end

local function empty(c, y, text)
  c:card(0, y, W.CW, 64)
  c:para(16, y + 14, W.CW - 32, text, { colour = S.MUTED, size = 0.9 })
  return y + 70
end

local function waiting(c, y)
  if C.ST.ready then return y end
  local text = C.waiting_too_long()
    and "The server isn't answering. Timeless Artisan needs the server's timeless_artisan module: the chances, storage and crafting wait for it."
    or "Asking the server for your skills and storage…"
  c:rect(0, y, W.CW, 26, { 240, 190, 90, 30 }, { round = 5 })
  c:text(10, y + 5, text, { colour = S.WARN, size = 0.8, w = W.CW - 20 })
  return y + 32
end

----------------------------------------------------------------------------------------------------------------------
-- Recipes
----------------------------------------------------------------------------------------------------------------------
function W.draw_recipes(c, y)
  local st, q = W.state, W.query()
  -- crafts
  local bw = (W.CW - 7 * 4) / 8
  for i, cr in ipairs(L.CRAFTS) do
    local lit = q == "" and st.craft == i
    local rank = L.RANKS[C.ST.rank[i] or 0] or ""
    c:button((i - 1) * (bw + 4), y, bw, 36, cr.name, { style = lit and "on" or "off", size = 0.8,
      sub = C.ST.ready and L.skill_text(C.ST.skill[i]) or "-", key = "craft_tab" .. i,
      title = cr.name, tip = C.ST.ready and ("Skill %s, %s. Cap %d."):format(L.skill_text(C.ST.skill[i]), rank, ((C.ST.rank[i] or 0) + 1) * 10) or nil,
      on = function() st.craft, st.page = i, 1 if q ~= "" then W.clear_query() end W.render() end })
  end
  y = y + 42
  if q == "" then
    local items = {}
    for i, b in ipairs(W.BANDS) do items[i] = { label = b[1], w = i == 1 and 44 or 48 } end
    c:switch(0, y, 22, items, st.band, { key = "band", on = function(i) st.band, st.page = i, 1 W.render() end })
    -- the craft's band for your skill: a mark under it
    y = y + 28
  else
    c:text(0, y + 4, ("Every craft: recipes named '%s' or using it."):format(W.state.query), { colour = S.MUTED, size = 0.85, w = 420 })
    y = y + 28
  end
  -- toggles
  c:button(0, y, 150, 22, (st.reach and "✓ " or "") .. "Within my skill", { style = st.reach and "on" or "off", size = 0.8, key = "reach",
    tip = "Hides recipes more than 15 levels above your skill (they'd almost always break).",
    on = function() st.reach, st.page = not st.reach, 1 W.render() end })
  c:button(156, y, 150, 22, (st.ready and "✓ " or "") .. "Have the materials", { style = st.ready and "on" or "off", size = 0.8, key = "ready",
    tip = "Shows only recipes you have every material for, counting your inventory and storage.",
    on = function() st.ready, st.page = not st.ready, 1 W.render() end })
  local list = W.recipe_list()
  c:text(W.CW, y + 4, C.plural(#list, "recipe"), { align = "right", colour = S.MUTED, size = 0.8 })
  y = y + 28
  y = waiting(c, y)
  if #list == 0 then return empty(c, y, q ~= "" and "No recipe matches that. Try part of a name: 'bronze', 'sole', 'ink'."
    or "No recipes here with these filters.") end
  local pages = math.max(1, math.ceil(#list / W.PER_PAGE))
  st.page = math.min(st.page, pages)
  for i = (st.page - 1) * W.PER_PAGE + 1, math.min(#list, st.page * W.PER_PAGE) do
    W.recipe_row(c, 0, y, W.CW, list[i], q ~= "")
    y = y + W.ROW
  end
  return pager(c, y + 4, st.page, pages)
end

----------------------------------------------------------------------------------------------------------------------
-- Favorites
----------------------------------------------------------------------------------------------------------------------
function W.draw_favorites(c, y)
  local q = W.query()
  y = waiting(c, y)
  local list = {}
  for _, r in ipairs(C.favorites()) do if W.matches(r, q) then list[#list + 1] = r end end
  if #C.favorites() == 0 then
    return empty(c, y, "No favorites yet. Click the star by a recipe (here, in Recipes or on its recipe card) to keep it at hand; your favorites also show in the HUD with a Craft button.")
  end
  if #list == 0 then return empty(c, y, "None of your favorites match that.") end
  local pages = math.max(1, math.ceil(#list / W.PER_PAGE))
  W.state.page = math.min(W.state.page, pages)
  for i = (W.state.page - 1) * W.PER_PAGE + 1, math.min(#list, W.state.page * W.PER_PAGE) do
    W.recipe_row(c, 0, y, W.CW, list[i], true)
    y = y + W.ROW
  end
  return pager(c, y + 4, W.state.page, pages)
end

----------------------------------------------------------------------------------------------------------------------
-- Guilds: today's guild point item of each guild
----------------------------------------------------------------------------------------------------------------------
function W.draw_guilds(c, y)
  local q = W.query()
  c:text(0, y + 2, "Each guild buys one item a day for guild points, up to a daily limit. The list turns over at midnight (Japan time).",
    { colour = S.MUTED, size = 0.8, w = W.CW - 90 })
  c:button(W.CW - 80, y, 80, 22, "Refresh", { size = 0.8, key = "gd", on = function() C.request("gd") end,
    tip = "Asks the server for today's items again." })
  y = y + 28
  y = waiting(c, y)
  if not C.ST.guilds_at then return empty(c, y, "Asking the guilds what they're buying today…") end
  local cw, ch = (W.CW - 8) / 2, 108
  local n = 0
  for gi = 1, 8 do
    local p = L.guild_plan(C.ST, C.IDX, gi, C.have, C.stored)
    local gname = L.GUILDS[gi]
    if p and (q == "" or gname:lower():find(q, 1, true) or C.name(p.item):lower():find(q, 1, true)) then
      local x = (n % 2) * (cw + 8)
      local yy = y + math.floor(n / 2) * (ch + 8)
      n = n + 1
      c:card(x, yy, cw, ch)
      c:text(x + 10, yy + 6, gname, { bold = true, colour = S.GOLD, size = 0.9 })
      c:text(x + cw - 10, yy + 7, ("%s rank"):format(L.RANKS[C.ST.rank[gi] or 0] or ""), { align = "right", colour = S.MUTED, size = 0.75 })
      c:icon(x + 10, yy + 26, 28, "item", p.item)
      c:text(x + 44, yy + 26, C.name(p.item), { w = cw - 54, size = 0.9 })
      c:text(x + 44, yy + 42, ("%d points each · %s today"):format(p.points, p.remaining > 0 and (C.commas(p.remaining) .. " of " .. C.commas(p.max) .. " points left") or "limit reached"),
        { w = cw - 54, size = 0.75, colour = p.remaining > 0 and S.MUTED or S.WARN })
      local line, colour
      if p.remaining <= 0 then
        line, colour = "Today's limit is reached: come back tomorrow.", S.MUTED
      elseif p.to_make <= 0 then
        line, colour = ("You have the %d needed: hand them to the guild."):format(p.items), S.GOOD
      else
        line, colour = ("%d more needed (you have %d)"):format(p.to_make, p.own), S.TEXT
      end
      c:text(x + 10, yy + 62, line, { w = cw - 20, size = 0.8, colour = colour })
      local r = p.recipe
      if r and p.to_make > 0 and p.remaining > 0 then
        local o = C.odds(r)
        local n_craft = math.max(1, math.min(p.queue, C.queue_room()))
        local why = C.block(r)
        c:text(x + 10, yy + 80, ("%s Lv %d · %s success"):format(L.CRAFTS[r.craft].short, r.level, o.too_hard and "too hard" or L.pct(o.success)),
          { w = cw - 130, size = 0.75, colour = S.MUTED })
        c:region(x + 10, yy + 78, cw - 130, 18, { on = function() W.open_card(r) end, key = "gr" .. gi, tip = "Open the recipe." })
        c:button(x + cw - 112, yy + 76, 102, 24, ("Craft %d"):format(n_craft), { style = "primary", enabled = why == nil, size = 0.85,
          key = "gcraft" .. gi, tip = why or (p.crafts > n_craft and ("Queues %d now (the queue holds 12); %d crafts make all %d."):format(n_craft, p.crafts, p.to_make)
            or ("Queues the %d crafts that make the %d you still need."):format(n_craft, p.to_make)),
          on = function() C.craft(r, n_craft) end })
      elseif not r then
        c:text(x + 10, yy + 80, "No enabled recipe makes it.", { size = 0.75, colour = S.FAINT })
      end
    end
  end
  if n == 0 then return empty(c, y, "No guild's item matches that.") end
  return y + math.ceil(n / 2) * (ch + 8)
end

----------------------------------------------------------------------------------------------------------------------
-- Storage
----------------------------------------------------------------------------------------------------------------------
W.STORE_PER_PAGE = 14

function W.storage_rows()
  local q = W.query()
  local ids, seen = {}, {}
  for _, t in ipairs({ C.ST.store, C.ST.store_own }) do
    for id, n in pairs(t) do if n > 0 and not seen[id] then seen[id] = true ids[#ids + 1] = id end end
  end
  -- items in your bags that could go in, so Deposit is at hand (only when you ask for them with Find)
  if q ~= "" then
    for _, id in ipairs(C.bag_materials(false)) do if not seen[id] then seen[id] = true ids[#ids + 1] = id end end
  end
  local groups = {}
  for _, id in ipairs(ids) do
    if q == "" or C.name(id):lower():find(q, 1, true) then
      local g = L.group_of(C.IDX, id)
      groups[g] = groups[g] or {}
      table.insert(groups[g], id)
    end
  end
  local rows = {}
  for g = 0, 9 do
    if groups[g] then
      table.sort(groups[g], function(a, b) return C.name(a) < C.name(b) end)
      local total = 0
      for _, id in ipairs(groups[g]) do total = total + C.stored(id) end
      rows[#rows + 1] = { head = g, n = #groups[g], total = total }
      if W.state.open[g] or q ~= "" then
        for _, id in ipairs(groups[g]) do rows[#rows + 1] = { id = id } end
      end
    end
  end
  return rows
end

local function storage_totals()
  local kinds, total = 0, 0
  for _, t in ipairs({ C.ST.store, C.ST.store_own }) do
    for _, n in pairs(t) do if n > 0 then kinds = kinds + 1 total = total + n end end
  end
  return kinds, total
end
W.storage_totals = storage_totals

function W.draw_storage_row(c, y, id)
  local s, inv, bags = C.stored(id), C.have(id), C.in_bags(id, true)
  local kept = C.kept_of(id)
  c:rect(16, y, W.CW - 16, 26, S.CARD, { round = 4 })
  c:icon(22, y + 3, 20, "item", id)
  c:text(48, y + 5, C.name(id), { w = 200, size = 0.85 })
  local own = (C.ST.store_own[id] or 0) > 0
  if own then c:text(250, y + 6, "this character", { size = 0.7, colour = S.FOCUS }) end
  c:region(250, y, 70, 26, own and { tip = "Rare or Ex: only this character can take these out." } or {})
  c:text(330, y + 5, C.commas(s), { bold = true, size = 0.85, colour = s > 0 and S.TEXT or S.FAINT, align = "right", w = 60 })
  c:text(334, y + 7, "stored", { size = 0.7, colour = S.FAINT })
  c:text(410, y + 5, C.commas(inv + bags), { size = 0.85, colour = inv + bags > 0 and S.TEXT or S.FAINT, align = "right", w = 50 })
  c:text(414, y + 7, "in bags", { size = 0.7, colour = S.FAINT })
  c:region(372, y, 92, 26, { tip = ("%d in your inventory, %d in your other bags."):format(inv, bags) })
  if kept > 0 then
    c:text(470, y + 7, ("keeping %d"):format(kept), { size = 0.7, colour = S.FOCUS })
    c:region(466, y, 60, 26, { tip = ("You withdrew these: auto-deposit leaves %d alone. Deposit stores them anyway."):format(kept) })
  end
  if C.auto_mode() == 2 then
    local on = C.auto_has(id)
    c:button(W.CW - 214, y + 3, 50, 20, on and "Auto ✓" or "Auto", { style = on and "on" or "off", size = 0.7, key = "auto" .. id,
      tip = on and "Auto-deposit stores this one: click to stop." or "Auto-deposit this item.",
      on = function() C.auto_toggle(id) end })
  end
  c:button(W.CW - 160, y + 3, 76, 20, "Withdraw", { size = 0.75, enabled = s > 0, key = "wd" .. id,
    tip = s > 0 and "Take some out: you type how many." or "None stored.",
    on = function() W.ask_amount(id) end })
  local can = inv + bags > 0 and not C.is_gear(id) and C.IDX.materials[id]
  c:button(W.CW - 80, y + 3, 76, 20, "Deposit", { size = 0.75, enabled = can and true or false, key = "dp" .. id,
    tip = can and "Store every one in your bags (as Settings > Mods > Timeless Artisan says which bags)." or "None in your bags.",
    on = function() C.deposit(id) end })
end

function W.draw_storage(c, y)
  local st = W.state
  local kinds, total = storage_totals()
  c:text(0, y, ("%s stored · %s"):format(C.plural(total, "item"), C.plural(kinds, "material type")), { bold = true, colour = S.GOLD, size = 0.9 })
  c:text(0, y + 18, "Shared by every character on your account. Crafting takes from here as well as your inventory.", { size = 0.75, colour = S.MUTED })
  c:switch(W.CW - 236, y, 22, { { label = "This character", w = 116 }, { label = "All characters", w = 116 } }, st.sto_view,
    { key = "view", on = function(i) st.sto_view, st.page = i, 1 if i == 2 then C.request("ac") end W.render() end })
  y = y + 40
  -- actions
  c:button(0, y, 150, 26, "Deposit all", { style = "primary", size = 0.85, key = "da", enabled = C.ST.ready,
    tip = "Stores every crafting material in your bags (not gear). Items you withdrew go back too.",
    on = function() C.deposit_all() end })
  local confirming = C.wa_confirm and C.now() < C.wa_confirm
  c:button(156, y, 190, 26, confirming and "Click again to withdraw all" or "Withdraw all materials", { style = confirming and "danger" or "plain",
    size = 0.85, key = "wa", enabled = C.ST.ready and total > 0,
    tip = "Takes every stored material out into your inventory, as far as it has room. Asks for a second click.",
    on = function()
      if confirming then C.wa_confirm = nil C.withdraw_all() else C.wa_confirm = C.now() + 4 end
      W.render()
    end })
  c:text(W.CW - 296, y + 6, "Auto-deposit", { size = 0.8, colour = S.MUTED })
  c:switch(W.CW - 220, y + 2, 22, { { label = "Off", w = 44, tip = "Nothing goes into storage by itself." },
    { label = "All materials", w = 96, tip = "Every crafting material you pick up goes into storage a few seconds later (not items you withdrew)." },
    { label = "Chosen", w = 72, tip = "Only the items you tick (Auto) below." } }, C.auto_mode() + 1,
    { key = "am", on = function(i) C.set_auto_mode(i - 1) end })
  y = y + 34
  y = waiting(c, y)
  if st.sto_view == 2 then return W.draw_account(c, y) end
  local rows = W.storage_rows()
  if #rows == 0 then
    return empty(c, y, W.query() ~= "" and "Nothing stored or in your bags matches that."
      or "Storage is empty. Deposit all stores the crafting materials in your bags; crystals and clusters go in too.")
  end
  local pages = math.max(1, math.ceil(#rows / W.STORE_PER_PAGE))
  st.page = math.min(st.page, pages)
  for i = (st.page - 1) * W.STORE_PER_PAGE + 1, math.min(#rows, st.page * W.STORE_PER_PAGE) do
    local row = rows[i]
    if row.head then
      local g = row.head
      local open = st.open[g] or W.query() ~= ""
      local hot = c:region(0, y, W.CW, 26, { key = "grp" .. g, on = function() st.open[g] = not st.open[g] W.render() end,
        tip = open and "Click to fold." or "Click to see these." })
      c:rect(0, y, W.CW, 26, hot and S.CARD_HI or { 255, 255, 255, 8 }, { round = 4 })
      W.chevron(c, 12, y + 13, open, S.GOLD)
      c:text(24, y + 5, L.GROUP_NAMES[g], { bold = true, size = 0.85, colour = S.GOLD })
      c:text(W.CW - 10, y + 6, ("%s · %s"):format(C.plural(row.n, "type"), C.plural(row.total, "item")), { align = "right", size = 0.75, colour = S.MUTED })
    else
      W.draw_storage_row(c, y, row.id)
    end
    y = y + 28
  end
  return pager(c, y + 4, st.page, pages)
end

-- Every character's own materials (Rare / Ex ones only they can take out) and what they carry
function W.draw_account(c, y)
  local q = W.query()
  if not C.ST.acct_at then return empty(c, y, "Asking the server about your other characters…") end
  if #C.ST.acct == 0 then return empty(c, y, "No characters on this account carry crafting materials.") end
  for _, ch in ipairs(C.ST.acct) do
    local ids = {}
    for id, n in pairs(ch.items) do
      if n > 0 and (q == "" or C.name(id):lower():find(q, 1, true)) then ids[#ids + 1] = id end
    end
    if #ids > 0 then
      table.sort(ids, function(a, b) return C.name(a) < C.name(b) end)
      y = c:heading(0, y, W.CW, ch.name, C.plural(#ids, "material type"))
      local col = 0
      for _, id in ipairs(ids) do
        local x = col * (W.CW / 3)
        c:icon(x, y, 18, "item", id)
        c:text(x + 22, y + 1, ("%s  x%s"):format(C.name(id), C.commas(ch.items[id])), { size = 0.8, w = W.CW / 3 - 28 })
        col = col + 1
        if col == 3 then col = 0 y = y + 22 end
      end
      if col > 0 then y = y + 22 end
      y = y + 6
      if y > 640 then
        c:text(0, y, "More characters: use Find to narrow it down.", { colour = S.MUTED, size = 0.8 })
        return y + 20
      end
    end
  end
  return y
end

----------------------------------------------------------------------------------------------------------------------
-- Queue and the session
----------------------------------------------------------------------------------------------------------------------
function W.draw_queue(c, y)
  local Q = C.ST.queue
  local v = C.queue_view()
  local busy = v ~= nil
  c:card(0, y, W.CW, 92)
  if v then
    c:text(12, y + 8, v.running and "Crafting" or "Next", { size = 0.8, colour = S.MUTED })
    c:text(12, y + 24, v.recipe and v.recipe.name or "…", { bold = true, colour = S.GOLD, w = 360 })
    c:text(W.CW - 140, y + 10, ("Synth %d of %d"):format(v.at, v.total), { align = "right", w = 120, size = 0.85 })
    c:bar(12, y + 48, W.CW - 160, 8, v.frac, S.GOLD)
    local rest = {}
    for i, e in ipairs(Q.list) do
      local rr = C.IDX.by_id[e.r]
      rest[#rest + 1] = ("%s x%d"):format(rr and rr.name or "?", e.n)
      if i >= 4 then break end
    end
    c:text(12, y + 64, #rest > 0 and ("Still queued: " .. table.concat(rest, ", ")) or "The last synth of the queue.",
      { size = 0.75, colour = S.MUTED, w = W.CW - 170 })
    c:button(W.CW - 136, y + 40, 124, 30, "Stop", { style = "danger", key = "stop", on = function() C.stop() end,
      tip = "Stops after the synth in progress; the rest of the queue is dropped." })
  else
    c:text(12, y + 10, "Nothing queued", { bold = true, colour = S.MUTED })
    c:para(12, y + 30, W.CW - 170, "Queue up to 12 crafts at a time from a recipe, a favorite or a guild item. Synths run one every 15 seconds, taking materials from your inventory and then storage.",
      { size = 0.8, colour = S.FAINT })
  end
  local last, n = C.last()
  if not busy then
    local why = last and C.block(last)
    c:button(W.CW - 136, y + 30, 124, 34, last and ("Repeat x%d"):format(n) or "Repeat", { style = "primary", key = "repeat",
      enabled = last ~= nil and why == nil, sub = last and last.name or nil, size = 0.85,
      tip = last and (why or ("Queues %s x%d again."):format(last.name, n)) or "Nothing crafted yet to repeat.",
      on = last and function() C.craft(last, n) end or nil })
  end
  y = y + 100
  -- the session
  local s = C.session
  local gains = {}
  for _, cr in ipairs(L.CRAFTS) do
    if (s.skill[cr.name] or 0) > 0 then gains[#gains + 1] = ("%s +%.1f"):format(cr.name, s.skill[cr.name]) end
  end
  y = c:heading(0, y, W.CW, "This session", ("%d made · %d HQ · %d broke"):format(s.made, s.hq, s.broke))
  if #gains > 0 then
    c:text(0, y, "Skill: " .. table.concat(gains, ", "), { size = 0.8, colour = S.GOLD, w = W.CW })
    y = y + 20
  end
  local shown = 0
  for _, e in ipairs(s.log) do
    if W.query() == "" or e.text:lower():find(W.query(), 1, true) then
      c:text(0, y, e.t, { size = 0.75, colour = S.FAINT })
      c:text(60, y, e.text, { size = 0.8, colour = e.colour, w = W.CW - 60 })
      y = y + 18
      shown = shown + 1
      if shown >= 16 then break end
    end
  end
  if shown == 0 then c:text(0, y, "Crafts, withdrawals and deposits show here.", { size = 0.8, colour = S.FAINT }) y = y + 18 end
  -- Masterworks: the ledger with mastery and the workbench
  y = y + 8
  c:button(0, y, 200, 24, "Open Masterworks", { size = 0.8, key = "mw", on = function() W.run("//masterworks") end,
    tip = "Your crafting ledger: recipe mastery, Masterwork chances, rested synths, orders and your apprentice." })
  c:text(210, y + 5, "Moon, day, weather and facing don't change any synth on this server.", { size = 0.75, colour = S.FAINT, w = W.CW - 210 })
  return y + 28
end

W.run = function(line) commands.run(line) end

----------------------------------------------------------------------------------------------------------------------
function W.draw(c)
  local y = 0
  local t = W.state.tab
  if t == 1 then y = W.draw_recipes(c, y)
  elseif t == 2 then y = W.draw_favorites(c, y)
  elseif t == 3 then y = W.draw_guilds(c, y)
  elseif t == 4 then y = W.draw_storage(c, y)
  else y = W.draw_queue(c, y) end
  c:grow(y)
end

function W.render()
  W.state.drawn_rev = C.rev
  if ui.is_open(W.WIN) then Canvas.render(W.WIN, "main") end
end

function W.clear_query()
  W.state.query = ""
  pcall(ui.update, W.WIN, "q", { text = "" })
end

-- What each tab asks the server for when it shows
function W.ask_for_tab(t)
  if t == 3 then C.request("gd")
  elseif t == 4 then C.request("st") if W.state.sto_view == 2 then C.request("ac") end
  elseif t == 5 then C.request("qs") end
end

function W.go(tab)
  W.state.tab, W.state.page = tab, 1
  pcall(ui.update, W.WIN, "tabs", { at = tab })
  W.ask_for_tab(tab)
  W.render()
end

function W.setup()
  ui.window(W.WIN, {
    title = "Timeless Artisan",
    width = W.CW + 20,
    help = "Recipes with your chances, Favorites, today's guild items, your account-wide crafting storage and the craft queue (12 at most). Craft takes the crystal and materials from your inventory first, then storage. Find filters the tab you're on.",
    elements = {
      { type = "tabs", id = "tabs", items = W.TABS, at = 1 },
      { type = "input", id = "q", label = "Find", text = "", filter = true,
        tip = "Type to filter: a recipe or material ('bronze', 'sheep leather'), a guild, or a line of the log." },
      { type = "canvas", id = "main", w = W.CW, h = 600, draw = {} },
    },
  })
  Canvas.add(W.WIN, "main", W.CW, W.draw)
  ui.on(W.WIN, function(win, e)
    if e.kind == "close" then return end
    if e.id == "tabs" and e.kind == "set" then
      W.state.tab, W.state.page = e.index, 1
      W.ask_for_tab(e.index)
      W.render()
      return
    end
    if e.id == "q" and (e.kind == "edit" or e.kind == "change") then
      W.state.query = e.text or ""
      W.state.page = 1
      W.render()
      return
    end
    Canvas.event(win, e)
  end)
end

function W.open(tab)
  if not ui.is_open(W.WIN) then
    ui.open(W.WIN)
    C.request("md")
    C.request("st")
  end
  W.go(tab or W.state.tab)
end

function W.toggle()
  if ui.is_open(W.WIN) then ui.close(W.WIN) else W.open() end
end

return W
