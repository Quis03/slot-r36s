# slot on the R36S (ArkOS)

A port of [slot](https://github.com/BrandonKowalski/slot) to the R36S. It runs
on top of ArkOS, which stays installed: the kernel, drivers and panel setup
are ArkOS' own, and slot only takes EmulationStation's place.

## What differs from slot for Anbernic

- **Display**: a KMS/GBM backend (`crates/slot-gfx/src/kms.rs`). The old fbdev
  backend stays as a fallback (`SLOT_GFX=fbdev|kms` forces one).
- **Buttons**: a map for the "GO-Super Gamepad" joypad (`crates/slot/src/input/evdev.rs`).
  FN = Menu, SELECT/START as labelled, the d-pad and left stick move.
- **Startup**: `card/System/r36s-launch.sh` and the scripts in `card/tools/`.
- **GBA core**: gpSP by default (`SLOT_GBA_CORE=gpsp` in the launcher), because
  mGBA cannot hold 60 fps on the RK3326. slot's core picker still works per game.
- **640×480 panel**: the GBA's 3× does not fit. Menu → Screen → GBA → Scale:
  *Integer* is a sharp 2× (480×320), *Fit* fills the width at 2.67× and is
  meant for the *Shimmerless* shader, which keeps the pixels even.
- **Audio**: speaker, headphone jack and USB-C sound cards. slot plays on a
  USB-C card while one is plugged in, and goes back to the speaker or
  headphones when it is pulled out, without restarting the game.

## Build

```bash
r36s/build.sh          # makes dist-r36s/ (needs Docker or Colima)
r36s/build.sh test     # runs the workspace tests in Linux
```

## Preparing the card

With the SD card in the computer:

```bash
python3 r36s/prepare_sd.py                              # macOS, card at /Volumes/EASYROMS
python3 r36s/prepare_sd.py --card /media/$USER/EASYROMS # Linux
```

It copies `slot/` and the tools to EASYROMS, unzips the ArkOS GB/GBC/GBA ROMs
into `slot/Games`, carries the `.srm` saves over to `slot/Saves` as `.sav` and
makes the cart labels. For each game it uses the first it finds:

1. the real cart label from slot's art set (art.slot-cfw.fyi, the one Cart
   Studio uses), by ROM checksum, or by name for ROM hacks;
2. the game's logo, from that set or from ArkOS' own scrape, over its blurred
   box art;
3. the box art, from ArkOS or thumbnails.libretro.com.

It deletes nothing and can be run again after adding games: it only makes the
labels that are missing, and `--relabel` remakes them all. If the ArkOS folders
are no longer on the card, `--arkos` says where they are (a backup, for
example). For a game with nothing, put a `.png` with the same name in
`slot/Labels/GBA` (or `GB`, `GBC`).

With `--clean`, after the import, it deletes everything slot does not use: the
folders of the other systems, the ArkOS gb/gbc/gba copies, BIOS, music, videos
and the ArkOS tools. `slot/`, the Slot tools and the EmulationStation themes
stay, since EmulationStation is still the fallback. It stops if a game did not
make it into `slot/Games`. Back the card up first:

```bash
diskutil list external   # find the card, for example disk4
sudo dd if=/dev/rdisk4 bs=4m status=progress | gzip > r36s-sd.img.gz
```

## On the console

ArkOS → Options → **Tools**:

| Entry | What it does |
|---|---|
| **Slot** | Try slot. Power off and Restart inside slot bring you back to ArkOS |
| **Slot - Install** | The console boots straight into slot. Also turns off Samba, SSH, Bluetooth, the other sharing services and the ArkOS helpers that only serve EmulationStation (Wi-Fi icon, video player buttons, auto suspend) |
| **Slot - Uninstall** | EmulationStation is back at boot, and the services Install turned off are on again |

Wallpapers are PNGs in `EASYROMS/slot/Wallpapers`, best at 640×480; pick one
from the quick menu (FN) → Wallpaper.

If slot hangs or does not start, EmulationStation comes back by itself at the
next boot. To start ArkOS once without uninstalling, create an empty file
`EASYROMS/slot/ARKOS` from a computer.

While slot runs, the audio codec's mixer (RK817) is routed to the speaker and
set to full, since slot sets the volume itself; ArkOS' levels come back when
it exits. The values before and after are in `slot-session.log`. ArkOS' hotkey
daemons (`ogage`, `oga_events`) are paused for as long as slot runs, because
slot handles volume and Power itself.

Logs and diagnostics are in `EASYROMS/slot/Logs` (`slot.log`,
`slot-session.log`, `diag.txt`). `diag.txt` is only rewritten on test runs
from Tools. For detailed traces (every button, audio pacing, latency) create
an empty file `EASYROMS/slot/TRACE`: they cost a few frames, because slot
syncs the card every second while the log grows, so remove it afterwards.
