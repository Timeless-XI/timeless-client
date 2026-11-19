--[[
* Timeless Journal - lib/protocol.lua (carried over unchanged from the questlog Ashita addon)
*
* Reads the server's quest log feed (modules/questlog). Every line is
*   QL1|<type>|<id>|key=value,...
*
*   B          begin a batch
*   S <log>    a storyline: story, state (active|accept|locked|done|none), code, title, num, of, guess
*   W <log>    k=<step>,t=<where>          (pieces with the same step are joined)
*   H <log>    k=<step>,t=<the full step>  (pieces with the same step are joined)
*   Q <n>      a custom quest: state (active|available), job, name
*   QH <n>     t=<its next step or how to start> (pieces joined)
*   R <n>      a retail quest in progress: area, name
*   C 0        counts: qdone, qtotal, rdone, ractive
*   Z          end: everything in the batch replaces what was shown
*
* Unknown types and keys are ignored. No game or UI code here, so it can be tested on its own.
--]]

local protocol = {};

protocol.VERSION = 'QL1';

local NUMBER_KEYS = { num = true, of = true, qdone = true, qtotal = true, rdone = true, ractive = true, guess = true };

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

local function empty_batch()
    return {
        stories = {},  -- list of { log, story, state, code, title, num, of, guess, steps = { { w, t } } }
        quests  = {},  -- list of { n, name, job, state, text }
        retail  = {},  -- list of { area, name }
        counts  = {},
    };
end

function protocol.new_model()
    local m = empty_batch();
    m.pending  = nil;
    m.loaded   = false;
    m.received = 0;
    return m;
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
    local line = string.sub(message, start);
    line = string.gsub(line, '[%c\128-\255]+.*$', '');
    return line;
end

local function story_of(b, log)
    for _, s in ipairs(b.stories) do
        if (s.log == log) then
            return s;
        end
    end
    return nil;
end

local function step_of(s, k)
    s.steps[k] = s.steps[k] or { w = '', t = '' };
    return s.steps[k];
end

--[[
* Feeds one line. Returns true when a batch has just completed.
--]]
function protocol.feed(model, line, now)
    local parts = split(line, '|', 4);
    if (parts[1] ~= protocol.VERSION or #parts < 3) then
        return false;
    end

    local kind = parts[2];
    local id   = tonumber(parts[3]) or 0;
    local body = parts[4] or '';

    if (kind == 'B') then
        model.pending = empty_batch();
        return false;
    end

    local b = model.pending;
    if (b == nil) then
        return false;
    end

    if (kind == 'S') then
        local kv = parse_pairs(body);
        kv.log   = id;
        kv.guess = (kv.guess == 1);
        kv.steps = {};
        table.insert(b.stories, kv);
    elseif (kind == 'W' or kind == 'H') then
        local k, t = string.match(body, '^k=(%d+),t=(.*)$');
        local s = story_of(b, id);
        if (k ~= nil and s ~= nil) then
            local step = step_of(s, tonumber(k));
            if (kind == 'W') then
                step.w = step.w .. t;
            else
                step.t = step.t .. t;
            end
        end
    elseif (kind == 'Q') then
        local kv = parse_pairs(body);
        kv.n    = id;
        kv.text = '';
        table.insert(b.quests, kv);
    elseif (kind == 'QH') then
        local t = string.match(body, '^t=(.*)$');
        for _, q in ipairs(b.quests) do
            if (q.n == id and t ~= nil) then
                q.text = q.text .. t;
            end
        end
    elseif (kind == 'R') then
        table.insert(b.retail, parse_pairs(body));
    elseif (kind == 'C') then
        b.counts = parse_pairs(body);
    elseif (kind == 'Z') then
        model.stories = b.stories;
        model.quests  = b.quests;
        model.retail  = b.retail;
        model.counts  = b.counts;
        model.pending = nil;
        model.loaded  = true;
        model.received = now or os.time();
        return true;
    end

    return false;
end

-- The steps of a storyline in order (the list may have gaps if a line was lost)
function protocol.steps_of(story)
    local out = {};
    local keys = {};
    for k in pairs(story.steps or {}) do
        table.insert(keys, k);
    end
    table.sort(keys);
    for _, k in ipairs(keys) do
        table.insert(out, story.steps[k]);
    end
    return out;
end

return protocol;
