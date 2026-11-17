# SPDX-License-Identifier: MIT OR Apache-2.0
"""Make WebP copies of the exported PNGs (lossless, keeps transparency).

    python3 brand/webp.py
"""
import os
from PIL import Image

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "export")
for name in ["buuz-buddy-512.png", "buuz-buddy-1024.png", "buuz-wordmark.png", "buuz-wordmark-light.png"]:
    Image.open(os.path.join(out, name)).save(os.path.join(out, name[:-4] + ".webp"), lossless=True)
print("wrote 4 webp files to", out)
