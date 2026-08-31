#!/usr/bin/env python3
"""Generate FidoCAD app icon: copper superellipse with a minimal circuit."""

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
N = 5.0  # superellipse exponent (iOS-like squircle)
PAD_RATIO = 0.012


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


def write_svg(path: Path) -> None:
    s = 1024.0
    inset = s * PAD_RATIO
    rx = ry = (s / 2) - inset
    cx = cy = s / 2
    shape = superellipse(cx, cy, rx, ry, N, 256)

    # Circuit in 1024 space
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


def render_png(path: Path, out_size: int = SIZE) -> None:
    hi = out_size * SUPERSAMPLE
    scale = hi / 1024.0
    inset = 1024 * PAD_RATIO * scale
    cx = cy = hi / 2
    rx = ry = (hi / 2) - inset
    pts = superellipse(cx, cy, rx, ry, N, 1440)

    yy, xx = np.mgrid[0:hi, 0:hi]
    t = np.clip((xx * 0.42 + yy * 0.58) / hi, 0, 1)
    # copper gradient
    c0 = np.array([232, 181, 122], dtype=np.float32)  # #E8B57A
    c1 = np.array([196, 122, 58], dtype=np.float32)  # #C47A3A
    c2 = np.array([138, 74, 34], dtype=np.float32)  # #8A4A22
    mid = np.where(t[..., None] < 0.42, t[..., None] / 0.42, (t[..., None] - 0.42) / 0.58)
    left = np.where(t[..., None] < 0.42, c0, c1)
    right = np.where(t[..., None] < 0.42, c1, c2)
    rgb = left + (right - left) * mid
    # top-left sheen
    sheen = np.clip(1.0 - np.sqrt(((xx / hi - 0.28) ** 2) + ((yy / hi - 0.22) ** 2)) * 1.35, 0, 1)
    rgb = rgb + sheen[..., None] * np.array([28, 16, 8])
    rgb = np.clip(rgb, 0, 255).astype(np.uint8)
    copper = Image.fromarray(rgb, "RGB")

    mask = Image.new("L", (hi, hi), 0)
    ImageDraw.Draw(mask).polygon(pts, fill=255)
    mask = mask.filter(ImageFilter.GaussianBlur(radius=SUPERSAMPLE * 0.35))

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
    for x, y in pads:
        draw.ellipse((x - r, y - r, x + r, y + r), fill=dark)
        draw.ellipse((x - rh, y - rh, x + rh, y + rh), fill=hole)

    base = Image.new("RGBA", (hi, hi), (0, 0, 0, 0))
    base.paste(copper, mask=mask)
    # keep circuit inside the squircle
    circ_m = Image.new("L", (hi, hi), 0)
    circ_m.paste(circuit.split()[3], mask=mask)
    tinted = Image.new("RGBA", (hi, hi), (0, 0, 0, 0))
    tinted.paste(circuit, mask=circ_m)
    out_hi = Image.alpha_composite(base, tinted)
    out = out_hi.resize((out_size, out_size), Image.Resampling.LANCZOS)
    path.parent.mkdir(parents=True, exist_ok=True)
    out.save(path, "PNG")


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
    source = TAURI_ICONS / "icon-source.png"
    render_png(source, 1024)
    master = Image.open(source)
    pngs = {
        TAURI_ICONS / "32x32.png": 32,
        TAURI_ICONS / "128x128.png": 128,
        TAURI_ICONS / "128x128@2x.png": 256,
        TAURI_ICONS / "icon.png": 512,
        PUBLIC / "favicon.png": 32,
        PUBLIC / "apple-touch-icon.png": 180,
    }
    for dest, size in pngs.items():
        master.resize((size, size), Image.Resampling.LANCZOS).save(dest, "PNG")
    write_ico(master, TAURI_ICONS / "icon.ico")
    write_icns(master, TAURI_ICONS / "icon.icns")
    print(f"wrote {svg_out}")
    print(f"wrote {source}")


if __name__ == "__main__":
    main()
