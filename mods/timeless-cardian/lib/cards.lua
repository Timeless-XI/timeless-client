-----------------------------------
-- Cardian: the cards. One list read by the server (drops, the game) and copied as-is into the timeless-cardian
-- client mod (lib/cards.lua), so both sides agree on every number.
--
-- A card: id (1-99 commons, 101-199 rares, 201-299 legendaries), key, name, sides { top, right, bottom, left }
-- (1-10; 10 shows as A), el (element), level (1-10: 1-5 field monsters, 6-7 notorious monsters, 8-10 legends),
-- fam (family: the world's bonuses read it), species (shown), rarity (1 common, 2 rare, 3 legendary), model (the
-- monster's model id, for the client's live 3D portrait; nil for townsfolk, who get a drawn picture), theme (the
-- card's header, after the shop windows), say (its voice).
-- Drops: family = an xi.mobFamily name (field monsters drop commons), nm = a monster's name (rares, legendaries).
--
-- Foils (Champion kills): id + 1000 x variant: 1 Gilded, 2 Nightborne, 3 Rift-touched, 4 Juggernaut.
-----------------------------------
local cards = {}

cards.ELEMENTS = { 'fire', 'ice', 'wind', 'earth', 'thunder', 'water', 'light', 'dark' }

cards.VARIANTS =
{
    [1] = { key = 'gilded', name = 'Gilded', text = 'Its lowest side +1.' },
    [2] = { key = 'nightborne', name = 'Nightborne', text = 'Every side +1 from 20:00 to 6:00.' },
    [3] = { key = 'rift', name = 'Rift-touched', text = 'Elemental squares, the day, conquest and the moon don\'t touch it.' },
    [4] = { key = 'juggernaut', name = 'Juggernaut', text = 'Its highest side +2 (to A at most).' },
}

-- Ten commons for anyone new to the game, and the hand of five they start with
cards.STARTER = { 11, 11, 13, 13, 14, 14, 16, 8, 8, 2 }
cards.STARTER_HAND = { 11, 13, 14, 16, 2 }

cards.list =
{
    -- Commons: field monsters. Sides: top, right, bottom, left.
    { id = 1, key = 'quadav', name = 'Quadav Helmsman', sides = { 6, 4, 5, 4 }, el = 'earth', level = 4, fam = 'Beastman', species = 'Quadav', rarity = 1, model = 652, theme = 'quadav', family = 'QUADAV', say = 'Hrrn. Shell holds.' },
    { id = 2, key = 'goblin', name = 'Goblin Tinkerer', sides = { 4, 2, 5, 4 }, el = 'fire', level = 2, fam = 'Beastman', species = 'Goblin', rarity = 1, model = 484, theme = 'goblin', family = 'GOBLIN', say = 'Hee hee! Cousin fix it!' },
    { id = 3, key = 'orc', name = 'Orcish Grunt', sides = { 6, 4, 3, 4 }, el = 'fire', level = 3, fam = 'Beastman', species = 'Orc', rarity = 1, model = 629, theme = 'orc', family = 'ORC', say = 'Grah! Smash the soft ones!' },
    { id = 4, key = 'yagudo', name = 'Yagudo Acolyte', sides = { 3, 5, 5, 4 }, el = 'wind', level = 3, fam = 'Beastman', species = 'Yagudo', rarity = 1, model = 587, theme = 'yagudo', family = 'YAGUDO', say = 'The Tzee Xicu watches, ho.' },
    { id = 5, key = 'bomb', name = 'Bomb', sides = { 9, 4, 2, 6 }, el = 'fire', level = 5, fam = 'Arcana', species = 'Bomb', rarity = 1, model = 280, theme = 'ember', family = 'BOMB', say = '*fizz* ...*FIZZ*' },
    { id = 6, key = 'skeleton', name = 'Skeleton Warrior', sides = { 5, 3, 6, 3 }, el = 'dark', level = 3, fam = 'Undead', species = 'Skeleton', rarity = 1, model = 572, theme = 'grave', family = 'SKELETON', say = '...the lamps are out...' },
    { id = 7, key = 'ghost', name = 'Ghost', sides = { 3, 6, 2, 4 }, el = 'dark', level = 2, fam = 'Undead', species = 'Ghost', rarity = 1, model = 368, theme = 'grave', family = 'GHOST', say = 'Ooooo... ooo?' },
    { id = 8, key = 'bat', name = 'Bat', sides = { 2, 4, 2, 5 }, el = 'wind', level = 1, fam = 'Beast', species = 'Bat', rarity = 1, model = 256, theme = 'wild', family = 'BAT', say = '*squeak squeak*' },
    { id = 9, key = 'worm', name = 'Tunnel Worm', sides = { 4, 4, 6, 3 }, el = 'earth', level = 3, fam = 'Vermin', species = 'Worm', rarity = 1, model = 424, theme = 'wild', family = 'WORM', say = '*the ground rumbles*' },
    { id = 10, key = 'crab', name = 'Land Crab', sides = { 2, 5, 5, 3 }, el = 'water', level = 2, fam = 'Aquan', species = 'Crab', rarity = 1, model = 356, theme = 'sea', family = 'CRAB', say = '*click click*' },
    { id = 11, key = 'rabbit', name = 'Wild Rabbit', sides = { 3, 1, 4, 5 }, el = 'wind', level = 1, fam = 'Beast', species = 'Rabbit', rarity = 1, model = 268, theme = 'wild', family = 'RABBIT', say = '*twitch*' },
    { id = 12, key = 'sheep', name = 'Ram', sides = { 5, 6, 3, 5 }, el = 'ice', level = 4, fam = 'Beast', species = 'Sheep', rarity = 1, model = 340, theme = 'wild', family = 'SHEEP', say = 'Baaaa.' },
    { id = 13, key = 'bee', name = 'Bee', sides = { 4, 3, 1, 5 }, el = 'wind', level = 1, fam = 'Vermin', species = 'Bee', rarity = 1, theme = 'wild', family = 'BEE', say = '*bzzzz*' },
    { id = 14, key = 'mandragora', name = 'Mandragora', sides = { 1, 5, 4, 3 }, el = 'earth', level = 1, fam = 'Plantoid', species = 'Mandragora', rarity = 1, model = 300, theme = 'wild', family = 'MANDRAGORA', say = '*shrieks*' },
    { id = 15, key = 'lizard', name = 'Hill Lizard', sides = { 6, 3, 5, 3 }, el = 'earth', level = 3, fam = 'Lizard', species = 'Lizard', rarity = 1, model = 328, theme = 'wild', family = 'LIZARD', say = '*basks*' },
    { id = 16, key = 'crawler', name = 'Crawler', sides = { 2, 4, 5, 2 }, el = 'earth', level = 1, fam = 'Vermin', species = 'Crawler', rarity = 1, model = 396, theme = 'wild', family = 'CRAWLER', say = '*munch munch*' },
    { id = 17, key = 'tiger', name = 'Forest Tiger', sides = { 6, 7, 3, 5 }, el = 'ice', level = 5, fam = 'Beast', species = 'Tiger', rarity = 1, model = 308, theme = 'wild', family = 'TIGER', say = '*low growl*' },
    { id = 18, key = 'leech', name = 'Leech', sides = { 5, 2, 4, 6 }, el = 'water', level = 3, fam = 'Amorph', species = 'Leech', rarity = 1, model = 276, theme = 'sea', family = 'LEECH', say = '*slurp*' },
    { id = 19, key = 'beetle', name = 'Beetle', sides = { 7, 3, 6, 3 }, el = 'earth', level = 4, fam = 'Vermin', species = 'Beetle', rarity = 1, model = 408, theme = 'wild', family = 'BEETLE', say = '*clack*' },
    { id = 20, key = 'coeurl', name = 'Coeurl', sides = { 5, 6, 6, 4 }, el = 'thunder', level = 5, fam = 'Beast', species = 'Coeurl', rarity = 1, model = 367, theme = 'wild', family = 'COEURL', say = '*whiskers crackle*' },
    { id = 21, key = 'pugil', name = 'Pugil', sides = { 4, 3, 4, 4 }, el = 'water', level = 2, fam = 'Aquan', species = 'Pugil', rarity = 1, model = 348, theme = 'sea', family = 'PUGIL', say = '*splash*' },
    { id = 22, key = 'dhalmel', name = 'Dhalmel', sides = { 4, 4, 7, 4 }, el = 'wind', level = 4, fam = 'Beast', species = 'Dhalmel', rarity = 1, model = 332, theme = 'wild', family = 'DHALMEL', say = '*snorts*' },
    { id = 23, key = 'funguar', name = 'Funguar', sides = { 5, 1, 5, 4 }, el = 'dark', level = 2, fam = 'Plantoid', species = 'Funguar', rarity = 1, model = 376, theme = 'grave', family = 'FUNGUAR', say = '*puff*' },
    { id = 24, key = 'gigas', name = 'Gigas', sides = { 7, 7, 2, 6 }, el = 'ice', level = 5, fam = 'Beastman', species = 'Gigas', rarity = 1, model = 707, theme = 'orc', family = 'GIGAS', say = 'GRAAAH.' },
    { id = 25, key = 'sahagin', name = 'Sahagin', sides = { 5, 5, 4, 5 }, el = 'water', level = 4, fam = 'Beastman', species = 'Sahagin', rarity = 1, theme = 'sea', family = 'SAHAGIN', say = 'Gloop! The tide rises!' },
    { id = 26, key = 'tonberry', name = 'Tonberry', sides = { 8, 2, 8, 3 }, el = 'dark', level = 5, fam = 'Beastman', species = 'Tonberry', rarity = 1, model = 1205, theme = 'tonberry', family = 'TONBERRY', say = '...' },

    -- Rares: notorious monsters
    { id = 101, key = 'lizzy', name = 'Leaping Lizzy', sides = { 7, 6, 4, 6 }, el = 'earth', level = 6, fam = 'Lizard', species = 'Notorious Monster', rarity = 2, model = 328, theme = 'guild', nm = 'Leaping_Lizzy', say = '*hiss* (it eyes your rations)' },
    { id = 102, key = 'emperor', name = 'Valkurm Emperor', sides = { 6, 7, 7, 3 }, el = 'wind', level = 6, fam = 'Vermin', species = 'Notorious Monster', rarity = 2, model = 448, theme = 'guild', nm = 'Valkurm_Emperor', say = '*BZZZZZT*' },
    { id = 103, key = 'billy', name = 'Bigmouth Billy', sides = { 8, 4, 7, 4 }, el = 'earth', level = 6, fam = 'Vermin', species = 'Notorious Monster', rarity = 2, model = 424, theme = 'guild', nm = 'Bigmouth_Billy', say = '*GULP*' },
    { id = 104, key = 'swamfisk', name = 'Swamfisk', sides = { 5, 8, 4, 6 }, el = 'water', level = 6, fam = 'Aquan', species = 'Notorious Monster', rarity = 2, model = 348, theme = 'guild', nm = 'Swamfisk', say = '*SPLOOSH*' },
    { id = 105, key = 'goldenbat', name = 'Golden Bat', sides = { 4, 8, 3, 8 }, el = 'wind', level = 6, fam = 'Beast', species = 'Notorious Monster', rarity = 2, model = 256, theme = 'guild', nm = 'Golden_Bat', say = '*glittering squeak*' },
    { id = 106, key = 'morbolger', name = 'Morbolger', sides = { 7, 7, 4, 7 }, el = 'earth', level = 7, fam = 'Plantoid', species = 'Notorious Monster', rarity = 2, model = 380, theme = 'guild', nm = 'Morbolger', say = '*HUAAAH*' },
    { id = 107, key = 'meedeggi', name = 'Mee Deggi the Punisher', sides = { 8, 5, 7, 5 }, el = 'wind', level = 7, fam = 'Beastman', species = 'Notorious Monster', rarity = 2, model = 580, theme = 'guild', nm = 'Mee_Deggi_the_Punisher', say = 'Kneel, heretic, ho!' },
    { id = 108, key = 'profblix', name = 'Mysticmaker Profblix', sides = { 5, 8, 5, 7 }, el = 'fire', level = 7, fam = 'Beastman', species = 'Notorious Monster', rarity = 2, model = 507, theme = 'guild', nm = 'Mysticmaker_Profblix', say = 'Profblix make it better! Hee!' },
    { id = 109, key = 'roc', name = 'Roc', sides = { 9, 7, 3, 6 }, el = 'wind', level = 7, fam = 'Bird', species = 'Notorious Monster', rarity = 2, model = 336, theme = 'guild', nm = 'Roc', say = '*SKREEEE*' },
    { id = 110, key = 'serket', name = 'Serket', sides = { 8, 4, 8, 5 }, el = 'earth', level = 7, fam = 'Vermin', species = 'Notorious Monster', rarity = 2, model = 284, theme = 'guild', nm = 'Serket', say = '*clack clack clack*' },

    -- Legendaries: a great wyrm, and the card players' own cards, won by beating them
    { id = 201, key = 'fafnir', name = 'Fafnir', sides = { 10, 8, 6, 8 }, el = 'dark', level = 10, fam = 'Dragon', species = 'Notorious Monster', rarity = 3, model = 783, theme = 'dragon', nm = 'Fafnir', say = '*the air itself trembles*' },
    { id = 202, key = 'toma', name = 'Old Toma, Lamplighter', sides = { 6, 8, 8, 6 }, el = 'light', level = 8, fam = 'Townsfolk', species = 'Hume', rarity = 3, theme = 'hume', say = 'Twenty lamps. Every night.' },
    { id = 203, key = 'hrodgar', name = 'Hrodgar, Card Shark', sides = { 9, 6, 9, 6 }, el = 'fire', level = 9, fam = 'Townsfolk', species = 'Galka', rarity = 3, theme = 'galka', say = 'I\'ll pay you on payday. Deal.' },
    { id = 204, key = 'dagny', name = 'Dagny by Candlelight', sides = { 8, 6, 5, 9 }, el = 'light', level = 8, fam = 'Townsfolk', species = 'Hume', rarity = 3, theme = 'hume', say = 'I\'ve seen that card before.' },
    { id = 205, key = 'fenwick', name = 'Steady Fenwick', sides = { 7, 9, 6, 8 }, el = 'earth', level = 9, fam = 'Townsfolk', species = 'Elvaan', rarity = 3, theme = 'elvaan', say = 'Patience wins hands.' },
    { id = 206, key = 'kipiko', name = 'Kipiko the Trickster', sides = { 5, 9, 9, 5 }, el = 'thunder', level = 8, fam = 'Townsfolk', species = 'Tarutaru', rarity = 3, theme = 'tarutaru', say = 'Hee! Now you see it...' },
    { id = 207, key = 'shaali', name = 'Shaali Nine-Lives', sides = { 9, 8, 4, 9 }, el = 'thunder', level = 9, fam = 'Townsfolk', species = 'Mithra', rarity = 3, theme = 'mithra', say = 'Nine lives, nine aces. Your deal.' },
}

cards.byId = {}
cards.byKey = {}
cards.byFamily = {}
cards.byNm = {}
for _, c in ipairs(cards.list) do
    cards.byId[c.id] = c
    cards.byKey[c.key] = c
    if c.family then
        cards.byFamily[c.family] = c
    end

    if c.nm then
        cards.byNm[c.nm] = c
    end
end

-- A card id with its foil: the base card, and the variant (nil for a plain card)
cards.split = function(id)
    id = tonumber(id) or 0
    local v = math.floor(id / 1000)
    return id - v * 1000, v > 0 and v or nil
end

-- The card for an id (a foil is its base card with the variant set)
cards.get = function(id)
    local base, v = cards.split(id)
    local c = cards.byId[base]
    if c == nil then
        return nil
    end

    if v == nil or cards.VARIANTS[v] == nil then
        return v == nil and c or nil
    end

    local f = {}
    for k, x in pairs(c) do
        f[k] = x
    end

    f.id, f.base, f.variant = id, base, v
    f.name = cards.VARIANTS[v].name .. ' ' .. c.name
    return f
end

-- The card's printed sides, a foil's change included (Nightborne's comes from the hour: not printed)
cards.printed = function(id)
    local c = cards.get(id)
    if c == nil then
        return { 1, 1, 1, 1 }
    end

    local s = { c.sides[1], c.sides[2], c.sides[3], c.sides[4] }
    if c.variant == 1 or c.variant == 4 then
        local pick = 1
        for i = 2, 4 do
            if (c.variant == 1 and s[i] < s[pick]) or (c.variant == 4 and s[i] > s[pick]) then
                pick = i
            end
        end

        s[pick] = math.min(10, s[pick] + (c.variant == 1 and 1 or 2))
    end

    return s
end

-- A side as the card shows it: 1-9, A for 10
cards.label = function(n)
    return n >= 10 and 'A' or tostring(n)
end

return cards
