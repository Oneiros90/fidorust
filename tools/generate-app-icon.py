#!/usr/bin/env python3
"""Generate FidoCAD app icon: copper plate with a minimal circuit.

Windows/Linux/web keep a full-bleed superellipse. macOS does not remask
.icns — the Dock draws whatever silhouette is in the file — so the Mac
asset is composited on Apple's Big Sur icon grid:

  1024 canvas, 824 plate, radius 185.4, continuous-corner smoothing 0.7,
  plus the template drop shadow. Otherwise the tile reads larger and
  rounder than every neighbouring app.

See: https://developer.apple.com/design/human-interface-guidelines/app-icons
     https://developer.apple.com/forums/thread/761179
"""

from __future__ import annotations

import math
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
TAURI_ICONS = ROOT / "apps/ui/src-tauri/icons"
PUBLIC = ROOT / "apps/ui/public"

SIZE = 1024
SUPERSAMPLE = 4
N = 5.0  # superellipse exponent (Windows / Linux / web)
PAD_RATIO = 0.012

# Apple's macOS app-icon grid, in 1024-canvas units. Measured against
# shipping system icons (Calculator, Mail, Notes, …) and Apple's template.
MAC_GRID = 1024
MAC_ART = 824
MAC_RADIUS_RATIO = 0.225  # 185.4 / 824
MAC_SMOOTHING = 0.7  # 0 = circular arc; 0.7 matches Apple's continuous corner
MAC_SHADOW_ALPHA = 0.26
MAC_SHADOW_BLUR = 16.0
MAC_SHADOW_DY = 6.0


def superellipse(cx: float, cy: float, rx: float, ry: float, n: float, steps: int = 720):
    pts: list[tuple[float, float]] = []
    exp = 2.0 / n
    for i in range(steps):
        t = 2.0 * math.pi * i / steps
        c, s = math.cos(t), math.sin(t)
        x = cx + rx * math.copysign(abs(c) ** exp, c)
        y = cy + ry * math.copysign(abs(s) ** exp, s)
        pts.append((x, y))
    return pts


def lerp(a: tuple[int, int, int], b: tuple[int, int, int], t: float):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def svg_path(pts: list[tuple[float, float]]) -> str:
    d = [f"M {pts[0][0]:.2f} {pts[0][1]:.2f}"]
    d += [f"L {x:.2f} {y:.2f}" for x, y in pts[1:]]
    d.append("Z")
    return " ".join(d)


def _cubic(p0, p1, p2, p3, steps: int, skip_start: bool) -> list[tuple[float, float]]:
    pts: list[tuple[float, float]] = []
    start = 1 if skip_start else 0
    for i in range(start, steps + 1):
        t = i / steps
        u = 1.0 - t
        x = u**3 * p0[0] + 3 * u**2 * t * p1[0] + 3 * u * t**2 * p2[0] + t**3 * p3[0]
        y = u**3 * p0[1] + 3 * u**2 * t * p1[1] + 3 * u * t**2 * p2[1] + t**3 * p3[1]
        pts.append((x, y))
    return pts


def _svg_arc(
    x1: float,
    y1: float,
    rx: float,
    ry: float,
    large: int,
    sweep: int,
    x2: float,
    y2: float,
    steps: int,
) -> list[tuple[float, float]]:
    """Sample an SVG endpoint-parameterized circular/elliptical arc, excluding the start."""
    rx, ry = abs(rx), abs(ry)
    if rx == 0 or ry == 0:
        return [(x2, y2)]
    dx = (x1 - x2) / 2.0
    dy = (y1 - y2) / 2.0
    lam = (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry)
    if lam > 1:
        s = math.sqrt(lam)
        rx *= s
        ry *= s
    num = rx * rx * ry * ry - rx * rx * dy * dy - ry * ry * dx * dx
    den = rx * rx * dy * dy + ry * ry * dx * dx
    coef = math.sqrt(max(0.0, num / den))
    if large == sweep:
        coef = -coef
    cxp = coef * (rx * dy) / ry
    cyp = coef * -(ry * dx) / rx
    cx = cxp + (x1 + x2) / 2.0
    cy = cyp + (y1 + y2) / 2.0

    def angle(ux: float, uy: float, vx: float, vy: float) -> float:
        n = math.hypot(ux, uy) * math.hypot(vx, vy)
        if n == 0:
            return 0.0
        cos_a = max(-1.0, min(1.0, (ux * vx + uy * vy) / n))
        a = math.acos(cos_a)
        if ux * vy - uy * vx < 0:
            a = -a
        return a

    theta1 = angle(1, 0, (x1 - cx) / rx, (y1 - cy) / ry)
    dtheta = angle((x1 - cx) / rx, (y1 - cy) / ry, (x2 - cx) / rx, (y2 - cy) / ry)
    if not sweep and dtheta > 0:
        dtheta -= 2 * math.pi
    elif sweep and dtheta < 0:
        dtheta += 2 * math.pi

    pts: list[tuple[float, float]] = []
    for i in range(1, steps + 1):
        th = theta1 + dtheta * i / steps
        pts.append((cx + rx * math.cos(th), cy + ry * math.sin(th)))
    return pts


def continuous_rounded_square(
    x: float, y: float, side: float, radius: float, smoothing: float, steps: int = 24
) -> list[tuple[float, float]]:
    """Apple/Figma continuous-corner rounded square (clockwise from top-right)."""
    r, s = radius, smoothing
    p = (1 + s) * r
    arc_measure = 90 * (1 - s)
    arc = math.sin(math.radians(arc_measure / 2)) * r * math.sqrt(2)
    angle_alpha = (90 - arc_measure) / 2
    angle_beta = 45 * s
    c = r * math.tan(math.radians(angle_alpha / 2)) * math.cos(math.radians(angle_beta))
    d = c * math.tan(math.radians(angle_beta))
    b = (p - arc - c - d) / 3
    a = 2 * b

    pts: list[tuple[float, float]] = []

    def cubic_rel(p0, dx1, dy1, dx2, dy2, dx, dy):
        p1 = (p0[0] + dx1, p0[1] + dy1)
        p2 = (p0[0] + dx2, p0[1] + dy2)
        p3 = (p0[0] + dx, p0[1] + dy)
        pts.extend(_cubic(p0, p1, p2, p3, steps, skip_start=bool(pts)))
        return p3

    def arc_rel(p0, dx, dy):
        end = (p0[0] + dx, p0[1] + dy)
        pts.extend(_svg_arc(p0[0], p0[1], r, r, 0, 1, end[0], end[1], steps))
        return end

    cur = (x + side - p, y)
    pts.append(cur)
    cur = cubic_rel(cur, a, 0, a + b, 0, a + b + c, d)
    cur = arc_rel(cur, arc, arc)
    cur = cubic_rel(cur, d, c, d, b + c, d, a + b + c)

    cur = (x + side, y + side - p)
    pts.append(cur)
    cur = cubic_rel(cur, 0, a, 0, a + b, -d, a + b + c)
    cur = arc_rel(cur, -arc, arc)
    cur = cubic_rel(cur, -c, d, -(b + c), d, -(a + b + c), d)

    cur = (x + p, y + side)
    pts.append(cur)
    cur = cubic_rel(cur, -a, 0, -(a + b), 0, -(a + b + c), -d)
    cur = arc_rel(cur, -arc, -arc)
    cur = cubic_rel(cur, -d, -c, -d, -(b + c), -d, -(a + b + c))

    cur = (x, y + p)
    pts.append(cur)
    cur = cubic_rel(cur, 0, -a, 0, -(a + b), d, -(a + b + c))
    cur = arc_rel(cur, arc, -arc)
    cubic_rel(cur, c, -d, b + c, -d, a + b + c, -d)
    return pts


def write_svg(path: Path) -> None:
    s = 1024.0
    inset = s * PAD_RATIO
    rx = ry = (s / 2) - inset
    cx = cy = s / 2
    shape = superellipse(cx, cy, rx, ry, N, 256)

    stroke = 58
    pads = [(300, 338), (724, 338), (512, 560), (724, 560)]
    traces = [
        [(300, 338), (724, 338)],
        [(512, 338), (512, 560)],
        [(512, 560), (724, 560)],
    ]
    via_r = 54
    via_hole = 22

    traces_d = []
    for t in traces:
        segs = [f"M {t[0][0]} {t[0][1]}"] + [f"L {x} {y}" for x, y in t[1:]]
        traces_d.append(" ".join(segs))

    svg = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">
  <defs>
    <linearGradient id="copper" x1="18%" y1="8%" x2="86%" y2="94%">
      <stop offset="0%" stop-color="#E8B57A"/>
      <stop offset="42%" stop-color="#C47A3A"/>
      <stop offset="100%" stop-color="#8A4A22"/>
    </linearGradient>
    <clipPath id="squircle">
      <path d="{svg_path(shape)}"/>
    </clipPath>
  </defs>
  <path d="{svg_path(shape)}" fill="url(#copper)"/>
  <g clip-path="url(#squircle)" fill="none" stroke="#2C160C" stroke-width="{stroke}" stroke-linecap="round" stroke-linejoin="round">
    {"".join(f'<path d="{d}"/>' for d in traces_d)}
  </g>
  <g clip-path="url(#squircle)">
    {"".join(f'<circle cx="{x}" cy="{y}" r="{via_r}" fill="#2C160C"/>' for x, y in pads)}
    {"".join(f'<circle cx="{x}" cy="{y}" r="{via_hole}" fill="#A05A2A"/>' for x, y in pads)}
  </g>
</svg>
'''
    path.write_text(svg)


def render_square_artwork(out_size: int) -> Image.Image:
    """Opaque copper square + circuit, in 1024 design space scaled to out_size."""
    hi = out_size
    scale = hi / 1024.0
    yy, xx = np.mgrid[0:hi, 0:hi]
    t = np.clip((xx * 0.42 + yy * 0.58) / hi, 0, 1)
    c0 = np.array([232, 181, 122], dtype=np.float32)  # #E8B57A
    c1 = np.array([196, 122, 58], dtype=np.float32)  # #C47A3A
    c2 = np.array([138, 74, 34], dtype=np.float32)  # #8A4A22
    left = np.where(t[..., None] < 0.42, c0, c1)
    right = np.where(t[..., None] < 0.42, c1, c2)
    mid = np.where(t[..., None] < 0.42, t[..., None] / 0.42, (t[..., None] - 0.42) / 0.58)
    rgb = left + (right - left) * mid
    sheen = np.clip(1.0 - np.sqrt(((xx / hi - 0.28) ** 2) + ((yy / hi - 0.22) ** 2)) * 1.35, 0, 1)
    rgb = rgb + sheen[..., None] * np.array([28, 16, 8])
    rgb = np.clip(rgb, 0, 255).astype(np.uint8)
    copper = Image.fromarray(rgb, "RGB").convert("RGBA")

    circuit = Image.new("RGBA", (hi, hi), (0, 0, 0, 0))
    draw = ImageDraw.Draw(circuit)
    sw = int(58 * scale)
    dark = (44, 22, 12, 255)
    hole = (160, 90, 42, 255)

    def P(x: float, y: float) -> tuple[int, int]:
        return (int(x * scale), int(y * scale))

    traces = [
        [P(300, 338), P(724, 338)],
        [P(512, 338), P(512, 560)],
        [P(512, 560), P(724, 560)],
    ]
    for tline in traces:
        draw.line(tline, fill=dark, width=sw, joint="curve")
    pads = [P(300, 338), P(724, 338), P(512, 560), P(724, 560)]
    r = int(54 * scale)
    rh = int(22 * scale)
    for px, py in pads:
        draw.ellipse((px - r, py - r, px + r, py + r), fill=dark)
        draw.ellipse((px - rh, py - rh, px + rh, py + rh), fill=hole)

    return Image.alpha_composite(copper, circuit)


def apply_superellipse_mask(square: Image.Image) -> Image.Image:
    hi = square.size[0]
    inset = 1024 * PAD_RATIO * (hi / 1024.0)
    cx = cy = hi / 2
    rx = ry = (hi / 2) - inset
    pts = superellipse(cx, cy, rx, ry, N, 1440)
    mask = Image.new("L", (hi, hi), 0)
    ImageDraw.Draw(mask).polygon(pts, fill=255)
    mask = mask.filter(ImageFilter.GaussianBlur(radius=max(SUPERSAMPLE * 0.35, 0.5)))
    rgba = np.array(square)
    rgba[..., 3] = np.array(mask)
    return Image.fromarray(rgba, "RGBA")


def apply_macos_icon_grid(square: Image.Image) -> Image.Image:
    """Place artwork on Apple's 824/1024 plate with continuous corners and shadow."""
    work = MAC_GRID * SUPERSAMPLE
    art = MAC_ART * SUPERSAMPLE
    inset = (work - art) // 2
    k = work / MAC_GRID

    art_img = square.resize((art, art), Image.Resampling.LANCZOS)
    radius = art * MAC_RADIUS_RATIO
    pts = continuous_rounded_square(inset, inset, art, radius, MAC_SMOOTHING, steps=32)

    mask = Image.new("L", (work, work), 0)
    ImageDraw.Draw(mask).polygon(pts, fill=255)
    mask = mask.filter(ImageFilter.GaussianBlur(radius=SUPERSAMPLE * 0.35))

    plate = Image.new("RGBA", (work, work), (0, 0, 0, 0))
    plate.paste(art_img, (inset, inset))
    pa = np.array(plate)
    pa[..., 3] = (pa[..., 3].astype(np.float32) * np.array(mask, dtype=np.float32) / 255.0).astype(
        np.uint8
    )
    plate = Image.fromarray(pa, "RGBA")

    blurred = mask.filter(ImageFilter.GaussianBlur(MAC_SHADOW_BLUR * k))
    shadow_a = (np.array(blurred, dtype=np.float32) * MAC_SHADOW_ALPHA).clip(0, 255).astype(np.uint8)
    shifted = Image.new("L", (work, work), 0)
    shifted.paste(Image.fromarray(shadow_a), (0, round(MAC_SHADOW_DY * k)))
    shadow = Image.new("RGBA", (work, work), (0, 0, 0, 0))
    shadow.putalpha(shifted)

    out = Image.alpha_composite(shadow, plate)
    return out.resize((MAC_GRID, MAC_GRID), Image.Resampling.LANCZOS)


def write_ico(master: Image.Image, path: Path) -> None:
    sizes = [16, 24, 32, 48, 64, 128, 256]
    master.save(path, format="ICO", sizes=[(s, s) for s in sizes])


def write_icns(master: Image.Image, path: Path) -> None:
    import shutil
    import subprocess
    import tempfile

    tmp = Path(tempfile.mkdtemp())
    iconset = tmp / "icon.iconset"
    iconset.mkdir()
    mapping = {
        "icon_16x16.png": 16,
        "icon_16x16@2x.png": 32,
        "icon_32x32.png": 32,
        "icon_32x32@2x.png": 64,
        "icon_128x128.png": 128,
        "icon_128x128@2x.png": 256,
        "icon_256x256.png": 256,
        "icon_256x256@2x.png": 512,
        "icon_512x512.png": 512,
        "icon_512x512@2x.png": 1024,
    }
    for name, size in mapping.items():
        dest = iconset / name
        master.resize((size, size), Image.Resampling.LANCZOS).save(dest, "PNG")
        subprocess.run(
            ["sips", "-s", "format", "png", str(dest), "--out", str(dest)],
            check=True,
            capture_output=True,
        )
    subprocess.run(["iconutil", "-c", "icns", "-o", str(path), str(iconset)], check=True)
    shutil.rmtree(tmp)


def main() -> None:
    TAURI_ICONS.mkdir(parents=True, exist_ok=True)
    PUBLIC.mkdir(parents=True, exist_ok=True)
    svg_out = TAURI_ICONS / "icon.svg"
    write_svg(svg_out)
    (PUBLIC / "favicon.svg").write_text(svg_out.read_text())

    square = render_square_artwork(SIZE * SUPERSAMPLE)
    windows = apply_superellipse_mask(square).resize((SIZE, SIZE), Image.Resampling.LANCZOS)
    macos = apply_macos_icon_grid(square)

    source = TAURI_ICONS / "icon-source.png"
    windows.save(source, "PNG")
    pngs = {
        TAURI_ICONS / "32x32.png": 32,
        TAURI_ICONS / "128x128.png": 128,
        TAURI_ICONS / "128x128@2x.png": 256,
        TAURI_ICONS / "icon.png": 512,
        PUBLIC / "favicon.png": 32,
        PUBLIC / "apple-touch-icon.png": 180,
    }
    for dest, size in pngs.items():
        windows.resize((size, size), Image.Resampling.LANCZOS).save(dest, "PNG")
    write_ico(windows, TAURI_ICONS / "icon.ico")
    write_icns(macos, TAURI_ICONS / "icon.icns")
    print(f"wrote {svg_out}")
    print(f"wrote {source}")
    print(f"wrote {TAURI_ICONS / 'icon.icns'} (macOS 824/1024 grid)")


if __name__ == "__main__":
    main()
