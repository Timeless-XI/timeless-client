-- Colours, job names and roles shared by both windows.

local S = {}

-- The modern UI's palette (its theme pack's charcoal, surface, bronze, gold, focus, text, muted), plus ours.
S.TEXT    = { 232, 226, 212, 255 }
S.MUTED   = { 160, 168, 176, 235 }
S.FAINT   = { 120, 126, 136, 200 }
S.GOLD    = { 228, 192, 119, 255 }
S.BRONZE  = { 170, 128, 72, 255 }
S.FOCUS   = { 120, 176, 232, 255 }
S.GOOD    = { 120, 214, 140, 255 }
S.BAD     = { 236, 108, 96, 255 }
S.WARN    = { 240, 190, 90, 255 }
S.BOND    = { 230, 150, 200, 255 }

S.CARD    = { 255, 255, 255, 12 }
S.CARD_HI = { 255, 255, 255, 26 }
S.EDGE    = { 255, 255, 255, 30 }
S.EDGE_HI = { 228, 192, 119, 200 }
S.TRACK   = { 0, 0, 0, 110 }
S.SLOT    = { 0, 0, 0, 80 }

S.HP = { 214, 92, 80, 255 }
S.MP = { 92, 148, 220, 255 }
S.TP = { 224, 196, 96, 255 }

S.JOBS = {
  { "WAR", "Warrior" }, { "MNK", "Monk" }, { "WHM", "White Mage" }, { "BLM", "Black Mage" },
  { "RDM", "Red Mage" }, { "THF", "Thief" }, { "PLD", "Paladin" }, { "DRK", "Dark Knight" },
  { "BST", "Beastmaster" }, { "BRD", "Bard" }, { "RNG", "Ranger" }, { "SAM", "Samurai" },
  { "NIN", "Ninja" }, { "DRG", "Dragoon" }, { "SMN", "Summoner" }, { "BLU", "Blue Mage" },
  { "COR", "Corsair" }, { "PUP", "Puppetmaster" }, { "DNC", "Dancer" }, { "SCH", "Scholar" },
  { "GEO", "Geomancer" }, { "RUN", "Rune Fencer" },
}

function S.job_short(id) return (S.JOBS[id] or { "---" })[1] end
function S.job_long(id) return (S.JOBS[id] or { nil, "None" })[2] end

S.RACES = { "Hume", "Hume", "Elvaan", "Elvaan", "Tarutaru", "Tarutaru", "Mithra", "Galka" }

-- Roles: the manager feed names each job's (J role); Companion View guesses from the job and the tank flag.
S.ROLES = {
  { key = "TANK",    label = "Tank",    colour = { 110, 166, 255, 255 } },
  { key = "HEALER",  label = "Healer",  colour = { 116, 226, 140, 255 } },
  { key = "SUPPORT", label = "Support", colour = { 204, 140, 255, 255 } },
  { key = "DPS",     label = "Damage",  colour = { 255, 128, 102, 255 } },
}
S.ROLE = {}
for _, r in ipairs(S.ROLES) do S.ROLE[r.key] = r end

local GUESS = { [7] = "TANK", [13] = "TANK", [3] = "HEALER", [5] = "HEALER", [20] = "HEALER",
  [10] = "SUPPORT", [15] = "SUPPORT", [17] = "SUPPORT" }

function S.role_guess(job, tank)
  if tank then return S.ROLE.TANK end
  return S.ROLE[GUESS[job] or "DPS"]
end

-- A colour with its alpha scaled.
function S.fade(c, k)
  return { c[1], c[2], c[3], math.floor((c[4] or 255) * k) }
end

-- "1:05", "12:00", "1:02:03"
function S.clock(seconds)
  seconds = math.max(0, math.floor(seconds or 0))
  if seconds >= 3600 then
    return string.format("%d:%02d:%02d", seconds // 3600, (seconds // 60) % 60, seconds % 60)
  end
  return string.format("%d:%02d", seconds // 60, seconds % 60)
end

return S
