"""Draws the app icon: a hard-disk platter with an AI sparkle, on a rounded
blue-violet tile. Run from the repo root; writes src-tauri/icons/*.

    python3 tools/make_icon.py
"""
from PIL import Image, ImageDraw, ImageFilter
import math, os

S = 1024          # master size
SS = 4            # supersampling
W = S * SS
out = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")

def rounded_tile():
    img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    # vertical gradient: deep blue -> violet
    grad = Image.new("RGBA", (W, W))
    top, bot = (37, 99, 235), (124, 58, 237)
    px = grad.load()
    for y in range(W):
        t = y / (W - 1)
        c = tuple(int(top[i] + (bot[i] - top[i]) * t) for i in range(3)) + (255,)
        for x in range(W):
            px[x, y] = c
    mask = Image.new("L", (W, W), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, W - 1, W - 1], radius=int(W * 0.22), fill=255)
    img.paste(grad, (0, 0), mask)
    return img

def platter(img):
    d = ImageDraw.Draw(img)
    cx = cy = W / 2 + W * 0.04
    r = W * 0.30
    # soft shadow
    sh = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    ImageDraw.Draw(sh).ellipse([cx - r, cy - r + W * 0.02, cx + r, cy + r + W * 0.02], fill=(0, 0, 0, 90))
    sh = sh.filter(ImageFilter.GaussianBlur(W * 0.02))
    img.alpha_composite(sh)
    d = ImageDraw.Draw(img)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(255, 255, 255, 255))
    # platter rings
    for k, a in ((0.78, 40), (0.58, 40), (0.40, 40)):
        rr = r * k
        d.ellipse([cx - rr, cy - rr, cx + rr, cy + rr], outline=(79, 70, 229, a), width=int(W * 0.008))
    # hub
    hr = r * 0.22
    d.ellipse([cx - hr, cy - hr, cx + hr, cy + hr], fill=(67, 56, 202, 255))
    hr2 = r * 0.08
    d.ellipse([cx - hr2, cy - hr2, cx + hr2, cy + hr2], fill=(255, 255, 255, 255))
    # read arm
    ax, ay = cx + r * 0.95, cy - r * 0.95
    d.line([(ax, ay), (cx + r * 0.30, cy + r * 0.05)], fill=(67, 56, 202, 255), width=int(W * 0.030))
    d.ellipse([ax - W * 0.028, ay - W * 0.028, ax + W * 0.028, ay + W * 0.028], fill=(67, 56, 202, 255))

def sparkle(img, cx, cy, r, color=(255, 255, 255, 255)):
    d = ImageDraw.Draw(img)
    pts = []
    for i in range(8):
        ang = math.pi / 4 * i - math.pi / 2
        rr = r if i % 2 == 0 else r * 0.28
        pts.append((cx + rr * math.cos(ang), cy + rr * math.sin(ang)))
    d.polygon(pts, fill=color)

img = rounded_tile()
platter(img)
sparkle(img, W * 0.76, W * 0.24, W * 0.13)
sparkle(img, W * 0.63, W * 0.13, W * 0.055)
sparkle(img, W * 0.22, W * 0.25, W * 0.045, (255, 255, 255, 210))

master = img.resize((S, S), Image.LANCZOS)
os.makedirs(out, exist_ok=True)
master.save(os.path.join(out, "icon.png"))
for name, size in (("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256)):
    master.resize((size, size), Image.LANCZOS).save(os.path.join(out, name))
master.save(os.path.join(out, "icon.ico"), sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
print("ok")
