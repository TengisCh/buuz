import math, sys, os
out = sys.argv[1]  # usage: python3 generate-concepts.py concepts
os.makedirs(out, exist_ok=True)

def pleats(cx, ky, n, spread, length, wid, col, hl):
    s = ''
    for i in range(n):
        t = (i/(n-1))*2-1  # -1..1
        a = t*spread
        x2 = cx + math.sin(a)*length*1.05
        y2 = ky + math.cos(a)*length*0.72 + 6
        c1x = cx + math.sin(a)*length*0.35 + t*10
        c1y = ky + length*0.55
        s += f'<path d="M{cx+t*9:.1f} {ky+10:.1f} Q{c1x:.1f} {c1y:.1f} {x2:.1f} {y2:.1f}" fill="none" stroke="{col}" stroke-width="{wid}" stroke-linecap="round" opacity=".9"/>'
        s += f'<path d="M{cx+t*9+3:.1f} {ky+10:.1f} Q{c1x+3:.1f} {c1y:.1f} {x2+3:.1f} {y2:.1f}" fill="none" stroke="{hl}" stroke-width="{wid*0.35}" stroke-linecap="round" opacity=".7"/>'
    return s

def face(cx, cy, kind='cute', ink='#3a211b', blush='#ff8fa3'):
    s = f'<ellipse cx="{cx-84}" cy="{cy+22}" rx="24" ry="15" fill="{blush}" opacity=".55"/>'
    s += f'<ellipse cx="{cx+84}" cy="{cy+22}" rx="24" ry="15" fill="{blush}" opacity=".55"/>'
    if kind == 'sparkle':
        for sx in (-50, 50):
            s += f'<ellipse cx="{cx+sx}" cy="{cy-4}" rx="19" ry="25" fill="{ink}"/>'
            s += f'<circle cx="{cx+sx-6}" cy="{cy-14}" r="8" fill="#fff"/><circle cx="{cx+sx+7}" cy="{cy+5}" r="4" fill="#fff"/>'
        s += f'<path d="M{cx-14} {cy+20} Q{cx} {cy+38} {cx+14} {cy+20}" fill="#ff6f8f" stroke="{ink}" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>'
    else:
        for sx in (-50, 50):
            s += f'<ellipse cx="{cx+sx}" cy="{cy}" rx="14" ry="19" fill="{ink}"/>'
            s += f'<circle cx="{cx+sx-4}" cy="{cy-7}" r="5.5" fill="#fff"/>'
        s += f'<path d="M{cx-14} {cy+14} Q{cx-7} {cy+26} {cx} {cy+14} Q{cx+7} {cy+26} {cx+14} {cy+14}" fill="none" stroke="{ink}" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>'
    return s

def dumpling(cx=256, cy=290, rx=190, ry=164, pal=None, kind='cute', with_face=True, glyph=False):
    p = pal
    ky = cy - ry + 18
    s = f'''
<ellipse cx="{cx}" cy="{cy+ry-6}" rx="{rx*0.9}" ry="20" fill="#000" opacity=".22" filter="url(#blur14)"/>
<path d="M{cx-rx} {cy+20} C{cx-rx} {cy-ry*0.55} {cx-60} {ky-8} {cx} {ky-18} C{cx+60} {ky-8} {cx+rx} {cy-ry*0.55} {cx+rx} {cy+20} C{cx+rx} {cy+ry*0.8} {cx+rx*0.55} {cy+ry} {cx} {cy+ry} C{cx-rx*0.55} {cy+ry} {cx-rx} {cy+ry*0.8} {cx-rx} {cy+20}Z" fill="url(#body)"/>
<path d="M{cx-rx} {cy+20} C{cx-rx} {cy-ry*0.55} {cx-60} {ky-8} {cx} {ky-18} C{cx+60} {ky-8} {cx+rx} {cy-ry*0.55} {cx+rx} {cy+20} C{cx+rx} {cy+ry*0.8} {cx+rx*0.55} {cy+ry} {cx} {cy+ry} C{cx-rx*0.55} {cy+ry} {cx-rx} {cy+ry*0.8} {cx-rx} {cy+20}Z" fill="url(#rim)"/>
{pleats(cx, ky, 9, 1.15, 128, 12, p['pleat'], p['hl'])}
<ellipse cx="{cx}" cy="{ky+4}" rx="30" ry="22" fill="url(#knot)"/>
<path d="M{cx-14} {ky+2} Q{cx} {ky-14} {cx+14} {ky+2} Q{cx} {ky+14} {cx-14} {ky+2}Z" fill="{p['pleat']}" opacity=".8"/>
<ellipse cx="{cx-rx*0.42}" cy="{cy-ry*0.18}" rx="40" ry="22" transform="rotate(-32 {cx-rx*0.42} {cy-ry*0.18})" fill="#fff" opacity=".55" filter="url(#blur6)"/>
<ellipse cx="{cx-rx*0.62}" cy="{cy+ry*0.12}" rx="9" ry="5" transform="rotate(-50 {cx-rx*0.62} {cy+ry*0.12})" fill="#fff" opacity=".8"/>
'''
    if with_face:
        s += face(cx, cy+26, kind, ink=p['ink'], blush=p['blush'])
    return s

def steam(cx=256, y=70, col='#fff', op=.85):
    s = ''
    for dx, h in ((-46, 54), (0, 70), (46, 54)):
        x = cx+dx
        s += f'<path d="M{x} {y+h} C{x-18} {y+h*0.72} {x+18} {y+h*0.45} {x} {y+h*0.2} C{x-8} {y+h*0.08} {x} {y} {x} {y}" fill="none" stroke="{col}" stroke-width="12" stroke-linecap="round" opacity="{op}"/>'
    return s

DEFS = '''<defs>
<filter id="blur14" x="-30%" y="-100%" width="160%" height="300%"><feGaussianBlur stdDeviation="14"/></filter>
<filter id="blur6" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="6"/></filter>
<filter id="ds" x="-20%" y="-20%" width="140%" height="150%"><feDropShadow dx="0" dy="6" stdDeviation="6" flood-color="#000" flood-opacity=".28"/></filter>
<radialGradient id="body" cx="36%" cy="30%" r="85%"><stop offset="0" stop-color="{b0}"/><stop offset=".55" stop-color="{b1}"/><stop offset="1" stop-color="{b2}"/></radialGradient>
<radialGradient id="rim" cx="50%" cy="45%" r="62%"><stop offset=".72" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="{rim}" stop-opacity=".45"/></radialGradient>
<radialGradient id="knot" cx="40%" cy="35%" r="80%"><stop offset="0" stop-color="{b0}"/><stop offset="1" stop-color="{b2}"/></radialGradient>
<linearGradient id="tile" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{t0}"/><stop offset="1" stop-color="{t1}"/></linearGradient>
<radialGradient id="tileglow" cx="50%" cy="0%" r="90%"><stop offset="0" stop-color="#fff" stop-opacity=".45"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></radialGradient>
{extra}
</defs>'''

def doc(inner, pal, tile=True, extra='', size=512):
    d = DEFS.format(extra=extra, **pal)
    bg = ''
    if tile:
        bg = '<rect width="512" height="512" rx="112" fill="url(#tile)"/><rect width="512" height="512" rx="112" fill="url(#tileglow)"/><rect x="1.5" y="1.5" width="509" height="509" rx="110.5" fill="none" stroke="#fff" stroke-opacity=".25" stroke-width="3"/>'
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="{size}" height="{size}">{d}{bg}{inner}</svg>'

CREAM = dict(b0='#FFFBF0', b1='#FFE9C7', b2='#F5BE86', rim='#C9702F', pleat='#E8A56D', hl='#FFFAF0', ink='#3a211b', blush='#ff8fa3')

concepts = {}

# 1. Classic - warm orange tile, cheerful buuz
pal1 = dict(CREAM, t0='#FFB35C', t1='#FF6A45')
inner = steam(256, 22, '#fff', .9) + dumpling(256, 306, pal=pal1)
concepts['1-classic'] = (inner, pal1, '')

# 2. Steamer pal - bamboo steamer
pal2 = dict(CREAM, t0='#8FD0FF', t1='#4C7DFF')
bam = '''
<g filter="url(#ds)">
<rect x="82" y="352" width="348" height="96" rx="26" fill="url(#bam)"/>
<rect x="82" y="352" width="348" height="26" rx="13" fill="#E9B766"/>
<rect x="82" y="418" width="348" height="30" rx="15" fill="#C98A3B" opacity=".75"/>
'''
for x in range(130, 420, 52):
    bam += f'<rect x="{x}" y="380" width="5" height="40" rx="2.5" fill="#B9772B" opacity=".55"/>'
bam += '<rect x="96" y="356" width="320" height="8" rx="4" fill="#fff" opacity=".35"/></g>'
extra2 = '<linearGradient id="bam" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#F6D08A"/><stop offset="1" stop-color="#D79A48"/></linearGradient>'
d2 = dumpling(256, 262, rx=150, ry=132, pal=pal2)
inner = steam(256, 10, '#fff', .9) + d2 + bam
concepts['2-steamer'] = (inner, pal2, extra2)

# 3. Glyph - deep blue, no face, Cyrillic Б embossed
pal3 = dict(CREAM, t0='#4D7BFF', t1='#1B2A9E')
gl = '''
<g stroke-linecap="round" stroke-linejoin="round" fill="none">
<path d="M312 252 H232 V362 H270 a38 38 0 0 0 0 -76 H232" stroke="#fff" stroke-opacity=".8" stroke-width="26" transform="translate(0 4)"/>
<path d="M312 252 H232 V362 H270 a38 38 0 0 0 0 -76 H232" stroke="url(#ink)" stroke-width="24"/>
</g>'''
extra3 = '<linearGradient id="ink" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#3C63E8"/><stop offset="1" stop-color="#1B2A9E"/></linearGradient>'
inner = steam(256, 22, '#fff', .85) + dumpling(256, 306, pal=pal3, with_face=False) + '<g transform="translate(256 300) scale(1.3) translate(-272 -307)">'+gl+'</g>'
concepts['3-glyph'] = (inner, pal3, extra3)

# 4. Clay / jelly - pink-lilac, sparkly eyes, sprinkles
pal4 = dict(b0='#FFF4F8', b1='#FFD3E4', b2='#F59AC0', rim='#B2407D', pleat='#EC86B1', hl='#FFF3F8', ink='#3b1a35', blush='#ff5d93', t0='#FFC0DE', t1='#9B7BFF')
spk = ''
for x, y, s_, o in ((74, 110, 20, .9), (440, 150, 26, .9), (420, 70, 12, .8), (90, 250, 13, .7)):
    spk += f'<path d="M{x} {y-s_} Q{x} {y} {x+s_} {y} Q{x} {y} {x} {y+s_} Q{x} {y} {x-s_} {y} Q{x} {y} {x} {y-s_}Z" fill="#fff" opacity="{o}"/>'
inner = spk + dumpling(256, 302, rx=192, ry=166, pal=pal4, kind='sparkle')
concepts['4-jelly'] = (inner, pal4, '')

for k, (inner, pal, extra) in concepts.items():
    open(f'{out}/concept-{k}-icon.svg', 'w').write(doc(inner, pal, True, extra))
    open(f'{out}/concept-{k}-mark.svg', 'w').write(doc(inner, pal, False, extra))
print('ok')
