<div align="center">

<img src=".github/resources/banner.png" width="100%" alt="slot. A bespoke, Game Boy-centric frontend for the Anbernic RG SP.">

<a href="LICENSE"><img src="https://img.shields.io/github/license/BrandonKowalski/slot?style=for-the-badge&labelColor=242429&color=4d4d57" height="36" alt="License"></a>
<a href="https://github.com/BrandonKowalski/slot/stargazers"><img src="https://img.shields.io/github/stars/BrandonKowalski/slot?style=for-the-badge&labelColor=242429&color=4d4d57" height="36" alt="Stars"></a>
<a href="https://github.com/BrandonKowalski/slot/releases"><img src="https://img.shields.io/github/downloads/BrandonKowalski/slot/total?style=for-the-badge&labelColor=242429&color=4d4d57" height="36" alt="Downloads"></a>
<a href="https://slot-cfw.fyi"><img src="https://img.shields.io/badge/Guide-slot--cfw.fyi-4d4d57?style=for-the-badge&labelColor=242429" height="36" alt="Docs"></a>
<a href="https://discord.gg/NYRWsSuPUu"><img src="https://img.shields.io/badge/Discord-4d4d57?style=for-the-badge&labelColor=242429&logo=discord&logoColor=white" height="36" alt="Discord"></a>

</div>

---

# slot su R36S

> [!NOTE]
> Questo è un fork non ufficiale che porta slot sull'**R36S con ArkOS**. Il
> progetto originale, per l'Anbernic RG SP, è
> [BrandonKowalski/slot](https://github.com/BrandonKowalski/slot): i problemi
> di questo port vanno segnalati qui, non lì.

slot prende il posto di EmulationStation: la console si accende direttamente
sulla libreria di cartucce GB, GBC e GBA. ArkOS resta installato sotto (kernel,
driver, schermo) e torna con un tasto se serve.

Cosa cambia rispetto a slot per RG SP: schermo via KMS, mappa dei tasti
dell'R36S, gpSP come core GBA di default (mGBA non tiene i 60 fps sull'RK3326),
audio da altoparlante, jack e schede USB-C, label vere delle cartucce anche per
le ROM hack. I dettagli tecnici sono in [`r36s/README.md`](r36s/README.md).

## Cosa serve

- Un R36S con **ArkOS** che si avvia e i giochi GB/GBC/GBA nella partizione
  EASYROMS. Provato con ArkOS per R36S (Ubuntu 19.10, kernel 4.4.189), pannello 4.
- Un Mac o un PC Linux con `git`, Python 3 e **Docker** (sul Mac va bene anche
  [Colima](https://github.com/abiosoft/colima)): slot si compila per ARM dentro
  un container.
- Per le label delle cartucce: `pip3 install pillow`.

## Installazione

1. **Fai un backup della SD.** Sul Mac, con la scheda inserita:

   ```bash
   diskutil list external   # trova la SD, per esempio disk4
   sudo dd if=/dev/rdisk4 bs=4m status=progress | gzip > r36s-sd.img.gz
   ```

2. **Compila slot.** La prima volta scarica la toolchain e ci mette un po':

   ```bash
   git clone -b r36s https://github.com/Quis03/slot-r36s.git
   cd slot-r36s
   r36s/build.sh
   ```

3. **Prepara la SD.** Con la scheda nel computer:

   ```bash
   python3 r36s/prepare_sd.py                              # Mac
   python3 r36s/prepare_sd.py --card /media/$USER/EASYROMS # Linux
   ```

   Copia slot e i suoi script in EASYROMS, porta i giochi delle cartelle
   `gb`, `gbc` e `gba` di ArkOS in `slot/Games` (estraendo gli zip), i
   salvataggi `.srm` in `slot/Saves` e crea le label delle cartucce. Non
   cancella niente: ArkOS continua a funzionare come prima. Con `--clean`
   toglie invece tutto quello che slot non usa (altre console, BIOS, video);
   prima fai il backup.

4. **Provalo.** Espelli la SD, avvia la console e apri ArkOS → Options →
   Tools → **Slot**. In questa prova *Spegni* e *Riavvia* dentro slot ti
   riportano ad ArkOS.

5. **Installalo.** Se va tutto bene: Options → Tools → **Slot - Installa**. La
   console si riavvia direttamente in slot. Installa spegne anche i servizi di
   ArkOS che slot non usa (Samba, SSH, Bluetooth, i programmi di supporto di
   EmulationStation) e ricorda quali erano accesi.

## Uso

- **FN**: premuto brevemente apre il menu; dentro un gioco apre il menu di
  gioco, tenuto premuto espelle la cartuccia.
- **Power**: premuto brevemente mette in pausa e spegne lo schermo (lo
  riaccende la pressione successiva); tenuto premuto un secondo salva la
  partita e spegne la console.
- Sullo schermo 640×480 il 3× del GBA non ci sta: in Menu → Screen → GBA →
  Scale scegli *Fit* con lo shader *Shimmerless* per riempire lo schermo, o
  *Integer* per un 2× nitido.
- **Sfondi**: metti dei PNG (meglio 640×480) in `EASYROMS/slot/Wallpapers` e
  sceglili da menu → Wallpaper.
- **Nuovi giochi**: copiali, non zippati, in `EASYROMS/slot/Games/GBA`
  (o `GB`, `GBC`). Rilanciando `prepare_sd.py` crea le label che mancano.

## Tornare ad ArkOS

- **Una volta sola**: crea dal PC un file vuoto `EASYROMS/slot/ARKOS`. Al
  prossimo avvio parte EmulationStation; cancellalo per tornare a slot.
- **Del tutto**: con il file `ARKOS`, da ArkOS apri Options → Tools →
  **Slot - Disinstalla**. Rimette EmulationStation e riaccende i servizi spenti.
- Se slot si blocca o non parte, al riavvio torna da sola EmulationStation.

## Se qualcosa non va

I log sono in `EASYROMS/slot/Logs` (`slot.log`, `slot-session.log`,
`diag.txt`). Per tracce dettagliate crea un file vuoto `EASYROMS/slot/TRACE`,
riproduci il problema e allega i log a una issue; poi toglilo, perché rallenta
un po' il gioco.

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
