--[[
* Timeless Companions - feed_cm.lua (from the companionmanager addon)
*
* Reads the server's manager feed (modules/companions, !companion mdata).
*
* Every line is:   CM1|<type>|<id>|key=value,key=value,...
*
*   B  begin a batch
*   L  limits: party, partymax, keptmax, presetmax, presetjobs, cd, downed
*   C  state: can, battle, raise, reason (merged into model.limits too)
*   M  modes: th, nuke, chat, wardps, plddps, drktank (1/0), engage (attack|retail)
*   J  a job (id = job id): cmd, role, wait (seconds), sub (job id, 0 = default), party (1/0),
*      egp (Everglass to call it while on cooldown; 0 = can't)
*   P  a companion in the party (id = target index): name, job, lv, sjob, slv, hpp, mpp, kept, bond
*   K  a kept companion (id = slot): name, job, race, points, rank, at (where the rank starts),
*      next, nextat, where, wait
*   R  a preset (id = slot): name, jobs (job ids joined with '.')
*   D  a downed companion: name, job, left (seconds)
*   X  a job's tactics rules (id = job id), only jobs not at the defaults: interrupt, dispel,
*      weakness, sleep, aoe, range, mp, talk. Jobs without an X line use protocol.TACTICS_DEFAULTS.
*   (P and K lines also carry pe, the personality; K carries kl, monsters defeated together)
*   Z  end of the batch: everything in it replaces what was shown
*   T  outside batches: the server's answer to one of this window's buttons (text=..., the rest
*      of the line as is), kept in model.messages
*
* Times (wait, left) count down from when the batch arrived (model.received), so the window can
* show them ticking without asking again. Unknown record types and keys are ignored, so the
* server can send more without breaking this addon.
*
* No game or UI code here, so it can be tested on its own.
--]]

local protocol = {}

protocol.VERSION = 'CM1'

local NUMBER_KEYS = {
    party = true, partymax = true, keptmax = true, presetmax = true, presetjobs = true, cd = true,
    downed = true, wait = true, sub = true, job = true, lv = true, sjob = true, slv = true,
    hpp = true, mpp = true, kept = true, race = true, points = true, at = true, nextat = true, left = true,
    mp = true, kl = true, egp = true, lg = true,
}

-- Tactics rules (modules/companions, companions_tactics.lua): every rule at its default
protocol.TACTICS_DEFAULTS = {
    interrupt = 'on', dispel = 'on', weakness = 'on', sleep = 'on', aoe = 'auto', range = 'auto', mp = 0, talk = 'on',
}

local FLAG_KEYS = {
    can = true, battle = true, raise = true, th = true, nuke = true, chat = true,
    wardps = true, plddps = true, drktank = true,
}

local function empty_batch()
    return {
        limits  = {},
        modes   = {},
        jobs    = {}, -- job id -> { id, cmd, role, wait, sub, party }
        jobOrder = {}, -- job ids, in the order sent
        party   = {}, -- list of companions in the party
        kept    = {}, -- list of kept companions
        presets = {}, -- list of presets
        downed  = {}, -- list of downed companions
        tactics = {}, -- job id -> rules (only jobs not at the defaults; see protocol.tactics_of)
    }
end

--[[
* Creates an empty data model.
--]]
function protocol.new_model()
    local m = empty_batch()
    m.pending  = nil;   -- batch being received
    m.received = 0;     -- os.time() when the last complete batch arrived
    m.loaded   = false; -- true once a batch has arrived
    m.messages = {};    -- the server's latest answers: { text, at }, newest last
    return m
end

local function split(str, sep)
    local out, pos = {}, 1
    while (true) do
        local s = string.find(str, sep, pos, true)
        if (s == nil) then
            table.insert(out, string.sub(str, pos))
            return out
        end
        table.insert(out, string.sub(str, pos, s - 1))
        pos = s + 1
    end
end

local function parse_pairs(body)
    local out = {}
    for pair in string.gmatch(body or '', '[^,]+') do
        local k, v = string.match(pair, '^([%w_]+)=(.*)$')
        if (k ~= nil) then
            if (NUMBER_KEYS[k]) then
                out[k] = tonumber(v) or 0
            else
                out[k] = v
            end
        end
    end
    return out
end

local function flags(kv)
    for k, v in pairs(kv) do
        if (FLAG_KEYS[k]) then
            kv[k] = (v == '1' or v == 1)
        end
    end
    return kv
end

local function job_list(text)
    local jobs = {}
    for id in string.gmatch(text or '', '%d+') do
        table.insert(jobs, tonumber(id))
    end
    return jobs
end

--[[
* Finds a feed line in a chat message. Returns the line, or nil if the message isn't one.
--]]
function protocol.extract(message)
    if (type(message) ~= 'string') then
        return nil
    end

    local start = string.find(message, protocol.VERSION .. '|', 1, true)
    if (start == nil) then
        return nil
    end

    -- Drop anything the chat log adds after the text (colors, line ends)
    local line = string.sub(message, start)
    line = string.gsub(line, '[%c\128-\255]+.*$', '')
    return line
end

protocol.MAX_MESSAGES = 6

--[[
* The text of a T line, or nil if the line isn't one.
--]]
function protocol.message_text(line)
    if (type(line) ~= 'string') then
        return nil
    end
    return string.match(line, '^' .. protocol.VERSION .. '|T|%d+|text=(.*)$')
end

--[[
* Feeds one line into the model. Returns true when a batch has just completed.
* now: the current os.time() (passed in so tests can control it).
--]]
function protocol.feed(model, line, now)
    local parts = split(line, '|')
    if (parts[1] ~= protocol.VERSION or #parts < 3) then
        return false
    end

    local kind = parts[2]
    local id   = tonumber(parts[3]) or 0

    if (kind == 'T') then
        local text = protocol.message_text(line)
        if (text ~= nil and text ~= '') then
            table.insert(model.messages, { text = text, at = now or os.time() })
            while (#model.messages > protocol.MAX_MESSAGES) do
                table.remove(model.messages, 1)
            end
        end
        return false
    end

    local kv   = parse_pairs(parts[4])

    if (kind == 'B') then
        model.pending = empty_batch()
        return false
    end

    local b = model.pending
    if (b == nil) then
        return false; -- only whole batches are used
    end

    if (kind == 'L' or kind == 'C') then
        -- L (limits) and C (state) both go in limits; an older server sends them all in L
        for k, v in pairs(flags(kv)) do
            b.limits[k] = v
        end
    elseif (kind == 'M') then
        b.modes = flags(kv)
    elseif (kind == 'J') then
        kv.id = id
        kv.party = (kv.party == 1)
        b.jobs[id] = kv
        table.insert(b.jobOrder, id)
    elseif (kind == 'P') then
        kv.id = id
        table.insert(b.party, kv)
    elseif (kind == 'K') then
        kv.slot = id
        table.insert(b.kept, kv)
    elseif (kind == 'R') then
        kv.slot = id
        kv.jobs = job_list(kv.jobs)
        table.insert(b.presets, kv)
    elseif (kind == 'D') then
        table.insert(b.downed, kv)
    elseif (kind == 'X') then
        b.tactics[id] = kv
    elseif (kind == 'Z') then
        for k, v in pairs(b) do
            model[k] = v; -- messages aren't part of a batch, so they stay
        end
        model.pending  = nil
        model.received = now or os.time()
        model.loaded   = true
        return true
    end

    return false
end

--[[
* A job's tactics rules: what the server sent, with defaults for anything missing.
--]]
function protocol.tactics_of(model, jobId)
    local out  = {}
    local sent = (model.tactics or {})[jobId] or {}
    for k, v in pairs(protocol.TACTICS_DEFAULTS) do
        if (sent[k] ~= nil) then
            out[k] = sent[k]
        else
            out[k] = v
        end
    end
    return out
end

--[[
* Seconds left of a time that was <seconds> when the batch arrived.
--]]
function protocol.remaining(model, seconds, now)
    local passed = (now or os.time()) - (model.received or 0)
    return math.max(0, (seconds or 0) - passed)
end

function protocol.clock(seconds)
    seconds = math.max(0, math.floor(seconds or 0))
    if (seconds >= 3600) then
        return string.format('%d:%02d:%02d', math.floor(seconds / 3600), math.floor(seconds / 60) % 60, seconds % 60)
    end
    return string.format('%d:%02d', math.floor(seconds / 60), seconds % 60)
end

return protocol
