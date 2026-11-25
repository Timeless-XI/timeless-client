-- Timeless Artisan's state and actions, shared by its windows: what the server told us (skills, modifiers, storage,
-- guild items, the queue), what our bags hold, the requests we send, the session log, favorites, kept items and
-- auto-deposit. The windows read it and call its actions; they redraw when C.rev changes.

local L = require("lib.logic")
local action = require("lib.action")

local C = {}

C.L = L
C.IDX = L.index(require("lib.data"))
C.ST = L.new_state()
C.rev = 1                     -- raised whenever anything a window shows changes
C.bags = {}                   -- [bag] = { [item] = count }; bags 0 1 2 4 5 6 7 9
C.inv_free = 0
C.session = { made = 0, hq = 0, broke = 0, skill = {}, log = {} }
C.net = { out = {}, next_send = 0, asked_at = nil, next_auto = nil, ki = nil, ki_at = -10 }
C.keep = { pending = {}, asked = {}, at = {} } -- hand withdrawals still landing: qty asked, when asked; when landed
C.wa_confirm = nil            -- Withdraw all: the second click must come before this time

C.now = function() return os.clock() end

local function changed() C.rev = C.rev + 1 end
C.changed = changed

-- Per-character preferences (storage.character.artisan): favorites, kept items, auto-deposit, the last craft
function C.prefs()
  storage.character.artisan = storage.character.artisan or {}
  local p = storage.character.artisan
  p.favorites = p.favorites or {}
  p.kept = p.kept or {}
  p.auto_items = p.auto_items or {}
  p.auto_mode = tonumber(p.auto_mode) or 0
  p.count = tonumber(p.count) or 1
  return p
end
local function save() pcall(storage.save) end
C.save = save

local function setting(key, fallback)
  local ok, v = pcall(settings.get, key)
  if ok and v ~= nil then return v end
  return fallback
end
C.setting = setting

----------------------------------------------------------------------------------------------------------------------
-- Items
----------------------------------------------------------------------------------------------------------------------
local info_cache = {}
function C.item(id)
  if not id or id == 0 then return nil end
  local r = info_cache[id]
  if r == nil then
    local ok, v = pcall(res.item, id)
    r = (ok and type(v) == "table") and v or false
    if r or ok then info_cache[id] = r end
  end
  return r or nil
end
function C.name(id)
  local r = C.item(id)
  return (r and r.name and r.name ~= "") and r.name or ("Item " .. tostring(id))
end
function C.stack(id)
  local r = C.item(id)
  return (r and (r.stack or 0) > 0) and r.stack or 1
end
-- Weapons and armour: some are ingredients, but "Deposit all" and auto-deposit leave gear alone
function C.is_gear(id)
  local r = C.item(id)
  return r ~= nil and (r.type == 4 or r.type == 5)
end

function C.have(id) return (C.bags[0] or {})[id] or 0 end
function C.stored(id) return (C.ST.store[id] or 0) + (C.ST.store_own[id] or 0) end
function C.in_bags(id, skip_inventory)
  local n = 0
  for bag, t in pairs(C.bags) do
    if not (skip_inventory and bag == 0) then n = n + (t[id] or 0) end
  end
  return n
end
-- Inventory + satchel + sack + case: where withdrawn items can wander to
function C.carried(id)
  local n = 0
  for _, b in ipairs({ 0, 5, 6, 7 }) do n = n + ((C.bags[b] or {})[id] or 0) end
  return n
end

function C.has_ki(k)
  local t = C.now()
  if not C.net.ki or t - C.net.ki_at > 2 then
    local set = {}
    local ok, list = pcall(game.key_items)
    if ok and type(list) == "table" then for _, id in ipairs(list) do set[id] = true end end
    C.net.ki, C.net.ki_at = set, t
  end
  return C.net.ki[k] == true
end

local BAGS = { 0, 1, 2, 4, 5, 6, 7, 9 }
local inv_sig = ""
-- Reads our bags; returns true when the inventory changed
function C.scan()
  local bags, sig, used = {}, {}, 0
  for _, b in ipairs(BAGS) do
    local t = {}
    local ok, list = pcall(game.items, b)
    if ok and type(list) == "table" then
      for _, it in ipairs(list) do
        if it.id and it.id > 0 and it.id ~= 65535 then
          t[it.id] = (t[it.id] or 0) + math.max(1, it.count or 1)
          if b == 0 then used = used + 1 end
        end
      end
    end
    bags[b] = t
  end
  local okn, size = pcall(game.bag_size, 0)
  local max = okn and tonumber(size) or 0
  if max <= 0 then max = 80 end
  for id, n in pairs(bags[0]) do sig[#sig + 1] = id .. ":" .. n end
  table.sort(sig)
  local s = table.concat(sig, ",")
  C.bags = bags
  C.inv_free = math.max(0, max - used)
  if s ~= inv_sig then
    inv_sig = s
    changed()
    return true
  end
  return false
end

----------------------------------------------------------------------------------------------------------------------
-- Talking to the server
----------------------------------------------------------------------------------------------------------------------
function C.request(text) C.net.out[#C.net.out + 1] = text end
function C.flush()
  if #C.net.out == 0 or C.now() < C.net.next_send then return end
  local text = table.remove(C.net.out, 1)
  pcall(action.send, text)
  C.net.next_send = C.now() + 0.12
end
function C.hello()
  C.request("hi")
  C.net.asked_at = C.now()
end
function C.waiting_too_long()
  return not C.ST.ready and C.net.asked_at ~= nil and C.now() - C.net.asked_at > 6
end

-- The bags the Deposit buttons take from (a bit per bag)
function C.deposit_mask(for_auto)
  local bags = for_auto and setting("auto_bags", false)
  if for_auto then
    return L.bag_mask(true, bags and setting("deposit_satchel", true), bags and setting("deposit_sack", true),
      bags and setting("deposit_case", true), false)
  end
  return L.bag_mask(true, setting("deposit_satchel", true), setting("deposit_sack", true), setting("deposit_case", true),
    setting("deposit_home", true))
end

----------------------------------------------------------------------------------------------------------------------
-- Session log
----------------------------------------------------------------------------------------------------------------------
local S = require("lib.style")
function C.log(text, colour)
  table.insert(C.session.log, 1, { t = os.date("%H:%M:%S"), text = text, colour = colour or S.TEXT })
  while #C.session.log > 60 do table.remove(C.session.log) end
  changed()
end
function C.note(text) print("Artisan: " .. text) end

local function plural(n, one, many)
  return ("%s %s"):format(C.commas(n), n == 1 and one or (many or (one .. "s")))
end
function C.commas(n)
  local t = tostring(math.floor((n or 0) + 0.5))
  local k
  repeat t, k = t:gsub("^(-?%d+)(%d%d%d)", "%1,%2") until k == 0
  return t
end
C.plural = plural

----------------------------------------------------------------------------------------------------------------------
-- Favorites, kept items
----------------------------------------------------------------------------------------------------------------------
function C.is_fav(id)
  for _, f in ipairs(C.prefs().favorites) do if f == id then return true end end
  return false
end
function C.toggle_fav(id)
  local list = C.prefs().favorites
  for i, f in ipairs(list) do
    if f == id then table.remove(list, i) save() changed() return end
  end
  list[#list + 1] = id
  save()
  changed()
end
function C.favorites()
  local out = {}
  for _, id in ipairs(C.prefs().favorites) do
    if C.IDX.by_id[id] then out[#out + 1] = C.IDX.by_id[id] end
  end
  return out
end

-- Kept: withdrawn by hand, so auto-deposit leaves that many alone (see L.kept_clamp / L.auto_plan)
function C.kept_map()
  local t = {}
  for k, n in pairs(C.prefs().kept) do
    local id, v = tonumber(k), tonumber(n)
    if id and v and v > 0 then t[id] = v end
  end
  for id, n in pairs(C.keep.pending) do t[id] = (t[id] or 0) + n end
  return t
end
function C.kept_of(id) return C.kept_map()[id] or 0 end
local function kept_save(t)
  local out = {}
  for id, n in pairs(t) do if n > 0 then out[tostring(id)] = n end end
  C.prefs().kept = out
  save()
  changed()
end
function C.kept_release(id)
  local t = {}
  for k, n in pairs(C.prefs().kept) do if tonumber(k) ~= id then t[tonumber(k)] = tonumber(n) end end
  C.keep.pending[id], C.keep.asked[id] = nil, nil
  kept_save(t)
end
function C.kept_tick()
  local p = C.prefs()
  if next(p.kept) == nil then return end
  local t, before, after = {}, 0, 0
  for k, n in pairs(p.kept) do t[tonumber(k)] = tonumber(n); before = before + tonumber(n) end
  L.kept_clamp(t, C.carried, function(id)
    return C.keep.pending[id] ~= nil or (C.keep.at[id] ~= nil and C.now() - C.keep.at[id] < 5)
  end)
  for _, n in pairs(t) do after = after + n end
  if after ~= before then kept_save(t) end
end
local function pending_expire()
  for id, t in pairs(C.keep.asked) do
    if C.now() - t > 10 then C.keep.pending[id], C.keep.asked[id] = nil, nil end -- no answer: forget it
  end
end

----------------------------------------------------------------------------------------------------------------------
-- Actions
----------------------------------------------------------------------------------------------------------------------
function C.queue_size() return (C.ST.queue.n or 0) + (C.ST.queue.run and 1 or 0) end

-- The queue as shown: the recipe on the bench (or next), which synth of how many, and the fraction done.
-- The server sends what's left; the total is the most it has held since it was last empty.
C.qpeak = 0
function C.queue_view()
  local Q = C.ST.queue
  local size = C.queue_size()
  if size == 0 then return nil end
  local id = (Q.run and Q.cur ~= 0) and Q.cur or (Q.list[1] and Q.list[1].r)
  local total = math.max(C.qpeak, size)
  local done = total - size
  return { recipe = id and C.IDX.by_id[id], at = math.min(total, done + 1), total = total, frac = done / total,
    running = Q.run, left = size }
end
function C.queue_room() return math.max(0, (C.ST.v.max or L.MAX_QUEUE) - C.queue_size()) end

local cache = { rev = 0, odds = {}, check = {} }
local function fresh() if cache.rev ~= C.rev then cache.rev, cache.odds, cache.check = C.rev, {}, {} end end
function C.odds(r)
  fresh()
  local o = cache.odds[r.id]
  if not o then o = L.odds(C.ST, r) cache.odds[r.id] = o end
  return o
end
function C.check(r)
  fresh()
  local c = cache.check[r.id]
  if not c then c = L.check(C.ST, r, C.have, C.stored, C.has_ki) cache.check[r.id] = c end
  return c
end

-- Why a recipe can't be crafted right now (nil: it can)
function C.block(r)
  if not C.ST.ready then return "Waiting for the server's Timeless Artisan module." end
  local o, c = C.odds(r), C.check(r)
  if o.too_hard then return "Too difficult: more than 15 levels above your skill." end
  if c.no_ki then return "Needs a key item you don't have (sold by the guild)." end
  if not c.ok then
    local parts = {}
    for _, m in ipairs(c.missing) do
      local what = L.is_crystal(m.id) and (L.ELEMENTS[L.CRYSTAL_ELEMENT[m.id]] .. " crystal") or C.name(m.id)
      parts[#parts + 1] = ("%s (%d of %d)"):format(what, m.have, m.need)
    end
    return "Missing: " .. table.concat(parts, ", ")
  end
  if C.queue_room() <= 0 then return "The craft queue is full (12 crafts at most)." end
  return nil
end

function C.count()
  return math.max(1, math.min(L.MAX_QUEUE, C.prefs().count or 1))
end
function C.set_count(n)
  C.prefs().count = math.max(1, math.min(L.MAX_QUEUE, n))
  save()
  changed()
end

function C.craft(r, n)
  n = math.max(1, math.min(n or C.count(), C.queue_room()))
  if C.queue_room() <= 0 then C.note("The craft queue is full (12 crafts at most).") return end
  C.request(("cq %d %d %d"):format(r.id, n, setting("return_leftovers", true) and 1 or 0))
  C.log(("Queued %s x%d"):format(r.name, n), S.FOCUS)
  C.prefs().last = { id = r.id, n = n }
  save()
end

function C.last()
  local l = C.prefs().last
  if type(l) ~= "table" or not tonumber(l.id) then return nil, 0 end
  return C.IDX.by_id[tonumber(l.id)], math.max(1, math.min(L.MAX_QUEUE, tonumber(l.n) or 1))
end

function C.stop() C.request("cx") end

function C.withdraw(id, n)
  if (n or 0) <= 0 then return end
  C.keep.pending[id] = (C.keep.pending[id] or 0) + n -- kept from the moment you ask
  C.keep.asked[id] = C.now()
  C.request(("wd %d %d"):format(id, n))
end

function C.withdraw_all()
  for id in pairs(C.ST.store) do C.keep.pending[id] = C.keep.pending[id] or 0; C.keep.asked[id] = C.now() end
  for id in pairs(C.ST.store_own) do C.keep.pending[id] = C.keep.pending[id] or 0; C.keep.asked[id] = C.now() end
  C.request("wa")
end

function C.deposit(id)
  C.kept_release(id)
  C.request(("dp %d %d:0"):format(C.deposit_mask(false), id))
end

-- Every crafting material in the deposit bags (gear only when deposited by its own button)
function C.bag_materials(for_auto)
  local bags = { 0 }
  if not for_auto or setting("auto_bags", false) then
    if setting("deposit_satchel", true) then bags[#bags + 1] = 5 end
    if setting("deposit_sack", true) then bags[#bags + 1] = 6 end
    if setting("deposit_case", true) then bags[#bags + 1] = 7 end
  end
  if not for_auto and setting("deposit_home", true) then
    for _, b in ipairs({ 1, 2, 4, 9 }) do bags[#bags + 1] = b end
  end
  local ids, seen = {}, {}
  for _, b in ipairs(bags) do
    for id in pairs(C.bags[b] or {}) do
      if C.IDX.materials[id] and not seen[id] and not C.is_gear(id) then seen[id] = true ids[#ids + 1] = id end
    end
  end
  table.sort(ids)
  return ids
end

function C.deposit_all()
  local ids = C.bag_materials(false)
  if #ids == 0 then C.log("No crafting materials in your bags to store.", S.MUTED) return end
  local parts = {}
  for _, id in ipairs(ids) do
    if C.kept_of(id) > 0 then C.kept_release(id) end
    parts[#parts + 1] = tostring(id)
  end
  for _, rq in ipairs(L.chunk_requests(("da %d"):format(C.deposit_mask(false)), parts)) do C.request(rq) end
end

-- Auto-deposit: 0 off, 1 every crafting material, 2 only the items picked on the Storage tab
C.AUTO = { [0] = "Off", "All materials", "Chosen items" }
function C.auto_mode() return math.max(0, math.min(2, C.prefs().auto_mode or 0)) end
function C.set_auto_mode(m)
  C.prefs().auto_mode = m % 3
  save()
  C.net.next_auto = C.now() + 2
  changed()
end
function C.auto_has(id)
  for _, x in ipairs(C.prefs().auto_items) do if x == id then return true end end
  return false
end
function C.auto_toggle(id)
  local list = C.prefs().auto_items
  for i, x in ipairs(list) do if x == id then table.remove(list, i) save() changed() return end end
  list[#list + 1] = id
  save()
  changed()
end

local function auto_candidates()
  local mode = C.auto_mode()
  if mode == 1 then return C.bag_materials(true) end
  local ids = {}
  if mode == 2 then
    for _, id in ipairs(C.prefs().auto_items) do
      if C.have(id) > 0 or (setting("auto_bags", false) and C.in_bags(id, true) > 0) then ids[#ids + 1] = id end
    end
  end
  return ids
end
local function auto_count(id)
  local n = C.have(id)
  if setting("auto_bags", false) then
    if setting("deposit_satchel", true) then n = n + ((C.bags[5] or {})[id] or 0) end
    if setting("deposit_sack", true) then n = n + ((C.bags[6] or {})[id] or 0) end
    if setting("deposit_case", true) then n = n + ((C.bags[7] or {})[id] or 0) end
  end
  return n
end

-- A few seconds after the inventory changes; never while crafts are queued or a withdrawal is landing
function C.auto_tick()
  local t = C.now()
  if C.auto_mode() == 0 or C.net.next_auto == nil or t < C.net.next_auto then return end
  C.net.next_auto = nil
  pending_expire()
  if C.ST.queue.run or (C.ST.queue.n or 0) > 0 or not C.ST.ready or next(C.keep.pending) ~= nil then
    C.net.next_auto = t + 5
    return
  end
  C.kept_tick()
  local plan = L.auto_plan(auto_candidates(), C.kept_map(), C.carried, auto_count)
  if #plan == 0 then return end
  local parts = {}
  for _, e in ipairs(plan) do parts[#parts + 1] = ("%d:%d"):format(e.id, e.qty) end
  for _, rq in ipairs(L.chunk_requests(("dp %d"):format(C.deposit_mask(true)), parts)) do C.request(rq) end
end
function C.inventory_changed()
  if C.auto_mode() ~= 0 then C.net.next_auto = C.now() + 4 end
end

----------------------------------------------------------------------------------------------------------------------
-- The server's lines
----------------------------------------------------------------------------------------------------------------------
function C.on_line(line)
  local kind, info = L.parse(C.ST, line)
  if kind == nil then return end
  changed()
  if kind == "Q" then
    local size = C.queue_size()
    C.qpeak = size == 0 and 0 or math.max(C.qpeak, size)
  end
  if kind == "R" then
    local r = C.IDX.by_id[info.recipe]
    local name = r and r.name or C.name(info.item)
    if info.res == 0 then
      C.session.broke = C.session.broke + 1
      C.log(("%s: the synthesis broke."):format(name), S.BAD)
    else
      C.session.made = C.session.made + 1
      if info.res >= 2 then C.session.hq = C.session.hq + 1 end
      C.log(("%s: %s x%d%s"):format(name, C.name(info.item), info.qty, info.res >= 2 and ("  (HQ" .. (info.res - 1) .. ")") or ""),
        info.res >= 2 and S.GOLD or S.GOOD)
    end
  elseif kind == "D" then
    if info.act == "all" then
      if info.q > 0 then C.log(("Stored %s (%s)."):format(plural(info.q, "item"), plural(info.i, "material type")), S.GOOD) end
    elseif info.act == "wall" then
      if info.q > 0 then C.log(("Withdrew %s (%s)."):format(plural(info.q, "item"), plural(info.i, "material type")), S.GOOD) end
      if info.why ~= "" then C.log(("Withdraw all stopped: %s."):format(info.why), S.WARN) end
      for id, n in pairs(C.keep.pending) do if n == 0 then C.keep.pending[id], C.keep.asked[id] = nil, nil end end
    elseif info.q > 0 then
      C.log(("%s %s x%d"):format(info.act == "wd" and "Withdrew" or "Stored", C.name(info.i), info.q), S.GOOD)
      if info.act == "wd" then
        -- a hand withdrawal: auto-deposit leaves these alone
        C.keep.pending[info.i], C.keep.asked[info.i] = nil, nil
        C.keep.at[info.i] = C.now()
        local t = C.kept_map()
        t[info.i] = (tonumber(C.prefs().kept[tostring(info.i)]) or 0) + info.q
        kept_save(t)
        if C.auto_mode() ~= 0 then C.log(("Auto-deposit will leave these %d alone."):format(info.q), S.FOCUS) end
      end
    elseif info.act == "wd" then
      C.keep.pending[info.i], C.keep.asked[info.i] = nil, nil
      C.log(("Couldn't withdraw %s: %s."):format(C.name(info.i), info.why ~= "" and info.why or "nothing moved"), S.WARN)
    elseif info.why == "in_use" then
      C.log(("%s is equipped or in use and stays in your bags."):format(C.name(info.i)), S.WARN)
    end
  elseif kind == "E" then
    local colour = info.kind == "done" and S.GOOD or ((info.kind == "stop" or info.kind == "no" or info.kind == "full") and S.WARN or S.MUTED)
    C.log(info.text, colour)
    if info.kind ~= "done" then C.note(info.text) end
  end
end

-- A chat line: a skill-up of a craft goes into the session tally
function C.on_chat(text)
  local craft, up = text:match("(%a+) skill rises ([%d%.]+) point")
  if not craft then return end
  for _, c in ipairs(L.CRAFTS) do
    if c.name == craft then
      C.session.skill[c.name] = (C.session.skill[c.name] or 0) + (tonumber(up) or 0)
      C.log(("%s skill +%s"):format(c.name, up), S.GOLD)
      C.request("md")
    end
  end
end

return C
