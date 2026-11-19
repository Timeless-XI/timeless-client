#!/usr/bin/env python3
"""Timeless XI: the Actions window's tables (crates/vana-client/assets/timeless/*.tsv) from a Phoenix server tree.

The server's own data, so the books show what this server allows:
  - abilities: sql/abilities.sql, then the era module's UPDATEs (modules/era/sql/<rov|abyssea|soa|wotg>, in
    modules/init.txt order), kept when the content tag's expansion is enabled in settings/main.lua, the job is one
    of the enabled expansions' jobs and the level is 1..MAX_LEVEL;
  - weapon skills: data/weapon_skills.yaml with modules/era/data/weapon_skills.yaml over it (a skill's fields
    replace the base ones), kept with a skill level up to the 75 cap's best (276);
  - spells: sql/spell_list.sql (jobs = one level byte per job, job 1 first) with the era UPDATEs, kept like the
    abilities.

Usage: book_data.py <Phoenix dir> <out dir>
"""
import os
import re
import sys

JOBS = ["", "WAR", "MNK", "WHM", "BLM", "RDM", "THF", "PLD", "DRK", "BST", "BRD", "RNG", "SAM", "NIN", "DRG", "SMN",
        "BLU", "COR", "PUP", "DNC", "SCH", "GEO", "RUN"]
# The expansion each job came with (jobs not listed: base game).
JOB_TAG = {16: "TOAU", 17: "TOAU", 18: "TOAU", 19: "WOTG", 20: "WOTG", 21: "SOA", 22: "SOA"}
WS_CAP = 276


def settings(phx):
    text = open(os.path.join(phx, "settings", "main.lua"), encoding="utf-8", errors="replace").read()
    on = {None}
    for tag in ["ROTZ", "COP", "TOAU", "WOTG", "ACP", "AMK", "ASA", "ABYSSEA", "SOA", "ROV", "TVR", "VOIDWATCH"]:
        m = re.search(r"ENABLE_%s\s*=\s*(\d+)" % tag, text)
        if m and m.group(1) != "0":
            on.add(tag)
    m = re.search(r"MAX_LEVEL\s*=\s*(\d+)", text)
    return on, int(m.group(1)) if m else 75


def split_top(s, sep=","):
    """Splits on `sep` outside quotes and parentheses."""
    out, cur, depth, q = [], "", 0, None
    for ch in s:
        if q:
            cur += ch
            if ch == q:
                q = None
        elif ch in "'\"":
            q = ch
            cur += ch
        elif ch == "(":
            depth += 1
            cur += ch
        elif ch == ")":
            depth -= 1
            cur += ch
        elif ch == sep and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def value(tok):
    tok = tok.strip()
    if tok.upper() == "NULL":
        return None
    if tok[:1] in "'\"":
        return tok[1:-1].replace("\\'", "'")
    if tok.lower().startswith("0x"):
        return bytes.fromhex(tok[2:])
    try:
        return int(tok)
    except ValueError:
        try:
            return float(tok)
        except ValueError:
            return tok  # @ELEMENT_LIGHT and the like


def load_table(path, table):
    text = open(path, encoding="utf-8", errors="replace").read()
    m = re.search(r"CREATE TABLE `%s` \((.*?)\n\)" % table, text, re.S)
    cols = re.findall(r"^\s*`(\w+)`", m.group(1), re.M)
    rows = []
    for vm in re.finditer(r"INSERT INTO `%s` VALUES \((.*)\);" % table, text):
        vals = [value(v) for v in split_top(vm.group(1))]
        rows.append(dict(zip(cols, vals)))
    return rows


def apply_updates(rows, path, table):
    text = open(path, encoding="utf-8", errors="replace").read()
    text = re.sub(r"--[^\n]*", "", text)
    for m in re.finditer(r"UPDATE\s+`?%s`?\s+SET\s+(.*?)\s+WHERE\s+(.*?);" % table, text, re.S | re.I):
        sets = {}
        for a in split_top(m.group(1)):
            k, v = a.split("=", 1)
            sets[k.strip().strip("`")] = value(v)
        conds = []
        for c in re.split(r"\s+AND\s+", m.group(2).strip(), flags=re.I):
            im = re.match(r"`?(\w+)`?\s+IN\s*\((.*)\)", c.strip(), re.I | re.S)
            if im:
                conds.append((im.group(1), [value(v) for v in split_top(im.group(2))]))
                continue
            em = re.match(r"`?(\w+)`?\s*=\s*(.+)", c.strip(), re.S)
            if em:
                conds.append((em.group(1), [value(em.group(2))]))
        for r in rows:
            if conds and all(r.get(k) in vs for k, vs in conds):
                r.update(sets)


def era_sql_dirs(phx):
    lines = open(os.path.join(phx, "modules", "init.txt"), encoding="utf-8").read().splitlines()
    dirs = [l.strip() for l in lines if l.strip().startswith("era/sql") and not l.strip().startswith("#")]
    return [os.path.join(phx, "modules", d) for d in dirs if os.path.isdir(os.path.join(phx, "modules", d))]


def parse_ws_yaml(path):
    """The weapon_skills: map of a weapon skills yaml: {name: {field: value}} (ints, words, [lists])."""
    out, cur = {}, None
    inside = False
    for raw in open(path, encoding="utf-8"):
        line = raw.split("#", 1)[0].rstrip()
        if not line.strip():
            continue
        if not line.startswith(" "):
            inside = line.startswith("weapon_skills:")
            continue
        if not inside:
            continue
        ind = len(line) - len(line.lstrip())
        key, _, rest = line.strip().partition(":")
        rest = rest.strip()
        if ind == 2:
            cur = out.setdefault(key, {})
            if rest.startswith("{"):
                for kv in split_top(rest.strip("{}")):
                    k, _, v = kv.partition(":")
                    cur[k.strip()] = yaml_value(v.strip())
        elif ind >= 4 and cur is not None:
            cur[key] = yaml_value(rest)
    return out


def yaml_value(v):
    if v.startswith("["):
        return [x.strip() for x in v.strip("[]").split(",") if x.strip()]
    try:
        return int(v)
    except ValueError:
        return v


def main(phx, out):
    on, cap = settings(phx)
    jobs_on = [j for j in range(1, 23) if JOB_TAG.get(j) in on]
    os.makedirs(out, exist_ok=True)
    era = era_sql_dirs(phx)

    abilities = load_table(os.path.join(phx, "sql", "abilities.sql"), "abilities")
    spells = load_table(os.path.join(phx, "sql", "spell_list.sql"), "spell_list")
    for d in era:
        for f in sorted(os.listdir(d)):
            if f.endswith(".sql"):
                apply_updates(abilities, os.path.join(d, f), "abilities")
                apply_updates(spells, os.path.join(d, f), "spell_list")

    n = 0
    with open(os.path.join(out, "abilities.tsv"), "w", encoding="utf-8", newline="\n") as f:
        f.write("# client ability id (0x200 + the server's abilityId)\tjob\tlevel\trange (yalms; 0 self)\tname (generated by tools/timeless/book_data.py)\n")
        # Level 0 is each job's two-hour (Mighty Strikes ...): usable from level 1.
        for a in abilities:
            if a["level"] == 0:
                a["level"] = 1
        for a in sorted(abilities, key=lambda r: (r["job"], r["level"], r["abilityId"])):
            if a["content_tag"] in on and a["job"] in jobs_on and 1 <= a["level"] <= cap:
                f.write("%d\t%d\t%d\t%g\t%s\n" % (0x200 + a["abilityId"], a["job"], a["level"], float(a["range"] or 0), a["name"]))
                n += 1
    print("abilities", n)

    n = 0
    with open(os.path.join(out, "spells.tsv"), "w", encoding="utf-8", newline="\n") as f:
        f.write("# spell id\tjob:level,... (generated by tools/timeless/book_data.py)\trange (yalms; spell_range / 10)\tname\n")
        for s in sorted(spells, key=lambda r: r["spellid"]):
            if s["content_tag"] not in on or not isinstance(s["jobs"], bytes):
                continue
            lv = [(j, s["jobs"][j - 1]) for j in jobs_on if j - 1 < len(s["jobs"]) and 1 <= s["jobs"][j - 1] <= cap]
            if lv:
                f.write("%d\t%s\t%g\t%s\n" % (s["spellid"], ",".join("%d:%d" % p for p in lv), (s["spell_range"] or 0) / 10.0, s["name"]))
                n += 1
    print("spells", n)

    ws = parse_ws_yaml(os.path.join(phx, "data", "weapon_skills.yaml"))
    over = os.path.join(phx, "modules", "era", "data", "weapon_skills.yaml")
    if os.path.exists(over):
        for k, v in parse_ws_yaml(over).items():
            if k in ws:
                ws[k].update(v)
    n = 0
    with open(os.path.join(out, "weaponskills.tsv"), "w", encoding="utf-8", newline="\n") as f:
        f.write("# weapon skill id\tskill\tskill level\tjobs (empty: any job with the skill)\tquest (1: unlocked by a quest)\trange (yalms)\tname (generated by tools/timeless/book_data.py)\n")
        for name, w in sorted(ws.items(), key=lambda kv: (str(kv[1].get("skill")), kv[1].get("level", 999), kv[1].get("id", 0))):
            lvl = w.get("level")
            if not isinstance(lvl, int) or lvl > WS_CAP or "id" not in w:
                continue
            jobs = [j.upper() for j in w.get("jobs", []) if j.upper() in JOBS and JOBS.index(j.upper()) in jobs_on]
            if w.get("jobs") and not jobs:
                continue
            f.write("%d\t%s\t%d\t%s\t%d\t%g\t%s\n" % (w["id"], w.get("skill", ""), lvl, ",".join(jobs), 1 if "unlock" in w else 0, float(w.get("range", 0) or 0), name))
            n += 1
    print("weapon skills", n)
    print("expansions on:", sorted(t for t in on if t), "cap", cap, "jobs", [JOBS[j] for j in jobs_on])


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
