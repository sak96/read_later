#!/usr/bin/env python3
import json
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
METADATA_DIR = SCRIPT_DIR.parent / "fastlane" / "metadata" / "android"
IMAGES_DIR = METADATA_DIR / "en-US" / "images"
OUTPUT = SCRIPT_DIR / "data.json"

SAMPLE_DATA = {
    "default": "en-US",
    "locales": {
        "en-US": {
            "title": "{app title}",
            "shortDescription": "{short description}",
            "fullDescription": "{full description}",
            "images": {
                "featureGraphic": "{feature graphic}",
                "icon": "{icon image}",
                "screenshots": [],
            },
        }
    },
}


def reset():
    OUTPUT.write_text(json.dumps(SAMPLE_DATA, indent=2, ensure_ascii=False) + "\n")
    print(f"Reset {OUTPUT} to sample data.")


def discover_images():
    images = {}
    if not IMAGES_DIR.exists():
        return images

    for f in sorted(IMAGES_DIR.iterdir()):
        if f.is_file() and f.suffix.lower() in (".png", ".jpg", ".jpeg", ".webp"):
            images[f.stem] = f.name

    screenshots_dir = IMAGES_DIR / "phoneScreenshots"
    if screenshots_dir.exists():
        shots = sorted(
            f.name
            for f in screenshots_dir.iterdir()
            if f.is_file() and f.suffix.lower() in (".png", ".jpg", ".jpeg", ".webp")
        )
        if shots:
            images["screenshots"] = shots

    return images


def generate():
    all_images = discover_images()
    data = {"default": "en-US", "locales": {}}

    for locale_dir in sorted(METADATA_DIR.iterdir()):
        if not locale_dir.is_dir():
            continue
        title_file = locale_dir / "title.txt"
        if not title_file.exists():
            continue

        locale = locale_dir.name
        title = (locale_dir / "title.txt").read_text().strip()
        short = (locale_dir / "short_description.txt").read_text().strip()
        full = (locale_dir / "full_description.txt").read_text().strip()

        images = {}
        for key in sorted(all_images):
            if key == "screenshots":
                images["screenshots"] = all_images["screenshots"]
            else:
                images[key] = all_images[key]

        data["locales"][locale] = {
            "title": title,
            "shortDescription": short,
            "fullDescription": full,
            "images": images,
        }

    OUTPUT.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
    print(f"Generated {OUTPUT} with {len(data['locales'])} locales.")


if __name__ == "__main__":
    if "--reset" in sys.argv:
        reset()
    else:
        generate()
