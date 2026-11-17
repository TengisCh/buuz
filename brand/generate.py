# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generate the Buuz 2.0 (v2) icon options as SVG.

    python3 brand/generate.py brand/icons

Every option is drawn on a 1024 x 1024 canvas and written as:
  <slug>-icon.svg   full app icon (tile + artwork), 128 px and up
  <slug>-small.svg  simplified glyph for 16-32 px (favicon, tray, menu bar)

All shapes are hand-built here; nothing is traced from another logo.
"""
import math
import os
import sys

C = dict(
    dough_hi="#FFFBF3", dough="#FFEBD0", dough_lo="#EDC08C", dough_edge="#D9A06A",
    pleat="#D99A60", filling="#8C4A2B", filling_hi="#B86A3E",
    ink="#24190F", blush="#FF8E8E",
    blue="#2E6BFF", blue_hi="#6FA0FF", blue_lo="#1638B8",
    orange="#FF6A3D", amber="#FFB648", sky="#BFE3FF",
    night="#241A4F", violet="#5B3FD9",
)

CX = 512


def tile(grad_id, stops, rx=228):
    s = "".join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in stops)
    return (
        f'<linearGradient id="{grad_id}" x1="0" y1="0" x2="0.35" y2="1">{s}</linearGradient>'
        f'<radialGradient id="{grad_id}Glow" cx="0.3" cy="0.15" r="0.7">'
        f'<stop offset="0" stop-color="#fff" stop-opacity=".35"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></radialGradient>',
        f'<rect width="1024" height="1024" rx="{rx}" fill="url(#{grad_id})"/>'
        f'<rect width="1024" height="1024" rx="{rx}" fill="url(#{grad_id}Glow)"/>'
        f'<rect x="6" y="6" width="1012" height="1012" rx="{rx-6}" fill="none" stroke="#fff" stroke-opacity=".25" stroke-width="6"/>',
    )


DOUGH_DEFS = (
    '<radialGradient id="dough" cx="0.36" cy="0.30" r="0.85">'
    f'<stop offset="0" stop-color="{C["dough_hi"]}"/><stop offset=".55" stop-color="{C["dough"]}"/>'
    f'<stop offset="1" stop-color="{C["dough_lo"]}"/></radialGradient>'
    '<radialGradient id="ao" cx="0.5" cy="0.5" r="0.5">'
    '<stop offset="0" stop-color="#000" stop-opacity=".35"/><stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>'
    '<radialGradient id="hole" cx="0.5" cy="0.35" r="0.7">'
    f'<stop offset="0" stop-color="{C["filling_hi"]}"/><stop offset="1" stop-color="{C["filling"]}"/></radialGradient>'
    '<linearGradient id="rim" x1="0" y1="0" x2="0" y2="1">'
    '<stop offset="0" stop-color="#fff" stop-opacity="0"/><stop offset=".8" stop-color="#fff" stop-opacity="0"/>'
    '<stop offset="1" stop-color="#fff" stop-opacity=".55"/></linearGradient>'
    '<filter id="tiny" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="3"/></filter>'
    '<filter id="soft" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="10"/></filter>'
    '<filter id="softer" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="22"/></filter>'
)


def buuz(cx=CX, base=770, w=300, h=430, face=True, steam=True, steam_color="#fff",
         n_pleats=14, twist=0.9, mood="smile"):
    """Front, slightly-from-above view of a Mongolian buuz.

    Real buuz: a soft round body, pleats pinched in a spiral toward the top,
    and the top left OPEN (the steam hole) so you can see the filling.
    """
    top = base - h
    hx, hy, hr = cx, top + 26, 40  # steam hole centre / radius
    L, R = cx - w, cx + w
    # body: soft rounded dome with a wide, slightly squashed base
    body = (
        f"M {L} {base-110} "
        f"C {L} {base-h*0.62}, {cx-w*0.62} {top}, {cx} {top} "
        f"C {cx+w*0.62} {top}, {R} {base-h*0.62}, {R} {base-110} "
        f"C {R} {base-20}, {cx+w*0.55} {base+8}, {cx} {base+8} "
        f"C {cx-w*0.55} {base+8}, {L} {base-20}, {L} {base-110} Z"
    )
    out = []
    out.append(f'<ellipse cx="{cx}" cy="{base+22}" rx="{w*0.98}" ry="{w*0.16}" fill="url(#ao)"/>')
    out.append(f'<path d="{body}" fill="url(#dough)"/>')
    cid = f"bc{cx}{base}{w}"
    out.append(f'<clipPath id="{cid}"><path d="{body}"/></clipPath>')
    out.append(f'<g clip-path="url(#{cid})">'
               f'<ellipse cx="{cx+w*0.25}" cy="{base+40}" rx="{w*1.1}" ry="{h*0.30}" fill="{C["dough_edge"]}" opacity=".45" filter="url(#softer)"/>'
               f'<ellipse cx="{cx-w*0.40}" cy="{top+h*0.38}" rx="{w*0.30}" ry="{h*0.17}" fill="#fff" opacity=".75" filter="url(#softer)"/>'
               f'<path d="{body}" fill="url(#rim)"/>')
    # spiral pleats: tapered folds pinched toward the open top, all turning one way
    plen_r, plen_y = w * 0.62, h * 0.30
    for i in range(n_pleats):
        a0 = 2 * math.pi * i / n_pleats + 0.15
        back = math.sin(a0 + twist * 0.5) < -0.25
        tmax = 0.45 if back else 1.0
        cen, nrm = [], []
        N = 14
        for k in range(N + 1):
            t = tmax * k / N
            a = a0 + twist * t
            r = hr + 6 + (plen_r - hr) * (t ** 0.8)
            yc = hy + plen_y * (t ** 1.15)
            cen.append((cx + r * math.cos(a), yc + r * 0.32 * math.sin(a)))
        def wid(t):
            return 4 + 26 * (math.sin(math.pi * min(t / tmax, 1)) ** 0.6) * (1 - 0.5 * (t / tmax))
        left, right = [], []
        for k, (x, y) in enumerate(cen):
            x2, y2 = cen[min(k + 1, N)]
            x1, y1 = cen[max(k - 1, 0)]
            dx, dy = x2 - x1, y2 - y1
            ln = math.hypot(dx, dy) or 1
            nx, ny = -dy / ln, dx / ln
            ww = wid(tmax * k / N)
            left.append((x + nx * ww, y + ny * ww)); right.append((x - nx * ww * 0.3, y - ny * ww * 0.3))
        poly = left + right[::-1]
        d = "M " + " L ".join(f"{x:.1f} {y:.1f}" for x, y in poly) + " Z"
        out.append(f'<path d="{d}" fill="{C["pleat"]}" opacity="{0.30 if back else 0.42}" filter="url(#tiny)"/>')
        hl = "M " + " L ".join(f"{x-4:.1f} {y-9:.1f}" for x, y in cen[1:-3])
        out.append(f'<path d="{hl}" fill="none" stroke="#fff" stroke-width="16" stroke-linecap="round" opacity="{0.35 if back else 0.75}" filter="url(#tiny)"/>')
    out.append('</g>')
    # the open top: a pinched dough knot with the filling showing
    for j in range(10):
        aa = 2 * math.pi * j / 10 + 0.3
        lx, ly = hx + (hr + 14) * math.cos(aa), hy + 4 + (hr + 14) * 0.45 * math.sin(aa)
        out.append(f'<ellipse cx="{lx:.1f}" cy="{ly:.1f}" rx="20" ry="14" fill="{C["dough_hi"]}" stroke="{C["pleat"]}" stroke-width="4" stroke-opacity=".45" transform="rotate({math.degrees(aa)+90:.0f} {lx:.1f} {ly:.1f})"/>')
    out.append(f'<ellipse cx="{hx}" cy="{hy+4}" rx="{hr+22}" ry="{(hr+22)*0.45}" fill="{C["dough_hi"]}" stroke="{C["pleat"]}" stroke-width="7" stroke-opacity=".55"/>')
    out.append(f'<ellipse cx="{hx}" cy="{hy+4}" rx="{hr}" ry="{hr*0.38}" fill="url(#hole)"/>')
    out.append(f'<ellipse cx="{hx-8}" cy="{hy}" rx="{hr*0.4}" ry="{hr*0.1}" fill="#fff" opacity=".4"/>')
    if face:
        ey = top + h * 0.66
        dx = w * 0.30
        for s in (-1, 1):
            ex = cx + s * dx
            out.append(f'<ellipse cx="{ex}" cy="{ey}" rx="27" ry="34" fill="{C["ink"]}"/>')
            out.append(f'<circle cx="{ex+9}" cy="{ey-12}" r="10" fill="#fff"/>')
            out.append(f'<circle cx="{ex-8}" cy="{ey+12}" r="4.5" fill="#fff" opacity=".8"/>')
            out.append(f'<ellipse cx="{ex+s*58}" cy="{ey+44}" rx="42" ry="24" fill="{C["blush"]}" opacity=".55" filter="url(#soft)"/>')
        my = ey + 34
        if mood == "smile":
            out.append(f'<path d="M {cx-30} {my} Q {cx-15} {my+24}, {cx} {my+4} Q {cx+15} {my+24}, {cx+30} {my}" fill="none" stroke="{C["ink"]}" stroke-width="10" stroke-linecap="round" stroke-linejoin="round"/>')
        else:  # open "o" mouth, sipping
            out.append(f'<ellipse cx="{cx}" cy="{my+10}" rx="16" ry="19" fill="{C["ink"]}"/><ellipse cx="{cx}" cy="{my+18}" rx="9" ry="7" fill="#E0585E"/>')
    if steam:
        for dx, op, ln in ((-95, .75, 0.8), (0, 1, 1.0), (95, .75, 0.8)):
            x0, y0 = hx + dx * 0.35, hy - 46
            d = (f"M {x0} {y0} c {dx*0.3+30} {-45*ln}, {dx*0.6-40} {-85*ln}, {dx*0.6} {-130*ln}"
                 f" s {35} {-70*ln}, {5} {-105*ln}")
            out.append(f'<path d="{d}" fill="none" stroke="{steam_color}" stroke-width="40" stroke-linecap="round" opacity="{op*0.35}" filter="url(#soft)"/>')
            out.append(f'<path d="{d}" fill="none" stroke="{steam_color}" stroke-width="28" stroke-linecap="round" opacity="{op*0.9}"/>')
    return "".join(out)


def svg(defs, body, size=1024):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="{size}" height="{size}">'
            f"<defs>{defs}</defs>{body}</svg>\n")


# ---------------------------------------------------------------- A. Buuz Buddy
def buddy():
    d, t = tile("bgA", [(0, C["amber"]), (1, C["orange"])])
    icon = svg(d + DOUGH_DEFS, t + buuz(base=800, w=320, h=470))
    d2, t2 = tile("bgA", [(0, C["amber"]), (1, C["orange"])], rx=200)
    small = svg(d2 + DOUGH_DEFS, t2 + buuz(base=880, w=420, h=640, steam=False, n_pleats=8, twist=0.7)
                .replace('rx="27" ry="34"', 'rx="46" ry="56"').replace('r="10" fill="#fff"', 'r="16" fill="#fff"'))
    return icon, small


# ---------------------------------------------------------------- B. Keycap
def glyph_be(x, y, w, h):
    """Cyrillic capital Б as one rounded stroke (no font needed)."""
    bx = x + w * 0.55
    r = h * 0.275
    return (f"M {x+w} {y} L {x} {y} L {x} {y+h} L {bx} {y+h} "
            f"A {r} {r} 0 0 0 {bx} {y+h-2*r} L {x} {y+h-2*r}")


def glyph_b(x, y, w, h):
    """Latin capital B as one rounded stroke."""
    r1, r2 = h * 0.24, h * 0.26
    m = y + 2 * r1
    return (f"M {x} {y+h} L {x} {y} L {x+w-r1*1.1} {y} A {r1} {r1} 0 0 1 {x+w-r1*1.1} {m} L {x} {m} "
            f"M {x+w-r2*1.1} {m} A {r2} {r2} 0 0 1 {x+w-r2*1.1} {y+h} L {x} {y+h}")


def keycap_shape(x, y, w, h, depth, top_fill, side_fill, r=120):
    return (f'<rect x="{x}" y="{y+depth}" width="{w}" height="{h}" rx="{r}" fill="{side_fill}"/>'
            f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{top_fill}"/>'
            f'<rect x="{x+40}" y="{y+26}" width="{w-80}" height="{h-90}" rx="{r-30}" fill="#fff" opacity=".18"/>')


def keycap():
    d, t = tile("bgB", [(0, "#2B2350"), (1, "#151033")])
    d += ('<linearGradient id="capTop" x1="0" y1="0" x2="0" y2="1">'
          f'<stop offset="0" stop-color="{C["blue_hi"]}"/><stop offset="1" stop-color="{C["blue"]}"/></linearGradient>')
    stroke = 'fill="none" stroke-width="22" stroke-linecap="round" stroke-linejoin="round"'
    legends = (f'<path d="{glyph_b(236, 236, 78, 104)}" stroke="#fff" opacity=".9" {stroke}/>'
               f'<path d="{glyph_be(710, 236, 80, 104)}" stroke="{C["amber"]}" {stroke}/>')
    body = (t + '<ellipse cx="512" cy="900" rx="380" ry="40" fill="#000" opacity=".35" filter="url(#softer)"/>'
            + keycap_shape(152, 160, 720, 650, 70, "url(#capTop)", C["blue_lo"], r=130)
            + legends + buuz(base=720, w=235, h=320, mood="sip")
            .replace('stroke-width="28"', 'stroke-width="22"').replace('stroke-width="40"', 'stroke-width="32"')
            .replace('rx="27" ry="34"', 'rx="22" ry="28"'))
    icon = svg(d + DOUGH_DEFS, body)
    small_body = (t + keycap_shape(110, 110, 804, 720, 90, "url(#capTop)", C["blue_lo"], r=170)
                  + f'<path d="{glyph_be(330, 250, 370, 440)}" fill="none" stroke="#fff" stroke-width="110" stroke-linecap="round" stroke-linejoin="round"/>')
    return icon, svg(d, small_body)


# ---------------------------------------------------------------- C. Rosette (top view)
def rosette_art(cx=512, cy=520, R=330, n=9, steam=True):
    out = [f'<ellipse cx="{cx+18}" cy="{cy+40}" rx="{R*1.02}" ry="{R*0.98}" fill="#000" opacity=".28" filter="url(#softer)"/>',
           f'<circle cx="{cx}" cy="{cy}" r="{R}" fill="url(#dough)"/>']
    # spiral pleats seen from above: crescent swooshes that all turn the same way
    for i in range(n):
        a = 2 * math.pi * i / n
        def p(rr, aa):
            return cx + rr * math.cos(aa), cy + rr * math.sin(aa)
        x0, y0 = p(R * 0.22, a)
        x1, y1 = p(R * 0.62, a + 0.55)
        x2, y2 = p(R * 0.93, a + 0.55)
        x3, y3 = p(R * 0.60, a + 0.95)
        x4, y4 = p(R * 0.26, a + 0.55)
        d = f"M {x0:.1f} {y0:.1f} Q {x1:.1f} {y1:.1f}, {x2:.1f} {y2:.1f} Q {x3:.1f} {y3:.1f}, {x4:.1f} {y4:.1f} Z"
        lit = 0.5 + 0.5 * math.cos(a + 0.55 + 2.2)  # light from top-left
        out.append(f'<path d="{d}" fill="{C["dough_hi"]}" opacity="{0.35+0.6*lit:.2f}"/>')
        out.append(f'<path d="M {x2:.1f} {y2:.1f} Q {x3:.1f} {y3:.1f}, {x4:.1f} {y4:.1f}" fill="none" stroke="{C["pleat"]}" stroke-width="10" stroke-linecap="round" opacity=".7"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R}" fill="none" stroke="{C["dough_edge"]}" stroke-width="10" opacity=".5"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R*0.22}" fill="{C["dough"]}"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{R*0.15}" fill="url(#hole)"/>')
    out.append(f'<circle cx="{cx-R*0.05}" cy="{cy-R*0.05}" r="{R*0.05}" fill="#fff" opacity=".35"/>')
    if steam:
        out.append(f'<path d="M {cx} {cy-20} C {cx+60} {cy-120}, {cx-90} {cy-200}, {cx-10} {cy-300} S {cx+60} {cy-420}, {cx+10} {cy-460}" '
                   f'fill="none" stroke="#fff" stroke-width="30" stroke-linecap="round" opacity=".85"/>')
    return "".join(out)


def rosette():
    d, t = tile("bgC", [(0, C["blue_hi"]), (1, C["blue_lo"])])
    icon = svg(d + DOUGH_DEFS, t + rosette_art(cy=500, R=350, steam=False))
    small = svg(d + DOUGH_DEFS, t + rosette_art(cy=512, R=400, n=6, steam=False)
                .replace('stroke-width="10" stroke-linecap', 'stroke-width="26" stroke-linecap'))
    return icon, small


# ---------------------------------------------------------------- D. Steam Б
def steam_b():
    d, t = tile("bgD", [(0, C["violet"]), (1, C["night"])])
    d += ('<linearGradient id="steamG" x1="0" y1="1" x2="0" y2="0">'
          '<stop offset="0" stop-color="#fff" stop-opacity=".55"/><stop offset=".45" stop-color="#fff"/></linearGradient>'
          '<filter id="glow" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="18"/></filter>')
    # a wisp leaves the open top and becomes a Б
    letter = glyph_be(392, 120, 250, 330)
    wisp = "M 512 548 C 470 520, 560 500, 520 486"
    art = ''
    for path, sw in ((letter, 56), (wisp, 40)):
        art += (f'<path d="{path}" fill="none" stroke="#B9A8FF" stroke-width="{sw+24}" stroke-linecap="round" stroke-linejoin="round" opacity=".55" filter="url(#glow)"/>'
                f'<path d="{path}" fill="none" stroke="#fff" stroke-width="{sw}" stroke-linecap="round" stroke-linejoin="round" opacity="{1 if path == letter else .7}"/>')
    body = t + art + buuz(base=880, w=300, h=330, steam=False, n_pleats=12)
    icon = svg(d + DOUGH_DEFS, body)
    small = svg(d, t + f'<path d="{glyph_be(300, 200, 430, 620)}" fill="none" stroke="#fff" stroke-width="120" stroke-linecap="round" stroke-linejoin="round"/>'
                + '<circle cx="770" cy="200" r="70" fill="#fff" opacity=".85"/>')
    return icon, small


OPTIONS = [("a-buuz-buddy", buddy), ("b-keycap", keycap), ("c-rosette", rosette), ("d-steam-b", steam_b)]

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "icons"
    os.makedirs(out, exist_ok=True)
    for slug, fn in OPTIONS:
        icon, small = fn()
        open(os.path.join(out, f"{slug}-icon.svg"), "w").write(icon)
        open(os.path.join(out, f"{slug}-small.svg"), "w").write(small)
    print("wrote", len(OPTIONS) * 2, "files to", out)
