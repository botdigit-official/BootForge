#!/usr/bin/env python3
import os
import subprocess
from PIL import Image, ImageDraw, ImageFont

def create_bootforge_icon():
    size = (1024, 1024)
    img = Image.new("RGBA", size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Draw rounded app background (macOS squircle style)
    # Background gradient from deep dark navy (#0B0F19) to dark blue (#131B2E)
    w, h = size
    r = 224 # macOS corner radius for 1024px

    # Create mask for rounded rectangle
    mask = Image.new("L", size, 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle([32, 32, 992, 992], radius=r, fill=255)

    # Draw gradient background onto a base layer
    base = Image.new("RGBA", size, (11, 15, 25, 255))
    base_draw = ImageDraw.Draw(base)

    # Gradient overlay
    for y in range(h):
        ratio = y / h
        r_c = int(11 + (24 - 11) * ratio)
        g_c = int(15 + (35 - 15) * ratio)
        b_c = int(25 + (58 - 25) * ratio)
        base_draw.line([(0, y), (w, y)], fill=(r_c, g_c, b_c, 255))

    # Outer border glow
    img.paste(base, (0, 0), mask)
    draw.rounded_rectangle([32, 32, 992, 992], radius=r, outline=(42, 59, 92, 255), width=8)

    # Inner subtle highlight
    draw.rounded_rectangle([40, 40, 984, 984], radius=r-8, outline=(59, 130, 246, 60), width=4)

    # Draw stylized USB / Boot / Bolt symbol in center
    # Center box / USB connector
    usb_top = 220
    usb_bottom = 360
    usb_left = 432
    usb_right = 592
    
    # USB plug
    draw.rounded_rectangle([usb_left, usb_top, usb_right, usb_bottom], radius=16, fill=(30, 41, 59, 255), outline=(59, 130, 246, 200), width=6)
    # Plug pins
    draw.rectangle([460, 250, 490, 310], fill=(59, 130, 246, 220))
    draw.rectangle([534, 250, 564, 310], fill=(59, 130, 246, 220))

    # USB Body / Drive
    body_top = 340
    body_bottom = 800
    body_left = 370
    body_right = 654
    draw.rounded_rectangle([body_left, body_top, body_right, body_bottom], radius=48, fill=(19, 27, 46, 255), outline=(34, 197, 94, 220), width=8)

    # Lightning / Power Bolt in center of body
    bolt_points = [
        (530, 390),
        (430, 540),
        (505, 540),
        (470, 730),
        (610, 510),
        (535, 510),
        (575, 390),
    ]
    draw.polygon(bolt_points, fill=(34, 197, 94, 255))
    # Inner bright core for bolt
    inner_bolt = [
        (525, 410),
        (450, 530),
        (510, 530),
        (485, 680),
        (585, 520),
        (530, 520),
        (560, 410),
    ]
    draw.polygon(inner_bolt, fill=(187, 247, 208, 255))

    # LED Indicator dot at bottom
    draw.ellipse([497, 745, 527, 775], fill=(59, 130, 246, 255), outline=(147, 197, 253, 255), width=3)

    os.makedirs("assets/icons", exist_ok=True)
    png_path = "assets/icons/bootforge-icon-1024.png"
    img.save(png_path, "PNG")
    print(f"Generated {png_path}")

    # Build .iconset and .icns
    iconset_dir = "assets/icons/BootForge.iconset"
    os.makedirs(iconset_dir, exist_ok=True)

    sizes = [16, 32, 64, 128, 256, 512, 1024]
    for s in sizes:
        if s <= 512:
            resized = img.resize((s, s), Image.Resampling.LANCZOS)
            resized.save(os.path.join(iconset_dir, f"icon_{s}x{s}.png"))
        if s >= 32:
            half = s // 2
            resized2x = img.resize((s, s), Image.Resampling.LANCZOS)
            resized2x.save(os.path.join(iconset_dir, f"icon_{half}x{half}@2x.png"))

    icns_path = "assets/icons/AppIcon.icns"
    subprocess.run(["iconutil", "-c", "icns", iconset_dir, "-o", icns_path], check=True)
    print(f"Generated {icns_path}")

if __name__ == "__main__":
    create_bootforge_icon()
