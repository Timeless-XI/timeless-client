-----------------------------------
-- Ironhand: the cards. One list read by the server (drops, the game) and copied as-is into the
-- timeless-ironhand client mod (lib/cards.lua), so both sides agree on every number.
--
-- A card: id (1-99 commons, 101-199 rares, 201-299 legendaries), key, name, cost (crystals), power,
-- el (element), fam (family, as abilities and the world see it), species (shown), rarity (1 common,
-- 2 rare, 3 legendary), model (the monster's model id, for the client's live 3D portrait; nil for
-- townsfolk, who get a drawn picture), theme (the card's header, after the shop windows), ab (its
-- abilities: key = value), text (what the card says, in the game's gear wording), say (its voice).
-- Drops: family = an xi.mobFamily name (field monsters drop commons), nm = a monster's name (rares).
--
-- Foils (champion kills): id + 1000 x variant: 1 Gilded, 2 Nightborne, 3 Rift-touched, 4 Juggernaut.
-----------------------------------
local cards = {}

cards.ELEMENTS = { 'fire', 'ice', 'wind', 'earth', 'thunder', 'water', 'light', 'dark' }
-- What each element beats (Vana'diel's wheel; light and dark beat each other)
cards.BEATS = { fire = 'ice', ice = 'wind', wind = 'earth', earth = 'thunder', thunder = 'water', water = 'fire', light = 'dark', dark = 'light' }

cards.VARIANTS =
{
    [1] = { key = 'gilded', name = 'Gilded', text = 'Power+1. Wins ties in its front.' },
    [2] = { key = 'nightborne', name = 'Nightborne', text = 'Power+1 at night. Its night lasts until 6:00.' },
    [3] = { key = 'rift', name = 'Rift-touched', text = 'Power+1. The front\'s weather and conquest don\'t touch it.' },
    [4] = { key = 'juggernaut', name = 'Juggernaut', text = 'Power+2. Costs 1 more.' },
}

cards.list =
{
    -- Commons: field monsters
    { id = 1, key = 'quadav', name = 'Quadav Helmsman', cost = 2, power = 3, el = 'earth', fam = 'Beastman', species = 'Quadav', rarity = 1, model = 652, theme = 'quadav', family = 'QUADAV',
      ab = { shellwall = 2, home = 'GUSTABERG' }, text = { { 'Shell Wall', 'Power+2 while another Beastman stands in this front.' }, { 'Latent effect', 'Power+1 in Gustaberg while the beastmen hold it.' } }, say = 'Hrrn. Shell hold. You no pass.' },
    { id = 2, key = 'goblin', name = 'Goblin Tinkerer', cost = 1, power = 1, el = 'fire', fam = 'Beastman', species = 'Goblin', rarity = 1, model = 484, theme = 'goblin', family = 'GOBLIN',
      ab = { tinker = 2 }, text = { { 'Tinker', 'When played: your weakest other card in this front gets Power+2.' } }, say = 'Hee hee! Cousin fix it!' },
    { id = 3, key = 'orc', name = 'Orcish Grunt', cost = 2, power = 2, el = 'fire', fam = 'Beastman', species = 'Orc', rarity = 1, model = 629, theme = 'orc', family = 'ORC',
      ab = { rally = 1 }, text = { { 'War Cry', 'When played: your other Beastmen in this front get Power+1.' } }, say = 'Grah! Smash the soft ones!' },
    { id = 4, key = 'yagudo', name = 'Yagudo Acolyte', cost = 2, power = 2, el = 'wind', fam = 'Beastman', species = 'Yagudo', rarity = 1, model = 587, theme = 'yagudo', family = 'YAGUDO',
      ab = { devout = 1 }, text = { { 'Devotion', 'Power+1 for each other Yagudo of yours in this front.' } }, say = 'The Tzee Xicu watches, ho.' },
    { id = 5, key = 'bomb', name = 'Bomb', cost = 4, power = 5, el = 'fire', fam = 'Arcana', species = 'Bomb', rarity = 1, model = 280, theme = 'ember', family = 'BOMB',
      ab = { selfdestruct = true }, text = { { 'Self-Destruct', 'End of round 5: destroys itself and the strongest enemy card in this front.' } }, say = '*fizz* ...*FIZZ*' },
    { id = 6, key = 'skeleton', name = 'Skeleton Warrior', cost = 2, power = 2, el = 'dark', fam = 'Undead', species = 'Skeleton', rarity = 1, model = 572, theme = 'grave', family = 'SKELETON',
      ab = { night = 3 }, text = { { 'Latent effect', 'Power+3 from 20:00 to 4:00.' } }, say = '...the lamps are out...' },
    { id = 7, key = 'ghost', name = 'Ghost', cost = 2, power = 1, el = 'dark', fam = 'Undead', species = 'Ghost', rarity = 1, model = 368, theme = 'grave', family = 'GHOST',
      ab = { night = 2, haunt = 1 }, text = { { 'Haunt', 'When played: the strongest enemy card in this front gets Power-1.' }, { 'Latent effect', 'Power+2 at night.' } }, say = 'Ooooo... ooo?' },
    { id = 8, key = 'bat', name = 'Bat', cost = 1, power = 1, el = 'wind', fam = 'Beast', species = 'Bat', rarity = 1, model = 256, theme = 'wild', family = 'BAT',
      ab = { swarm = 1 }, text = { { 'Swarm', 'Power+1 for each other Bat of yours on the table.' } }, say = '*squeak squeak*' },
    { id = 9, key = 'worm', name = 'Tunnel Worm', cost = 2, power = 3, el = 'earth', fam = 'Vermin', species = 'Worm', rarity = 1, model = 424, theme = 'wild', family = 'WORM',
      ab = { immune = true }, text = { { 'Burrow', 'Enemy abilities can\'t weaken or destroy it.' } }, say = '*the ground rumbles*' },
    { id = 10, key = 'crab', name = 'Land Crab', cost = 2, power = 2, el = 'water', fam = 'Aquan', species = 'Crab', rarity = 1, model = 356, theme = 'sea', family = 'CRAB',
      ab = { rain = 2 }, text = { { 'Latent effect', 'Power+2 in rain or squalls.' } }, say = '*click click*' },
    { id = 11, key = 'rabbit', name = 'Wild Rabbit', cost = 1, power = 1, el = 'wind', fam = 'Beast', species = 'Rabbit', rarity = 1, model = 268, theme = 'wild', family = 'RABBIT',
      ab = { quick = 1 }, text = { { 'Quick', 'When played: draw a card.' } }, say = '*twitch*' },
    { id = 12, key = 'sheep', name = 'Ram', cost = 2, power = 3, el = 'ice', fam = 'Beast', species = 'Sheep', rarity = 1, model = 340, theme = 'wild', family = 'SHEEP',
      ab = {}, text = { { 'Stubborn', 'No tricks. Just horns.' } }, say = 'Baaaa.' },
    { id = 13, key = 'bee', name = 'Bee', cost = 1, power = 1, el = 'wind', fam = 'Vermin', species = 'Bee', rarity = 1, theme = 'wild', family = 'BEE',
      ab = { sting = 1 }, text = { { 'Sting', 'When played: a random enemy card in this front gets Power-1.' } }, say = '*bzzzz*' },
    { id = 14, key = 'mandragora', name = 'Mandragora', cost = 1, power = 2, el = 'earth', fam = 'Plantoid', species = 'Mandragora', rarity = 1, model = 300, theme = 'wild', family = 'MANDRAGORA',
      ab = {}, text = { { 'Rooted', 'Small, stubborn and surprisingly loud.' } }, say = '*shrieks*' },
    { id = 15, key = 'lizard', name = 'Hill Lizard', cost = 2, power = 2, el = 'earth', fam = 'Lizard', species = 'Lizard', rarity = 1, model = 328, theme = 'wild', family = 'LIZARD',
      ab = { sun = 2 }, text = { { 'Latent effect', 'Power+2 in a hot spell or heat wave.' } }, say = '*basks*' },
    { id = 16, key = 'crawler', name = 'Crawler', cost = 1, power = 1, el = 'earth', fam = 'Vermin', species = 'Crawler', rarity = 1, model = 396, theme = 'wild', family = 'CRAWLER',
      ab = { grow = 3 }, text = { { 'Cocoon', 'Power+1 at the start of each round it\'s on the table, up to +3.' } }, say = '*munch munch*' },
    { id = 17, key = 'tiger', name = 'Forest Tiger', cost = 3, power = 4, el = 'ice', fam = 'Beast', species = 'Tiger', rarity = 1, model = 308, theme = 'wild', family = 'TIGER',
      ab = { prey = 2 }, text = { { 'Hunt', 'Power+2 while an enemy Rabbit, Ram or Dhalmel is in this front.' } }, say = '*low growl*' },
    { id = 18, key = 'leech', name = 'Leech', cost = 2, power = 2, el = 'water', fam = 'Amorph', species = 'Leech', rarity = 1, model = 276, theme = 'sea', family = 'LEECH',
      ab = { drain = 1 }, text = { { 'Drain', 'When played: the strongest enemy card in this front gets Power-1, and this Power+1.' } }, say = '*slurp*' },
    { id = 19, key = 'beetle', name = 'Beetle', cost = 3, power = 3, el = 'earth', fam = 'Vermin', species = 'Beetle', rarity = 1, model = 408, theme = 'wild', family = 'BEETLE',
      ab = { harden = 1 }, text = { { 'Harden', 'Power+1 for each other card of yours in this front.' } }, say = '*clack*' },
    { id = 20, key = 'coeurl', name = 'Coeurl', cost = 3, power = 4, el = 'thunder', fam = 'Beast', species = 'Coeurl', rarity = 1, model = 367, theme = 'wild', family = 'COEURL',
      ab = { stun = true }, text = { { 'Petrifying Gaze', 'When played: the strongest enemy card in this front loses its abilities.' } }, say = '*whiskers crackle*' },
    { id = 21, key = 'pugil', name = 'Pugil', cost = 2, power = 2, el = 'water', fam = 'Aquan', species = 'Pugil', rarity = 1, model = 348, theme = 'sea', family = 'PUGIL',
      ab = { rain = 2 }, text = { { 'Latent effect', 'Power+2 in rain or squalls.' } }, say = '*splash*' },
    { id = 22, key = 'dhalmel', name = 'Dhalmel', cost = 2, power = 3, el = 'wind', fam = 'Beast', species = 'Dhalmel', rarity = 1, model = 332, theme = 'wild', family = 'DHALMEL',
      ab = {}, text = { { 'Long Stride', 'Plain, tall and sturdy.' } }, say = '*snorts*' },
    { id = 23, key = 'funguar', name = 'Funguar', cost = 2, power = 1, el = 'dark', fam = 'Plantoid', species = 'Funguar', rarity = 1, model = 376, theme = 'grave', family = 'FUNGUAR',
      ab = { night = 2 }, text = { { 'Latent effect', 'Power+2 at night.' } }, say = '*puff*' },
    { id = 24, key = 'gigas', name = 'Gigas', cost = 5, power = 7, el = 'ice', fam = 'Beastman', species = 'Gigas', rarity = 1, model = 707, theme = 'orc', family = 'GIGAS',
      ab = {}, text = { { 'Giant', 'Big. Slow. Big.' } }, say = 'GRAAAH.' },
    { id = 25, key = 'sahagin', name = 'Sahagin', cost = 3, power = 3, el = 'water', fam = 'Beastman', species = 'Sahagin', rarity = 1, theme = 'sea', family = 'SAHAGIN',
      ab = { rain = 2 }, text = { { 'Latent effect', 'Power+2 in rain or squalls.' } }, say = 'Gloop! The tide rises!' },
    { id = 26, key = 'tonberry', name = 'Tonberry', cost = 3, power = 2, el = 'dark', fam = 'Beastman', species = 'Tonberry', rarity = 1, model = 1205, theme = 'tonberry', family = 'TONBERRY',
      ab = { grudge = 1 }, text = { { 'Everyone\'s Grudge', 'Power+1 for each of your cards destroyed this game.' } }, say = '...' },

    -- Rares: notorious monsters
    { id = 101, key = 'lizzy', name = 'Leaping Lizzy', cost = 3, power = 4, el = 'earth', fam = 'Lizard', species = 'Notorious Monster', rarity = 2, model = 328, theme = 'guild', nm = 'Leaping_Lizzy',
      ab = { leap = true }, text = { { 'Leap', 'Can be played to a full front. When played: knocks out the weakest enemy card in this front.' } }, say = '*hiss* (it eyes your rations)' },
    { id = 102, key = 'emperor', name = 'Valkurm Emperor', cost = 3, power = 3, el = 'wind', fam = 'Vermin', species = 'Notorious Monster', rarity = 2, model = 448, theme = 'guild', nm = 'Valkurm_Emperor',
      ab = { lord = 1 }, text = { { 'Swarm Lord', 'Power+1 for each other Wind card of yours in this front.' } }, say = '*BZZZZZT*' },
    { id = 103, key = 'billy', name = 'Bigmouth Billy', cost = 4, power = 4, el = 'earth', fam = 'Vermin', species = 'Notorious Monster', rarity = 2, model = 424, theme = 'guild', nm = 'Bigmouth_Billy',
      ab = { swallow = true }, text = { { 'Swallow', 'When played: destroys the weakest enemy card in this front.' } }, say = '*GULP*' },
    { id = 104, key = 'swamfisk', name = 'Swamfisk', cost = 3, power = 4, el = 'water', fam = 'Aquan', species = 'Notorious Monster', rarity = 2, model = 348, theme = 'guild', nm = 'Swamfisk',
      ab = { rain = 3 }, text = { { 'Latent effect', 'Power+3 in rain or squalls.' } }, say = '*SPLOOSH*' },
    { id = 105, key = 'goldenbat', name = 'Golden Bat', cost = 2, power = 2, el = 'wind', fam = 'Beast', species = 'Notorious Monster', rarity = 2, model = 256, theme = 'guild', nm = 'Golden_Bat',
      ab = { swarm = 1, night = 2 }, text = { { 'Swarm', 'Power+1 for each other Bat of yours on the table.' }, { 'Latent effect', 'Power+2 at night.' } }, say = '*glittering squeak*' },
    { id = 106, key = 'morbolger', name = 'Morbolger', cost = 4, power = 4, el = 'earth', fam = 'Plantoid', species = 'Notorious Monster', rarity = 2, model = 380, theme = 'guild', nm = 'Morbolger',
      ab = { breath = 1 }, text = { { 'Bad Breath', 'When played: every enemy card in this front gets Power-1.' } }, say = '*HUAAAH*' },
    { id = 107, key = 'meedeggi', name = 'Mee Deggi the Punisher', cost = 4, power = 4, el = 'wind', fam = 'Beastman', species = 'Notorious Monster', rarity = 2, model = 580, theme = 'guild', nm = 'Mee_Deggi_the_Punisher',
      ab = { punish = 3 }, text = { { 'Punish', 'When played: Power+3 if you\'re behind in this front.' } }, say = 'Kneel, heretic, ho!' },
    { id = 108, key = 'profblix', name = 'Mysticmaker Profblix', cost = 3, power = 3, el = 'fire', fam = 'Beastman', species = 'Notorious Monster', rarity = 2, model = 507, theme = 'guild', nm = 'Mysticmaker_Profblix',
      ab = { tinker2 = 2 }, text = { { 'Master Tinker', 'When played: two of your other cards on the table get Power+2.' } }, say = 'Profblix make it better! Hee!' },
    { id = 109, key = 'roc', name = 'Roc', cost = 5, power = 6, el = 'wind', fam = 'Bird', species = 'Notorious Monster', rarity = 2, model = 336, theme = 'guild', nm = 'Roc',
      ab = { gale = 2 }, text = { { 'Latent effect', 'Power+2 in wind or gales.' } }, say = '*SKREEEE*' },
    { id = 110, key = 'serket', name = 'Serket', cost = 5, power = 5, el = 'earth', fam = 'Vermin', species = 'Notorious Monster', rarity = 2, model = 284, theme = 'guild', nm = 'Serket',
      ab = { venom = 3 }, text = { { 'Venom', 'End of round 5: the strongest enemy card in this front gets Power-3.' } }, say = '*clack clack clack*' },

    -- Legendaries: a great wyrm, and townsfolk won at the Steaming Sheep's table
    { id = 201, key = 'fafnir', name = 'Fafnir', cost = 6, power = 8, el = 'dark', fam = 'Dragon', species = 'Notorious Monster', rarity = 3, model = 783, theme = 'dragon', nm = 'Fafnir',
      ab = { dread = true }, text = { { 'Dread', 'Enemy cards in this front lose every bonus.' } }, say = '*the air itself trembles*' },
    { id = 202, key = 'toma', name = 'Old Toma, Lamplighter', cost = 2, power = 1, el = 'light', fam = 'Townsfolk', species = 'Hume', rarity = 3, theme = 'hume',
      ab = { lamps = true, night = 2 }, text = { { 'Light the Lamps', 'Enemy cards in this front lose their night bonus.' }, { 'Latent effect', 'Power+2 at night.' } }, say = 'Twenty lamps. Every night.' },
    { id = 203, key = 'hrodgar', name = 'Hrodgar, Card Shark', cost = 3, power = 2, el = 'fire', fam = 'Townsfolk', species = 'Galka', rarity = 3, theme = 'galka',
      ab = { bluff = 4 }, text = { { 'Bluff', 'When played: if your opponent holds a legendary, Power+4.' } }, say = 'I\'ll pay you on payday. Deal.' },
    { id = 204, key = 'dagny', name = 'Dagny by Candlelight', cost = 2, power = 2, el = 'light', fam = 'Townsfolk', species = 'Hume', rarity = 3, theme = 'hume',
      ab = { draw = 2 }, text = { { 'Card Counter', 'When played: draw two cards.' } }, say = 'I\'ve seen that card before.' },
    { id = 205, key = 'fenwick', name = 'Steady Fenwick', cost = 3, power = 4, el = 'earth', fam = 'Townsfolk', species = 'Elvaan', rarity = 3, theme = 'elvaan',
      ab = { immune = true, harden = 1 }, text = { { 'Unshaken', 'Enemy abilities can\'t weaken or destroy it.' }, { 'Harden', 'Power+1 for each other card of yours in this front.' } }, say = 'Patience wins hands.' },
    { id = 206, key = 'kipiko', name = 'Kipiko the Trickster', cost = 2, power = 1, el = 'thunder', fam = 'Townsfolk', species = 'Tarutaru', rarity = 3, theme = 'tarutaru',
      ab = { mimic = true }, text = { { 'Mimic', 'Its Power is that of the strongest other card in this front.' } }, say = 'Hee! Now you see it...' },
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

-- A card id with its foil: the base card, the variant (0 none) and the card as played
cards.split = function(id)
    local v = math.floor(id / 1000)
    return cards.byId[id % 1000], v
end

cards.get = function(id)
    local base, v = cards.split(id or 0)
    if base == nil or v > 4 then
        return nil
    end

    if v == 0 then
        return base
    end

    local c = {}
    for k, val in pairs(base) do
        c[k] = val
    end

    c.id, c.variant, c.base = id, v, base.id
    c.name = cards.VARIANTS[v].name .. ' ' .. base.name
    if v == 4 then
        c.cost = base.cost + 1
    end

    return c
end

-- The starter deck every player gets: fifteen commons
cards.STARTER = { 1, 1, 2, 2, 3, 6, 6, 8, 8, 11, 12, 14, 15, 16, 22 }

return cards
