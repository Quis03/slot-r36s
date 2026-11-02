<div align="center">

<img src=".github/resources/banner.png" width="100%" alt="slot. A bespoke, Game Boy-centric frontend for the Anbernic RG SP.">

<a href="LICENSE"><img src="https://img.shields.io/github/license/BrandonKowalski/slot?style=for-the-badge&labelColor=242429&color=4d4d57" height="36" alt="License"></a>
<a href="https://github.com/BrandonKowalski/slot/stargazers"><img src="https://img.shields.io/github/stars/BrandonKowalski/slot?style=for-the-badge&labelColor=242429&color=4d4d57" height="36" alt="Stars"></a>
<a href="https://github.com/BrandonKowalski/slot/releases"><img src="https://img.shields.io/github/downloads/BrandonKowalski/slot/total?style=for-the-badge&labelColor=242429&color=4d4d57" height="36" alt="Downloads"></a>
<a href="https://slot-cfw.fyi"><img src="https://img.shields.io/badge/Guide-slot--cfw.fyi-4d4d57?style=for-the-badge&labelColor=242429" height="36" alt="Docs"></a>
<a href="https://discord.gg/NYRWsSuPUu"><img src="https://img.shields.io/badge/Discord-4d4d57?style=for-the-badge&labelColor=242429&logo=discord&logoColor=white" height="36" alt="Discord"></a>

</div>

---

# slot on the R36S

> [!NOTE]
> This is an unofficial fork that ports slot to the **R36S running ArkOS**.
> The original project, for the Anbernic RG SP, is
> [BrandonKowalski/slot](https://github.com/BrandonKowalski/slot). Please
> report problems with this port here, not there.

slot takes EmulationStation's place: the console boots straight into a shelf
of GB, GBC and GBA carts. ArkOS stays installed underneath (kernel, drivers,
display) and is one step away if you need it.

What differs from slot on the RG SP: KMS display output, the R36S button map,
gpSP as the default GBA core (mGBA cannot hold 60 fps on the RK3326), audio
through the speaker, headphone jack and USB-C sound cards, and real cart
labels, ROM hacks included. The technical details are in
[`r36s/README.md`](r36s/README.md).

## What you need

- An R36S running **ArkOS** that boots, with your GB/GBC/GBA games on the
  EASYROMS partition. Tested on ArkOS for the R36S (Ubuntu 19.10, kernel
  4.4.189), panel 4.
- A computer that can read the SD card, and Python 3 to prepare it.
- For cart labels: `pip3 install pillow`.

## Install

1. **Back up the SD card.** On a Mac, with the card inserted:

   ```bash
   diskutil list external   # find the card, for example disk4
   sudo dd if=/dev/rdisk4 bs=4m status=progress | gzip > r36s-sd.img.gz
   ```

   On Windows, [Win32 Disk Imager](https://sourceforge.net/projects/win32diskimager/)
   can save the whole card to a file.

2. **Download** `slot-r36s-*.zip` from the
   [latest release](https://github.com/Quis03/slot-r36s/releases/latest) and
   unzip it.

3. **Prepare the card.** With the card in the computer, from the unzipped
   folder:

   ```bash
   python3 prepare_sd.py                              # macOS
   python3 prepare_sd.py --card /media/$USER/EASYROMS # Linux
   python3 prepare_sd.py --card E:\                   # Windows, if EASYROMS is E:
   ```

   It copies slot and its tools to EASYROMS, moves the games from the ArkOS
   `gb`, `gbc` and `gba` folders into `slot/Games` (unzipping them), carries
   the `.srm` saves over to `slot/Saves` and makes a cart label for every
   game. It deletes nothing, so ArkOS keeps working as before. With `--clean`
   it removes whatever slot does not use (other systems, BIOS, videos); back
   the card up first.

   Without Python: copy the `slot` and `tools` folders from the zip to the
   root of EASYROMS, then copy your unzipped ROMs into `slot/Games/GBA`
   (or `GB`, `GBC`).

4. **Try it.** Eject the card, start the console and open ArkOS → Options →
   Tools → **Slot**. In this trial run, *Power off* and *Restart* inside slot
   bring you back to ArkOS.

5. **Install it.** If everything works: Options → Tools → **Slot - Install**.
   The console restarts straight into slot. Install also turns off the ArkOS
   services slot does not use (Samba, SSH, Bluetooth and the helpers that only
   serve EmulationStation), and remembers which ones were on.

### Building from source

Instead of steps 2 and 3, on macOS or Linux with `git` and **Docker** (on a
Mac, [Colima](https://github.com/abiosoft/colima) works too), since slot is
built for ARM inside a container:

```bash
git clone https://github.com/Quis03/slot-r36s.git
cd slot-r36s
r36s/build.sh                  # the first build downloads the toolchain and takes a while
python3 r36s/prepare_sd.py     # same options as above
```

## Using it

- **FN**: a short press opens the menu (the game menu while playing); hold it
  to eject the cart.
- **Power**: a short press pauses and turns the screen off (the next press
  turns it back on); hold it for a second to save the game and power off.
- The GBA's 3× does not fit the 640×480 screen: in Menu → Screen → GBA →
  Scale, pick *Fit* with the *Shimmerless* shader to fill the screen, or
  *Integer* for a sharp 2×.
- **Wallpapers**: put PNGs (640×480 is best) in `EASYROMS/slot/Wallpapers`
  and pick one from the menu → Wallpaper.
- **New games**: copy them, unzipped, into `EASYROMS/slot/Games/GBA` (or `GB`,
  `GBC`). Running `prepare_sd.py` again makes the labels that are missing.

## Going back to ArkOS

- **Once**: create an empty file `EASYROMS/slot/ARKOS` from a computer. The
  next boot starts EmulationStation; delete the file to get slot back.
- **For good**: with the `ARKOS` file in place, open ArkOS → Options → Tools →
  **Slot - Uninstall**. EmulationStation is back, and the services Install
  turned off are on again.
- If slot hangs or does not start, EmulationStation comes back by itself at
  the next boot.

## If something goes wrong

The logs are in `EASYROMS/slot/Logs` (`slot.log`, `slot-session.log`,
`diag.txt`). For detailed traces, create an empty file `EASYROMS/slot/TRACE`,
reproduce the problem and attach the logs to an
[issue](https://github.com/Quis03/slot-r36s/issues). Remove the file
afterwards, as it slows games down a little.

---

# What is slot?

A bespoke, Game Boy-centric frontend for the Anbernic RG SP.

Has support for GBA, GBC, and GB titles only.

> [!IMPORTANT]
> ROMs must be unzipped. slot does not read `.zip` or `.7z` files.

A full user guide can be found at [slot-cfw.fyi](https://slot-cfw.fyi).

---

# What does it look like?

<p align="center">
<img src=".github/resources/screenshots/carousel.gif" width="48%" alt="Scrolling the cart carousel across the Game Boy Advance, Game Boy and Game Boy Color shelves">
<img src=".github/resources/screenshots/core-picker.gif" width="48%" alt="Flipping a cart to its circuit board to choose mGBA or gpSP">
</p>

<p align="center">
<img src=".github/resources/screenshots/link.gif" width="96.5%" alt="Two SPs racing in Mario Kart over the emulated link">
</p>

---

# Supported Devices

| Device             | Supported                                    | Since Version |
|--------------------|----------------------------------------------|---------------|
| Anbernic RG SP     | Yes                                          | Always        |
| Anbernic RG34XX    | Yes                                          | 1.4.0         |
| Anbernic RG34XXSP  | Yes                                          | 1.4.0         |
| Anbernic RG35XXSP  | Yes                                          | 1.5.0         |
| Anbernic 35XX Plus | Yes                                          | 1.5.0         |
| Anbernic RG28XX    | Yes                                          | 1.6.0         |
| Anbernic RG35XX-H  | Yes                                          | 1.7.0         |

If you have one of the untested devices, or have tried slot on a device not listed,
please [open an issue on GitHub](https://github.com/BrandonKowalski/slot/issues) and let me know how it went.

---

# Discord Community

slot has a [Discord server](https://discord.gg/NYRWsSuPUu) for questions, sharing setups, and talking about the project.

It is open to everyone in the slot universe. That includes folks running or building forks of slot.

There is no reason for this community to be fragmented so long as we are having fun!

---

# AI Disclosure

The Rust frontend, and the SVGs on [slot-cfw.fyi](https://slot-cfw.fyi), were put together by Claude.
I reviewed everything that was produced. All documentation is 100% free-range, meatbag prose.

My goal with this project was to recreate the feeling of playing my GBA SP in the back seat as a kid,
but with a few modern creature comforts.

Nothing about this project is exciting or groundbreaking from a technological perspective, and as a
result, I delegated the glue code. mGBA and gpSP have been around forever and are the true stars of
this show. The incredible skeuomorphic art is the work of [Markus](https://github.com/MarkusMaribu), an
artist who saw the bare-bones version and offered to dress it up.

I understand if my use of AI is a dealbreaker for you. Just know it's the least interesting part of
what's here.

Figured I should share the end result of all the wasted water. ✌🏻

---

# Spread Joy!

I've spent a lot of time building slot.

If you enjoy using it and feel inclined to pay it forward, go do something nice for someone! ❤️
