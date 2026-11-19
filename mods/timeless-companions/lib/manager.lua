-- Companion Manager (/cmgr): call, keep, save and set up companions.
--
-- Tabs: Party (who's with you: keep, two-hour, why, dismiss; downed companions and Raise), Call (every job, ready or
-- its cooldown, a click calls one), Kept (bond and where they are: call, errand, forget), Presets (load, edit, delete,
-- save your party, build one from jobs), Tactics (each job's rules) and Settings (modes, roles, sub jobs, chat).
-- Buttons send their !companion command as an action packet (link.send), never as chat. The server's answers show at
-- the top of the window for a while.

local S = require("lib.style")
local Canvas = require("lib.canvas")
local link = require("lib.link")
local feed = require("lib.feed_cm")
local live = require("lib.live")
local jobicons = require("lib.jobicons")

local M = {}

M.WIN = "companion_manager"
M.WIDTH = 620
local CW = M.WIDTH - 24
M.MESSAGE_SECONDS = 30
M.open_view = function() end -- set by init.lua

M.TABS = { "party", "call", "kept", "presets", "tactics", "settings" }
M.TAB_LABELS = { "Party", "Call", "Kept", "Presets", "Tactics", "Settings" }

local state = { tab = 1, build = { jobs = {} }, save_name = "", build_name = "", tactics_job = nil, note = nil }
M.state = state

local SLOW, FAST = link.SLOW, 0

local function model() return link.cm end
local function remaining(seconds) return feed.remaining(model(), seconds, os.time()) end

function M.role_of(jobId)
  local job = model().jobs[jobId]
  return S.ROLE[job and job.role or "DPS"] or S.ROLE.DPS
end

-- Party slots left, and whether companions can be called at all (and why not).
local function call_state()
  local L = model().limits
  local free = (L.partymax or 6) - (L.party or 1)
  if L.can == false then
    return false, (L.reason or "") ~= "" and L.reason or "Companions can't be called here.", free
  end
  if free <= 0 then return false, "Your party is full.", free end
  return true, nil, free
end

local function send(cmd, delay, in_chat)
  link.send(cmd, delay, in_chat)
  M.render_all()
end

-- Asks before an action that can't be taken back.
local function confirm(text, yes, cmd, delay)
  ui.prompt(text, { yes, "Cancel" }, function(i)
    if i == 1 then send(cmd, delay) end
  end)
end

-- Kept companions using a slot (legends have their own).
function M.kept_count()
  local n = 0
  for _, k in ipairs(model().kept) do
    if (k.lg or 0) == 0 then n = n + 1 end
  end
  return n
end

local function in_battle_chip(c, x, y)
  local w = Canvas.measure("In battle", 0.8, true) + 14
  c:rect(x, y, w, 18, S.fade(S.WARN, 0.2), { round = 9 })
  c:text(x + w / 2, y + 2, "In battle", { size = 0.8, bold = true, align = "center", colour = S.WARN })
  c:tip(x, y, w, 18, "In battle", "Role, gear and sub job changes wait until the fight is over.")
  return w
end

----------------------------------------------------------------------------------------------------------------------
-- Header: refresh, status, the server's latest answers

function M.draw_head(c)
  c:button(0, 0, 84, 26, "Refresh", { style = "primary", key = "refresh", tip = "Ask the server again.",
    enabled = not link.waiting.cm, on = function() link.ask("cm") M.render_all() end })
  local status, bad = link.status("cm")
  c:text(96, 5, status, { colour = bad and S.BAD or S.MUTED, w = CW - 96 - 130 })
  c:button(CW - 118, 0, 118, 26, "Companion View", { key = "view", size = 0.85,
    tip = "Your companions' gear and stats (/cv).", on = function() M.open_view() end })
  local y = 34
  local shown, now = 0, os.time()
  local msgs = model().messages or {}
  for i = #msgs, 1, -1 do
    local m = msgs[i]
    if now - m.at <= M.MESSAGE_SECONDS and shown < 3 then
      c:text(0, y, m.text, { colour = shown == 0 and S.GOLD or S.MUTED, size = shown == 0 and 0.95 or 0.85, w = CW })
      y = y + 18
      shown = shown + 1
    end
  end
  c:grow(y)
end

----------------------------------------------------------------------------------------------------------------------
-- Party

local MP_JOBS = { [3] = 1, [4] = 1, [5] = 1, [7] = 1, [8] = 1, [10] = 1, [15] = 1, [16] = 1, [20] = 1 }

function M.draw_party(c)
  local m, L = model(), model().limits
  local can, why, free = call_state()
  local x = 0
  local tw = c:text(0, 4, ("Party %d / %d"):format(L.party or 1, L.partymax or 6), { bold = true })
  if L.battle then in_battle_chip(c, tw + 12, 3) end

  local bx = CW
  bx = bx - 96
  c:button(bx, 0, 96, 26, "Dismiss all", { style = "danger", key = "rel_all", enabled = #m.party > 0,
    tip = "Send all your companions home (regular Trusts stay).",
    on = function() confirm("Send all your companions home?", "Dismiss all", "!companion release", SLOW) end })
  bx = bx - 104
  c:button(bx, 0, 98, 26, "Two-hour: all", { key = "2hr_all", enabled = #m.party > 0,
    tip = "Every companion uses its two-hour ability now.", on = function() send("!companion 2hr all", FAST) end })
  bx = bx - 98
  c:button(bx, 0, 92, 26, "Fill party", { style = "primary", key = "fill", enabled = can,
    tip = can and ("Call companions for the %d open slots: tank, healer, support, then damage."):format(free) or why,
    on = function() send("!companion", SLOW) end })
  local y = 34
  if not can and why and free > 0 then
    c:text(0, y, why, { colour = S.BAD, w = CW })
    y = y + 20
  end

  y = c:heading(0, y + 4, CW, "With you")
  if #m.party == 0 then
    c:text(0, y, "No companions in your party. Use Fill party, the Call tab, a preset or a kept companion.",
      { colour = S.MUTED, size = 0.9, w = CW })
    y = y + 22
  end

  local lives = {}
  for _, lm in ipairs(live.members()) do lives[lm.index] = lm end
  for _, p in ipairs(m.party) do
    local role = M.role_of(p.job)
    local lm = lives[p.id]
    local hpp = lm and lm.hpp or p.hpp or 0
    local mpp = lm and lm.mpp or p.mpp or 0
    c:card(0, y, CW, 56)
    c:rect(1, y + 8, 3, 40, role.colour, { round = 1 })
    jobicons.draw(c, 9, y + 11, 34, p.job, { role = role })
    c:text(49, y + 8, p.name, { bold = true, colour = role.colour, w = 96 })
    c:tip(9, y + 4, 136, 48, p.name, ("%s (%s)%s"):format(S.job_long(p.job), role.label,
      (p.pe or "") ~= "" and ("\nPersonality: " .. p.pe) or ""))
    local sub = (p.sjob or 0) ~= 0 and ("/%s%d"):format(S.job_short(p.sjob), p.slv) or ""
    c:text(49, y + 27, ("%s%d%s"):format(S.job_short(p.job), p.lv or 0, sub), { size = 0.85, colour = S.MUTED, w = 96 })

    c:text(150, y + 6, "HP", { size = 0.75, bold = true, colour = S.HP })
    c:text(260, y + 6, ("%d%%"):format(hpp), { size = 0.75, align = "right" })
    c:bar(150, y + 22, 110, 6, hpp / 100, S.HP)
    if mpp > 0 or MP_JOBS[p.job] then
      c:text(272, y + 6, "MP", { size = 0.75, bold = true, colour = S.MP })
      c:text(352, y + 6, ("%d%%"):format(mpp), { size = 0.75, align = "right" })
      c:bar(272, y + 22, 80, 6, mpp / 100, S.MP)
    end
    local act, colour = live.activity(lm and lm.id, lm and lm.entity)
    if hpp == 0 then act, colour = "Knocked out", S.BAD end
    if act then c:text(150, y + 35, act, { size = 0.75, colour = colour, w = 210 }) end

    local by = y + 15
    if (p.kept or 0) > 0 then
      local label = (p.bond or "") ~= "" and p.bond or "Kept"
      c:text(408, y + 19, label, { bold = true, colour = S.BOND, align = "center", w = 84, size = 0.9 })
      c:tip(366, y, 84, 56, "Kept", "A companion you keep. Their bond grows with every monster you defeat together.")
    else
      local full = M.kept_count() >= (L.keptmax or 8)
      c:button(366, by, 78, 26, "Keep", { key = "keep" .. p.id, enabled = not full,
        tip = full and "You keep as many companions as you can. Forget one first (Kept tab)."
          or ("Keep %s: they answer every call for a %s from now on and earn bond."):format(p.name, S.job_long(p.job)),
        on = function() send("!companion keep " .. p.name, FAST) end })
    end
    c:button(450, by, 40, 26, "2hr", { key = "2hr" .. p.id, size = 0.85,
      tip = ("%s uses their two-hour ability now."):format(p.name), on = function() send("!companion 2hr " .. p.name, FAST) end })
    c:button(494, by, 42, 26, "Why", { key = "why" .. p.id, size = 0.85,
      tip = ("What %s is doing and why, and their last few decisions (in chat)."):format(p.name),
      on = function() send("!companion why " .. p.name, 0, true) end })
    c:button(540, by, CW - 540 - 6, 26, "Dismiss", { style = "danger", key = "rel" .. p.id, size = 0.85,
      tip = ("Send %s home."):format(p.name),
      on = function() confirm(("Send %s home?"):format(p.name), "Dismiss", "!companion release " .. p.name, FAST) end })
    y = y + 62
  end

  if #m.downed > 0 then
    y = c:heading(0, y + 6, CW, "Downed")
    for i, d in ipairs(m.downed) do
      local left = remaining(d.left)
      c:card(0, y, CW, 36)
      c:text(12, y + 9, d.name, { bold = true, colour = S.BAD, w = 140 })
      c:text(160, y + 10, ("%s, %s left"):format(S.job_short(d.job), S.clock(left)), { size = 0.9, colour = S.MUTED })
      c:tip(0, y, 360, 36, d.name, "A healer companion who knows Raise brings them back between fights. If nobody "
        .. "does, they go home and their job can be called again at once.")
      c:button(CW - 90, y + 5, 84, 26, "Raise", { key = "raise" .. i, enabled = L.raise == true and left > 0,
        tip = L.raise and ("Raise %s yourself."):format(d.name) or "Your job can't cast Raise.",
        on = function() send("!companion raise " .. d.name, SLOW) end })
      y = y + 42
    end
  end
  c:grow(y)
end

----------------------------------------------------------------------------------------------------------------------
-- Call

function M.draw_call(c)
  local m = model()
  local can, why = call_state()
  c:text(0, 0, can and "Click a job to call a companion of it. A companion you keep answers for their job." or why,
    { colour = can and S.MUTED or S.BAD, size = 0.9, w = CW })
  local y = 26

  local keptByJob = {}
  for _, k in ipairs(m.kept) do keptByJob[k.job] = keptByJob[k.job] or k.name end

  local tw, th, gap, per = 92, 46, 8, 6
  for _, role in ipairs(S.ROLES) do
    local ids = {}
    for _, id in ipairs(m.jobOrder) do
      if M.role_of(id) == role then ids[#ids + 1] = id end
    end
    if #ids > 0 then
      c:text(0, y, role.label, { bold = true, colour = role.colour })
      y = y + 22
      for i, id in ipairs(ids) do
        local job = m.jobs[id]
        local cx = ((i - 1) % per) * (tw + gap)
        local cy = y + ((i - 1) // per) * (th + gap)
        local wait = remaining(job.wait)
        local price = job.egp or 0
        local more = job.party and "Call another one." or "Call one now."
        local sub, ok, hint, sub_colour
        if wait > 0 and price > 0 then
          sub, ok, sub_colour = ("%d EG"):format(price), can, S.GOLD
          hint = can and ("On cooldown for %s. %s for %d Everglass."):format(S.clock(wait), more, price) or why
        elseif wait > 0 then
          sub, ok, hint = S.clock(wait), false, ("Ready again in %s."):format(S.clock(wait))
        else
          sub, ok, hint = job.party and "in party" or "ready", can, can and more or why
          sub_colour = job.party and S.FOCUS or S.GOOD
        end
        local subText = (job.sub or 0) > 0 and ("/" .. S.job_short(job.sub)) or "default sub job"
        hint = ("%s\n%s"):format(subText, hint)
        if keptByJob[id] then hint = hint .. ("\n%s answers (kept)."):format(keptByJob[id]) end
        local hot = c:region(cx, cy, tw, th, { key = "call" .. id, enabled = ok, title = S.job_long(id), tip = hint,
          on = function() send("!companion " .. job.cmd, SLOW) end })
        local k = ok and 1 or 0.5
        c:rect(cx, cy, tw, th, S.fade(hot and S.CARD_HI or S.CARD, ok and 1.4 or 0.8), { round = 7 })
        c:rect(cx, cy, tw, th, S.fade(hot and S.EDGE_HI or S.EDGE, k), { round = 7, line = hot and 1.5 or 1 })
        jobicons.draw(c, cx + 6, cy + 7, 32, id, { role = role, dim = not ok })
        c:text(cx + 44, cy + 7, S.job_short(id), { bold = true, size = 1.05, colour = S.fade(role.colour, k) })
        c:text(cx + 44, cy + 25, sub, { size = 0.8, colour = S.fade(sub_colour or S.MUTED, k), w = tw - 48 })
      end
      y = y + ((#ids - 1) // per + 1) * (th + gap) + 8
    end
  end
  c:grow(y)
end

----------------------------------------------------------------------------------------------------------------------
-- Kept

-- Errand kinds (modules/companions/lua/companions_errands.lua)
M.ERRANDS = {
  { key = "log", label = "Log", tip = "Logging in a zone you've visited: logs and what grows there." },
  { key = "mine", label = "Mine", tip = "Mining in a zone you've visited: ores and stones." },
  { key = "harvest", label = "Harvest", tip = "Harvesting in a zone you've visited: plants and herbs." },
  { key = "excavate", label = "Excavate", tip = "Excavating in a zone you've visited." },
  { key = "dig", label = "Dig", tip = "Chocobo digging in a zone you've visited, at your digging rank." },
  { key = "crew", label = "Crew", tip = "Help the expedition crew: their chance goes up and you earn Everglass now." },
}

function M.draw_kept(c)
  local m, L = model(), model().limits
  local can, why = call_state()
  local legends = #m.kept - M.kept_count()
  c:text(0, 4, ("Kept %d / %d%s"):format(M.kept_count(), L.keptmax or 8,
    legends > 0 and (" + %d legend%s"):format(legends, legends == 1 and "" or "s") or ""), { bold = true })
  c:tip(0, 0, 200, 22, "Kept companions",
    "Keep a companion from the Party tab. They answer every call for their job, as the same person, and grow in bond.")

  local back = 0
  for _, k in ipairs(m.kept) do
    if k.where == "back" then back = back + 1 end
  end
  if back > 0 then
    c:button(CW - 140, 0, 140, 26, ("Collect (%d back)"):format(back), { style = "primary", key = "collect",
      tip = "Take what your companions brought back from their errands.",
      on = function() send("!companion collect", FAST) end })
  end
  local y = 34

  if #m.kept == 0 then
    c:text(0, y, "You keep no companions yet. Press Keep next to one in the Party tab.", { colour = S.MUTED, size = 0.9 })
    c:grow(y + 20)
    return
  end

  for _, k in ipairs(m.kept) do
    local role = M.role_of(k.job)
    c:card(0, y, CW, 52)
    c:rect(1, y + 8, 3, 36, role.colour, { round = 1 })
    jobicons.draw(c, 9, y + 9, 34, k.job, { role = role })
    c:text(49, y + 8, k.name, { bold = true, colour = role.colour, w = 86 })
    c:tip(9, y + 4, 126, 44, k.name, ("%s%s%s"):format(
      (k.lg or 0) > 0 and "Legendary companion: has a place of their own, not one of your kept slots.\n" or "",
      (k.pe or "") ~= "" and ("Personality: " .. k.pe .. "\n") or "",
      ("%d monsters defeated together."):format(k.kl or 0)))
    c:text(49, y + 27, S.job_short(k.job) .. ((k.lg or 0) > 0 and "  ·  Legend" or ""), { size = 0.85, colour = S.MUTED, w = 86 })

    -- Bond: rank, and progress within it to the next
    c:text(140, y + 7, k.rank or "", { bold = true, colour = S.BOND, size = 0.9 })
    local points = k.points or 0
    if (k.next or "") ~= "" and (k.nextat or 0) > 0 then
      local from = math.min(k.at or 0, points)
      local need = math.max(1, k.nextat - from)
      local done = math.min(points - from, need)
      c:text(300, y + 8, ("%d/%d to %s"):format(done, need, k.next), { size = 0.75, align = "right", colour = S.MUTED })
      c:bar(140, y + 30, 160, 6, done / need, S.BOND)
      c:tip(140, y, 160, 52, "Bond", ("%d bond in all. %s at %d.\nBond grows with every monster you defeat together.")
        :format(points, k.next, k.nextat))
    else
      c:text(300, y + 8, ("%d, top rank"):format(points), { size = 0.75, align = "right", colour = S.MUTED })
      c:bar(140, y + 30, 160, 6, 1, S.BOND)
      c:tip(140, y, 160, 52, "Bond", "The top rank. They use their two-hour by themselves in an emergency.")
    end

    -- Where they are
    local wait = remaining(k.wait)
    local status, colour, ok, hint
    if k.where == "party" then
      status, colour, ok, hint = "with you", S.GOOD, false, "Already in your party."
    elseif k.where == "down" then
      status, colour, ok, hint = "down", S.BAD, false, "Downed: raise them, or wait until they go home."
    elseif k.where == "errand" then
      status, colour, ok, hint = "away " .. S.clock(wait), S.FOCUS, false, ("On an errand, back in %s."):format(S.clock(wait))
    elseif k.where == "back" then
      status, colour, ok, hint = "back!", S.GOLD, false, "Back from an errand: Collect what they brought first."
    elseif wait > 0 then
      status, colour = S.clock(wait), S.MUTED
      local job = m.jobs[k.job]
      local price = job and job.egp or 0
      if price > 0 then
        ok = can
        hint = can and ("Their job is on cooldown for %s. Call %s now for %d Everglass."):format(S.clock(wait), k.name, price) or why
      else
        ok, hint = false, ("Their job is ready again in %s."):format(S.clock(wait))
      end
    else
      status, colour, ok, hint = "ready", S.TEXT, can, can and ("Call %s."):format(k.name) or why
    end
    c:text(374, y + 17, status, { size = 0.9, colour = colour, align = "center", w = 70 })

    c:button(412, y + 13, 50, 26, "Call", { style = "primary", key = "kcall" .. k.slot, enabled = ok, tip = hint, size = 0.85,
      on = function() send("!companion call " .. k.name, SLOW) end })
    local home = k.where ~= "errand" and k.where ~= "back" and k.where ~= "down"
    c:button(466, y + 13, 62, 26, "Errand", { key = "errand" .. k.slot, enabled = home, size = 0.85,
      tip = home and ("Send %s away for an hour to gather, dig or help the expedition crew."):format(k.name)
        or "Not while they're away, back with finds, or down.",
      on = function()
        local labels = {}
        for _, e in ipairs(M.ERRANDS) do labels[#labels + 1] = e.label .. ": " .. e.tip end
        ui.menu(labels, function(i)
          if i then send(("!companion errand %s %s"):format(k.name, M.ERRANDS[i].key), SLOW) end
        end)
      end })
    c:button(532, y + 13, CW - 532 - 6, 26, "Forget", { style = "danger", key = "forget" .. k.slot, size = 0.85,
      tip = ("Stop keeping %s. Their bond is lost."):format(k.name),
      on = function() confirm(("Stop keeping %s? Their bond is lost."):format(k.name), "Forget", "!companion forget " .. k.name, FAST) end })
    y = y + 58
  end
  c:grow(y)
end

----------------------------------------------------------------------------------------------------------------------
-- Presets

local function jobs_text(jobs)
  local names = {}
  for _, id in ipairs(jobs) do names[#names + 1] = S.job_short(id) end
  return table.concat(names, " ")
end

-- What the server accepts as a name: letters, digits, spaces, - and ', up to 15.
function M.clean_name(name)
  name = (name or ""):gsub("[^%w %-']", ""):gsub("^%s+", ""):gsub("%s+$", ""):gsub("%s+", " ")
  return name:sub(1, 15)
end

local function preset_named(name)
  for _, p in ipairs(model().presets) do
    if p.name:lower() == name:lower() then return p end
  end
  return nil
end

function M.draw_presets(c)
  local m, L = model(), model().limits
  c:text(0, 4, ("Presets %d / %d"):format(#m.presets, L.presetmax or 5), { bold = true })
  c:tip(0, 0, 160, 22, "Presets", "Loading a preset sends home the companions not in it and calls the missing jobs.")
  local y = 32
  if #m.presets == 0 then
    c:text(0, y, "No presets yet. Save your party or build one below.", { colour = S.MUTED, size = 0.9 })
    y = y + 22
  end
  for _, p in ipairs(m.presets) do
    c:card(0, y, CW, 40)
    c:text(12, y + 11, p.name, { bold = true, colour = S.GOLD, w = 130 })
    -- The jobs as small role-coloured tags
    local x = 150
    for _, id in ipairs(p.jobs) do
      jobicons.draw(c, x, y + 7, 26, id, { role = M.role_of(id) })
      c:tip(x, y + 7, 26, 26, S.job_long(id), nil)
      x = x + 30
    end
    local loadable = L.can ~= false
    c:button(CW - 200, y + 7, 60, 26, "Load", { style = "primary", key = "load" .. p.slot, enabled = loadable, size = 0.85,
      tip = loadable and ('Call "%s".'):format(p.name) or (L.reason or ""), on = function() send("!companion load " .. p.name, SLOW) end })
    c:button(CW - 136, y + 7, 60, 26, "Edit", { key = "edit" .. p.slot, size = 0.85, tip = "Put these jobs in the builder below.",
      on = function()
        state.build.jobs = {}
        for _, id in ipairs(p.jobs) do state.build.jobs[#state.build.jobs + 1] = id end
        state.build_name = p.name
        pcall(ui.update, M.WIN, "build_name", { text = p.name })
        M.render_all()
      end })
    c:button(CW - 72, y + 7, 66, 26, "Delete", { style = "danger", key = "del" .. p.slot, size = 0.85,
      tip = ('Forget "%s".'):format(p.name),
      on = function() confirm(('Delete the preset "%s"?'):format(p.name), "Delete", "!companion delete " .. p.name, FAST) end })
    y = y + 46
  end
  y = c:heading(0, y + 6, CW, "Save your party", ("%d companion%s with you"):format(#m.party, #m.party == 1 and "" or "s"))
  c:grow(y - 6)
end

-- Saving the party: checks the name; returns nil, or the reason it can't.
function M.save_problem(name)
  local m, L = model(), model().limits
  if #m.party == 0 then return "Call the companions you want first." end
  if name == "" then return "Name the preset (letters, digits and spaces, up to 15)." end
  if #m.presets >= (L.presetmax or 5) and not preset_named(name) then
    return "You have as many presets as you can. Delete one first."
  end
  return nil
end

function M.build_problem(name)
  local m, L = model(), model().limits
  if #state.build.jobs == 0 then return "Pick the jobs first." end
  if name == "" then return "Name the preset (letters, digits and spaces, up to 15)." end
  if #m.presets >= (L.presetmax or 5) and not preset_named(name) then
    return "You have as many presets as you can. Delete one first."
  end
  return nil
end

function M.draw_builder(c)
  local m, L = model(), model().limits
  local y = 0
  if state.note then
    c:text(0, y, state.note.text, { colour = state.note.bad and S.BAD or S.GOOD, size = 0.9, w = CW })
    y = y + 20
  end
  y = c:heading(0, y + 6, CW, "Build a preset", "Pick up to " .. (L.presetjobs or 5) .. " jobs")
  local max = L.presetjobs or 5
  local picked = {}
  for _, id in ipairs(state.build.jobs) do picked[id] = true end
  local tw, th, gap, per = 50, 26, 4, 11
  for i, id in ipairs(m.jobOrder) do
    local cx = ((i - 1) % per) * (tw + gap)
    local cy = y + ((i - 1) // per) * (th + gap)
    local on = picked[id] == true
    local role = M.role_of(id)
    c:button(cx, cy, tw, th, S.job_short(id), { key = "b" .. id, style = on and "on" or "off", size = 0.85, bold = true,
      colour = on and role.colour or S.MUTED, enabled = on or #state.build.jobs < max, title = S.job_long(id),
      tip = on and "Click to take it out." or "Click to add it.",
      on = function()
        if picked[id] then
          for j, x in ipairs(state.build.jobs) do
            if x == id then table.remove(state.build.jobs, j) break end
          end
        else
          state.build.jobs[#state.build.jobs + 1] = id
        end
        M.render_all()
      end })
  end
  y = y + ((#m.jobOrder - 1) // per + 1) * (th + gap) + 4
  c:text(0, y, #state.build.jobs > 0 and jobs_text(state.build.jobs) or "No jobs picked.",
    { colour = #state.build.jobs > 0 and S.TEXT or S.MUTED, size = 0.9 })
  c:grow(y + 20)
end

function M.save_party()
  local name = M.clean_name(state.save_name)
  local problem = M.save_problem(name)
  if problem then
    state.note = { text = problem, bad = true }
  else
    send("!companion save " .. name, FAST)
    state.note = { text = ('Saving your party as "%s"...'):format(name) }
    state.save_name = ""
    pcall(ui.update, M.WIN, "save_name", { text = "" })
  end
  M.render_all()
end

function M.save_build()
  local m = model()
  local name = M.clean_name(state.build_name)
  local problem = M.build_problem(name)
  if problem then
    state.note = { text = problem, bad = true }
  else
    local cmds = {}
    for _, id in ipairs(state.build.jobs) do
      cmds[#cmds + 1] = m.jobs[id] and m.jobs[id].cmd or S.job_short(id):lower()
    end
    send(("!companion define %s %s"):format(table.concat(cmds, ","), name), FAST)
    state.note = { text = ('Saving "%s": %s...'):format(name, jobs_text(state.build.jobs)) }
    state.build = { jobs = {} }
    state.build_name = ""
    pcall(ui.update, M.WIN, "build_name", { text = "" })
  end
  M.render_all()
end

----------------------------------------------------------------------------------------------------------------------
-- Tactics

M.TACTICS = {
  { key = "interrupt", label = "Interrupts", values = { "on", "off" },
    hint = "Stun dangerous spells and TP moves (Stun, Shield Bash, Weapon Bash; a stun weapon skill for deadly ones)." },
  { key = "dispel", label = "Dispel", values = { "on", "off" },
    hint = "Red Mages and Bards dispel buffs worth removing (Stoneskin, Blink, Haste...)." },
  { key = "weakness", label = "Weakness", values = { "on", "off" },
    hint = "Nukers cast the element the monster takes the most from (resistances, day, weather)." },
  { key = "sleep", label = "Sleep adds", values = { "on", "off" }, hint = "Sleepers put monsters that join the fight to sleep." },
  { key = "aoe", label = "Area spells", values = { "auto", "never", "free" },
    hint = "auto: no area spells while a monster sleeps near the target. never: single target only. free: no limits." },
  { key = "range", label = "Caster range", values = { "auto", "close", "far" },
    hint = "How far casters stand. auto also steps back past area moves the monster has used." },
  { key = "talk", label = "Callouts", values = { "on", "quiet" }, hint = "Tactics callouts in party chat (interrupts, weaknesses, dispels)." },
}
M.MP_STEPS = { 0, 10, 20, 30, 40, 50 }

function M.draw_tactics(c)
  local m = model()
  c:text(0, 0, "Rules for each job's companions. Presets save them too.", { colour = S.MUTED, size = 0.9 })
  local y = 24
  state.tactics_job = state.tactics_job or m.jobOrder[1]
  local tw, th, gap, per = 50, 26, 4, 11
  for i, id in ipairs(m.jobOrder) do
    local cx = ((i - 1) % per) * (tw + gap)
    local cy = y + ((i - 1) // per) * (th + gap)
    local custom = m.tactics ~= nil and m.tactics[id] ~= nil
    local on = state.tactics_job == id
    c:button(cx, cy, tw, th, S.job_short(id) .. (custom and "*" or ""), { key = "tj" .. id, style = on and "on" or "off",
      size = 0.85, bold = true, title = S.job_long(id), tip = custom and "Changed from the defaults." or nil,
      on = function() state.tactics_job = id M.render_all() end })
  end
  y = y + ((#m.jobOrder - 1) // per + 1) * (th + gap) + 6
  local jobId = state.tactics_job
  local job = m.jobs[jobId]
  if not job then c:grow(y) return end

  y = c:heading(0, y, CW, S.job_long(jobId))
  local rules = feed.tactics_of(m, jobId)
  for _, r in ipairs(M.TACTICS) do
    c:text(0, y + 5, r.label, { size = 0.9, colour = S.MUTED })
    c:tip(0, y, 120, 26, r.label, r.hint)
    local items, lit = {}, 0
    for i, v in ipairs(r.values) do
      items[i] = { label = v, tip = r.hint, w = 64 }
      if rules[r.key] == v then lit = i end
    end
    c:switch(130, y, 26, items, lit, { key = "t_" .. r.key,
      on = function(i) send(("!companion tactics %s %s %s"):format(job.cmd, r.key, r.values[i]), FAST) end })
    y = y + 32
  end
  c:text(0, y + 5, "MP kept back", { size = 0.9, colour = S.MUTED })
  c:tip(0, y, 120, 26, "MP kept back", "No offensive magic below this much MP.")
  local items, lit = {}, 0
  for i, v in ipairs(M.MP_STEPS) do
    items[i] = { label = v == 0 and "none" or (v .. "%"), tip = "Keep this much MP back.", w = 52 }
    if (rules.mp or 0) == v then lit = i end
  end
  c:switch(130, y, 26, items, lit, { key = "t_mp",
    on = function(i) send(("!companion tactics %s mp %d"):format(job.cmd, M.MP_STEPS[i]), FAST) end })
  y = y + 38
  c:button(0, y, 96, 26, "Defaults", { key = "t_reset", enabled = m.tactics ~= nil and m.tactics[jobId] ~= nil,
    tip = "Every rule back to its default for this job.",
    on = function() send(("!companion tactics %s reset"):format(job.cmd), FAST) end })
  c:grow(y + 30)
end

----------------------------------------------------------------------------------------------------------------------
-- Settings

local function setting_row(c, y, label, hint, a, a_on, a_hint, a_cmd, b, b_hint, b_cmd, delay, key)
  c:text(0, y + 5, label, { size = 0.9, colour = S.MUTED })
  c:tip(0, y, 140, 26, label, hint)
  c:switch(150, y, 26, { { label = a, tip = a_hint, w = 84 }, { label = b, tip = b_hint, w = 84 } }, a_on and 1 or 2,
    { key = key, on = function(i) send(i == 1 and a_cmd or b_cmd, delay) end })
  return y + 32
end

function M.draw_settings(c)
  local m = model()
  local Md = m.modes or {}
  local y = 0
  if m.limits.battle then
    c:text(0, y, "In battle: role, gear and sub job changes wait until the fight is over.", { colour = S.WARN, size = 0.9 })
    y = y + 22
  end
  y = c:heading(0, y, CW, "Fighting")
  y = setting_row(c, y, "Engage", "When companions join the fight.", "Attack", Md.engage ~= "retail", "As soon as you engage.",
    "!companion engage attack", "Retail", "After your first swing (applies to your regular Trusts too).",
    "!companion engage retail", FAST, "engage")
  y = setting_row(c, y, "Treasure Hunter", "What Thieves wear. They swap out and back in with the new gear, between fights.",
    "On", Md.th, "Thieves wear the most Treasure Hunter.", "!companion th on", "Off", "Thieves wear their best damage gear.",
    "!companion th off", SLOW, "th")
  y = setting_row(c, y, "Nuke mode", "Black Mages swap out and back in, between fights.", "On", Md.nuke,
    "Black Mages cast nukes nonstop.", "!companion nuke on", "Off", "Black Mages nuke at their normal pace.",
    "!companion nuke off", SLOW, "nuke")
  y = setting_row(c, y, "Party chat", "Companions call out in party chat (skillchains, two-hours, low MP...).", "On",
    Md.chat ~= false, "Callouts on.", "!companion chat on", "Off", "Callouts off.", "!companion chat off", FAST, "chat")

  y = c:heading(0, y + 8, CW, "Roles")
  y = setting_row(c, y, "Warrior", "Warriors in your party swap for the new role, between fights.", "Tank", not Md.wardps,
    "Axe or sword and shield, tanking gear.", "!companion war tank", "Damage", "Great axe and damage gear.",
    "!companion war dps", SLOW, "war")
  y = setting_row(c, y, "Paladin", "Paladins in your party swap for the new role, between fights.", "Tank", not Md.plddps,
    "Sword and shield.", "!companion pld tank", "Damage", "Great sword and damage gear.", "!companion pld dps", SLOW, "pld")
  y = setting_row(c, y, "Dark Knight", "Dark Knights in your party swap for the new role, between fights.", "Tank",
    Md.drktank, "Great sword and tanking gear; holds the monster.", "!companion drk tank", "Damage",
    "Damage dealer (the default).", "!companion drk dps", SLOW, "drk")

  y = c:heading(0, y + 8, CW, "Sub jobs", "Click one to change it")
  local per = 4
  local colw, rowh = CW / per, 30
  for i, id in ipairs(m.jobOrder) do
    local job = m.jobs[id]
    local cx = ((i - 1) % per) * colw
    local cy = y + ((i - 1) // per) * rowh
    jobicons.draw(c, cx, cy + 1, 22, id, { role = M.role_of(id) })
    c:text(cx + 25, cy + 5, S.job_short(id), { bold = true, colour = M.role_of(id).colour, size = 0.8 })
    local cur = (job.sub or 0) > 0 and S.job_short(job.sub) or "Default"
    c:button(cx + 56, cy, colw - 62, 24, cur .. "  ▾", { key = "sub" .. id, size = 0.85,
      title = S.job_long(id) .. " companions' sub job",
      tip = "Ones in your party switch between fights.",
      on = function()
        local labels, ids = { "Default" }, { 0 }
        for _, sid in ipairs(m.jobOrder) do
          if sid ~= id then
            labels[#labels + 1] = S.job_short(sid) .. "  " .. S.job_long(sid)
            ids[#ids + 1] = sid
          end
        end
        ui.menu(labels, function(k)
          if not k then return end
          local sid = ids[k]
          if sid == (job.sub or 0) then return end
          send(("!companion sub %s %s"):format(job.cmd, sid == 0 and "default" or m.jobs[sid].cmd), SLOW)
        end)
      end })
  end
  y = y + ((#m.jobOrder - 1) // per + 1) * rowh + 6

  y = c:heading(0, y + 4, CW, "In chat", "The answer goes to your chat log")
  local x = 0
  for _, b in ipairs({
    { "Status", "!companion status", "Every job's cooldown." },
    { "Fight report", "!companion report", "Damage, weapon skills and healing for the last fight." },
    { "Weapon skills", "!companion ws", "What each companion uses." },
    { "Gear", "!companion gear", "What each companion wears." },
  }) do
    local w = Canvas.measure(b[1], 0.9) + 26
    c:button(x, y, w, 26, b[1], { key = "chat_" .. b[1], tip = b[3], on = function() send(b[2], 0, true) end })
    x = x + w + 6
  end
  c:grow(y + 30)
end

----------------------------------------------------------------------------------------------------------------------

local PAGE_CANVASES = {
  { "party" }, { "call" }, { "kept" }, { "presets", "builder" }, { "tactics" }, { "settings" },
}

function M.render_all()
  if not ui.is_open(M.WIN) then return end
  Canvas.render(M.WIN, "head")
  if not model().loaded then
    Canvas.render(M.WIN, "waiting")
    return
  end
  for _, id in ipairs(PAGE_CANVASES[state.tab] or {}) do Canvas.render(M.WIN, id) end
end

local function waiting_draw(c)
  if model().loaded then c:grow(0) return end
  c:card(0, 0, CW, 60)
  c:text(CW / 2, 20, link.waiting.cm and "Asking the server for your companions..." or "No answer from the server yet. Press Refresh.",
    { align = "center", colour = S.MUTED })
end

-- The window's elements: the header, a "waiting" note until the first feed, then the tabs and their pages.
local function elements()
  local loaded = model().loaded
  local list = {
    { type = "canvas", id = "head", w = CW, h = 30, draw = {} },
  }
  if not loaded then
    list[#list + 1] = { type = "canvas", id = "waiting", w = CW, h = 60, draw = {} }
    return list
  end
  list[#list + 1] = { type = "tabs", id = "tabs", items = M.TAB_LABELS, at = state.tab }
  list[#list + 1] = { type = "canvas", id = "party", w = CW, h = 100, draw = {}, page = 1 }
  list[#list + 1] = { type = "canvas", id = "call", w = CW, h = 100, draw = {}, page = 2 }
  list[#list + 1] = { type = "canvas", id = "kept", w = CW, h = 100, draw = {}, page = 3 }
  list[#list + 1] = { type = "canvas", id = "presets", w = CW, h = 100, draw = {}, page = 4 }
  list[#list + 1] = { type = "input", id = "save_name", label = "Name", text = state.save_name, max = 15, page = 4,
    tip = "Letters, digits and spaces, up to 15. A name you already use replaces that preset." }
  list[#list + 1] = { type = "buttons", id = "save", items = { "Save party" }, page = 4,
    tip = "Save the companions in your party now as a preset." }
  list[#list + 1] = { type = "canvas", id = "builder", w = CW, h = 100, draw = {}, page = 4 }
  list[#list + 1] = { type = "input", id = "build_name", label = "Name", text = state.build_name, max = 15, page = 4,
    tip = "Letters, digits and spaces, up to 15." }
  list[#list + 1] = { type = "buttons", id = "build", items = { "Save preset", "Clear" }, page = 4 }
  list[#list + 1] = { type = "canvas", id = "tactics", w = CW, h = 100, draw = {}, page = 5 }
  list[#list + 1] = { type = "canvas", id = "settings", w = CW, h = 100, draw = {}, page = 6 }
  return list
end

local built_loaded = nil

-- (Re)defines the window: once with the waiting note, again with the tabs when the first feed arrives.
function M.define()
  local loaded = model().loaded
  if built_loaded == loaded then return end
  built_loaded = loaded
  ui.window(M.WIN, {
    title = "Companion Manager",
    width = M.WIDTH,
    help = "Call, keep, save and set up your companions. Buttons act at once; the server answers at the top.",
    elements = elements(),
  })
end

local function on_event(win, e)
  if Canvas.event(win, e) then return end
  if e.kind == "set" and e.id == "tabs" then
    state.tab = e.index
    state.note = nil
    M.render_all()
  elseif (e.kind == "edit" or e.kind == "change") and e.id == "save_name" then
    state.save_name = e.text or ""
  elseif (e.kind == "edit" or e.kind == "change") and e.id == "build_name" then
    state.build_name = e.text or ""
  elseif e.kind == "press" and e.id == "save" then
    state.save_name = ui.get(win, "save_name").text or state.save_name
    M.save_party()
  elseif e.kind == "press" and e.id == "build" then
    if e.index == 1 then
      state.build_name = ui.get(win, "build_name").text or state.build_name
      M.save_build()
    else
      state.build = { jobs = {} }
      state.build_name = ""
      state.note = nil
      pcall(ui.update, win, "build_name", { text = "" })
      M.render_all()
    end
  end
end

function M.setup()
  M.define()
  Canvas.add(M.WIN, "head", CW, M.draw_head)
  Canvas.add(M.WIN, "waiting", CW, waiting_draw)
  Canvas.add(M.WIN, "party", CW, M.draw_party)
  Canvas.add(M.WIN, "call", CW, M.draw_call)
  Canvas.add(M.WIN, "kept", CW, M.draw_kept)
  Canvas.add(M.WIN, "presets", CW, M.draw_presets)
  Canvas.add(M.WIN, "builder", CW, M.draw_builder)
  Canvas.add(M.WIN, "tactics", CW, M.draw_tactics)
  Canvas.add(M.WIN, "settings", CW, M.draw_settings)
  ui.on(M.WIN, on_event)
end

-- A feed came in: switch from the waiting note to the tabs if this was the first, then redraw.
function M.refresh()
  local was_open = ui.is_open(M.WIN)
  M.define()
  if was_open and not ui.is_open(M.WIN) then ui.show(M.WIN) end
  M.render_all()
end

function M.is_open() return ui.is_open(M.WIN) end

function M.open(tab)
  if tab then
    state.tab = tab
    if model().loaded then pcall(ui.update, M.WIN, "tabs", { at = tab }) end
  end
  ui.show(M.WIN)
  link.ask("cm")
  M.render_all()
end

function M.toggle()
  if ui.is_open(M.WIN) then ui.close(M.WIN) else M.open() end
end

-- "party" -> 1 ...
function M.tab_index(name)
  for i, t in ipairs(M.TABS) do
    if t == name then return i end
  end
  return nil
end

return M
