#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Make macos/Resources/Buuz-menu.png: the single-colour Б glyph for the
Input menu. Run once, then commit the PNG. Needs Pillow and a bold sans font
with Cyrillic (DejaVu Sans Bold)."""
from PIL import Image, ImageDraw, ImageFont

FONT = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"
SIZE = 32  # 16 pt at 2x
LIFT = 3  # px; the menu bar draws a centred glyph too low, so lift it

img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)
font = ImageFont.truetype(FONT, 28)
box = draw.textbbox((0, 0), "Б", font=font)
x = (SIZE - (box[2] - box[0])) / 2 - box[0]
y = (SIZE - (box[3] - box[1])) / 2 - box[1] - LIFT
draw.text((x, y), "Б", font=font, fill=(0, 0, 0, 255))
img.save("macos/Resources/Buuz-menu.png")
