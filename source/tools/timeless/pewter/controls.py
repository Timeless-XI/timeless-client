"""Pewter Hairline controls, drawn at the pack's logical size (rendered 2x)."""
import pewter
SIL = '<linearGradient id="sv" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="0" y2="{h}"><stop offset="0" stop-color="#f4f6fa"/><stop offset=".55" stop-color="#b7c0cf"/><stop offset="1" stop-color="#7d879b"/></linearGradient>'
def doc(w, h, body, defs=""):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w*2}" height="{h*2}" viewBox="0 0 {w} {h}"><defs>{SIL.format(h=h)}{defs}</defs>{body}</svg>'
def grad(id_, a, b): return f'<linearGradient id="{id_}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient>'
def panel(w, h, r, top, bot, edge, inner="rgba(214,225,240,.10)", glow=None, ew=1.0):
    g = f'<rect x="0.5" y="0.5" width="{w-1}" height="{h-1}" rx="{r+1}" fill="none" stroke="{glow}" stroke-opacity=".35" stroke-width="1"/>' if glow else ""
    return (grad("bg", top, bot), g +
            f'<rect x="1.5" y="1.5" width="{w-3}" height="{h-3}" rx="{r}" fill="url(#bg)" stroke="{edge}" stroke-width="{ew}"/>'
            f'<rect x="3" y="3" width="{w-6}" height="{h-6}" rx="{max(r-1.5,0.5)}" fill="none" stroke="{inner}" stroke-width=".6"/>')
OUT = {}
BTN = {"normal": ("#1d2333", "#10131c", "#6b7892", None), "hover": ("#262e43", "#151a26", "#a3aec3", None),
       "pressed": ("#0d1018", "#181d2a", "#6b7892", None), "disabled": ("#15181f", "#101218", "#363d4d", None),
       "selected": ("#2c4268", "#172238", "#8fb2e6", "#8fb2e6"), "danger": ("#33202a", "#170f14", "#c25a66", None),
       "confirm": ("#2a3a56", "#141d2f", "#d6e1f0", "#9fbbe0")}
for k, (a, b, e, gl) in BTN.items():
    d, body = panel(160, 40, 3, a, b, e, glow=gl)
    if k == "pressed": body += '<rect x="3" y="3" width="154" height="3" fill="#000" opacity=".25"/>'
    OUT[f"controls/button_{k}"] = doc(160, 40, body, d)
for k, (e, gl) in {"normal": ("#4a5468", None), "focused": ("#8fb2e6", "#8fb2e6"), "invalid": ("#c25a66", "#e0606c")}.items():
    d, body = panel(256, 36, 2.5, "#07080d", "#0e1119", e, inner="rgba(0,0,0,.35)", glow=gl)
    OUT[f"controls/input_{k}"] = doc(256, 36, body, d)
TAB = {"normal": ("#151925", "#10131c", "#4a5468", .0), "hover": ("#1e2433", "#141824", "#7d899f", .0),
       "active": ("#232a3d", "#161a26", "#a3aec3", 1.0), "disabled": ("#121419", "#0f1116", "#2c323f", .0)}
for k, (a, b, e, act) in TAB.items():
    body = (f'<path d="M1.5 36 L1.5 5 Q1.5 1.5 5 1.5 L123 1.5 Q126.5 1.5 126.5 5 L126.5 36" fill="url(#bg)" stroke="{e}" stroke-width="1"/>'
            f'<path d="M3.5 36 L3.5 5.5 Q3.5 3.5 5.5 3.5 L122.5 3.5 Q124.5 3.5 124.5 5.5 L124.5 36" fill="none" stroke="rgba(214,225,240,.10)" stroke-width=".6"/>')
    if act:
        body += '<path d="M20 2 L108 2" stroke="#d6e1f0" stroke-opacity=".55" stroke-width="1"/><path d="M64 0 L66.5 2.5 L64 5 L61.5 2.5 Z" fill="#d6e1f0"/>'
    OUT[f"controls/tab_{k}"] = doc(128, 36, body, grad("bg", a, b))
SLOT = {"normal": ("#4a5468", None, ""), "hover": ("#a3aec3", None, ""), "selected": ("#d6e1f0", "#8fb2e6", ""),
        "invalid": ("#c25a66", None, ""), "drop_valid": ("#8fb2e6", "#8fb2e6", ""), "drop_invalid": ("#e0606c", "#e0606c", ""), "locked": ("#363d4d", None, "lock")}
for k, (e, gl, extra) in SLOT.items():
    d, body = panel(68, 68, 4, "#141824", "#0b0d13", e, glow=gl, ew=1.2)
    if gl: body = f'<rect x="3" y="3" width="62" height="62" rx="3" fill="{gl}" opacity=".10"/>' + body
    if extra == "lock":
        inner = pewter.svg("lock", 64, design=16, glow=False)
        inner = inner[inner.index("<defs>"):-6]
        body += f'<svg x="46" y="46" width="18" height="18" viewBox="0 0 64 64">{inner}</svg>'
    OUT[f"controls/item_slot_{k}"] = doc(68, 68, body, d)
def chevron(dirn):
    pts = {"right": "7.5 4.5 13 10 7.5 15.5", "left": "12.5 4.5 7 10 12.5 15.5", "up": "4.5 12.5 10 7 15.5 12.5", "down": "4.5 7.5 10 13 15.5 7.5"}[dirn]
    return doc(20, 20, f'<polyline points="{pts}" fill="none" stroke="url(#sv)" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>')
for dn in ("up", "down", "left", "right"): OUT[f"controls/arrow_{dn}"] = chevron(dn)
box = '<rect x="3.5" y="3.5" width="17" height="17" rx="2.5" fill="url(#bg)" stroke="{e}" stroke-width="1"/><rect x="5" y="5" width="14" height="14" rx="1.5" fill="none" stroke="rgba(214,225,240,.12)" stroke-width=".5"/>'
OUT["controls/checkbox_off"] = doc(24, 24, box.format(e="#7d899f"), grad("bg", "#141824", "#0b0d13"))
OUT["controls/checkbox_on"] = doc(24, 24, box.format(e="#a3aec3") + '<polyline points="7.5 12.5 10.8 15.8 16.8 8.5" fill="none" stroke="url(#sv)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>', grad("bg", "#2c4268", "#172238"))
OUT["controls/close"] = doc(24, 24, '<path d="M7.5 7.5 L16.5 16.5 M16.5 7.5 L7.5 16.5" stroke="url(#sv)" stroke-width="1.5" stroke-linecap="round"/>')
OUT["controls/resize"] = doc(24, 24, '<path d="M20 9 L9 20 M20 14 L14 20 M20 18.5 L18.5 20" stroke="url(#sv)" stroke-width="1.2" stroke-linecap="round"/>')
OUT["controls/drag"] = doc(24, 24, "".join(f'<circle cx="{x}" cy="{y}" r="1.4" fill="url(#sv)"/>' for x in (9.5, 14.5) for y in (7, 12, 17)))
for nm in ("search", "move"):
    src = {"search": "check", "move": "drag_drop"}[nm]
    inner = pewter.svg(src, 64, design=20, glow=False); inner = inner[inner.index("<defs>"):-6]
    OUT[f"controls/{nm}"] = f'<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 64 64">{inner}</svg>'
for k, fill in (("empty", 0.0), ("half", 0.5), ("full", 1.0)):
    body = '<rect x="6.5" y="2.5" width="7" height="79" rx="3.5" fill="#0b0d13" stroke="#4a5468" stroke-width="1"/>'
    if fill:
        h = 79 * fill
        body += f'<rect x="7.2" y="{2.5+79-h:.1f}" width="5.6" height="{h-0.7:.1f}" rx="2.8" fill="url(#sv)" opacity=".9"/>'
    OUT[f"controls/scroll_thumb_{k}"] = doc(20, 84, body)
