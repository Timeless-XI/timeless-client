#!/usr/bin/env python3
"""Builds mods/timeless-artisan/lib/data.lua (Timeless Artisan's recipes and crafting materials) from the TimelessXI
server's tables.

usage: python3 tools/artisan_recipes.py <timeless-server root> mods/timeless-artisan/lib/data.lua

Reads sql/synth_recipes.sql (commented-out "-- INSERT" rows are not on the server), applies the
UPDATEs of every era/sql/<expansion>/*synth_recipes*.sql folder listed in modules/init.txt, and keeps
synthesis recipes (Desynth = 0) whose content tag is enabled in the server's settings
(tools/timeless/settings/main.lua: ENABLE_<TAG> = 1; no tag = always). The same rules as
modules/timeless_artisan/cpp/timeless_artisan.cpp and modules/craft_plus.
Re-run after any change to synth_recipes or the expansion settings.
"""
import datetime
import os
import re
import sys

root, out = sys.argv[1], sys.argv[2]

COLS = ['ID', 'Desynth', 'KeyItem', 'Wood', 'Smith', 'Gold', 'Cloth', 'Leather', 'Bone', 'Alchemy', 'Cook',
        'Crystal', 'HQCrystal', 'Ingredient1', 'Ingredient2', 'Ingredient3', 'Ingredient4', 'Ingredient5',
        'Ingredient6', 'Ingredient7', 'Ingredient8', 'Result', 'ResultHQ1', 'ResultHQ2', 'ResultHQ3',
        'ResultQty', 'ResultHQ1Qty', 'ResultHQ2Qty', 'ResultHQ3Qty', 'ResultName', 'content_tag']
CRAFTS = ['Wood', 'Smith', 'Gold', 'Cloth', 'Leather', 'Bone', 'Alchemy', 'Cook']
CRYSTALS = {4096 + e: e for e in range(8)}
CRYSTALS.update({4238 + e: e for e in range(8)})
CRYSTALS.update({6506 + e: e for e in range(8)})


def split_row(s):
    """Split the inside of a VALUES (...) row into raw values (strings unquoted)."""
    out, cur, i, q = [], '', 0, False
    while i < len(s):
        c = s[i]
        if q:
            if c == '\\':
                cur += s[i + 1]; i += 2; continue
            if c == "'":
                if i + 1 < len(s) and s[i + 1] == "'":
                    cur += "'"; i += 2; continue
                q = False; i += 1; continue
            cur += c; i += 1; continue
        if c == "'":
            q = True; i += 1; continue
        if c == ',':
            out.append(cur.strip()); cur = ''; i += 1; continue
        cur += c; i += 1
    out.append(cur.strip())
    return out


recipes = {}
for line in open(os.path.join(root, 'sql', 'synth_recipes.sql'), encoding='utf-8', errors='replace'):
    m = re.match(r"^INSERT INTO `synth_recipes` VALUES \((.*)\);\s*$", line)
    if not m:
        continue
    vals = split_row(m.group(1))
    if len(vals) != len(COLS):
        raise SystemExit('unexpected row: ' + line[:120])
    r = dict(zip(COLS, vals))
    for k in COLS:
        if k not in ('ResultName', 'content_tag'):
            r[k] = int(r[k])
    r['content_tag'] = '' if r['content_tag'] in ('NULL', '') else r['content_tag']
    recipes[r['ID']] = r

# era/sql/<expansion> folders listed in init.txt
init = open(os.path.join(root, 'modules', 'init.txt'), encoding='utf-8').read().splitlines()
listed = [l.split('#')[0].strip().rstrip('/') for l in init]
listed = [l for l in listed if l]
era_dirs = []
for l in listed:
    if l == 'era' or l == 'era/sql':
        era_dirs += [os.path.join('era', 'sql', d) for d in os.listdir(os.path.join(root, 'modules', 'era', 'sql'))]
    elif l.startswith('era/sql/'):
        era_dirs.append(l)
updates = 0
for d in era_dirs:
    full = os.path.join(root, 'modules', d)
    if not os.path.isdir(full):
        continue
    for f in sorted(os.listdir(full)):
        if 'synth_recipes' not in f:
            continue
        for line in open(os.path.join(full, f), encoding='utf-8'):
            m = re.match(r"^UPDATE `?synth_recipes`? SET (.*) WHERE `?ID`? = (\d+);", line.strip())
            if not m:
                continue
            rid = int(m.group(2))
            if rid not in recipes:
                continue
            for part in re.finditer(r"`?(\w+)`?\s*=\s*('(?:[^']|'')*'|\d+)", m.group(1)):
                k, v = part.group(1), part.group(2)
                recipes[rid][k] = v[1:-1].replace("''", "'") if v.startswith("'") else int(v)
            updates += 1

# enabled content tags
settings = open(os.path.join(root, 'tools', 'timeless', 'settings', 'main.lua'), encoding='utf-8').read()
enabled = {m.group(1) for m in re.finditer(r"ENABLE_(\w+)\s*=\s*(?:1|true)\b", settings)}

kept = []
for r in sorted(recipes.values(), key=lambda r: r['ID']):
    if r['Desynth'] != 0:
        continue
    if r['content_tag'] and r['content_tag'].upper() not in enabled:
        continue
    skills = [r[c] for c in CRAFTS]
    level = max(skills)
    if level == 0 or r['Result'] == 0 or r['Crystal'] not in CRYSTALS:
        continue
    craft = skills.index(level) + 1
    counts = []
    for i in range(1, 9):
        ing = r['Ingredient%d' % i]
        if ing == 0:
            continue
        for e in counts:
            if e[0] == ing:
                e[1] += 1
                break
        else:
            counts.append([ing, 1])
    if not counts:
        continue
    kept.append((r, craft, level, skills, counts))

# Items in an Auction House materials category also count as materials (server: same rule in
# timeless_artisan.cpp load()), so crafted materials such as Sailcloth are storable even when no
# enabled recipe uses them. ah_era (if listed in init.txt) hides out-of-era items with aH = 0.
AH_GROUP = {35: 0, 43: 1, 38: 2, 39: 3, 40: 4, 41: 5, 42: 6, 44: 7, 63: 7, 59: 8}
consts = {}
ah_of = {}
for line in open(os.path.join(root, 'sql', 'item_basic.sql'), encoding='utf-8', errors='replace'):
    m = re.match(r"^SET @(\w+)\s*=\s*(\d+);", line)
    if m:
        consts[m.group(1)] = int(m.group(2))
        continue
    m = re.match(r"^INSERT INTO `item_basic` VALUES \((.*)\);", line)
    if not m:
        continue
    vals = split_row(m.group(1))
    if len(vals) < 10:
        continue
    a = vals[8]
    a = consts.get(a[1:], 0) if a.startswith('@') else int(a)
    ah_of[int(vals[0])] = a
if any(l == 'ah_era/sql' for l in listed):
    for line in open(os.path.join(root, 'modules', 'ah_era', 'sql', 'ah_era.sql'), encoding='utf-8'):
        m = re.match(r"^UPDATE `?item_basic`? SET `?aH`? = 0 WHERE `?itemid`? = (\d+);", line.strip())
        if m:
            ah_of[int(m.group(1))] = 0
ingredients = {g[0] for _, _, _, _, counts in kept for g in counts}
ah_mats = sorted((i, AH_GROUP[a]) for i, a in ah_of.items() if a in AH_GROUP and i not in ingredients)

def lua_str(s):
    return "'" + s.replace('\\', '\\\\').replace("'", "\\'") + "'"

lines = []
lines.append('-- GENERATED by tools/artisan_recipes.py from the Timeless XI server (%s). Do not edit.' % datetime.date.today().isoformat())
lines.append('-- %d synthesis recipes (enabled content: %s); era updates applied: %d.' % (len(kept), ', '.join(sorted(enabled & {'ROTZ', 'COP', 'TOAU', 'WOTG', 'ACP', 'AMK', 'ASA', 'ABYSSEA', 'SOA', 'ROV', 'TVR'})), updates))
lines.append('-- Recipe: { id, craft 1-8, level, crystal, key item, { 8 craft levels }, { item, count, ... }, { NQ, qty, HQ1, qty, HQ2, qty, HQ3, qty }, name }')
lines.append('local R = {')
for r, craft, level, skills, counts in kept:
    ing = ','.join('%d,%d' % (a, b) for a, b in counts)
    res = ','.join(str(r[k]) for k in ('Result', 'ResultQty', 'ResultHQ1', 'ResultHQ1Qty', 'ResultHQ2', 'ResultHQ2Qty', 'ResultHQ3', 'ResultHQ3Qty'))
    lines.append('{%d,%d,%d,%d,%d,{%s},{%s},{%s},%s},' % (
        r['ID'], craft, level, r['Crystal'], r['KeyItem'], ','.join(str(s) for s in skills), ing, res, lua_str(r['ResultName'])))
lines.append('}')
lines.append('-- %d more materials from Auction House materials categories (no enabled recipe uses them): { item, group }' % len(ah_mats))
lines.append('-- group 0 crystals, 1-8 the craft (woodworking .. cooking) of the category')
lines.append('local M = {' + ','.join('{%d,%d}' % m for m in ah_mats) + '}')
lines.append('return { recipes = R, materials = M, version = %s }' % lua_str(datetime.date.today().isoformat()))
open(out, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines) + '\n')
print('%d recipes, %d extra materials -> %s' % (len(kept), len(ah_mats), out))
