#!/usr/bin/env python3
"""Timeless XI: maps every spell, ability, weapon skill and Misc Book action to its painted Timeless View icon.

Reads crates/vana-client/assets/timeless/{spells,abilities,weaponskills}.tsv (book_data.py's output) and writes
  assets/timeless/icons/map.tsv   kind+id -> icon name  (s<spell id>, a<client ability id>, m<misc key>)
  src/client/timeless/icon_files.rs   the embedded PNGs (only the ones the map uses)
copying the used PNGs from <png dir> (icons/build.py's output) into assets/timeless/icons/.

Usage: icon_map.py <png dir> [<crate dir>]
Re-run after book_data.py, or after adding icons, then rebuild the client.
"""
import os
import re
import shutil
import sys

PNG = sys.argv[1]
CRATE = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "crates", "vana-client")
ASSETS = os.path.join(CRATE, "assets", "timeless")
OUT = os.path.join(ASSETS, "icons")
have = {f[:-4] for f in os.listdir(PNG) if f.endswith(".png")}


def rows(name):
    for line in open(os.path.join(ASSETS, name), encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        yield line.rstrip("\n").split("\t")


ROMAN = {"": 1, "ii": 2, "iii": 3, "iv": 4, "v": 5, "vi": 6}
ELEM = {"fire": "fire", "ice": "ice", "wind": "wind", "air": "wind", "earth": "earth", "lightning": "thunder", "thunder": "thunder",
        "water": "water", "light": "light", "dark": "dark"}


def tier(name):
    m = re.match(r"^(.*?)(?:_(ii|iii|iv|v|vi))?$", name)
    return m.group(1), ROMAN[m.group(2) or ""]


def spell_icon(sid, name, jobs):
    base, t = tier(name)
    nuke = {"fire": "sp_fire", "blizzard": "sp_blizzard", "aero": "sp_aero", "stone": "sp_stone", "thunder": "sp_thunder", "water": "sp_water", "banish": "sp_banish"}
    ga = {"firaga": "sp_firaga", "blizzaga": "sp_blizzaga", "aeroga": "sp_aeroga", "stonega": "sp_stonega", "thundaga": "sp_thunderga", "waterga": "sp_waterga", "banishga": "sp_banishga"}
    if base in nuke:
        return f"{nuke[base]}{min(t, 4)}"
    if base in ga:
        return f"{ga[base]}{min(t, 3)}"
    ancient = {"flare": "sp_fire4", "freeze": "sp_blizzard4", "tornado": "sp_aero4", "quake": "sp_stone4", "burst": "sp_thunder4", "flood": "sp_water4"}
    debuff = {"burn": "sp_fire1", "frost": "sp_blizzard1", "choke": "sp_aero1", "rasp": "sp_stone1", "shock": "sp_thunder1", "drown": "sp_water1"}
    if base in ancient:
        return ancient[base]
    if base in debuff:
        return debuff[base]
    if base == "cure":
        return f"sp_cure{min(t, 5)}"
    if base == "curaga":
        return f"sp_curaga{min(t, 3)}"
    if base == "bio":
        return f"sp_bio{min(t, 3)}"
    simple = {"raise": "sp_raise", "reraise": "sp_reraise", "protect": "sp_protect", "protectra": "sp_protect", "shell": "sp_shell", "shellra": "sp_shell",
              "haste": "sp_haste", "slow": "sp_slow", "refresh": "sp_refresh", "regen": "sp_regen", "dia": "sp_dia", "diaga": "sp_dia", "holy": "sp_holy",
              "flash": "sp_flash", "sneak": "sp_sneak", "invisible": "sp_invisible", "deodorize": "sp_deodorize", "stoneskin": "sp_stoneskin",
              "blink": "sp_blink", "aquaveil": "sp_aquaveil", "phalanx": "sp_phalanx", "warp": "sp_warp", "retrace": "sp_warp", "escape": "sp_warp",
              "sleep": "sp_sleep", "sleepga": "sp_sleep", "repose": "sp_sleep", "poison": "sp_poison", "poisonga": "sp_poison", "paralyze": "sp_paralyze",
              "silence": "sp_silence", "blind": "sp_blind", "gravity": "sp_gravity", "bind": "sp_bind", "dispel": "sp_dispel", "stun": "sp_stun",
              "drain": "sp_drain", "aspir": "sp_aspir", "tractor": "sp_tractor", "erase": "sp_cleanse", "poisona": "sp_cleanse", "paralyna": "sp_cleanse",
              "blindna": "sp_cleanse", "silena": "sp_cleanse", "stona": "sp_cleanse", "viruna": "sp_cleanse", "cursna": "sp_cleanse",
              "tonko": "sp_tonko", "monomi": "sp_tonko", "magic_finale": "sp_song_finale", "goddesss_hymnus": "sp_song_hymnus",
              "knights_minne": "sp_song_minne", "carbuncle": "sp_avatar_carbuncle", "ifrit": "sp_avatar_ifrit", "shiva": "sp_avatar_shiva",
              "garuda": "sp_avatar_garuda", "titan": "sp_avatar_titan", "ramuh": "sp_avatar_ramuh", "leviathan": "sp_avatar_leviathan",
              "fenrir": "sp_avatar_fenrir", "diabolos": "sp_avatar_diabolos"}
    if base in simple:
        return simple[base]
    if name.startswith("absorb-"):
        return "sp_absorb"
    if name.startswith("teleport-"):
        return "sp_teleport"
    m = re.match(r"^en(fire|blizzard|aero|stone|thunder|water)$", name)
    if m:
        return "sp_en_" + {"fire": "fire", "blizzard": "ice", "aero": "wind", "stone": "earth", "thunder": "thunder", "water": "water"}[m.group(1)]
    m = re.match(r"^bar(fire|blizzard|aero|stone|thunder|water)(a|ra)?$", name)
    if m:
        return "sp_bar_" + {"fire": "fire", "blizzard": "ice", "aero": "wind", "stone": "earth", "thunder": "thunder", "water": "water"}[m.group(1)]
    if name.startswith("bar"):
        return "sp_bar_status"
    spikes = {"blaze_spikes": "sp_spikes_fire", "ice_spikes": "sp_spikes_ice", "shock_spikes": "sp_spikes_thunder", "dread_spikes": "sp_spikes_dark"}
    if name in spikes:
        return spikes[name]
    m = re.match(r"^(fire|ice|air|earth|thunder|water|light|dark)_spirit$", name)
    if m:
        return "sp_spirit_" + ELEM[m.group(1)]
    m = re.match(r"^(fire|ice|wind|earth|lightning|water|light|dark)_(carol|threnody)(_ii)?$", name)
    if m:
        return f"sp_{m.group(2)}_{ELEM[m.group(1)]}"
    for kind in ("minuet", "paeon", "ballad", "march", "madrigal", "mambo", "etude", "requiem", "elegy", "lullaby", "mazurka", "prelude"):
        if name.endswith("_" + kind) or ("_" + kind + "_") in name:
            return "sp_song_" + kind
    if re.search(r"_(minne|virelai|operetta|capriccio|gavotte|aubade|pastoral|fantasia|sirvente|dirge|hymnus)", name) or name in ("foe_sirvente", "adventurers_dirge", "warding_round"):
        return "sp_song_song"
    if base.startswith("utsusemi"):
        return "sp_utsusemi"
    m = re.match(r"^(katon|hyoton|huton|doton|raiton|suiton)_", name)
    if m:
        return "sp_nin_" + {"katon": "fire", "hyoton": "ice", "huton": "wind", "doton": "earth", "raiton": "thunder", "suiton": "water"}[m.group(1)]
    if re.match(r"^(tonko|monomi)_", name):
        return "sp_tonko"
    if name == "siren":
        return "sp_avatar_garuda"
    if re.match(r"^(jubaku|hojo|kurayami|dokumori)_", name):
        return "sp_nin_enfeeble"
    if sid >= 896:
        return "sp_trust"
    if jobs.startswith("16:") and "," not in jobs:
        return "sp_bluemagic"
    return None


TWO_HOURS = {"mighty_strikes": "war", "hundred_fists": "mnk", "benediction": "whm", "manafont": "blm", "chainspell": "rdm", "perfect_dodge": "thf",
             "invincible": "pld", "blood_weapon": "drk", "familiar": "bst", "soul_voice": "brd", "eagle_eye_shot": "rng", "meikyo_shisui": "sam",
             "mijin_gakure": "nin", "spirit_surge": "drg", "astral_flow": "smn", "azure_lore": "blu", "wild_card": "cor", "overdrive": "pup"}
JOB = {1: "war", 2: "mnk", 3: "whm", 4: "blm", 5: "rdm", 6: "thf", 7: "pld", 8: "drk", 9: "bst", 10: "brd", 11: "rng", 12: "sam", 13: "nin",
       14: "drg", 15: "smn", 16: "blu", 17: "cor", 18: "pup"}
PACTS = {
    "carbuncle": "healing_ruby searing_light poison_nails shining_ruby glittering_ruby meteorite healing_ruby_ii holy_mist soothing_ruby pacifying_ruby",
    "fenrir": "howling_moon moonlit_charge crescent_fang lunar_cry lunar_roar ecliptic_growl ecliptic_howl eclipse_bite heavenward_howl",
    "ifrit": "punch inferno fire_ii fire_iv burning_strike double_punch crimson_howl flaming_crush meteor_strike conflag_strike inferno_howl",
    "titan": "rock_throw earthen_fury stone_ii stone_iv rock_buster megalith_throw earthen_ward mountain_buster geocrush crag_throw earthen_armor",
    "leviathan": "barracuda_dive tidal_wave water_ii water_iv tail_whip slowga spring_water spinning_dive grand_fall tidal_roar soothing_current",
    "garuda": "claw aerial_blast aero_ii aero_iv aerial_armor whispering_wind hastega predator_claws wind_blade fleet_wind hastega_ii",
    "shiva": "axe_kick diamond_dust blizzard_ii blizzard_iv frost_armor sleepga double_slap rush heavenly_strike crystal_blessing diamond_storm",
    "ramuh": "shock_strike judgment_bolt thunder_ii thunder_iv thunderspark rolling_thunder lightning_armor chaotic_strike thunderstorm volt_strike",
    "diabolos": "camisado ruinous_omen somnolence nightmare ultimate_terror noctoshield dream_shroud nether_blast night_terror pavor_nocturnus",
}
PACT = {n: k for k, v in PACTS.items() for n in v.split()}
UNIQUE = {"provoke", "berserk", "defender", "warcry", "aggressor", "boost", "focus", "dodge", "chakra", "divine_seal", "elemental_seal", "convert",
          "steal", "sneak_attack", "trick_attack", "flee", "hide", "sentinel", "holy_circle", "shield_bash", "cover", "rampart", "arcane_circle",
          "last_resort", "souleater", "weapon_bash", "charm", "reward", "call_beast", "fight", "scavenge", "sharpshot", "barrage", "camouflage",
          "meditate", "third_eye", "hasso", "seigan", "jump", "high_jump", "super_jump", "call_wyvern", "ancient_circle", "release", "assault",
          "retreat", "phantom_roll", "double_up", "quick_draw", "random_deal", "activate", "deploy", "repair", "mug", "feint", "counterstance",
          "mantra", "chi_blast", "warding_circle", "shadowbind", "spirit_link", "dismiss"}
ALIAS = {"sic": "ja_fight", "tame": "ja_charm", "heel": "ja_pet_cmd", "stay": "ja_pet_cmd", "leave": "ja_pet_cmd", "gauge": "ja_job_bst",
         "deactivate": "ja_activate", "retrieve": "ja_repair", "role_reversal": "ja_repair", "ventriloquy": "ja_deploy", "fold": "ja_double_up",
         "snake_eye": "ja_phantom_roll", "blood_pact_rage": "ja_assault", "blood_pact_ward": "ja_release", "corsairs_roll": "ja_phantom_roll",
         "angon": "ja_jump", "deep_breathing": "ja_call_wyvern", "blade_bash": "ja_hasso", "shikikoyo": "ja_meditate", "sange": "ja_job_nin",
         "unlimited_shot": "ja_sharpshot", "stealth_shot": "ja_camouflage", "flashy_shot": "ja_sharpshot", "nightingale": "ja_job_brd",
         "troubadour": "ja_job_brd", "dark_seal": "ja_arcane_circle", "diabolic_eye": "ja_aggressor", "fealty": "ja_sentinel", "chivalry": "ja_rampart",
         "devotion": "ja_convert", "martyr": "ja_convert", "formless_strikes": "ja_boost", "assassins_charge": "ja_sneak_attack",
         "warriors_charge": "ja_berserk", "tomahawk": "ja_job_war", "burst_affinity": "ja_job_blu", "chain_affinity": "ja_job_blu",
         "convergence": "ja_job_blu", "diffusion": "ja_job_blu", "perfect_defense": "ja_2h_smn", "pet_commands": "ja_pet_cmd",
         "killer_instinct": "ja_familiar", "feral_howl": "ja_provoke"}


def ability_icon(name, job):
    if name in TWO_HOURS:
        return "ja_2h_" + TWO_HOURS[name]
    if name in ALIAS:
        return ALIAS[name]
    if name in UNIQUE:
        return "ja_" + name
    if name in PACT:
        return "sp_avatar_" + PACT[name]
    if name.endswith("_roll"):
        return "ja_phantom_roll"
    if name.endswith("_maneuver"):
        return "ja_maneuver"
    if job == 17 and name.endswith("_shot"):
        return "ja_quick_draw"
    if job == 9:
        return "ja_ready"
    if job == 15:
        return "ja_job_smn"
    return "ja_job_" + JOB[job]


MISC = {"attack": "misc_attack", "ranged": "misc_ranged", "check": "misc_check", "assist": "misc_assist", "lockon": "misc_lockon",
        "follow": "misc_follow", "rest": "misc_rest", "sit": "misc_sit", "widescan": "misc_widescan", "map": "misc_map", "region": "misc_region",
        "status": "misc_status", "equipment": "misc_equipment", "items": "misc_inventory", "party": "misc_party", "trade": "misc_trade",
        "search": "misc_search", "linkshell": "misc_linkshell", "synthesis": "misc_synthesis", "missions": "misc_missions", "quests": "misc_quests",
        "key_items": "misc_keyitems", "disengage": "misc_disengage", "gear": "misc_equipment", "fishing": "helm_fishing",
        "mining": "helm_mining", "logging": "helm_logging", "harvesting": "helm_harvesting", "excavation": "helm_excavation"}

out, missing, unmapped = [], set(), []
for r in rows("spells.tsv"):
    sid, jobs, name = int(r[0]), r[1], r[3]
    ic = spell_icon(sid, name, jobs)
    if ic is None:
        unmapped.append(name)
        continue
    out.append((f"s{sid}", ic))
for r in rows("abilities.tsv"):
    out.append((f"a{int(r[0])}", ability_icon(r[4], int(r[1]))))
for r in rows("weaponskills.tsv"):
    out.append((f"a{int(r[0])}", "ws_" + r[1].replace("_", "").replace("handtohand", "h2h")))
for k, v in MISC.items():
    out.append((f"m{k}", v))
used = sorted({ic for _, ic in out})
for ic in used:
    if ic not in have:
        missing.add(ic)
if missing:
    sys.exit(f"missing icons: {sorted(missing)}")
os.makedirs(OUT, exist_ok=True)
for f in os.listdir(OUT):
    if f.endswith(".png"):
        os.remove(os.path.join(OUT, f))
for ic in used:
    shutil.copyfile(os.path.join(PNG, ic + ".png"), os.path.join(OUT, ic + ".png"))
with open(os.path.join(OUT, "map.tsv"), "w", encoding="utf-8", newline="\n") as f:
    f.write("# kind+id\ticon (generated by tools/timeless/icon_map.py: s spell id, a client ability id, m Misc Book key)\n")
    for k, v in out:
        f.write(f"{k}\t{v}\n")
rs = os.path.join(CRATE, "src", "client", "timeless", "icon_files.rs")
with open(rs, "w", encoding="utf-8", newline="\n") as f:
    f.write("// Timeless XI: generated by tools/timeless/icon_map.py; the painted Timeless View icons (128 x 128 PNG).\n\n")
    f.write("pub const FILES: &[(&str, &[u8])] = &[\n")
    for ic in used:
        f.write(f'    ("{ic}", include_bytes!("../../../assets/timeless/icons/{ic}.png")),\n')
    f.write("];\n")
print(f"{len(out)} entries, {len(used)} icons; unmapped spells (game icon kept): {len(unmapped)}")
if unmapped:
    print("  " + " ".join(unmapped))
