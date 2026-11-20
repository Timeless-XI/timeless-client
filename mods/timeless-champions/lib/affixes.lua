-- What each Champion affix is called, which of the game's icons stands for it, and what its tooltip says. Kept in
-- step with the server's champions_config.lua (the numbers in the tips are its settings).

local A = {}

A.RANK = {
  champion = { name = "Champion", colour = { 170, 205, 240 }, light = { 150, 190, 255 }, range = 6, intensity = 7, halo = 0 },
  elite = { name = "Elite Champion", colour = { 255, 200, 80 }, light = { 255, 196, 90 }, range = 8, intensity = 11, halo = 0.5 },
  legendary = { name = "Legendary Champion", colour = { 255, 110, 40 }, light = { 255, 120, 50 }, range = 10, intensity = 15,
    halo = 0.8 },
}

A.RAGE = { 255, 50, 40 }

A.ELEMENT = {
  fire = { prefix = "Pyre", icon = 94, colour = { 255, 90, 40 } },
  ice = { prefix = "Frost", icon = 95, colour = { 130, 200, 255 } },
  wind = { prefix = "Gale", icon = 96, colour = { 110, 230, 140 } },
  earth = { prefix = "Stone", icon = 97, colour = { 200, 160, 90 } },
  lightning = { prefix = "Storm", icon = 98, colour = { 200, 120, 255 } },
  water = { prefix = "Tide", icon = 99, colour = { 60, 140, 255 } },
  light = { prefix = "Radiant", icon = 274, colour = { 255, 245, 200 } },
  dark = { prefix = "Umbral", icon = 288, colour = { 150, 80, 200 } },
}

-- icon: a status effect's id (hud.icon kind "status")
A.AFFIX = {
  hasted = { name = "Hasted", icon = 33, text = "It attacks much faster." },
  shielded = { name = "Shielded", icon = 40, text = "Weapons do 30% less to it. Magic does full damage." },
  warded = { name = "Warded", icon = 41, text = "Magic does 30% less to it. Weapons do full damage." },
  frenzied = { name = "Frenzied", icon = 91, text = "It often strikes twice." },
  vampiric = { name = "Vampiric", icon = 173, text = "It drains life with every blow: 3% of its HP each round that hits." },
  venomous = { name = "Venomous", icon = 3, text = "Its blows can poison." },
  enraged = { name = "Enraged", icon = 56, text = "Below half HP it flies into a rage: +30% attack, +20 accuracy." },
  thorned = { name = "Thorned", icon = 34, text = "Striking it in melee burns you (Blaze Spikes). Ranged attacks and magic are safe." },
  regen = { name = "Renewing", icon = 42, text = "It heals 1% of its HP every few seconds unless it is poisoned, Bio'd or Dia'd." },
  undying = { name = "Undying", icon = 113, text = "Once, at 20% HP, it recovers to 50%. One big enough blow past 20% skips it." },
  elemental = { name = "Elemental", icon = 94, text = "One element's magic heals it." },
  blinking = { name = "Blinking", icon = 36, text = "Whenever it has no shadows it puts up 3 more, every 15 seconds. Area attacks hit through them." },
  pack = { name = "Alpha", icon = 68, text = "On its first fight it rallies up to 2 monsters of its kind within 15 yalms, with +20% attack." },
}

-- A badge for one affix of Champion c: {name, icon, title, text, dim, alert}
-- dim: the affix is spent or stopped right now; alert: it is acting right now (rage, shadows up).
function A.badge(key, c)
  local a = A.AFFIX[key]
  if not a then return nil end
  local b = { key = key, name = a.name, icon = a.icon, text = a.text, dim = false, alert = false }
  if key == "elemental" then
    local e = A.ELEMENT[c.element or ""]
    if e then
      b.name, b.icon = e.prefix, e.icon
      b.text = ("%s magic heals it. Use another element, or weapons."):format(c.element:sub(1, 1):upper() .. c.element:sub(2))
    end
  elseif key == "enraged" and c.raged then
    b.alert, b.text = true, "Enraged now: +30% attack and +20 accuracy until it falls."
  elseif key == "undying" and c.undying == false then
    b.dim, b.text = true, "Its one recovery is spent: it stays down this time."
  elseif key == "regen" and c.regen == false then
    b.dim, b.text = true, "Not healing now: poison, Bio or Dia is stopping it. Keep it on."
  elseif key == "regen" and c.regen then
    b.text = a.text .. " Healing now."
  elseif key == "blinking" and c.shadows then
    b.alert, b.text = true, "Shadows up now. " .. a.text
  elseif key == "pack" and #(c.rallied or {}) > 0 then
    b.text = ("It rallied %d of its kind (+20%% attack). The lines show which."):format(#c.rallied)
  end
  b.title = b.name
  return b
end

function A.badges(c)
  local out = {}
  for _, key in ipairs(c.affixes or {}) do
    local b = A.badge(key, c)
    if b then out[#out + 1] = b end
  end
  return out
end

return A
