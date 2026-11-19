//! Timeless XI: the Actions window's data (drawn by modern/tlbook.rs). Every job ability, weapon skill and spell this
//! server allows, from its own tables (assets/timeless/*.tsv, made by tools/timeless/book_data.py from the Phoenix
//! tree: the era module's levels, only the enabled expansions, the 75 cap), and the general actions of the Misc Book.

use super::super::lua::actions::{Action, MENUS, apply};
use super::super::{World, magic, textcmd};
use std::collections::HashMap;
use std::sync::OnceLock;

/// The jobs' short names by job id (1 = WAR).
pub const JOBS: [&str; 23] = [
    "", "WAR", "MNK", "WHM", "BLM", "RDM", "THF", "PLD", "DRK", "BST", "BRD", "RNG", "SAM", "NIN", "DRG", "SMN", "BLU",
    "COR", "PUP", "DNC", "SCH", "GEO", "RUN",
];

/// The window's books (tabs) and the Ability Book's pages.
pub const BOOKS: [&str; 5] = ["Ability Book", "Spell Book", "Item Book", "Misc Book", "Commands"];
pub const ABILITY_PAGES: [&str; 3] = ["Job Abilities", "Weapon Skills", "Blood Pacts"];

/// The Blood Pact lists (ability record types under Pet Commands): Rage, Ward.
pub const PACTS: [u16; 2] = [6, 10];

/// The Spell Book's pages: the spell record's type and its name.
pub const MAGIC: [(u16, &str); 7] = [
    (1, "White Magic"),
    (2, "Black Magic"),
    (3, "Summoning"),
    (4, "Ninjutsu"),
    (5, "Songs"),
    (6, "Blue Magic"),
    // Timeless XI: the Trusts the server has (its Companions, formerly Recruits, are built on them), so their icon can
    // go on a bar.
    (8, "Companion"),
];

/// The Weapon Skills page's weapon types: the server's skill name and its label.
pub const WEAPONS: [(&str, &str); 14] = [
    ("hand_to_hand", "Hand-to-Hand"),
    ("dagger", "Dagger"),
    ("sword", "Sword"),
    ("great_sword", "Great Sword"),
    ("axe", "Axe"),
    ("great_axe", "Great Axe"),
    ("scythe", "Scythe"),
    ("polearm", "Polearm"),
    ("katana", "Katana"),
    ("great_katana", "Great Katana"),
    ("club", "Club"),
    ("staff", "Staff"),
    ("archery", "Archery"),
    ("marksmanship", "Marksmanship"),
];

/// A job ability: the client's ability id (0x200 + the server's), its job and level.
#[derive(Clone, Copy, Debug)]
pub struct JobAbility {
    pub id: u16,
    pub job: u8,
    pub level: u8,
    /// The server's range in yalms (0: on ourselves).
    pub range: f32,
}

/// A weapon skill: id, weapon type (WEAPONS index), skill level, the jobs that may use it (none: any with the skill),
/// unlocked by a quest.
#[derive(Clone, Debug)]
pub struct WeaponSkill {
    pub id: u16,
    pub weapon: usize,
    pub skill: u16,
    pub jobs: Vec<u8>,
    pub quest: bool,
    /// The server's range in yalms.
    pub range: f32,
}

/// A spell and each job's level.
#[derive(Clone, Debug)]
pub struct Spell {
    pub id: u16,
    pub levels: Vec<(u8, u8)>,
    /// The server's range in yalms (0: the server's table gives none).
    pub range: f32,
}

pub struct Data {
    pub abilities: Vec<JobAbility>,
    pub weapon_skills: Vec<WeaponSkill>,
    pub spells: Vec<Spell>,
}

fn rows(text: &str) -> impl Iterator<Item = Vec<&str>> {
    text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| l.split('\t').collect())
}

fn job_of(abbr: &str) -> Option<u8> {
    JOBS.iter().position(|j| !j.is_empty() && j.eq_ignore_ascii_case(abbr)).map(|p| p as u8)
}

fn parse(abilities: &str, weapon_skills: &str, spells: &str) -> Data {
    let abilities = rows(abilities)
        .filter_map(|f| {
            Some(JobAbility {
                id: f.first()?.parse().ok()?,
                job: f.get(1)?.parse().ok()?,
                level: f.get(2)?.parse().ok()?,
                range: f.get(3).and_then(|r| r.parse().ok()).unwrap_or(0.0),
            })
        })
        .collect();
    let weapon_skills = rows(weapon_skills)
        .filter_map(|f| {
            let skill_name = *f.get(1)?;
            Some(WeaponSkill {
                id: f.first()?.parse().ok()?,
                weapon: WEAPONS.iter().position(|w| w.0 == skill_name)?,
                skill: f.get(2)?.parse().ok()?,
                jobs: f.get(3)?.split(',').filter_map(job_of).collect(),
                quest: f.get(4) == Some(&"1"),
                range: f.get(5).and_then(|r| r.parse().ok()).unwrap_or(0.0),
            })
        })
        .collect();
    let spells = rows(spells)
        .filter_map(|f| {
            let levels = f
                .get(1)?
                .split(',')
                .filter_map(|p| {
                    let (j, l) = p.split_once(':')?;
                    Some((j.parse().ok()?, l.parse().ok()?))
                })
                .collect();
            let range = f.get(2).and_then(|r| r.parse().ok()).unwrap_or(0.0);
            Some(Spell { id: f.first()?.parse().ok()?, levels, range })
        })
        .collect();
    Data { abilities, weapon_skills, spells }
}

/// The server's range of an action, by kind (0 spell, 1 job ability or weapon skill by name-table id): None when
/// the tables have none (or 0, on ourselves).
pub fn range(kind: u8, id: u16) -> Option<f32> {
    static R: OnceLock<HashMap<(u8, u16), f32>> = OnceLock::new();
    let r = R.get_or_init(|| {
        let d = data();
        let mut m = HashMap::new();
        for s in &d.spells {
            m.insert((0, s.id), s.range);
        }
        for a in &d.abilities {
            m.insert((1, a.id), a.range);
        }
        for w in &d.weapon_skills {
            m.insert((1, w.id), w.range);
        }
        m
    });
    r.get(&(kind, id)).copied().filter(|&x| x > 0.0)
}

pub fn data() -> &'static Data {
    static D: OnceLock<Data> = OnceLock::new();
    D.get_or_init(|| {
        parse(
            include_str!("../../../assets/timeless/abilities.tsv"),
            include_str!("../../../assets/timeless/weaponskills.tsv"),
            include_str!("../../../assets/timeless/spells.tsv"),
        )
    })
}

/// The jobs with job abilities in the book, in job order.
pub fn ability_jobs() -> Vec<u8> {
    let mut j: Vec<u8> = data().abilities.iter().map(|a| a.job).collect();
    j.sort_unstable();
    j.dedup();
    j
}

/// A job ability's level by job (for a tooltip), from the book.
pub fn ability_levels(id: u16) -> Vec<(u8, u8)> {
    data().abilities.iter().filter(|a| a.id == id).map(|a| (a.job, a.level)).collect()
}

/// The spells of a page (a spell record type), in id order.
pub fn spells_of(w: &World, ty: u16) -> Vec<&'static Spell> {
    let Some(acts) = w.actions.as_ref() else { return Vec::new() };
    data().spells.iter().filter(|s| acts.spell(s.id).is_some_and(|r| r.kind == ty)).collect()
}

/// "WHM Lv. 1 · RDM Lv. 3" for the given (job, level) pairs.
pub fn levels_text(levels: &[(u8, u8)]) -> String {
    levels
        .iter()
        .map(|(j, l)| format!("{} Lv. {l}", JOBS.get(*j as usize).copied().unwrap_or("?")))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Whether we have learned a job ability or weapon skill (the server's 0x0AC bits; a weapon skill: in the list
/// for our weapon and skill now), or a spell.
pub fn has_ability(w: &World, id: u16) -> bool {
    if (0x200..0x400).contains(&id) {
        let bit = (id - 0x200) as usize;
        return w.commands_known.get(64 + bit / 8).is_some_and(|b| b & (1 << (bit % 8)) != 0);
    }
    let Some(i) = super::hotbar::ability_index(w, id) else { return false };
    magic::ability_rows(w, 3).contains(&i)
}

pub fn has_spell(w: &World, id: u16) -> bool {
    textcmd::spell_learned(w, id)
}

/// A general action of the Misc Book: its key (saved in a slot), label, the art pack's icon and what it does.
pub struct Misc {
    pub key: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub help: &'static str,
    what: Do,
}

enum Do {
    /// A main menu row by its name in lua::actions::MENUS (as a pick there opens it).
    Menu(&'static str),
    /// A command line, run as the menus run theirs.
    Line(&'static str),
    Scan,
}

pub const MISC: [Misc; 28] = [
    Misc {
        key: "attack",
        label: "Attack",
        icon: "attack",
        help: "Engages your target; while you fight, disengages.",
        what: Do::Line(""),
    },
    // Timeless XI: Disengage as its own action (the Attack slot also turns into it while fighting).
    Misc {
        key: "disengage",
        label: "Disengage",
        icon: "disengage",
        help: "Stops attacking your target.",
        what: Do::Line("/attackoff"),
    },
    Misc {
        key: "ranged",
        label: "Ranged Attack",
        icon: "range",
        help: "Shoots your target.",
        what: Do::Line("/ra <t>"),
    },
    Misc { key: "check", label: "Check", icon: "check", help: "Checks your target.", what: Do::Line("/check <t>") },
    Misc {
        key: "assist",
        label: "Assist",
        icon: "assist",
        help: "Targets what your target is fighting.",
        what: Do::Line("/assist <t>"),
    },
    Misc {
        key: "lockon",
        label: "Lock On",
        icon: "lock",
        help: "Locks your view on your target.",
        what: Do::Line("/lockon"),
    },
    Misc {
        key: "follow",
        label: "Follow",
        icon: "follow",
        help: "Follows your target.",
        what: Do::Line("/follow <t>"),
    },
    Misc {
        key: "rest",
        label: "Rest",
        icon: "rest",
        help: "Rests (/heal) to recover HP and MP.",
        what: Do::Line("/heal"),
    },
    Misc { key: "sit", label: "Sit", icon: "emote", help: "Sits down.", what: Do::Line("/sit") },
    Misc {
        key: "widescan",
        label: "Wide Scan",
        icon: "find",
        help: "Lists what is around you (jobs with Wide Scan).",
        what: Do::Scan,
    },
    Misc { key: "map", label: "Map", icon: "map", help: "Opens the map.", what: Do::Menu("map") },
    Misc {
        key: "region",
        label: "Region Info",
        icon: "pin",
        help: "Opens Region Info: conquest and the area's nation.",
        what: Do::Menu("region"),
    },
    Misc { key: "status", label: "Status", icon: "status", help: "Opens your status.", what: Do::Menu("status") },
    Misc {
        key: "equipment",
        label: "Equipment",
        icon: "equipment",
        help: "Opens your equipment.",
        what: Do::Menu("equipment"),
    },
    Misc { key: "items", label: "Items", icon: "items", help: "Opens your items.", what: Do::Menu("items") },
    Misc { key: "party", label: "Party", icon: "party", help: "Opens the party menu.", what: Do::Menu("party") },
    Misc { key: "trade", label: "Trade", icon: "trade", help: "Offers your target a trade.", what: Do::Menu("trade") },
    Misc {
        key: "search",
        label: "Search",
        icon: "inspect",
        help: "Opens the player search.",
        what: Do::Menu("search"),
    },
    Misc {
        key: "linkshell",
        label: "Linkshell",
        icon: "linkshell",
        help: "Opens the linkshell menu.",
        what: Do::Menu("linkshell"),
    },
    Misc { key: "synthesis", label: "Synthesis", icon: "craft", help: "Opens synthesis.", what: Do::Menu("synthesis") },
    Misc {
        key: "missions",
        label: "Missions",
        icon: "scroll",
        help: "Opens your missions.",
        what: Do::Menu("missions"),
    },
    Misc { key: "quests", label: "Quests", icon: "quest_log", help: "Opens your quests.", what: Do::Menu("quests") },
    Misc {
        key: "key_items",
        label: "Key Items",
        icon: "key",
        help: "Opens your key items.",
        what: Do::Menu("key_items"),
    },
    // Timeless XI: fishing and HELM. The gathering points take the tool as a trade, as the game's macros do it.
    Misc {
        key: "fishing",
        label: "Fishing",
        icon: "materials",
        help: "Casts your line (rod and bait equipped).",
        what: Do::Line("/fish"),
    },
    Misc {
        key: "mining",
        label: "Mining",
        icon: "materials",
        help: "Uses a Pickaxe on the targeted Mining Point.",
        what: Do::Line("/item \"Pickaxe\" <t>"),
    },
    Misc {
        key: "logging",
        label: "Logging",
        icon: "materials",
        help: "Uses a Hatchet on the targeted Logging Point.",
        what: Do::Line("/item \"Hatchet\" <t>"),
    },
    Misc {
        key: "harvesting",
        label: "Harvesting",
        icon: "materials",
        help: "Uses a Sickle on the targeted Harvesting Point.",
        what: Do::Line("/item \"Sickle\" <t>"),
    },
    Misc {
        key: "excavation",
        label: "Excavation",
        icon: "materials",
        help: "Uses a Pickaxe on the targeted Excavation Point.",
        what: Do::Line("/item \"Pickaxe\" <t>"),
    },
];

/// Whether a Misc action opens a game window (a second press closes it).
pub fn opens_window(key: &str) -> bool {
    misc(key).is_some_and(|m| matches!(m.what, Do::Menu(_)))
}

pub fn misc(key: &str) -> Option<&'static Misc> {
    MISC.iter().find(|m| m.key == key)
}

/// Runs a Misc Book action.
pub fn run_misc(w: &mut World, key: &str) {
    let Some(m) = misc(key) else { return };
    match m.what {
        Do::Line("") => super::hotbar::run(w, &super::hotbar::Slot::Attack),
        Do::Line(line) => textcmd::run(w, line, textcmd::Mode::Internal),
        Do::Scan => apply(w, None, Action::Scan),
        Do::Menu(name) => {
            if let Some(&(_, page, child)) = MENUS.iter().find(|(n, _, _)| *n == name) {
                apply(w, None, Action::Menu(page, child));
            }
        }
    }
}

/// The open Actions window: its book, the Ability Book's page, the job, weapon type and magic page shown.
#[derive(Clone, Copy, Debug, Default)]
pub struct BookState {
    pub book: usize,
    pub page: usize,
    pub job: u8,
    pub weapon: usize,
    pub magic: usize,
    /// The Blood Pacts page's list (PACTS index).
    pub pact: usize,
}

/// The Ability Book's pages by job: the job's abilities in level order.
pub fn job_abilities(job: u8) -> Vec<JobAbility> {
    let mut v: Vec<JobAbility> = data().abilities.iter().filter(|a| a.job == job).copied().collect();
    v.sort_by_key(|a| (a.level, a.id));
    v
}

/// The Weapon Skills page of a weapon type, in skill order.
pub fn weapon_skills(weapon: usize) -> Vec<&'static WeaponSkill> {
    let mut v: Vec<&WeaponSkill> = data().weapon_skills.iter().filter(|s| s.weapon == weapon).collect();
    v.sort_by_key(|s| (s.skill, s.id));
    v
}

/// Labels by key, for the slots' names.
pub fn misc_names() -> &'static HashMap<&'static str, &'static str> {
    static M: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    M.get_or_init(|| MISC.iter().map(|m| (m.key, m.label)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_tables_parse() {
        let d = data();
        // 18 jobs with abilities (Base, RotZ, CoP, ToAU), each with its two-hour at level 1.
        assert_eq!(ability_jobs().len(), 18);
        for j in ability_jobs() {
            assert!(job_abilities(j).first().is_some_and(|a| a.level == 1), "{}", JOBS[j as usize]);
        }
        // Provoke: WAR 5 (server id 35 -> client 0x223).
        assert!(d.abilities.iter().any(|a| a.id == 0x223 && a.job == 1 && a.level == 5));
        // Cure: WHM 1, RDM 3, PLD 5; no SCH (WotG off).
        let cure = d.spells.iter().find(|s| s.id == 1).unwrap();
        assert_eq!(cure.levels, vec![(3, 1), (5, 3), (7, 5)]);
        assert_eq!(levels_text(&cure.levels), "WHM Lv. 1 · RDM Lv. 3 · PLD Lv. 5");
        // Every weapon type has skills, all at the 75 cap's skill or under.
        for (k, w) in WEAPONS.iter().enumerate() {
            assert!(!weapon_skills(k).is_empty(), "{}", w.1);
        }
        assert!(d.weapon_skills.iter().all(|s| s.skill <= 276));
        // Fast Blade (32): sword.
        assert!(d.weapon_skills.iter().any(|s| s.id == 32 && WEAPONS[s.weapon].0 == "sword"));
    }

    #[test]
    fn misc_menus_exist() {
        for m in &MISC {
            if let Do::Menu(name) = m.what {
                assert!(MENUS.iter().any(|(n, _, _)| *n == name), "{name}");
            }
        }
        assert_eq!(misc_names()["widescan"], "Wide Scan");
    }
}
