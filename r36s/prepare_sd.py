#!/usr/bin/env python3
"""Put slot on an ArkOS card for the R36S, from a Mac or Linux PC.

  python3 r36s/prepare_sd.py                     # card mounted at /Volumes/EASYROMS
  python3 r36s/prepare_sd.py --card /media/EASYROMS --no-scrape

Additive unless --clean: ArkOS keeps working and nothing on the card is
deleted or changed outside EASYROMS/slot and EASYROMS/tools.

  1. copies dist-r36s/slot (from r36s/build.sh) to EASYROMS/slot, refreshing
     System/ and leaving saves, states and settings alone
  2. copies the Slot scripts to EASYROMS/tools (ArkOS > Options > Tools)
  3. imports the ArkOS gb, gbc and gba folders: unzips ROMs into
     slot/Games/<GB|GBC|GBA>, and carries RetroArch .srm saves over as .sav
  4. makes a cart label for every game that has none, from the box art
     EmulationStation already scraped or from thumbnails.libretro.com

  5. with --clean, once every game is in slot: deletes the folders of all the
     other systems, the ArkOS gb/gbc/gba copies, BIOS, music, videos and the
     ArkOS tools, keeping slot/, the Slot tools and the EmulationStation themes
     (EmulationStation is still the fallback when slot cannot start)

Needs Pillow for step 4 (pip install pillow); --no-scrape skips it.
"""

import argparse
import difflib
import html
import io
import re
import shutil
import subprocess
import sys
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

SYSTEMS = {
    ".gba": "GBA",
    ".gb": "GB",
    ".gbc": "GBC",
}

ARKOS_FOLDERS = ["gba", "gbc", "gb"]

LIBRETRO = {
    "GBA": "Nintendo - Game Boy Advance",
    "GB": "Nintendo - Game Boy",
    "GBC": "Nintendo - Game Boy Color",
}

# slot's cart label sizes, from card/Labels/README.txt
LABEL_SIZE = {
    "GBA": (1280, 661),
    "GB": (1024, 901),
    "GBC": (1024, 901),
}

THUMBS = "https://thumbnails.libretro.com"

FUZZY_CUTOFF = 0.88

# what --clean leaves at the top of EASYROMS
KEEP = {"slot", "tools", "themes"}


def say(*parts):
    print(*parts, flush=True)


def copy_tree(src: Path, dst: Path, overwrite: bool):
    for path in sorted(src.rglob("*")):
        rel = path.relative_to(src)
        out = dst / rel
        if path.is_dir():
            out.mkdir(parents=True, exist_ok=True)
        elif overwrite or not out.exists():
            out.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, out)


def install_slot(card: Path, dist: Path):
    built = dist / "slot"
    if not (built / "System" / "slot").is_file():
        sys.exit(f"no build at {built}, run r36s/build.sh first")
    root = card / "slot"
    for child in sorted(built.iterdir()):
        copy_tree(child, root / child.name, overwrite=child.name == "System")
    for script in (root / "System").glob("*.sh"):
        script.chmod(0o755)
    tools = card / "tools"
    tools.mkdir(exist_ok=True)
    for script in sorted((dist / "tools").glob("*.sh")):
        shutil.copyfile(script, tools / script.name)
    say(f"slot  -> {root}")
    say(f"tools -> {tools}: " + ", ".join(p.name for p in sorted((dist / 'tools').glob('*.sh'))))
    return root


def rom_in_zip(zf: zipfile.ZipFile):
    for info in zf.infolist():
        ext = Path(info.filename).suffix.lower()
        if ext in SYSTEMS and not Path(info.filename).name.startswith("."):
            return info, ext
    return None, None


def import_roms(card: Path, root: Path, skipped: list):
    """Returns [(system, stem, arkos folder)] for every game now in slot, and
    adds to skipped the ROMs it could not bring over."""
    games = []
    for folder in ARKOS_FOLDERS:
        src = card / folder
        if not src.is_dir():
            continue
        for path in sorted(src.iterdir()):
            if path.name.startswith(".") or not path.is_file():
                continue
            ext = path.suffix.lower()
            stem = path.stem
            if ext == ".zip":
                try:
                    with zipfile.ZipFile(path) as zf:
                        info, inner = rom_in_zip(zf)
                        if info is None:
                            say(f"  skip {path.name}: no GB/GBC/GBA ROM inside")
                            skipped.append(path)
                            continue
                        system = SYSTEMS[inner]
                        out = root / "Games" / system / f"{stem}{inner}"
                        if not out.exists():
                            out.parent.mkdir(parents=True, exist_ok=True)
                            with zf.open(info) as rom, open(out, "wb") as dst:
                                shutil.copyfileobj(rom, dst)
                            say(f"  rom  {system}/{out.name}")
                except zipfile.BadZipFile:
                    say(f"  skip {path.name}: not a readable zip")
                    skipped.append(path)
                    continue
            elif ext in SYSTEMS:
                system = SYSTEMS[ext]
                out = root / "Games" / system / path.name
                if not out.exists():
                    out.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(path, out)
                    say(f"  rom  {system}/{out.name}")
            elif ext == ".7z":
                say(f"  skip {path.name}: unzip it to .{folder} first, slot reads no .7z")
                skipped.append(path)
                continue
            else:
                continue
            games.append((system, stem, src))
    return games


def import_saves(root: Path, games):
    for system, stem, src in games:
        out = root / "Saves" / system / f"{stem}.sav"
        if out.exists():
            continue
        found = [p for p in [src / f"{stem}.srm", *src.glob(f"*/{glob_escape(stem)}.srm")] if p.is_file()]
        if not found:
            continue
        newest = max(found, key=lambda p: p.stat().st_mtime)
        out.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(newest, out)
        say(f"  save {system}/{out.name}  <- {newest.relative_to(src.parent)}")


def glob_escape(text: str) -> str:
    return re.sub(r"([*?\[])", r"[\1]", text)


def libretro_name(name: str) -> str:
    return re.sub(r'[&*/:`<>?\\|"]', "_", name)


def bare_title(name: str) -> str:
    title = re.sub(r"\([^)]*\)|\[[^\]]*\]", " ", name)
    return " ".join(re.sub(r"[^a-z0-9]+", " ", title.lower()).split())


def tags(name: str):
    return set(re.findall(r"\(([^)]*)\)", name))


class Thumbnails:
    def __init__(self):
        self.index = {}

    def names(self, system: str):
        if system not in self.index:
            url = f"{THUMBS}/{urllib.parse.quote(LIBRETRO[system])}/Named_Boxarts/"
            try:
                with urllib.request.urlopen(url, timeout=30) as r:
                    page = r.read().decode("utf-8", "replace")
                hrefs = re.findall(r'href="([^"]+\.png)"', page)
                self.index[system] = [html.unescape(urllib.parse.unquote(h)).rsplit("/", 1)[-1][:-4] for h in hrefs]
            except OSError as e:
                say(f"  thumbnails.libretro.com unreachable for {system}: {e}")
                self.index[system] = []
        return self.index[system]

    def match(self, system: str, stem: str):
        names = self.names(system)
        if not names:
            return None
        exact = libretro_name(stem)
        if exact in names:
            return exact
        title = bare_title(stem)
        same = [n for n in names if bare_title(n) == title]
        if not same:
            by_title = {}
            for n in names:
                by_title.setdefault(bare_title(n), []).append(n)
            close = difflib.get_close_matches(title, by_title.keys(), n=1, cutoff=FUZZY_CUTOFF)
            same = by_title[close[0]] if close else []
        if not same:
            return None
        want = tags(stem)
        return max(same, key=lambda n: (len(tags(n) & want), "Europe" in n or "USA" in n, -len(n)))

    def fetch(self, system: str, name: str) -> bytes:
        url = f"{THUMBS}/{urllib.parse.quote(LIBRETRO[system])}/Named_Boxarts/{urllib.parse.quote(name)}.png"
        with urllib.request.urlopen(url, timeout=30) as r:
            return r.read()


def make_label(art: bytes, size, out: Path):
    from PIL import Image, ImageEnhance, ImageFilter, ImageOps

    box = Image.open(io.BytesIO(art)).convert("RGB")
    back = ImageOps.fit(box, size, Image.LANCZOS)
    back = back.filter(ImageFilter.GaussianBlur(size[0] // 40))
    back = ImageEnhance.Brightness(back).enhance(0.55)
    front = ImageOps.contain(box, size, Image.LANCZOS)
    back.paste(front, ((size[0] - front.width) // 2, (size[1] - front.height) // 2))
    out.parent.mkdir(parents=True, exist_ok=True)
    back.save(out, optimize=True)


def scrape(root: Path, games, card: Path):
    try:
        import PIL  # noqa: F401
    except ImportError:
        say("  Pillow missing: pip install pillow, or pass --no-scrape")
        return
    thumbs = Thumbnails()
    missing = []
    for system, stem, src in games:
        out = root / "Labels" / system / f"{stem}.png"
        if out.exists():
            continue
        art, source = None, None
        for candidate in [src / "images" / f"{stem}.png", src / "images" / f"{stem}-image.png"]:
            if candidate.is_file():
                art, source = candidate.read_bytes(), "ArkOS scrape"
                break
        if art is None:
            name = thumbs.match(system, stem)
            if name:
                try:
                    art, source = thumbs.fetch(system, name), f"libretro: {name}"
                except OSError as e:
                    say(f"  art  {stem}: {e}")
        if art is None:
            missing.append(f"{system}/{stem}")
            continue
        try:
            make_label(art, LABEL_SIZE[system], out)
            say(f"  art  {system}/{out.name}  <- {source}")
        except Exception as e:  # a broken image should not stop the rest
            say(f"  art  {stem}: {e}")
            missing.append(f"{system}/{stem}")
    if missing:
        say("  no art found for: " + "; ".join(missing))
        say(f"  drop a box art .png into {root}/Labels/<GBA|GB|GBC> named after the ROM, or make one at https://studio.slot-cfw.fyi")


def clean(card: Path, games, skipped):
    if skipped:
        say("  not cleaning, these would be lost: " + "; ".join(p.name for p in skipped))
        return
    for system, stem, _ in games:
        if not any((card / "slot" / "Games" / system).glob(f"{glob_escape(stem)}.*")):
            say(f"  not cleaning, {system}/{stem} is not in slot/Games")
            return
    for path in sorted(card.iterdir()):
        if path.name.startswith("._") and path.is_file():
            path.unlink()  # macOS metadata
            continue
        if path.name.startswith(".") or path.name in KEEP or not path.is_dir():
            continue
        shutil.rmtree(path)
        say(f"  rm   {path.name}/")
    for path in sorted((card / "tools").iterdir()):
        if path.name.startswith("Slot") or path.name.startswith("."):
            continue
        shutil.rmtree(path) if path.is_dir() else path.unlink()
        say(f"  rm   tools/{path.name}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--card", type=Path, default=Path("/Volumes/EASYROMS"), help="the EASYROMS partition")
    ap.add_argument("--dist", type=Path, default=REPO / "dist-r36s", help="output of r36s/build.sh")
    ap.add_argument("--no-scrape", action="store_true", help="skip box art")
    ap.add_argument("--no-import", action="store_true", help="skip ROMs and saves from the ArkOS folders")
    ap.add_argument("--clean", action="store_true", help="then delete everything slot does not use (back the card up first)")
    args = ap.parse_args()

    card = args.card
    if not (card / "tools").is_dir() or not ((card / "gba").is_dir() or (card / "slot").is_dir()):
        sys.exit(f"{card} does not look like an ArkOS EASYROMS partition")

    say("== slot")
    root = install_slot(card, args.dist)
    games, skipped = [], []
    if not args.no_import:
        say("== ROMs")
        games = import_roms(card, root, skipped)
        say("== saves")
        import_saves(root, games)
    else:
        for system in set(SYSTEMS.values()):
            for rom in sorted((root / "Games" / system).glob("*")):
                if not rom.name.startswith("."):
                    games.append((system, rom.stem, card / system.lower()))
    if not args.no_scrape:
        say("== art")
        scrape(root, games, card)
    if args.clean:
        if args.no_import:
            sys.exit("--clean needs the import, drop --no-import")
        say("== clean")
        clean(card, games, skipped)
    if sys.platform == "darwin":
        subprocess.run(["dot_clean", "-m", str(root)], check=False)
        subprocess.run(["dot_clean", "-m", str(card / "tools")], check=False)
    say(f"== done: {len(games)} games. Eject the card, then on the R36S open Options > Tools > Slot.")


if __name__ == "__main__":
    main()
