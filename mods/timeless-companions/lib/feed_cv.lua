--[[
* Timeless Companions - feed_cv.lua (from the companionview addon)
*
* Reads the server's companion data feed (modules/companions, !companion data).
*
* Every line is:   CV1|<type>|<id>|key=value,key=value,...
*
*   B  begin a batch (count=<companions>)
*   M  the player's modes (id 0): th, nuke, wardps, plddps, drktank = 1 or 0
*   H  who a companion is: name, mjob, mlv, sjob, slv, race, tank, kept (1/0), bond (rank name or empty)
*   G  gear: slot=itemId (0 = empty); sent in more than one line
*   S  combat stats: hp, mhp, mp, mmp, tp, att, def, acc, eva, ratt, racc
*   A  attributes as base+bonus: str=60+7, dex=..., chr=...
*   Z  end of the batch: the companions in it replace the ones shown
*
* <id> is the companion's target index. Record types and keys this addon doesn't
* know are kept (rec.extra) or ignored, so the server can send more without
* breaking this addon. Only a new version tag (RV2) means an incompatible change.
--]]

local protocol = {}

protocol.VERSION = 'CV1'

-- Equipment slots, in the order of the game's equipment menu (4 x 4)
protocol.SLOTS = {
    'main', 'sub', 'ranged', 'ammo',
    'head', 'neck', 'ear1', 'ear2',
    'body', 'hands', 'ring1', 'ring2',
    'back', 'waist', 'legs', 'feet',
}

protocol.ATTRIBUTES = { 'str', 'dex', 'vit', 'agi', 'int', 'mnd', 'chr' }

--[[
* Creates an empty data model.
--]]
function protocol.new_model()
    return {
        companions = {},  -- id -> companion
        order    = {},  -- ids, in the order the server sent them
        modes    = {},  -- the player's modes from the server (th = true/false, ...)
        pending  = nil, -- batch being received
        updated  = 0,   -- os.time() of the last complete batch
    }
end

local function split(str, sep)
    -- Lua 5.1 gmatch also matches the empty string after each field, so walk with find
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
            out[k] = v
        end
    end
    return out
end

local function new_companion(id)
    return {
        id         = id,
        name       = '?',
        mjob       = 0,
        mlv        = 0,
        sjob       = 0,
        slv        = 0,
        race       = 0,
        tank       = false,
        kept       = false,
        bond       = '',
        gear       = {}, -- slot -> item id
        stats      = {}, -- key -> number
        attributes = {}, -- key -> { base = n, bonus = n }
        extra      = {}, -- record type -> key/value pairs this addon doesn't use (yet)
    }
end

local function apply(rec, kind, kv)
    if (kind == 'H') then
        rec.name = kv.name or rec.name
        rec.mjob = tonumber(kv.mjob) or rec.mjob
        rec.mlv  = tonumber(kv.mlv) or rec.mlv
        rec.sjob = tonumber(kv.sjob) or rec.sjob
        rec.slv  = tonumber(kv.slv) or rec.slv
        rec.race = tonumber(kv.race) or rec.race
        rec.tank = kv.tank == '1'
        rec.kept = kv.kept == '1'
        rec.bond = kv.bond or ''
    elseif (kind == 'G') then
        for slot, id in pairs(kv) do
            rec.gear[slot] = tonumber(id) or 0
        end
    elseif (kind == 'S') then
        for k, v in pairs(kv) do
            rec.stats[k] = tonumber(v)
        end
    elseif (kind == 'A') then
        for k, v in pairs(kv) do
            local base, bonus = string.match(v, '^(%-?%d+)([%+%-]%d+)$')
            if (base ~= nil) then
                rec.attributes[k] = { base = tonumber(base), bonus = tonumber(bonus) }
            else
                rec.attributes[k] = { base = tonumber(v) or 0, bonus = 0 }
            end
        end
    else
        -- A record type from a newer server: keep it for future use
        rec.extra[kind] = rec.extra[kind] or {}
        for k, v in pairs(kv) do
            rec.extra[kind][k] = v
        end
    end
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

    -- Drop anything the chat log adds around the text (colors, line ends)
    local line = string.sub(message, start)
    line = string.gsub(line, '[%c\128-\255]+.*$', '')
    return line
end

--[[
* Feeds one line into the model. Returns true when a batch has just completed.
--]]
function protocol.feed(model, line)
    local parts = split(line, '|')
    if (parts[1] ~= protocol.VERSION or #parts < 3) then
        return false
    end

    local kind = parts[2]
    local id   = tonumber(parts[3]) or 0
    local kv   = parse_pairs(parts[4])

    if (kind == 'B') then
        model.pending = { companions = {}, order = {}, modes = {} }
        return false
    end

    if (kind == 'M') then
        local target = model.pending or model
        for k, v in pairs(kv) do
            target.modes[k] = v == '1'
        end
        return false
    end

    if (kind == 'Z') then
        if (model.pending ~= nil) then
            model.companions = model.pending.companions
            model.order    = model.pending.order
            model.modes    = model.pending.modes
            model.pending  = nil
            model.updated  = os.time()
            return true
        end
        return false
    end

    -- A record outside a batch (e.g. a single update pushed by the server): update in place
    local target = model.pending or model
    local rec = target.companions[id]
    if (rec == nil) then
        rec = new_companion(id)
        target.companions[id] = rec
        table.insert(target.order, id)
    end

    apply(rec, kind, kv)
    return false
end

return protocol
