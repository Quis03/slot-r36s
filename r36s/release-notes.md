slot for the **R36S running ArkOS**, based on upstream [slot v1.12.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.12.0). This is an unofficial port; please report problems [here](https://github.com/Quis03/slot-r36s/issues), not upstream.

slot replaces EmulationStation: the console boots straight into a shelf of GB, GBC and GBA carts. ArkOS stays installed underneath and is one step away.

## What's in this port

- KMS display output
- The R36S button map
- gpSP as the default GBA core (mGBA cannot hold 60 fps on the RK3326)
- Audio through the speaker, the headphone jack and USB-C sound cards
- Real cart labels, ROM hacks included

## Install

The full guide is in the [README](https://github.com/Quis03/slot-r36s#install).

1. Back up the SD card.
2. Download `slot-r36s-v1.12.0-r36s.1.zip` below and unzip it.
3. With the card in your computer, run `python3 prepare_sd.py` from the unzipped folder (add `--card <path to EASYROMS>` on Linux or Windows). It copies `slot/` and `tools/` to EASYROMS, imports your ArkOS gb/gbc/gba games and saves, and makes cart labels (`pip3 install pillow` for labels). It deletes nothing unless you pass `--clean`. Without Python, copy `slot/` and `tools/` to the root of EASYROMS and put your ROMs in `slot/Games/GBA`, `GB` or `GBC`.
4. Try it: ArkOS → Options → Tools → **Slot**. Power off and Restart inside slot bring you back to ArkOS.
5. Install it: Options → Tools → **Slot - Install**. The console then boots straight into slot.

## Tested on

- ArkOS for the R36S (Ubuntu 19.10, kernel 4.4.189), panel 4
- `prepare_sd.py` has not been tested on Windows yet.

## Contents

- `slot/`: the slot card tree for EASYROMS
- `tools/`: the Slot, Slot - Install and Slot - Uninstall scripts for ArkOS → Options → Tools
- `prepare_sd.py`: prepares the card, using `slot/` and `tools/` next to it
- `LICENSE`: GPL-3.0
