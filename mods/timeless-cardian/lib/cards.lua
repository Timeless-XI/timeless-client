-----------------------------------
-- Cardian: the cards. One list read by the server (drops, the game) and copied as-is into the timeless-cardian
-- client mod (lib/cards.lua), so both sides agree on every number.
--
-- A card: id (1-99 commons, 101-199 rares, 201-299 legendaries, 301-311 crests, 401-405 lost, 501-522 jobs, 601-606 club), key, name, sides { top, right, bottom, left }
-- (1-10; 10 shows as A), el (element), level (1-10: 1-5 field monsters, 6-7 notorious monsters, 8-10 legends),
-- fam (family: the world's bonuses read it), species (shown), rarity (1 common, 2 rare, 3 legendary), model (the
-- monster's model id, for the client's live 3D portrait; nil for townsfolk, who get a drawn picture), theme (the
-- card's header, after the shop windows), say (its voice).
-- Drops: family = an xi.mobFamily name (field monsters drop commons), nm = a monster's name (rares, legendaries).
-- when: the world's own cards drop only { night = true } (20:00-6:00), in { weather = element }, where
-- { beastHeld = true } (the beastmen hold the region) or in one { region = xi.region name }.
-- crest = true: a set's reward (cards.SETS), never dropped.
-- job = 'WAR'..: a job card (501-522), won from that job's master while you're on the job; at the table its ability
--   (cards.ABILITIES) works only for someone on that job. One in a hand. Never put up, broken or taken.
-- club = true: the Card Club's own (601-606), won from each member in turn. lost = true: a lost card (401-405),
--   one on the whole server, found by solving its riddle (cardian_config.lost) and won from whoever holds it.
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

-- What each job card's ability does (cardian_engine.lua does it)
cards.ABILITIES =
{
    berserk = { name = 'Berserk', text = 'Berserk: every side +1 when placed.' },
    counter = { name = 'Counterstance', text = 'Counterstance: plain captures can\'t flip it (only Same, Plus and Combo).' },
    cure = { name = 'Cure', text = 'Cure: your cards beside it get +1 on every side when it\'s placed.' },
    burst = { name = 'Magic Burst', text = 'Magic Burst: every side +2 when placed on a marked square.' },
    enspell = { name = 'Enspell', text = 'Enspell: when placed, it also captures cards whose touching number equals its own.' },
    steal = { name = 'Steal', text = 'Steal: its weakest side takes the highest number of an enemy card beside it.' },
    sentinel = { name = 'Sentinel', text = 'Sentinel: nothing can flip it during your opponent\'s next turn.' },
    souleater = { name = 'Souleater', text = 'Souleater: every side +2 when placed, but its weakest side drops to 1.' },
    charm = { name = 'Charm', text = 'Charm: after it captures, the enemy beside it with the lowest touching number comes over too.' },
    march = { name = 'March', text = 'March: your other cards in its column get +1 on every side.' },
    barrage = { name = 'Barrage', text = 'Barrage: it also captures across one empty square, in a straight line.' },
    meditate = { name = 'Meditate', text = 'Meditate: every side +1 for each enemy card beside it when placed.' },
    utsusemi = { name = 'Utsusemi', text = 'Utsusemi: the first time it would be flipped, a shadow takes it instead.' },
    jump = { name = 'Jump', text = 'Jump: the side facing the strongest enemy beside it gets +3 when placed.' },
    avatar = { name = 'Avatar', text = 'Avatar: any marked square counts as its own element.' },
    azure = { name = 'Azure Lore', text = 'Azure Lore: when placed, it learns the numbers of the strongest card beside it, if they\'re better.' },
    roll = { name = 'Phantom Roll', text = 'Phantom Roll: each side gets +0 to +2 when placed, as the dice fall.' },
    automaton = { name = 'Automaton', text = 'Automaton: the next card you play gets +1 on every side.' },
    step = { name = 'Box Step', text = 'Box Step: enemy cards beside it lose 1 on every side when it\'s placed.' },
    arts = { name = 'Addendum', text = 'Addendum: it plays Same and Plus, whatever the table\'s rules.' },
    geocolure = { name = 'Geocolure', text = 'Geocolure: your other cards in its row get +1 on every side.' },
    lunge = { name = 'Lunge', text = 'Lunge: every side +1 for each marked square on the board (3 at most).' },
}

cards.JOBS = { 'WAR', 'MNK', 'WHM', 'BLM', 'RDM', 'THF', 'PLD', 'DRK', 'BST', 'BRD', 'RNG', 'SAM', 'NIN', 'DRG', 'SMN', 'BLU', 'COR', 'PUP', 'DNC', 'SCH', 'GEO', 'RUN' }

-- The quests' words (the server's cardian_config.lua and the client's cardbook both read them)
-- The Card Club, rank by rank: who, where, and what you hear about them
cards.CLUB =
{
    { rank = 1, card = 601, name = 'Harmodios', npc = 'Harmodios', zone = 235, place = 'Bastok Markets', title = 'Jack',
      hint = 'They say the Card Club\'s Jack sells instruments in Bastok Markets, and plays a mean hand between customers.' },
    { rank = 2, card = 602, name = 'Legata', npc = 'Legata', zone = 230, place = 'Southern San d\'Oria', title = 'Queen of Clubs',
      hint = 'The Queen of Clubs keeps to Southern San d\'Oria. An Elvaan who never smiles at the table.' },
    { rank = 3, card = 603, name = 'Kohlo-Lakolo', npc = 'Kohlo-Lakolo', zone = 240, place = 'Port Windurst', title = 'Queen of Diamonds',
      hint = 'In Port Windurst, the Queen of Diamonds hides among the Tarutaru who love a good story.' },
    { rank = 4, card = 604, name = 'Squintrox Dryeyes', npc = 'Squintrox_Dryeyes', zone = 246, place = 'Port Jeuno', title = 'Queen of Spades',
      hint = 'The Queen of Spades is a Goblin in Port Jeuno, they say, who\'ll deal with anyone.' },
    { rank = 5, card = 605, name = 'Ghebi Damomohe', npc = 'Ghebi_Damomohe', zone = 245, place = 'Lower Jeuno', title = 'Queen of Hearts',
      hint = 'The Queen of Hearts deals for the Tenshodo in Lower Jeuno, and never shows her hand.' },
    { rank = 6, card = 606, name = 'Maat', npc = 'Maat', zone = 243, place = 'Ru\'Lude Gardens', title = 'King',
      hint = 'Every card player in Jeuno whispers it: the King of the Card Club is an old master in Ru\'Lude Gardens.' },
}

-- The jobs' masters: { job, job name, the NPC's name, zone, place }; their job card is 500 + their place here
cards.MASTERS =
{
    { 'WAR', 'Warrior', 'Phara', 234, 'Bastok Mines' },
    { 'MNK', 'Monk', 'Oggbi', 236, 'Port Bastok' },
    { 'WHM', 'White Mage', 'Narcheral', 231, 'Northern San d\'Oria' },
    { 'BLM', 'Black Mage', 'Shantotto', 239, 'Windurst Walls' },
    { 'RDM', 'Red Mage', 'Sharzalion', 230, 'Southern San d\'Oria' },
    { 'THF', 'Thief', 'Nanaa_Mihgo', 241, 'Windurst Woods' },
    { 'PLD', 'Paladin', 'Balasiel', 230, 'Southern San d\'Oria' },
    { 'DRK', 'Dark Knight', 'Raibaht', 237, 'Metalworks' },
    { 'BST', 'Beastmaster', 'Brutus', 244, 'Upper Jeuno' },
    { 'BRD', 'Bard', 'Mertaire', 245, 'Lower Jeuno' },
    { 'RNG', 'Ranger', 'Perih_Vashai', 241, 'Windurst Woods' },
    { 'SAM', 'Samurai', 'Jaucribaix', 252, 'Norg' },
    { 'NIN', 'Ninja', 'Romualdo', 237, 'Metalworks' },
    { 'DRG', 'Dragoon', 'Rahal', 233, 'Chateau d\'Oraguille' },
    { 'SMN', 'Summoner', 'Koru-Moru', 239, 'Windurst Walls' },
    { 'BLU', 'Blue Mage', 'Waoud', 50, 'Aht Urhgan Whitegate' },
    { 'COR', 'Corsair', 'Ratihb', 50, 'Aht Urhgan Whitegate' },
    { 'PUP', 'Puppetmaster', 'Iruki-Waraki', 50, 'Aht Urhgan Whitegate' },
    { 'DNC', 'Dancer', 'Laila', 244, 'Upper Jeuno' },
    { 'SCH', 'Scholar', 'Erlene', 175, 'the Eldieme Necropolis (past)' },
    { 'GEO', 'Geomancer', 'Sylvie', 256, 'Western Adoulin' },
    { 'RUN', 'Rune Fencer', 'Octavien', 257, 'Eastern Adoulin' },
}

-- The lost cards' riddles
cards.RIDDLES =
{
    [401] = 'Where the sands of Altepa burn under a burning sky, on the day of flame, the great wyrm\'s kin calls its king.',
    [402] = 'When no moon rises over Lufaise, in the deepest hours of night, bring darkness in your hand.',
    [403] = 'At the stroke of midnight in the Shadow Lord\'s keep, his captains kneel: bring three of them in your hand.',
    [404] = 'When thunder rolls over Qufim, a hand of storms, and nothing else, wakes the dark rider.',
    [405] = 'On the day of light under a full moon, in the Empire\'s white city, four of its own in your hand call judgment.',
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
    { id = 27, key = 'slime', name = 'Slime', sides = { 3, 5, 4, 3 }, el = 'water', level = 2, fam = 'Amorph', species = 'Slime', rarity = 1, model = 292, theme = 'sea', family = 'SLIME', say = '*blorp*' },
    { id = 28, key = 'scorpion', name = 'Desert Scorpion', sides = { 6, 4, 4, 5 }, el = 'earth', level = 4, fam = 'Vermin', species = 'Scorpion', rarity = 1, model = 284, theme = 'wild', family = 'SCORPION', say = '*click-SNAP*' },
    { id = 29, key = 'spider', name = 'Spider', sides = { 5, 4, 3, 5 }, el = 'dark', level = 3, fam = 'Vermin', species = 'Spider', rarity = 1, model = 310, theme = 'grave', family = 'SPIDER', say = '*skitter skitter*' },
    { id = 30, key = 'fly', name = 'Damselfly', sides = { 4, 3, 5, 3 }, el = 'wind', level = 2, fam = 'Vermin', species = 'Fly', rarity = 1, model = 448, theme = 'wild', family = 'FLY', say = '*hummm*' },
    { id = 31, key = 'morbol', name = 'Morbol', sides = { 7, 5, 4, 5 }, el = 'earth', level = 5, fam = 'Plantoid', species = 'Morbol', rarity = 1, model = 380, theme = 'wild', family = 'MORBOL', say = '*gurgle*' },
    { id = 32, key = 'treant', name = 'Treant', sides = { 6, 6, 6, 3 }, el = 'earth', level = 5, fam = 'Plantoid', species = 'Treant', rarity = 1, model = 388, theme = 'wild', family = 'TREANT', say = '*creak...*' },
    { id = 33, key = 'sapling', name = 'Sapling', sides = { 3, 4, 4, 4 }, el = 'earth', level = 2, fam = 'Plantoid', species = 'Sapling', rarity = 1, model = 392, theme = 'wild', family = 'SAPLING', say = '*rustle*' },
    { id = 34, key = 'goobbue', name = 'Goobbue', sides = { 5, 6, 5, 3 }, el = 'earth', level = 4, fam = 'Plantoid', species = 'Goobbue', rarity = 1, model = 296, theme = 'wild', family = 'GOOBBUE', say = '*goob goob*' },
    { id = 35, key = 'cactuar', name = 'Cactuar', sides = { 6, 2, 6, 3 }, el = 'wind', level = 3, fam = 'Plantoid', species = 'Cactuar', rarity = 1, model = 372, theme = 'wild', family = 'CACTAUR', say = '*a thousand needles*' },
    { id = 36, key = 'ahriman', name = 'Evil Eye', sides = { 6, 5, 3, 5 }, el = 'dark', level = 4, fam = 'Demon', species = 'Ahriman', rarity = 1, model = 265, theme = 'grave', family = 'AHRIMAN', say = '*stares*' },
    { id = 37, key = 'hound', name = 'Hellhound', sides = { 5, 6, 5, 3 }, el = 'fire', level = 4, fam = 'Undead', species = 'Hound', rarity = 1, model = 364, theme = 'grave', family = 'HOUND', say = '*howls at nothing*' },
    { id = 38, key = 'doll', name = 'Doll', sides = { 4, 6, 3, 4 }, el = 'earth', level = 3, fam = 'Arcana', species = 'Doll', rarity = 1, model = 304, theme = 'ember', family = 'DOLL', say = '*clatter*' },
    { id = 39, key = 'manticore', name = 'Manticore', sides = { 7, 4, 6, 4 }, el = 'fire', level = 5, fam = 'Beast', species = 'Manticore', rarity = 1, model = 402, theme = 'wild', family = 'MANTICORE', say = '*ROAR*' },
    { id = 40, key = 'bird', name = 'Giant Bird', sides = { 4, 4, 3, 4 }, el = 'wind', level = 2, fam = 'Bird', species = 'Bird', rarity = 1, model = 444, theme = 'wild', family = 'BIRD', say = '*caw*' },
    { id = 41, key = 'ziz', name = 'Ziz', sides = { 5, 5, 4, 5 }, el = 'wind', level = 4, fam = 'Bird', species = 'Greater Bird', rarity = 1, model = 336, theme = 'wild', family = 'GREATER_BIRD', say = '*SKREE*' },
    { id = 42, key = 'wyvern', name = 'Wyvern', sides = { 6, 7, 3, 5 }, el = 'wind', level = 5, fam = 'Dragon', species = 'Wyvern', rarity = 1, model = 398, theme = 'dragon', family = 'WYVERN', say = '*hisss*' },
    { id = 43, key = 'raptor', name = 'Raptor', sides = { 5, 5, 3, 4 }, el = 'earth', level = 3, fam = 'Lizard', species = 'Raptor', rarity = 1, model = 317, theme = 'wild', family = 'RAPTOR', say = '*chirr*' },
    { id = 44, key = 'cockatrice', name = 'Cockatrice', sides = { 4, 3, 5, 5 }, el = 'wind', level = 3, fam = 'Bird', species = 'Cockatrice', rarity = 1, model = 288, theme = 'wild', family = 'COCKATRICE', say = '*cluck... petrify*' },
    { id = 45, key = 'doomed', name = 'Doomed', sides = { 4, 5, 5, 5 }, el = 'dark', level = 4, fam = 'Undead', species = 'Doomed', rarity = 1, model = 320, theme = 'grave', family = 'DOOMED', say = '*croak*' },
    { id = 46, key = 'magicpot', name = 'Magic Pot', sides = { 5, 3, 4, 5 }, el = 'light', level = 3, fam = 'Arcana', species = 'Magic Pot', rarity = 1, model = 412, theme = 'ember', family = 'MAGIC_POT', say = '*clink*' },
    { id = 47, key = 'evilweapon', name = 'Evil Weapon', sides = { 7, 3, 6, 3 }, el = 'dark', level = 4, fam = 'Arcana', species = 'Evil Weapon', rarity = 1, model = 451, theme = 'ember', family = 'EVIL_WEAPON', say = '*rattle*' },
    { id = 48, key = 'golem', name = 'Golem', sides = { 6, 5, 6, 4 }, el = 'earth', level = 5, fam = 'Arcana', species = 'Golem', rarity = 1, model = 432, theme = 'ember', family = 'GOLEM', say = '...' },
    { id = 49, key = 'cardian', name = 'Ace Cardian', sides = { 4, 5, 4, 4 }, el = 'light', level = 3, fam = 'Arcana', species = 'Cardian', rarity = 1, model = 431, theme = 'tarutaru', family = 'CARDIAN', say = 'Ace of Cups, reporting!' },
    { id = 50, key = 'demon', name = 'Demon', sides = { 7, 6, 3, 5 }, el = 'dark', level = 5, fam = 'Demon', species = 'Demon', rarity = 1, model = 741, theme = 'grave', family = 'DEMON', say = 'Kneel before the dream.' },
    { id = 51, key = 'hecteyes', name = 'Hecteyes', sides = { 5, 4, 5, 3 }, el = 'dark', level = 3, fam = 'Amorph', species = 'Hecteyes', rarity = 1, model = 384, theme = 'grave', family = 'HECTEYES', say = '*blink blink blink*' },
    { id = 52, key = 'seamonk', name = 'Sea Monk', sides = { 4, 5, 3, 5 }, el = 'water', level = 3, fam = 'Aquan', species = 'Sea Monk', rarity = 1, model = 352, theme = 'sea', family = 'SEA_MONK', say = '*bloop*' },
    { id = 53, key = 'opoopo', name = 'Opo-opo', sides = { 4, 5, 2, 4 }, el = 'earth', level = 2, fam = 'Beast', species = 'Opo-opo', rarity = 1, model = 416, theme = 'wild', family = 'OPO_OPO', say = 'Ook! Ook!' },
    { id = 54, key = 'corse', name = 'Corse', sides = { 5, 4, 6, 4 }, el = 'dark', level = 4, fam = 'Undead', species = 'Corse', rarity = 1, model = 1363, theme = 'grave', family = 'CORSE', say = '...ashes...' },
    { id = 55, key = 'mimic', name = 'Mimic', sides = { 3, 7, 3, 6 }, el = 'dark', level = 4, fam = 'Arcana', species = 'Mimic', rarity = 1, model = 259, theme = 'ember', family = 'MIMIC', say = '*the chest grins*' },
    { id = 56, key = 'antica', name = 'Antica', sides = { 6, 4, 5, 4 }, el = 'earth', level = 4, fam = 'Beastman', species = 'Antica', rarity = 1, model = 1263, theme = 'ember', family = 'ANTICA', say = 'For the sand!' },
    { id = 57, key = 'buffalo', name = 'Buffalo', sides = { 5, 5, 5, 4 }, el = 'earth', level = 4, fam = 'Beast', species = 'Buffalo', rarity = 1, model = 1357, theme = 'wild', family = 'BUFFALO', say = '*snort*' },
    { id = 58, key = 'flan', name = 'Flan', sides = { 2, 5, 3, 5 }, el = 'water', level = 2, fam = 'Amorph', species = 'Flan', rarity = 1, model = 1799, theme = 'sea', family = 'FLAN', say = '*wobble*' },
    { id = 59, key = 'eft', name = 'Eft', sides = { 4, 4, 4, 3 }, el = 'fire', level = 2, fam = 'Lizard', species = 'Eft', rarity = 1, model = 1349, theme = 'wild', family = 'EFT', say = '*flick*' },
    { id = 60, key = 'imp', name = 'Imp', sides = { 3, 6, 3, 5 }, el = 'dark', level = 3, fam = 'Demon', species = 'Imp', rarity = 1, model = 2236, theme = 'grave', family = 'IMP', say = 'Hee hee hee!' },
    { id = 61, key = 'bugard', name = 'Bugard', sides = { 6, 5, 3, 5 }, el = 'earth', level = 4, fam = 'Lizard', species = 'Bugard', rarity = 1, model = 1351, theme = 'wild', family = 'BUGARD', say = '*stomp*' },
    { id = 62, key = 'troll', name = 'Troll', sides = { 7, 5, 5, 4 }, el = 'earth', level = 5, fam = 'Beastman', species = 'Troll', rarity = 1, model = 1712, theme = 'orc', family = 'TROLL', say = 'Troll strong!' },
    { id = 63, key = 'mamoolja', name = 'Mamool Ja', sides = { 5, 6, 4, 4 }, el = 'fire', level = 4, fam = 'Beastman', species = 'Mamool Ja', rarity = 1, model = 1619, theme = 'orc', family = 'MAMOOL_JA', say = 'Mamook!' },
    { id = 64, key = 'lamia', name = 'Lamia', sides = { 4, 5, 5, 5 }, el = 'water', level = 4, fam = 'Beastman', species = 'Lamia', rarity = 1, model = 1654, theme = 'sea', family = 'LAMIAE', say = 'Ssso pretty...' },
    { id = 65, key = 'behemothcalf', name = 'Behemoth Calf', sides = { 6, 6, 5, 4 }, el = 'thunder', level = 5, fam = 'Beast', species = 'Behemoth', rarity = 1, model = 404, theme = 'wild', family = 'BEHEMOTH', say = '*grrruuh*' },
    { id = 66, key = 'ram', name = 'Battering Ram', sides = { 6, 4, 4, 5 }, el = 'earth', level = 4, fam = 'Beast', species = 'Ram', rarity = 1, model = 344, theme = 'wild', family = 'RAM', say = '*CLONK*' },
    { id = 67, key = 'fire_elemental', name = 'Fire Elemental', sides = { 6, 6, 5, 4 }, el = 'fire', level = 5, fam = 'Elemental', species = 'Elemental', rarity = 1, model = 436, theme = 'ember', family = 'ELEMENTAL', when = { weather = 'fire' }, say = '*the air hums*' },
    { id = 68, key = 'ice_elemental', name = 'Ice Elemental', sides = { 5, 6, 6, 4 }, el = 'ice', level = 5, fam = 'Elemental', species = 'Elemental', rarity = 1, model = 437, theme = 'sea', family = 'ELEMENTAL', when = { weather = 'ice' }, say = '*the air hums*' },
    { id = 69, key = 'wind_elemental', name = 'Air Elemental', sides = { 6, 5, 4, 6 }, el = 'wind', level = 5, fam = 'Elemental', species = 'Elemental', rarity = 1, model = 438, theme = 'wild', family = 'ELEMENTAL', when = { weather = 'wind' }, say = '*the air hums*' },
    { id = 70, key = 'earth_elemental', name = 'Earth Elemental', sides = { 5, 4, 6, 6 }, el = 'earth', level = 5, fam = 'Elemental', species = 'Elemental', rarity = 1, model = 439, theme = 'quadav', family = 'ELEMENTAL', when = { weather = 'earth' }, say = '*the air hums*' },
    { id = 71, key = 'thunder_elemental', name = 'Thunder Elemental', sides = { 7, 5, 4, 5 }, el = 'thunder', level = 5, fam = 'Elemental', species = 'Elemental', rarity = 1, model = 441, theme = 'tarutaru', family = 'ELEMENTAL', when = { weather = 'thunder' }, say = '*the air hums*' },
    { id = 72, key = 'water_elemental', name = 'Water Elemental', sides = { 4, 6, 6, 5 }, el = 'water', level = 5, fam = 'Elemental', species = 'Elemental', rarity = 1, model = 440, theme = 'sea', family = 'ELEMENTAL', when = { weather = 'water' }, say = '*the air hums*' },
    { id = 73, key = 'light_elemental', name = 'Light Elemental', sides = { 6, 5, 5, 5 }, el = 'light', level = 5, fam = 'Elemental', species = 'Elemental', rarity = 1, model = 442, theme = 'hume', family = 'ELEMENTAL', when = { weather = 'light' }, say = '*the air hums*' },
    { id = 74, key = 'dark_elemental', name = 'Dark Elemental', sides = { 5, 5, 6, 5 }, el = 'dark', level = 5, fam = 'Elemental', species = 'Elemental', rarity = 1, model = 443, theme = 'grave', family = 'ELEMENTAL', when = { weather = 'dark' }, say = '*the air hums*' },
    { id = 75, key = 'banshee', name = 'Banshee', sides = { 4, 7, 3, 5 }, el = 'dark', level = 4, fam = 'Undead', species = 'Ghost', rarity = 1, model = 369, theme = 'grave', family = 'GHOST', when = { night = true }, say = 'Ahhhhhhh...' },
    { id = 76, key = 'vampirebat', name = 'Vampire Bat', sides = { 3, 5, 4, 5 }, el = 'dark', level = 3, fam = 'Beast', species = 'Bat', rarity = 1, model = 257, theme = 'grave', family = 'BAT', when = { night = true }, say = '*flutter*' },
    { id = 77, key = 'wight', name = 'Wight', sides = { 7, 4, 6, 4 }, el = 'dark', level = 5, fam = 'Undead', species = 'Skeleton', rarity = 1, model = 564, theme = 'grave', family = 'SKELETON', when = { night = true }, say = '...the dead march...' },
    { id = 78, key = 'infernobomb', name = 'Inferno Bomb', sides = { 8, 5, 3, 6 }, el = 'fire', level = 5, fam = 'Arcana', species = 'Bomb', rarity = 1, model = 278, theme = 'ember', family = 'BOMB', when = { weather = 'fire' }, say = '*FWOOSH*' },
    { id = 79, key = 'snowtiger', name = 'Snow Tiger', sides = { 6, 6, 4, 6 }, el = 'ice', level = 5, fam = 'Beast', species = 'Tiger', rarity = 1, model = 2248, theme = 'wild', family = 'TIGER', when = { weather = 'ice' }, say = '*a cold growl*' },
    { id = 80, key = 'goblinraider', name = 'Goblin Raider', sides = { 6, 4, 5, 4 }, el = 'fire', level = 4, fam = 'Beastman', species = 'Goblin', rarity = 1, model = 1087, theme = 'goblin', family = 'GOBLIN', when = { beastHeld = true }, say = 'Raid! Raid! Shiny for Goblin!' },
    { id = 81, key = 'quadavironclad', name = 'Quadav Ironclad', sides = { 7, 5, 6, 4 }, el = 'earth', level = 5, fam = 'Beastman', species = 'Quadav', rarity = 1, model = 1053, theme = 'quadav', family = 'QUADAV', when = { beastHeld = true }, say = 'The shell does not break.' },
    { id = 82, key = 'yagudozealot', name = 'Yagudo Zealot', sides = { 5, 6, 6, 5 }, el = 'wind', level = 5, fam = 'Beastman', species = 'Yagudo', rarity = 1, model = 1081, theme = 'yagudo', family = 'YAGUDO', when = { beastHeld = true }, say = 'The heretics flee, ho!' },
    { id = 83, key = 'orcwarlord', name = 'Orcish Warlord', sides = { 8, 4, 5, 5 }, el = 'fire', level = 5, fam = 'Beastman', species = 'Orc', rarity = 1, model = 1046, theme = 'orc', family = 'ORC', when = { beastHeld = true }, say = 'The land is ours now!' },
    { id = 84, key = 'sandlizard', name = 'Sand Lizard', sides = { 6, 4, 5, 4 }, el = 'earth', level = 4, fam = 'Lizard', species = 'Lizard', rarity = 1, model = 329, theme = 'wild', family = 'LIZARD', when = { region = 'KUZOTZ' }, say = '*buries itself*' },
    { id = 85, key = 'glacierbat', name = 'Glacier Bat', sides = { 4, 4, 4, 5 }, el = 'ice', level = 3, fam = 'Beast', species = 'Bat', rarity = 1, model = 256, theme = 'sea', family = 'BAT', when = { region = 'FAUREGANDI' }, say = '*squeak* (frost on its wings)' },

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
    { id = 111, key = 'jack', name = 'Jaggedy-Eared Jack', sides = { 7, 5, 6, 5 }, el = 'wind', level = 6, fam = 'Beast', species = 'Notorious Monster', rarity = 2, model = 268, theme = 'guild', nm = 'Jaggedy-Eared_Jack', say = '*thump thump*' },
    { id = 112, key = 'ropipi', name = 'Sharp-Eared Ropipi', sides = { 5, 7, 5, 6 }, el = 'wind', level = 6, fam = 'Beast', species = 'Notorious Monster', rarity = 2, model = 269, theme = 'guild', nm = 'Sharp-Eared_Ropipi', say = '*ears twitch*' },
    { id = 113, key = 'tomtittat', name = 'Tom Tit Tat', sides = { 4, 7, 6, 6 }, el = 'earth', level = 6, fam = 'Plantoid', species = 'Notorious Monster', rarity = 2, model = 300, theme = 'guild', nm = 'Tom_Tit_Tat', say = '*SHRIEEEK*' },
    { id = 114, key = 'spook', name = 'Spook', sides = { 6, 6, 4, 7 }, el = 'dark', level = 6, fam = 'Undead', species = 'Notorious Monster', rarity = 2, model = 368, theme = 'guild', nm = 'Spook', say = 'Boo.' },
    { id = 115, key = 'lumberjack', name = 'Lumber Jack', sides = { 8, 4, 6, 5 }, el = 'earth', level = 6, fam = 'Vermin', species = 'Notorious Monster', rarity = 2, model = 409, theme = 'guild', nm = 'Lumber_Jack', say = '*CHOMP*' },
    { id = 116, key = 'kingarthro', name = 'King Arthro', sides = { 6, 7, 7, 5 }, el = 'water', level = 7, fam = 'Aquan', species = 'Notorious Monster', rarity = 2, model = 357, theme = 'guild', nm = 'King_Arthro', say = '*CLACK CLACK*' },
    { id = 117, key = 'simurgh', name = 'Simurgh', sides = { 8, 7, 4, 6 }, el = 'wind', level = 7, fam = 'Bird', species = 'Notorious Monster', rarity = 2, model = 336, theme = 'guild', nm = 'Simurgh', say = '*a wind like a storm*' },
    { id = 118, key = 'argus', name = 'Argus', sides = { 7, 5, 7, 6 }, el = 'dark', level = 7, fam = 'Amorph', species = 'Notorious Monster', rarity = 2, model = 384, theme = 'guild', nm = 'Argus', say = '*a hundred eyes open*' },
    { id = 119, key = 'charybdis', name = 'Charybdis', sides = { 6, 8, 5, 6 }, el = 'water', level = 7, fam = 'Aquan', species = 'Notorious Monster', rarity = 2, model = 352, theme = 'guild', nm = 'Charybdis', say = '*the sea churns*' },
    { id = 120, key = 'cactrot', name = 'Cactrot Rapido', sides = { 9, 3, 9, 4 }, el = 'wind', level = 7, fam = 'Plantoid', species = 'Notorious Monster', rarity = 2, model = 372, theme = 'guild', nm = 'Cactrot_Rapido', say = '*gone in a blur*' },
    { id = 121, key = 'culberry', name = 'Golden-Tongued Culberry', sides = { 8, 5, 5, 7 }, el = 'dark', level = 7, fam = 'Beastman', species = 'Notorious Monster', rarity = 2, model = 1262, theme = 'guild', nm = 'Golden-Tongued_Culberry', say = '...you did this.' },
    { id = 122, key = 'bombqueen', name = 'Bomb Queen', sides = { 9, 6, 4, 6 }, el = 'fire', level = 7, fam = 'Arcana', species = 'Notorious Monster', rarity = 2, model = 280, theme = 'guild', nm = 'Bomb_Queen', say = '*her children fizz*' },
    { id = 123, key = 'ose', name = 'Ose', sides = { 7, 6, 6, 6 }, el = 'thunder', level = 7, fam = 'Beast', species = 'Notorious Monster', rarity = 2, model = 367, theme = 'guild', nm = 'Ose', say = '*sparks crawl over its fur*' },

    -- Legendaries: a great wyrm, and the card players' own cards, won by beating them
    { id = 201, key = 'fafnir', name = 'Fafnir', sides = { 10, 8, 6, 8 }, el = 'dark', level = 10, fam = 'Dragon', species = 'Notorious Monster', rarity = 3, model = 783, theme = 'dragon', nm = 'Fafnir', say = '*the air itself trembles*' },
    { id = 202, key = 'toma', name = 'Old Toma, Lamplighter', sides = { 6, 8, 8, 6 }, el = 'light', level = 8, fam = 'Townsfolk', species = 'Hume', rarity = 3, theme = 'hume', say = 'Twenty lamps. Every night.' },
    { id = 203, key = 'hrodgar', name = 'Hrodgar, Card Shark', sides = { 9, 6, 9, 6 }, el = 'fire', level = 9, fam = 'Townsfolk', species = 'Galka', rarity = 3, theme = 'galka', say = 'I\'ll pay you on payday. Deal.' },
    { id = 204, key = 'dagny', name = 'Dagny by Candlelight', sides = { 8, 6, 5, 9 }, el = 'light', level = 8, fam = 'Townsfolk', species = 'Hume', rarity = 3, theme = 'hume', say = 'I\'ve seen that card before.' },
    { id = 205, key = 'fenwick', name = 'Steady Fenwick', sides = { 7, 9, 6, 8 }, el = 'earth', level = 9, fam = 'Townsfolk', species = 'Elvaan', rarity = 3, theme = 'elvaan', say = 'Patience wins hands.' },
    { id = 206, key = 'kipiko', name = 'Kipiko the Trickster', sides = { 5, 9, 9, 5 }, el = 'thunder', level = 8, fam = 'Townsfolk', species = 'Tarutaru', rarity = 3, theme = 'tarutaru', say = 'Hee! Now you see it...' },
    { id = 207, key = 'shaali', name = 'Shaali Nine-Lives', sides = { 9, 8, 4, 9 }, el = 'thunder', level = 9, fam = 'Townsfolk', species = 'Mithra', rarity = 3, theme = 'mithra', say = 'Nine lives, nine aces. Your deal.' },
    { id = 208, key = 'behemoth', name = 'Behemoth', sides = { 9, 7, 7, 7 }, el = 'thunder', level = 9, fam = 'Beast', species = 'Notorious Monster', rarity = 3, model = 404, theme = 'dragon', nm = 'Behemoth', say = '*the ground splits*' },
    { id = 209, key = 'adamantoise', name = 'Adamantoise', sides = { 6, 9, 9, 6 }, el = 'earth', level = 9, fam = 'Lizard', species = 'Notorious Monster', rarity = 3, model = 400, theme = 'dragon', nm = 'Adamantoise', say = '*a mountain moves*' },
    { id = 210, key = 'kingbehemoth', name = 'King Behemoth', sides = { 10, 8, 7, 7 }, el = 'thunder', level = 10, fam = 'Beast', species = 'Notorious Monster', rarity = 3, model = 404, theme = 'dragon', nm = 'King_Behemoth', say = '*METEOR*' },

    -- Crests: finish a set and its crest is yours (never dropped)
    { id = 301, key = 'crest_gustaberg', name = 'Gustaberg Crest', sides = { 7, 6, 6, 6 }, el = 'earth', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of Gustaberg, in one book.' },
    { id = 302, key = 'crest_ronfaure', name = 'Ronfaure Crest', sides = { 6, 7, 6, 6 }, el = 'wind', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of Ronfaure, in one book.' },
    { id = 303, key = 'crest_sarutabaruta', name = 'Sarutabaruta Crest', sides = { 6, 6, 7, 6 }, el = 'thunder', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of Sarutabaruta, in one book.' },
    { id = 304, key = 'crest_coast', name = 'Crest of the Coasts', sides = { 6, 6, 6, 7 }, el = 'water', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of the Coasts, in one book.' },
    { id = 305, key = 'crest_moonless', name = 'Moonless Nights Crest', sides = { 7, 7, 6, 5 }, el = 'dark', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of Moonless Nights, in one book.' },
    { id = 306, key = 'crest_altepa', name = 'Altepa Crest', sides = { 5, 7, 7, 6 }, el = 'fire', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of Altepa, in one book.' },
    { id = 307, key = 'crest_elements', name = 'Crest of the Elements', sides = { 7, 6, 5, 7 }, el = 'light', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of the Elements, in one book.' },
    { id = 308, key = 'crest_beastmen', name = 'Crest of the Beastmen', sides = { 6, 5, 7, 7 }, el = 'fire', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of the Beastmen, in one book.' },
    { id = 309, key = 'crest_urhgan', name = 'Aht Urhgan Crest', sides = { 7, 5, 6, 7 }, el = 'water', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of Aht Urhgan, in one book.' },
    { id = 310, key = 'crest_elshimo', name = 'Elshimo Crest', sides = { 6, 7, 5, 7 }, el = 'earth', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of Elshimo, in one book.' },
    { id = 311, key = 'crest_arcana', name = 'Crest of the Arcana', sides = { 7, 6, 7, 5 }, el = 'light', level = 7, fam = 'Crest', species = 'Set reward', rarity = 2, crest = true, theme = 'guild', say = 'Every card of the Arcana, in one book.' },
    { id = 501, key = 'job_war', name = 'The Warrior', sides = { 8, 6, 6, 5 }, el = 'fire', level = 7, fam = 'Job', species = 'Warrior', rarity = 2, job = 'WAR', ability = 'berserk', theme = 'guild', say = 'A warrior\'s card: won from Phara.' },
    { id = 502, key = 'job_mnk', name = 'The Monk', sides = { 6, 7, 6, 6 }, el = 'earth', level = 7, fam = 'Job', species = 'Monk', rarity = 2, job = 'MNK', ability = 'counter', theme = 'guild', say = 'A monk\'s card: won from Oggbi.' },
    { id = 503, key = 'job_whm', name = 'The White Mage', sides = { 7, 6, 5, 7 }, el = 'light', level = 7, fam = 'Job', species = 'White Mage', rarity = 2, job = 'WHM', ability = 'cure', theme = 'guild', say = 'A white mage\'s card: won from Narcheral.' },
    { id = 504, key = 'job_blm', name = 'The Black Mage', sides = { 5, 8, 7, 5 }, el = 'dark', level = 7, fam = 'Job', species = 'Black Mage', rarity = 2, job = 'BLM', ability = 'burst', theme = 'guild', say = 'A black mage\'s card: won from Shantotto.' },
    { id = 505, key = 'job_rdm', name = 'The Red Mage', sides = { 6, 7, 6, 6 }, el = 'fire', level = 7, fam = 'Job', species = 'Red Mage', rarity = 2, job = 'RDM', ability = 'enspell', theme = 'guild', say = 'A red mage\'s card: won from Sharzalion.' },
    { id = 506, key = 'job_thf', name = 'The Thief', sides = { 7, 5, 6, 7 }, el = 'wind', level = 7, fam = 'Job', species = 'Thief', rarity = 2, job = 'THF', ability = 'steal', theme = 'guild', say = 'A thief\'s card: won from Nanaa Mihgo.' },
    { id = 507, key = 'job_pld', name = 'The Paladin', sides = { 6, 6, 7, 6 }, el = 'light', level = 7, fam = 'Job', species = 'Paladin', rarity = 2, job = 'PLD', ability = 'sentinel', theme = 'guild', say = 'A paladin\'s card: won from Balasiel.' },
    { id = 508, key = 'job_drk', name = 'The Dark Knight', sides = { 7, 6, 7, 5 }, el = 'dark', level = 7, fam = 'Job', species = 'Dark Knight', rarity = 2, job = 'DRK', ability = 'souleater', theme = 'guild', say = 'A dark knight\'s card: won from Raibaht.' },
    { id = 509, key = 'job_bst', name = 'The Beastmaster', sides = { 6, 6, 6, 7 }, el = 'earth', level = 7, fam = 'Job', species = 'Beastmaster', rarity = 2, job = 'BST', ability = 'charm', theme = 'guild', say = 'A beastmaster\'s card: won from Brutus.' },
    { id = 510, key = 'job_brd', name = 'The Bard', sides = { 5, 7, 7, 6 }, el = 'wind', level = 7, fam = 'Job', species = 'Bard', rarity = 2, job = 'BRD', ability = 'march', theme = 'guild', say = 'A bard\'s card: won from Mertaire.' },
    { id = 511, key = 'job_rng', name = 'The Ranger', sides = { 7, 6, 5, 7 }, el = 'wind', level = 7, fam = 'Job', species = 'Ranger', rarity = 2, job = 'RNG', ability = 'barrage', theme = 'guild', say = 'A ranger\'s card: won from Perih Vashai.' },
    { id = 512, key = 'job_sam', name = 'The Samurai', sides = { 7, 6, 6, 6 }, el = 'fire', level = 7, fam = 'Job', species = 'Samurai', rarity = 2, job = 'SAM', ability = 'meditate', theme = 'guild', say = 'A samurai\'s card: won from Jaucribaix.' },
    { id = 513, key = 'job_nin', name = 'The Ninja', sides = { 6, 7, 5, 7 }, el = 'water', level = 7, fam = 'Job', species = 'Ninja', rarity = 2, job = 'NIN', ability = 'utsusemi', theme = 'guild', say = 'A ninja\'s card: won from Romualdo.' },
    { id = 514, key = 'job_drg', name = 'The Dragoon', sides = { 8, 5, 6, 6 }, el = 'thunder', level = 7, fam = 'Job', species = 'Dragoon', rarity = 2, job = 'DRG', ability = 'jump', theme = 'guild', say = 'A dragoon\'s card: won from Rahal.' },
    { id = 515, key = 'job_smn', name = 'The Summoner', sides = { 6, 6, 6, 7 }, el = 'light', level = 7, fam = 'Job', species = 'Summoner', rarity = 2, job = 'SMN', ability = 'avatar', theme = 'guild', say = 'A summoner\'s card: won from Koru-Moru.' },
    { id = 516, key = 'job_blu', name = 'The Blue Mage', sides = { 6, 6, 7, 6 }, el = 'water', level = 7, fam = 'Job', species = 'Blue Mage', rarity = 2, job = 'BLU', ability = 'azure', theme = 'guild', say = 'A blue mage\'s card: won from Waoud.' },
    { id = 517, key = 'job_cor', name = 'The Corsair', sides = { 6, 6, 6, 7 }, el = 'water', level = 7, fam = 'Job', species = 'Corsair', rarity = 2, job = 'COR', ability = 'roll', theme = 'guild', say = 'A corsair\'s card: won from Ratihb.' },
    { id = 518, key = 'job_pup', name = 'The Puppetmaster', sides = { 7, 6, 6, 6 }, el = 'thunder', level = 7, fam = 'Job', species = 'Puppetmaster', rarity = 2, job = 'PUP', ability = 'automaton', theme = 'guild', say = 'A puppetmaster\'s card: won from Iruki-Waraki.' },
    { id = 519, key = 'job_dnc', name = 'The Dancer', sides = { 6, 7, 6, 6 }, el = 'wind', level = 7, fam = 'Job', species = 'Dancer', rarity = 2, job = 'DNC', ability = 'step', theme = 'guild', say = 'A dancer\'s card: won from Laila.' },
    { id = 520, key = 'job_sch', name = 'The Scholar', sides = { 6, 6, 7, 6 }, el = 'light', level = 7, fam = 'Job', species = 'Scholar', rarity = 2, job = 'SCH', ability = 'arts', theme = 'guild', say = 'A scholar\'s card: won from Erlene.' },
    { id = 521, key = 'job_geo', name = 'The Geomancer', sides = { 6, 7, 6, 6 }, el = 'earth', level = 7, fam = 'Job', species = 'Geomancer', rarity = 2, job = 'GEO', ability = 'geocolure', theme = 'guild', say = 'A geomancer\'s card: won from Sylvie.' },
    { id = 522, key = 'job_run', name = 'The Rune Fencer', sides = { 7, 6, 6, 6 }, el = 'ice', level = 7, fam = 'Job', species = 'Rune Fencer', rarity = 2, job = 'RUN', ability = 'lunge', theme = 'guild', say = 'A rune fencer\'s card: won from Octavien.' },
    { id = 601, key = 'club_jack', name = 'Harmodios, Jack of the Club', sides = { 8, 7, 6, 6 }, el = 'wind', level = 8, fam = 'Townsfolk', species = 'Card Club (Hume)', rarity = 3, club = true, theme = 'hume', say = 'The Club? Never heard of it. ...Fine. One game.' },
    { id = 602, key = 'club_clubs', name = 'Legata, Queen of Clubs', sides = { 6, 8, 7, 6 }, el = 'earth', level = 8, fam = 'Townsfolk', species = 'Card Club (Elvaan)', rarity = 3, club = true, theme = 'elvaan', say = 'The Club has rules. Rule one: you don\'t win.' },
    { id = 603, key = 'club_diamonds', name = 'Kohlo-Lakolo, Queen of Diamonds', sides = { 7, 7, 8, 7 }, el = 'water', level = 9, fam = 'Townsfolk', species = 'Card Club (Tarutaru)', rarity = 3, club = true, theme = 'tarutaru', say = 'Sparkly, isn\'t it? So are my cards!' },
    { id = 604, key = 'club_spades', name = 'Squintrox, Queen of Spades', sides = { 9, 6, 7, 7 }, el = 'dark', level = 9, fam = 'Townsfolk', species = 'Card Club (Goblin)', rarity = 3, club = true, theme = 'goblin', say = 'Shovel, spade, same thing. Dig in!' },
    { id = 605, key = 'club_hearts', name = 'Ghebi, Queen of Hearts', sides = { 7, 8, 6, 8 }, el = 'fire', level = 9, fam = 'Townsfolk', species = 'Card Club (Hume)', rarity = 3, club = true, theme = 'hume', say = 'The Tenshodo deals cards too, friend.' },
    { id = 606, key = 'club_king', name = 'Maat, King of the Card Club', sides = { 9, 8, 7, 7 }, el = 'light', level = 10, fam = 'Townsfolk', species = 'Card Club (Hume)', rarity = 3, club = true, theme = 'hume', say = 'You came all this way to play an old man? Good.' },
    { id = 401, key = 'lost_bahamut', name = 'Bahamut, King of Wyrms', sides = { 10, 8, 8, 7 }, el = 'fire', level = 10, fam = 'Legend', species = 'Wyrmking', rarity = 3, lost = true, model = 1307, theme = 'dragon', say = 'The sky remembers.' },
    { id = 402, key = 'lost_promathia', name = 'Promathia', sides = { 9, 7, 9, 8 }, el = 'dark', level = 10, fam = 'Legend', species = 'Twilight God', rarity = 3, lost = true, model = 1072, theme = 'grave', say = 'All things return to the void.' },
    { id = 403, key = 'lost_shadowlord', name = 'The Shadow Lord', sides = { 10, 7, 7, 9 }, el = 'ice', level = 10, fam = 'Legend', species = 'Shadow Lord', rarity = 3, lost = true, model = 789, theme = 'orc', say = 'Kneel, and be spared... or play.' },
    { id = 404, key = 'lost_odin', name = 'Odin', sides = { 8, 10, 7, 8 }, el = 'thunder', level = 10, fam = 'Legend', species = 'Dark Rider', rarity = 3, lost = true, model = 1829, theme = 'grave', say = 'Zantetsuken.' },
    { id = 405, key = 'lost_alexander', name = 'Alexander', sides = { 8, 8, 10, 7 }, el = 'light', level = 10, fam = 'Legend', species = 'Holy Machine', rarity = 3, lost = true, model = 1834, theme = 'guild', say = 'Judgment, in nine squares.' },
}

-- The sets: every card of one, and its crest is yours
cards.SETS =
{
    { key = 'gustaberg', name = 'Gustaberg', crest = 301, cards = { 1, 2, 15, 13, 9, 101 } },
    { key = 'ronfaure', name = 'Ronfaure', crest = 302, cards = { 3, 11, 12, 23, 111 } },
    { key = 'sarutabaruta', name = 'Sarutabaruta', crest = 303, cards = { 4, 14, 16, 8, 49, 112, 113 } },
    { key = 'coast', name = 'the Coasts', crest = 304, cards = { 10, 21, 52, 30, 102, 116, 119 } },
    { key = 'moonless', name = 'Moonless Nights', crest = 305, cards = { 6, 7, 75, 76, 77, 37, 45, 54, 114 } },
    { key = 'altepa', name = 'Altepa', crest = 306, cards = { 56, 35, 28, 84, 120 } },
    { key = 'elements', name = 'the Elements', crest = 307, cards = { 67, 68, 69, 70, 71, 72, 73, 74 } },
    { key = 'beastmen', name = 'the Beastmen', crest = 308, cards = { 80, 81, 82, 83, 24, 26 } },
    { key = 'urhgan', name = 'Aht Urhgan', crest = 309, cards = { 58, 59, 60, 61, 62, 63, 64 } },
    { key = 'elshimo', name = 'Elshimo', crest = 310, cards = { 53, 33, 34, 32, 31, 106, 121 } },
    { key = 'arcana', name = 'the Arcana', crest = 311, cards = { 38, 46, 47, 48, 55, 51, 36, 118, 122 } },
}

cards.byId = {}
cards.byKey = {}
cards.byFamily = {}   -- family -> its everyday card (no `when`)
cards.ofFamily = {}   -- family -> every card it can drop, the world's own (`when`) included
cards.byNm = {}
for _, c in ipairs(cards.list) do
    cards.byId[c.id] = c
    cards.byKey[c.key] = c
    if c.family then
        cards.ofFamily[c.family] = cards.ofFamily[c.family] or {}
        table.insert(cards.ofFamily[c.family], c)
        if c.when == nil then
            cards.byFamily[c.family] = c
        end
    end

    if c.nm then
        cards.byNm[c.nm] = c
    end
end

-- A crest wears the portrait of its set's strongest card
for _, set in ipairs(cards.SETS) do
    local crest, best = cards.byId[set.crest], nil
    for _, id in ipairs(set.cards) do
        local c = cards.byId[id]
        if c and c.model and (best == nil or c.rarity > best.rarity or (c.rarity == best.rarity and c.level > best.level)) then
            best = c
        end
    end

    if crest and best and crest.model == nil then
        crest.model = best.model
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
