-- Timeless Artisan: all the maths and the server protocol, no game calls (tested offline: lua tests/run.lua).
--
-- Chances follow the server's own synthesis code (src/map/utils/synthutils.cpp):
--   success  per craft: difficulty = recipe level - floor(skill) - craft skill gear
--            >= 4: 80 - 10 x (d - 3)   1..3: 95 - 5 x d   else 95; + SYNTH_SUCCESS_RATE; craft ring +1; max 99
--            every craft of the recipe rolls on its own, so the chances multiply
--   HQ       only if no craft is above your skill and no craft ring blocks it. Tier by the craft closest to its
--            level: 0-10 over 1/64, 11-30 1/16, 31-50 1/4, 51+ 1/2; + 100 x HQ rate / 512; x the HQ multiplier;
--            max 80%. HQ2 / HQ3: one 25% upgrade roll at tier 2, two at 3-4.
--   skill-up per craft (real skill only): not at the rank cap; modern rules: recipe level - floor(skill) > -11
--            (era rules: > 0); chance = d x (3 - ln(1.2 + skill/10)) / 5 when d > 1, else (3 - ln(1.2 + skill/10)) /
--            (6 - d); era rules 60% under 50.0, 25% from 50.0; + SYNTH_SKILL_GAIN %, x the chance multiplier; a break
--            halves it and only counts when the recipe is 1-5 levels above you.
--   Craft Plus (the server's craft_plus module) adds rested synths, lessons, recipe mastery, the guild feast and Mog
--            House furniture for each synth; Flash of Insight is random.
-- Nothing in the server's synthesis code looks at the moon, the day, the weather or the way you face.

local L = {};

L.CRAFTS = {
    { name = 'Woodworking',  short = 'WW', guild = 1 }, { name = 'Smithing',     short = 'SM', guild = 2 },
    { name = 'Goldsmithing', short = 'GS', guild = 3 }, { name = 'Clothcraft',   short = 'CL', guild = 4 },
    { name = 'Leathercraft', short = 'LT', guild = 5 }, { name = 'Bonecraft',    short = 'BN', guild = 6 },
    { name = 'Alchemy',      short = 'AL', guild = 7 }, { name = 'Cooking',      short = 'CK', guild = 8 },
};
L.GUILDS = { [0] = 'Fishing', 'Woodworking', 'Smithing', 'Goldsmithing', 'Clothcraft', 'Leathercraft', 'Bonecraft', 'Alchemy', 'Cooking' };
L.RANKS = { [0] = 'Amateur', 'Recruit', 'Initiate', 'Novice', 'Apprentice', 'Journeyman', 'Craftsman', 'Artisan', 'Adept', 'Veteran', 'Expert' };
L.ELEMENTS = { 'Fire', 'Ice', 'Wind', 'Earth', 'Lightning', 'Water', 'Light', 'Dark' };
L.MAX_QUEUE = 12;
L.SYNTH_GAP = 15;     -- seconds between synths (the synthesis menu's own wait)

-- Crystal id -> element 1..8; clusters 4104-4111 hold 12 crystals of 4096-4103
L.CRYSTAL_SETS = {};
for e = 1, 8 do L.CRYSTAL_SETS[e] = { 4095 + e, 4237 + e, 6505 + e }; end
L.CRYSTAL_ELEMENT = {};
for e, set in ipairs(L.CRYSTAL_SETS) do for _, id in ipairs(set) do L.CRYSTAL_ELEMENT[id] = e; end end
function L.cluster_of(e) return 4103 + e; end
function L.is_cluster(id) return id >= 4104 and id <= 4111; end
function L.is_crystal(id) return L.CRYSTAL_ELEMENT[id] ~= nil; end

-- Storage groups: 0 = crystals and clusters, 1-8 = the craft that uses the material most, 9 = other
L.GROUP_NAMES = { [0] = 'Crystals and Clusters', 'Woodworking', 'Smithing', 'Goldsmithing', 'Clothcraft', 'Leathercraft', 'Bonecraft', 'Alchemy', 'Cooking', [9] = 'Other' };
function L.group_of(idx, id)
    if (L.is_crystal(id) or L.is_cluster(id)) then return 0; end
    return idx.mat_group[id] or 9;
end

-- A typed withdraw amount: digits only, 1..stored (nil = not a number)
function L.parse_amount(text, max)
    local n = tonumber((tostring(text or ''):gsub('[^%d]', '')));
    if (n == nil or n < 1 or max < 1) then return nil; end
    return math.min(n, max);
end

----------------------------------------------------------------------------
-- Recipes (data.lua rows -> tables)
----------------------------------------------------------------------------
-- row: { id, craft, level, crystal, keyitem, {8 levels}, {item, count, ...}, {NQ, q, HQ1, q, HQ2, q, HQ3, q}, name }
function L.index(D)
    local list, by_id, by_result, by_craft, materials, uses = {}, {}, {}, {}, {}, {};
    for i = 1, 8 do by_craft[i] = {}; end
    for _, row in ipairs(D.recipes) do
        local r = { id = row[1], craft = row[2], level = row[3], crystal = row[4], ki = row[5], need = row[6], ing = {}, res = row[8], name = row[9] };
        for j = 1, #row[7], 2 do
            r.ing[#r.ing + 1] = { id = row[7][j], qty = row[7][j + 1] };
            materials[row[7][j]] = true;
            uses[row[7][j]] = uses[row[7][j]] or {};
            table.insert(uses[row[7][j]], r.id);
        end
        r.result, r.qty = r.res[1], r.res[2];
        r.element = L.CRYSTAL_ELEMENT[r.crystal] or 0;
        list[#list + 1] = r;
        by_id[r.id] = r;
        by_result[r.result] = by_result[r.result] or {};
        table.insert(by_result[r.result], r);
        table.insert(by_craft[r.craft], r);
    end
    -- each material's crafts: how many recipes of each craft use it; its group = the craft using it most
    local mat_crafts, mat_group = {}, {};
    for id, rids in pairs(uses) do
        local t = {};
        for _, rid in ipairs(rids) do local c = by_id[rid].craft; t[c] = (t[c] or 0) + 1; end
        local best, n = 0, -1;
        for c = 1, 8 do if ((t[c] or 0) > n) then best, n = c, t[c] or 0; end end
        mat_crafts[id], mat_group[id] = t, best;
    end
    -- materials no enabled recipe uses (Auction House materials categories): grouped by their category
    for _, m in ipairs(D.materials or {}) do
        materials[m[1]] = true;
        if (mat_group[m[1]] == nil and m[2] > 0) then mat_group[m[1]] = m[2]; mat_crafts[m[1]] = {}; end
    end
    for _, set in ipairs(L.CRYSTAL_SETS) do for _, id in ipairs(set) do materials[id] = true; end end
    for e = 1, 8 do materials[L.cluster_of(e)] = true; end
    local by_level = function (a, b) if (a.level ~= b.level) then return a.level < b.level; end return a.name < b.name; end
    for i = 1, 8 do table.sort(by_craft[i], by_level); end
    for _, rs in pairs(by_result) do table.sort(rs, by_level); end
    return { list = list, by_id = by_id, by_result = by_result, by_craft = by_craft, materials = materials, uses = uses, mat_crafts = mat_crafts, mat_group = mat_group };
end

-- The other crafts a recipe needs: { { craft, level }, ... }
function L.subcrafts(r)
    local out = {};
    for i = 1, 8 do
        if (r.need[i] > 0 and i ~= r.craft) then out[#out + 1] = { craft = i, level = r.need[i] }; end
    end
    return out;
end

----------------------------------------------------------------------------
-- State the server sends (defaults until it answers)
----------------------------------------------------------------------------
function L.new_state()
    local st = {
        ready = false, v = { modern = true, cm = 1, hm = 1, max = L.MAX_QUEUE },
        skill = {}, rank = {},
        mods = { sr = 0, hq = 0, sg = 0, ml = 0, c = {}, ah = {}, cl = {} },
        cp = { on = false, r = 0, rm = 50, rg = 15, l = {}, lg = 10, ins = 2, rdy = false, gl = 0, fs = 0, hh = 0, tiers = {} },
        store = {}, store_own = {}, acct = {}, mastery = {}, guilds = {},
        queue = { run = false, n = 0, cur = 0, list = {} },
        pending = {},   -- lists being received (B .. Z)
    };
    for i = 1, 8 do st.skill[i] = 0; st.rank[i] = 0; st.mods.c[i] = 0; st.mods.ah[i] = 0; st.mods.cl[i] = 0; st.cp.l[i] = 0; end
    return st;
end

local function kv(s)
    local t = {};
    for k, v in s:gmatch('([%w_]+)=([^,]*)') do t[k] = v; end
    return t;
end
local function nums(s, sep)
    local t = {};
    for n in (s or ''):gmatch('[^' .. (sep or '.') .. ']+') do t[#t + 1] = tonumber(n) or 0; end
    return t;
end
local function pairs_of(s)   -- "id:q.id:q" -> { [id] = q }
    local t = {};
    for a, b in (s or ''):gmatch('(%d+):(%d+)') do t[tonumber(a)] = (t[tonumber(a)] or 0) + tonumber(b); end
    return t;
end

-- One server line ("TA1|..."). Returns the line's kind (and extra details for events).
function L.parse(st, line)
    local body = line:match('TA1|(.*)$');
    if (body == nil) then return nil; end
    body = body:gsub('[\30\31\127].', ''):gsub('[%z\1-\31]', ''):gsub('%s+$', '');
    local kind, rest = body:match('^(%a)|?(.*)$');
    if (kind == nil) then return nil; end
    if (kind == 'V') then
        local t = kv(rest);
        st.v = { modern = t.modern == '1', cm = tonumber(t.cm) or 1, hm = tonumber(t.hm) or 1, max = tonumber(t.max) or L.MAX_QUEUE, ver = tonumber(t.ver) or 0 };
        st.ready = true;
    elseif (kind == 'K') then
        local i = 0;
        for s, r in rest:gmatch('(%d+):(%d+)') do i = i + 1; st.skill[i] = tonumber(s); st.rank[i] = tonumber(r); end
    elseif (kind == 'M') then
        local t = kv(rest);
        st.mods = { sr = tonumber(t.sr) or 0, hq = tonumber(t.hq) or 0, sg = tonumber(t.sg) or 0, ml = tonumber(t.ml) or 0,
                    c = nums(t.c), ah = nums(t.ah), cl = nums(t.cl) };
        for i = 1, 8 do st.mods.c[i] = st.mods.c[i] or 0; st.mods.ah[i] = st.mods.ah[i] or 0; st.mods.cl[i] = st.mods.cl[i] or 0; end
    elseif (kind == 'P') then
        local t = kv(rest);
        local cp = { on = t.on == '1', r = tonumber(t.r) or 0, rm = tonumber(t.rm) or 50, rg = tonumber(t.rg) or 0, l = nums(t.l), lg = tonumber(t.lg) or 0,
                     ins = tonumber(t.ins) or 0, rdy = t.rdy == '1', gl = tonumber(t.gl) or 0, fs = tonumber(t.fs) or 0, hh = tonumber(t.hh) or 0, tiers = {} };
        for made, hq, loss in (t.t or ''):gmatch('(%d+):(%d+):(%d+)') do
            cp.tiers[#cp.tiers + 1] = { made = tonumber(made), hq = tonumber(hq), loss = tonumber(loss) };
        end
        for i = 1, 8 do cp.l[i] = cp.l[i] or 0; end
        st.cp = cp;
    elseif (kind == 'B') then
        st.pending[rest] = {};
    elseif (kind == 'S' or kind == 'A' or kind == 'T' or kind == 'G') then
        local p = st.pending[({ S = 'st', A = 'ac', T = 'ms', G = 'gd' })[kind]];
        if (p ~= nil) then p[#p + 1] = rest; end
    elseif (kind == 'Z') then
        local p = st.pending[rest] or {};
        st.pending[rest] = nil;
        if (rest == 'st') then
            st.store, st.store_own = {}, {};
            for _, l in ipairs(p) do
                local owner, list = l:match('^(%d+)|(.*)$');
                local into = (owner == '1') and st.store_own or st.store;
                for id, q in pairs(pairs_of(list)) do into[id] = (into[id] or 0) + q; end
            end
            st.store_at = os.time();
        elseif (rest == 'ac') then
            st.acct = {};
            for _, l in ipairs(p) do
                local name, loc, list = l:match('^([^|]+)|(%d+)|(.*)$');
                if (name) then st.acct[#st.acct + 1] = { name = name, loc = tonumber(loc), items = pairs_of(list) }; end
            end
            st.acct_at = os.time();
        elseif (rest == 'ms') then
            st.mastery = {};
            for _, l in ipairs(p) do
                for id, made in pairs(pairs_of(l:match('|(.*)$') or '')) do st.mastery[id] = made; end
            end
        elseif (rest == 'gd') then
            st.guilds = {};
            for _, l in ipairs(p) do
                local g, item, rem, pts, max, con = l:match('^(%d+)|(%d+)|(%d+)|(%d+)|(%d+)|(%d+)');
                if (g) then
                    st.guilds[tonumber(g)] = { item = tonumber(item), remaining = tonumber(rem), points = tonumber(pts), max = tonumber(max), contract = con == '1' };
                end
            end
            st.guilds_at = os.time();
        end
        return 'Z', rest;
    elseif (kind == 'Q') then
        local t = kv(rest);
        local list = {};
        for r, n in (t.list or ''):gmatch('(%d+)%*(%d+)') do list[#list + 1] = { r = tonumber(r), n = tonumber(n) }; end
        st.queue = { run = t.run == '1', n = tonumber(t.n) or 0, cur = tonumber(t.cur) or 0, list = list };
    elseif (kind == 'R') then
        local t = kv(rest);
        return 'R', { recipe = tonumber(t.recipe) or 0, res = tonumber(t.res) or 0, item = tonumber(t.item) or 0, qty = tonumber(t.qty) or 0 };
    elseif (kind == 'D') then
        local t = kv(rest);
        return 'D', { act = t.act, i = tonumber(t.i) or 0, q = tonumber(t.q) or 0, why = t.why or '' };
    elseif (kind == 'E') then
        local k, text = rest:match('^([^|]*)|(.*)$');
        return 'E', { kind = k or '', text = text or rest };
    end
    return kind;
end

-- Split "dp" / "da" requests with many items so each fits a packet (the server reads 124 letters)
L.MAX_REQUEST = 112
function L.chunk_requests(head, parts)
    local out, cur = {}, head;
    for _, p in ipairs(parts) do
        if (#cur + #p + 1 > L.MAX_REQUEST) then out[#out + 1] = cur; cur = head; end
        cur = cur .. ' ' .. p;
    end
    if (cur ~= head or #parts == 0) then out[#out + 1] = cur; end
    return out;
end

----------------------------------------------------------------------------
-- Chances
----------------------------------------------------------------------------
local HQ_BASE = { [1] = 1.5625, [2] = 6.25, [3] = 25, [4] = 50 };

function L.mastery_tier(st, recipe_id)
    local made = st.mastery[recipe_id] or 0;
    local tier = nil;
    for _, t in ipairs(st.cp.tiers or {}) do if (made >= t.made) then tier = t; end end
    return tier, made;
end

-- Can this recipe skill up each craft? (Craft Plus' rested and lessons depend on it)
function L.eligible(st, r)
    local out, any = {}, false;
    local min_diff = st.v.modern and -11 or 0;
    for i = 1, 8 do
        local need = r.need[i];
        if (need > 0) then
            local skill = st.skill[i] or 0;
            local cap = ((st.rank[i] or 0) + 1) * 100;
            local d = need - math.floor(skill / 10);
            if (skill < cap and d > min_diff) then out[i] = true; any = true; end
        end
    end
    return out, any;
end

-- Everything about one recipe for you: { success, hq, hq1, hq2, hq3, can_hq, crafts = { [i] = { diff, success, skillup, why } },
--   skillup (best craft's chance per synth), loss (% lost per material on a break), too_hard, made, tier }
function L.odds(st, r)
    local o = { crafts = {}, success = 1, can_hq = true, too_hard = false };
    local final_tier = 4;
    local mods = st.mods;
    local tier, made = L.mastery_tier(st, r.id);
    o.made, o.tier = made, tier;
    local elig, any = L.eligible(st, r);

    for i = 1, 8 do
        local need = r.need[i];
        if (need > 0) then
            local skill = st.skill[i] or 0;
            local d = need - math.floor(skill / 10) - (mods.c[i] or 0);
            local rate = 95;
            local t = 0;
            if (d >= 4) then rate = 80 - 10 * (d - 3); o.can_hq = false;
            elseif (d >= 1) then rate = 95 - 5 * d; o.can_hq = false;
            elseif (d >= -10) then t = 1;
            elseif (d >= -30) then t = 2;
            elseif (d >= -50) then t = 3;
            else t = 4; end
            if (t < final_tier) then final_tier = t; end
            rate = rate + mods.sr;
            if ((mods.ah[i] or 0) ~= 0) then o.can_hq = false; rate = rate + 1; end
            rate = math.max(0, math.min(99, rate));
            o.success = o.success * rate / 100;
            if (skill < need * 10 - 150) then o.too_hard = true; end
            o.crafts[i] = { diff = d, success = rate / 100, need = need, skill = skill };
        end
    end

    -- HQ
    if (o.can_hq and final_tier >= 1) then
        local hq_rate = mods.hq + (tier and tier.hq or 0) + (st.cp.on and st.cp.hh or 0);
        local chance = (HQ_BASE[final_tier] + 100 * hq_rate / 512) * (st.v.hm or 1);
        chance = math.max(0, math.min(80, chance)) / 100;
        local up = (final_tier == 1) and 0 or ((final_tier == 2) and 1 or 2);
        local p2 = (up >= 1) and 0.25 or 0;
        local p3 = (up >= 2) and 0.25 * 0.25 or 0;
        o.hq_given = chance;
        o.hq = o.success * chance;
        o.hq3 = o.hq * p3;
        o.hq2 = o.hq * (p2 - p3);
        o.hq1 = o.hq - o.hq2 - o.hq3;
        o.hq_tier = final_tier;
    else
        o.hq, o.hq1, o.hq2, o.hq3, o.hq_given, o.hq_tier = 0, 0, 0, 0, 0, 0;
    end

    -- Skill-ups
    local gain = mods.sg;
    if (st.cp.on) then
        if (any and (st.cp.r or 0) > 0) then gain = gain + st.cp.rg; o.rested = true; end
        local lessons = 0;
        for i in pairs(elig) do lessons = math.max(lessons, st.cp.l[i] or 0); end
        gain = gain + lessons * st.cp.lg + st.cp.fs;
        o.lessons = lessons;
    end
    o.skillup = 0;
    for i, c in pairs(o.crafts) do
        local skill = c.skill;
        local cap = ((st.rank[i] or 0) + 1) * 100;
        local d = c.need - math.floor(skill / 10);
        if (skill >= cap) then
            c.skillup, c.why = 0, (skill >= 1000) and 'max' or 'cap';
        elseif (not elig[i]) then
            c.skillup, c.why = 0, 'low';
        else
            local ch;
            if (st.v.modern) then
                local k = 3 - math.log(1.2 + skill / 100);
                ch = (d > 1) and (d * k / 5) or (k / (6 - d));
            else
                ch = (skill < 500) and 0.6 or 0.25;
            end
            ch = (ch + gain / 100) * (st.v.cm or 1);
            ch = math.max(0, math.min(1, ch));
            local on_break = (d > 0 and d <= 5) and ch / 2 or 0;
            c.skillup = o.success * ch + (1 - o.success) * on_break;
            c.on_success = ch;
        end
        if (c.skillup > o.skillup) then o.skillup = c.skillup; end
    end

    -- Material loss on a break (Craft Plus guild rank and mastery included)
    local reduce = mods.ml + (st.cp.on and st.cp.gl or 0) + (tier and tier.loss or 0) + (mods.cl[r.craft] or 0);
    o.loss = math.max(20, math.min(100, 50 - reduce)) / 100;
    return o;
end

----------------------------------------------------------------------------
-- Materials: what you have (inventory now + storage) and what a craft is missing
----------------------------------------------------------------------------
-- have(id) -> inventory count; stored(id) -> storage count (shared + yours)
function L.check(st, r, have, stored, has_ki)
    local out = { missing = {}, ok = true, max = 999 };
    -- crystal: the recipe's, a stored cluster, or another crystal of the element
    local crystals = 0;
    local e = r.element;
    if (e > 0) then
        for _, id in ipairs(L.CRYSTAL_SETS[e]) do crystals = crystals + have(id) + stored(id); end
        crystals = crystals + 12 * stored(L.cluster_of(e));
    end
    out.crystals = crystals;
    if (crystals <= 0) then out.ok = false; out.missing[#out.missing + 1] = { id = r.crystal, need = 1, have = 0 }; end
    out.max = math.min(out.max, crystals);
    for _, g in ipairs(r.ing) do
        local n = have(g.id) + stored(g.id);
        if (n < g.qty) then out.ok = false; out.missing[#out.missing + 1] = { id = g.id, need = g.qty, have = n }; end
        out.max = math.min(out.max, math.floor(n / g.qty));
    end
    if (r.ki ~= 0 and has_ki and not has_ki(r.ki)) then out.ok = false; out.no_ki = true; end
    if (not out.ok) then out.max = 0; end
    return out;
end

----------------------------------------------------------------------------
-- Guild point items
----------------------------------------------------------------------------
-- What today's item means for you: items still worth points, crafts to make them
function L.guild_plan(st, idx, guild, have, stored)
    local g = st.guilds[guild];
    if (g == nil) then return nil; end
    local p = { item = g.item, points = g.points, max = g.max, remaining = g.remaining, contract = g.contract };
    p.items = (g.points > 0) and math.ceil(g.remaining / g.points) or 0;
    p.full_day = (g.points > 0) and math.ceil(g.max / g.points) or 0;
    p.own = have(g.item) + stored(g.item);
    p.to_make = math.max(0, p.items - p.own);
    p.recipes = idx.by_result[g.item] or {};
    -- the easiest recipe that makes it
    local best = nil;
    for _, r in ipairs(p.recipes) do
        if (best == nil or r.level < best.level) then best = r; end
    end
    p.recipe = best;
    if (best) then
        p.crafts = math.ceil(p.to_make / math.max(1, best.qty));
        p.queue = math.min(p.crafts, L.MAX_QUEUE);
    else
        p.crafts, p.queue = 0, 0;
    end
    return p;
end

----------------------------------------------------------------------------
-- Formatting
----------------------------------------------------------------------------
function L.pct(x)
    if (x == nil) then return '-'; end
    if (x > 0 and x < 0.005) then return '<1%'; end
    if (x > 0.995 and x < 1) then return '>99%'; end
    return ('%d%%'):format(math.floor(x * 100 + 0.5));
end
function L.skill_text(s) return ('%.1f'):format((s or 0) / 10); end

L.BAGS = { [0] = 'Inventory', 'Mog Safe', 'Storage', 'Temporary', 'Mog Locker', 'Mog Satchel', 'Mog Sack', 'Mog Case',
           'Wardrobe', 'Mog Safe 2', 'Wardrobe 2', 'Wardrobe 3', 'Wardrobe 4', 'Wardrobe 5', 'Wardrobe 6', 'Wardrobe 7', 'Wardrobe 8' };

-- Deposit bag mask: bit = container id
function L.bag_mask(inv, satchel, sack, case, home)
    local m = 0;
    if (inv) then m = m + 1; end
    if (satchel) then m = m + 32; end
    if (sack) then m = m + 64; end
    if (case) then m = m + 128; end
    if (home) then m = m + 2 + 4 + 16 + 512; end
    return m;
end

L.REASON_TEXT = {
    missing = 'Missing materials', inventory_full = 'Inventory full', too_hard = 'Too difficult', key_item = 'Needs a key item',
    rare = 'Rare: you already have one', crafting = 'Busy crafting', wait = 'Waiting', busy = 'Busy',
};

----------------------------------------------------------------------------
-- Auto-deposit with "kept" items
-- Withdrawing an item marks that many as kept: auto-deposit leaves them alone and only stores
-- what you have beyond them. The kept number shrinks as you use, sell or trade the items (it never
-- exceeds what you still carry) and is gone once you carry none, or when you deposit the item by hand.
----------------------------------------------------------------------------
-- kept: { [id] = n }; carried(id): count in the inventory + satchel / sack / case.
-- skip(id): true while a withdrawal of it is still landing (don't shrink on a stale count).
function L.kept_clamp(kept, carried, skip)
    for id, n in pairs(kept) do
        if (not (skip and skip(id))) then
            local c = carried(id);
            if (c <= 0) then kept[id] = nil;
            elseif (c < n) then kept[id] = c; end
        end
    end
end

-- What auto-deposit should store: { { id, qty } } for each candidate, never touching kept items.
-- auto_count(id): count in the bags auto-deposit takes from; carried(id) as above.
function L.auto_plan(candidates, kept, carried, auto_count)
    local out = {};
    for _, id in ipairs(candidates) do
        local excess = carried(id) - (kept[id] or 0);
        local qty = math.min(auto_count(id), excess);
        if (qty > 0) then out[#out + 1] = { id = id, qty = qty }; end
    end
    return out;
end

return L;
