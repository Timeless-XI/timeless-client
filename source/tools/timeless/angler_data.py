#!/usr/bin/env python3
"""Timeless XI: the fishing tables the client's Timeless Angler (client/timeless/angler.rs) embeds, as JSON, from the
server's fishing SQL (the same reading as timeless-tools TimelessAngler/gen.py, which builds the old add-on's data.lua).
usage: angler_data.py <server root (sql/ + modules/phoenix/sql/)> <out .json>
Run it after a fishing table changes on the server, then rebuild the client:
    python3 tools/timeless/angler_data.py ../timeless-server crates/vana-client/assets/timeless/angler.json"""
import re, struct, sys, os, datetime

root, out = sys.argv[1], sys.argv[2]

def values(path):
    """Yield the tuples of every INSERT ... VALUES (...) in a dump."""
    txt = open(path, encoding='utf-8', errors='replace').read()
    for m in re.finditer(r"INSERT INTO `\w+` VALUES\s*", txt):
        i = m.end()
        while i < len(txt) and txt[i] == '(':
            i += 1; row = []; cur = ''; s = False
            while True:
                c = txt[i]
                if s:
                    if c == '\\': cur += txt[i+1]; i += 2; continue
                    if c == "'":
                        if txt[i+1] == "'": cur += "'"; i += 2; continue
                        s = False; i += 1; continue
                    cur += c; i += 1; continue
                if c == "'": s = True; cur = cur or ''; row_is_str = True; i += 1; continue
                if c in ',)':
                    row.append(cur.strip() if not isinstance(cur, str) or True else cur); cur = ''
                    i += 1
                    if c == ')': break
                    continue
                cur += c; i += 1
            yield row
            while i < len(txt) and txt[i] in ', \n\r\t': i += 1

def num(v):
    if v in ('NULL', ''): return 0
    try: return int(v)
    except ValueError: return float(v)

sql = os.path.join(root, 'sql')
T = lambda n: list(values(os.path.join(sql, 'fishing_%s.sql' % n)))

# ---- fish (server: disabled = 0 AND ranking < 99)
FCOLS = 'fishid name skill_level difficulty base_delay base_move min_length max_length ranking size_type water_type log quest quest_status flags hour_pattern moon_pattern month_pattern legendary legendary_flags item max_hook rarity required_keyitem required_catches family quest_only contest disabled'.split()
fish = {}
for r in T('fish'):
    d = dict(zip(FCOLS, r))
    if num(d['disabled']) != 0 or num(d['ranking']) >= 99: continue
    fish[num(d['fishid'])] = d

BCOLS = 'baitid name type maxhook losable flags mmm rankmod'.split()
bait = {num(r[0]): dict(zip(BCOLS, r)) for r in T('bait')}
aff = {}
for r in T('bait_affinity'):
    b, f, p = num(r[0]), num(r[1]), num(r[2])
    aff.setdefault(f, {})[b] = p

RCOLS = 'rodid name material size_type flags min_rank max_rank fish_attack lgd_bonus_attack fish_recovery fish_time lgd_bonus_time sm_delay_bonus sm_move_bonus lg_delay_bonus lg_move_bonus multiplier breakable broken_rodid mmm legendary rating'.split()
rods = {num(r[0]): dict(zip(RCOLS, r)) for r in T('rod')}
# Phoenix module: UPDATE fishing_rod SET max_rank = N where name = '...'
pr = os.path.join(root, 'modules', 'phoenix', 'sql', 'fishing_rod.sql')
if os.path.exists(pr):
    for m in re.finditer(r"SET\s+max_rank\s*=\s*(\d+)\s+where\s+name\s*=\s*'([^']+)'", open(pr).read(), re.I):
        for rd in rods.values():
            if rd['name'] == m.group(2): rd['max_rank'] = m.group(1)

groups = {}
for r in T('group'):
    groups.setdefault(num(r[0]), []).append((num(r[1]), num(r[2]), num(r[3])))

catch = {}
for r in T('catch'):
    catch[(num(r[0]), num(r[1]))] = num(r[2])     # server keeps the last row per zone/area

zones = {num(r[0]): {'name': r[1], 'diff': num(r[2])} for r in T('zone')}

areas = {}
for r in T('area'):
    z, a = num(r[0]), num(r[1])
    blob = r[6]
    pts = []
    if blob.startswith('0x') and len(blob) > 2:
        raw = bytes.fromhex(blob[2:])
        fl = [x[0] for x in struct.iter_unpack('<f', raw[:len(raw)//4*4])]
        tri = [tuple(fl[i:i+3]) for i in range(0, len(fl) - 2, 3)][:32]
        last = 0
        for i, t in enumerate(tri):
            if t != (0.0, 0.0, 0.0): last = i + 1
        pts = tri[:last]
    areas.setdefault(z, {})[a] = dict(name=r[2], bt=num(r[3]), bh=num(r[4]), br=num(r[5]), pts=pts,
                                      cx=num(r[7]), cy=num(r[8]), cz=num(r[9]))

MCOLS = 'mobid name zoneid level min_length max_length ranking difficulty base_delay base_move log quest nm nm_flags areaid rarity min_respawn max_respawn required_baitid alternative_baitid required_keyitem quest_only disabled'.split()
mobs = {}
for r in T('mob'):
    d = dict(zip(MCOLS, r))
    if num(d['disabled']): continue
    key = (num(d['zoneid']), d['name'], num(d['areaid']), num(d['nm']))
    if key in mobs: continue
    mobs[key] = d

# city zones (zone.yaml type: [city])
city = set()
zdir = os.path.join(root, 'data', 'zones')
cityfile = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'angler_city_zones.txt')
citynames = set(open(cityfile).read().split()) if os.path.exists(cityfile) else set()
def norm(s): return re.sub(r'[^a-z0-9]', '', s.lower())
cityn = {norm(c) for c in citynames}
for zid, z in zones.items():
    if norm(z['name']) in cityn: city.add(zid)

# Gear with Mod::FISH (127) from sql/item_mods.sql
gear = {}
im = os.path.join(sql, 'item_mods.sql')
GEARNAMES = {}
if os.path.exists(im):
    for m in re.finditer(r"VALUES \((\d+),127,(-?\d+)\);\s*--\s*(.*)", open(im, errors='replace').read()):
        gear[int(m.group(1))] = int(m.group(2))

# ---- emit
# NPC sell price (item_basic.BaseSell, then the Phoenix pre-RMT price reverts) for every catchable fish / item
sell = {}
for line in open(os.path.join(sql, 'item_basic.sql'), encoding='utf-8', errors='replace'):
    m = re.match(r"INSERT INTO `item_basic` VALUES \((\d+),.*,(\d+)\);\s*$", line)
    if m: sell[int(m.group(1))] = int(m.group(2))
rv = os.path.join(root, 'modules', 'phoenix', 'sql', 'pre_rmt_basesell_vendor_revert.sql')
if os.path.exists(rv):
    for m in re.finditer(r"SET\s+baseSell\s*=\s*(\d+)\s+WHERE\s+itemid\s*=\s*(\d+)", open(rv, errors='replace').read(), re.I):
        sell[int(m.group(2))] = int(m.group(1))
import json
used_groups = set(catch.values())
D = {'built': datetime.date.today().isoformat(), 'fish': {}, 'bait': {}, 'aff': {}, 'rods': {}, 'groups': {}, 'zones': {},
     'sell': {}, 'gear': {}}
for fid in sorted(fish):
    d = fish[fid]
    D['fish'][str(fid)] = dict(n=d['name'], lv=num(d['skill_level']), sz=num(d['size_type']), rk=num(d['ranking']),
        lg=num(d['legendary']), it=num(d['item']), rar=num(d['rarity']), hp=num(d['hour_pattern']), mp=num(d['moon_pattern']),
        mo=num(d['month_pattern']), fl=num(d['flags']), ki=num(d['required_keyitem']), qo=num(d['quest_only']))
for b in sorted(bait):
    d = bait[b]
    D['bait'][str(b)] = dict(n=d['name'], t=num(d['type']), fl=num(d['flags']))
for f in sorted(aff):
    if f in fish:
        D['aff'][str(f)] = {str(b): p for b, p in sorted(aff[f].items())}
for r in sorted(rods):
    d = rods[r]
    D['rods'][str(r)] = dict(n=d['name'], sz=num(d['size_type']), lg=num(d['legendary']))
for g in sorted(used_groups):
    D['groups'][str(g)] = [x[0] for x in sorted(groups.get(g, [])) if x[0] in fish]
for zid in sorted({z for (z, a) in catch}):
    za = areas.get(zid, {})
    zinfo = zones.get(zid, {'name': 'Zone %d' % zid, 'diff': 0})
    ar = {}
    for aid in sorted(za):
        a = za[aid]
        g = catch.get((zid, aid))
        if g is None: continue
        ar[str(aid)] = dict(n=a['name'], bt=a['bt'], bh=a['bh'], br=a['br'], cx=a['cx'], cy=a['cy'], cz=a['cz'], g=g,
                            pts=[[p[0], p[1], p[2]] for p in a['pts']])
    zm = []
    for (mz, nm, a, isnm), d in sorted(mobs.items()):
        if mz == zid:
            zm.append(dict(n=nm.replace('_', ' '), lv=num(d['level']), a=a, nm=isnm, qo=num(d['quest_only'])))
    D['zones'][str(zid)] = dict(n=zinfo['name'].replace('_', ' '), city=zid in city, areas=ar, mobs=zm)
D['sell'] = {str(f): sell[f] for f in sorted(fish) if sell.get(f, 0) > 0}
D['gear'] = {str(k): v for k, v in sorted(gear.items())}
open(out, 'w', encoding='utf-8', newline='\n').write(json.dumps(D, separators=(',', ':'), sort_keys=True) + '\n')
print('fish', len(fish), 'bait', len(bait), 'rods', len(rods), 'zones', len(D['zones']), 'groups', len(used_groups), 'gear', len(gear))
