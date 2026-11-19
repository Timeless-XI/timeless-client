--[[
* Timeless Gauges - lib/protocol.lua (carried over unchanged from the timelessgauges Ashita addon)
*
* Reads the server's gauge feed (modules/gauges). Every line is
*   GA1|<type>|<id>|key=value,...
*
*   J <job id>   n=<count>: the gauges of your main job start over (n = 0: no gauges)
*   G <i>        gauge i: k (key), l (label), t (bar|pips|timer|cd|flag), v, m, h (1 = full or
*                ready), s (status text), c (colour name), tot, left (seconds, counted down here)
*
* Only changed gauges are sent; a timer's seconds count down on this side. Unknown types and keys
* are ignored. No game or UI code here, so it can be tested on its own.
--]]

local protocol = {};

protocol.VERSION = 'GA1';

local NUMBER_KEYS = { n = true, v = true, m = true, h = true, tot = true, left = true };

-- FFXI job ids
protocol.JOBS = {
    [1] = 'Warrior', [2] = 'Monk', [3] = 'White Mage', [4] = 'Black Mage', [5] = 'Red Mage',
    [6] = 'Thief', [7] = 'Paladin', [8] = 'Dark Knight', [9] = 'Beastmaster', [10] = 'Bard',
    [11] = 'Ranger', [12] = 'Samurai', [13] = 'Ninja', [14] = 'Dragoon', [15] = 'Summoner',
    [16] = 'Blue Mage', [17] = 'Corsair', [18] = 'Puppetmaster', [19] = 'Dancer', [20] = 'Scholar',
};

local function split(str, sep, max)
    local out, pos = {}, 1;
    while (true) do
        if (max ~= nil and #out == max - 1) then
            table.insert(out, string.sub(str, pos));
            return out;
        end
        local s = string.find(str, sep, pos, true);
        if (s == nil) then
            table.insert(out, string.sub(str, pos));
            return out;
        end
        table.insert(out, string.sub(str, pos, s - 1));
        pos = s + 1;
    end
end

local function parse_pairs(body)
    local out = {};
    for pair in string.gmatch(body or '', '[^,]+') do
        local k, v = string.match(pair, '^([%w_]+)=(.*)$');
        if (k ~= nil) then
            out[k] = NUMBER_KEYS[k] and (tonumber(v) or 0) or v;
        end
    end
    return out;
end

function protocol.new_model()
    return {
        job      = 0,     -- main job id (0 = nothing yet)
        count    = 0,     -- how many gauges the job has
        gauges   = {},    -- [i] = { k, l, t, v, m, h, s, c, tot, left, at }
        loaded   = false, -- a J line has arrived
        received = 0,     -- clock of the last line
    };
end

--[[
* Finds a feed line in a chat message. Returns the line, or nil if the message isn't one.
--]]
function protocol.extract(message)
    if (type(message) ~= 'string') then
        return nil;
    end
    local start = string.find(message, protocol.VERSION .. '|', 1, true);
    if (start == nil) then
        return nil;
    end
    -- Chat may add colour codes or a line break after the text
    local line = string.sub(message, start);
    line = string.gsub(line, '[%c].*$', '');
    return line;
end

--[[
* Reads one line into the model. now: a clock in seconds (os.clock() in game). Returns true when
* the line was understood.
--]]
function protocol.feed(model, line, now)
    local parts = split(line or '', '|', 4);
    if (parts[1] ~= protocol.VERSION or #parts < 3) then
        return false;
    end

    local kind = parts[2];
    local id   = tonumber(parts[3]) or 0;
    local data = parse_pairs(parts[4]);
    model.received = now or 0;

    if (kind == 'J') then
        model.job    = id;
        model.count  = math.max(0, math.min(8, data.n or 0));
        model.gauges = {};
        model.loaded = true;
        return true;
    elseif (kind == 'G') then
        if (id < 1 or id > 8) then
            return false;
        end
        data.v    = data.v or 0;
        data.m    = math.max(1, data.m or 1);
        data.h    = data.h or 0;
        data.left = data.left or 0;
        data.tot  = math.max(1, data.tot or 1);
        data.at   = now or 0;
        model.gauges[id] = data;
        return true;
    end

    return false;
end

-- Seconds left on a gauge now (timers count down from when the line arrived)
function protocol.left(g, now)
    if (g == nil or (g.left or 0) <= 0) then
        return 0;
    end
    return math.max(0, g.left - ((now or 0) - (g.at or 0)));
end

-- How full to draw a gauge (0..1)
function protocol.fill(g, now)
    if (g == nil) then
        return 0;
    end
    if (g.t == 'timer') then
        return math.min(1, protocol.left(g, now) / g.tot);
    elseif (g.t == 'cd') then
        return 1 - math.min(1, protocol.left(g, now) / g.tot);
    elseif (g.t == 'flag') then
        return (g.v or 0) > 0 and 1 or 0;
    end
    return math.max(0, math.min(1, (g.v or 0) / (g.m or 1)));
end

-- Is it full or ready right now? (a cooldown counting down here turns ready when it reaches 0)
function protocol.ready(g, now)
    if (g == nil) then
        return false;
    end
    if (g.t == 'cd') then
        return protocol.left(g, now) <= 0;
    elseif (g.t == 'timer') then
        return protocol.left(g, now) > 0;
    end
    return (g.h or 0) > 0;
end

-- The text drawn on a gauge
function protocol.text(g, now)
    if (g == nil) then
        return '';
    end
    local s = g.s or '';
    if (g.t == 'timer') then
        local left = protocol.left(g, now);
        if (left <= 0) then
            return '';
        end
        return (s ~= '' and (s .. '  ') or '') .. string.format('%ds', math.ceil(left));
    elseif (g.t == 'cd') then
        local left = protocol.left(g, now);
        if (left <= 0) then
            return 'Ready';
        end
        return string.format('%d:%02d', math.floor(math.ceil(left) / 60), math.ceil(left) % 60);
    elseif (g.t == 'flag') then
        return (g.v or 0) > 0 and (s ~= '' and s or 'On') or '';
    elseif (g.t == 'bar') then
        if (s ~= '') then
            return s;
        end
        return string.format('%d / %d', g.v or 0, g.m or 0);
    end
    -- pips: the status text, plus the window running out
    local left = protocol.left(g, now);
    if (left > 0) then
        return (s ~= '' and (s .. '  ') or '') .. string.format('%ds', math.ceil(left));
    end
    return s;
end

-- The gauges to draw, in order (skips any not received yet)
function protocol.list(model)
    local out = {};
    for i = 1, model.count do
        if (model.gauges[i] ~= nil) then
            table.insert(out, model.gauges[i]);
        end
    end
    return out;
end

function protocol.job_name(model)
    return protocol.JOBS[model.job] or '';
end

return protocol;
