"""Make the installer artwork: the Welcome/Finish side panel and the inner-page header.
NSIS MUI sizes: side panel 164x314, header 150x57 (drawn at 2x and stretched for sharp high-DPI display)."""
import math
from PIL import Image, ImageDraw, ImageFont, ImageFilter

B = '/home/claude/tunas/brand'
OUT = '/home/claude/tunas/installer'
F = lambda w, s: ImageFont.truetype(f'/tmp/pjs-{w}.ttf', s)


def fit(path, max_w, max_h):
    im = Image.open(path).convert('RGBA')
    im = im.crop(im.getbbox())
    r = min(max_w / im.width, max_h / im.height)
    return im.resize((max(1, round(im.width * r)), max(1, round(im.height * r))), Image.LANCZOS)


def navy_bg(w, h):
    bg = Image.new('RGB', (w, h))
    px = bg.load()
    for y in range(h):
        for x in range(w):
            # radial glow top-left, navy body
            d = math.hypot(x - w * 0.15, y - h * 0.05) / (h * 0.9)
            t = min(1, d)
            c0, c1 = (0x1D, 0x2A, 0x5E), (0x0B, 0x10, 0x22)
            px[x, y] = tuple(round(c0[i] * (1 - t) + c1[i] * t) for i in range(3))
    return bg


def waves(img, y0, amp, count=5):
    """The S.A.L.T multicolour wave artwork, drawn as soft thin lines near the bottom."""
    w, h = img.size
    layer = Image.new('RGBA', (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    cols = [(0x2F, 0x74, 0xD8), (0x6D, 0x4F, 0xE0), (0xE3, 0xC2, 0x54), (0xF0, 0x5A, 0x28), (0x3F, 0xC8, 0xE4)]
    for i in range(count):
        pts = []
        for x in range(-10, w + 11, 4):
            y = y0 + i * amp * 0.35 + math.sin(x / w * math.pi * 2 + i * 0.9) * amp * (0.6 + i * 0.12)
            pts.append((x, y))
        d.line(pts, fill=cols[i % len(cols)] + (120,), width=3)
    layer = layer.filter(ImageFilter.GaussianBlur(0.6))
    img.paste(layer, (0, 0), layer)


def center_text(d, y, text, font, fill, w):
    tw = d.textlength(text, font=font)
    d.text(((w - tw) / 2, y), text, font=font, fill=fill)


def side_panel(scale=2):
    w, h = 164 * scale, 314 * scale
    img = navy_bg(w, h)
    waves(img, h * 0.955, 9 * scale, 4)
    d = ImageDraw.Draw(img)
    # Top: TUNAS mark and name
    mark = fit('/tmp/mark112.png', 54 * scale, 54 * scale)
    img.paste(mark, ((w - mark.width) // 2, 20 * scale), mark)
    center_text(d, 80 * scale, 'TUNAS', F(800, 19 * scale), (255, 255, 255), w)
    center_text(d, 104 * scale, 'Your own AI, on your notebook', F(400, 7.5 * scale), (0xB9, 0xC3, 0xE0), w)
    # Middle: the gold S.A.L.T logo (largest of the three)
    salt = fit(f'{B}/salt/salt-gold-3d.png', 88 * scale, 98 * scale)
    img.paste(salt, ((w - salt.width) // 2, 128 * scale), salt)
    center_text(d, 230 * scale, 'POWERED BY S.A.L.T CI', F(700, 6 * scale), (0xE3, 0xC2, 0x54), w)
    # Bottom: owner and distributor, smaller, side by side
    tech = fit(f'{B}/salt/salt-ci-tech-on-dark.png', 64 * scale, 22 * scale)
    pem = fit(f'{B}/salt/pemangkin-3d-silver-horizontal.png', 64 * scale, 18 * scale)
    gap = 8 * scale
    total = tech.width + gap + pem.width
    x0 = (w - total) // 2
    yb = 262 * scale
    img.paste(tech, (x0, yb + (22 * scale - tech.height) // 2), tech)
    img.paste(pem, (x0 + tech.width + gap, yb + (22 * scale - pem.height) // 2), pem)
    center_text(d, 290 * scale, 'www.salt.ci  ·  www.pemangkin.tech', F(600, 5.5 * scale), (0x8E, 0x98, 0xB8), w)
    return img


def header(scale=2):
    w, h = 150 * scale, 57 * scale
    img = Image.new('RGB', (w, h), (255, 255, 255))
    salt = fit(f'{B}/salt/salt-gold-3d.png', 60 * scale, 49 * scale)
    img.paste(salt, (w - salt.width - 6 * scale, (h - salt.height) // 2), salt)
    return img


if __name__ == '__main__':
    sp = side_panel()
    sp.save(f'{OUT}/welcome.bmp')
    sp.save('/tmp/welcome_preview.png')
    hd = header()
    hd.save(f'{OUT}/header.bmp')
    print('ok', sp.size, hd.size)
