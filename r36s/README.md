# slot su R36S (ArkOS)

Port di [slot](https://github.com/BrandonKowalski/slot) sull'R36S. Gira sopra
ArkOS, che resta installato: kernel, driver e pannello restano i suoi, e slot
prende solo il posto di EmulationStation.

## Cosa cambia rispetto a slot per Anbernic

- **Schermo**: backend KMS/GBM (`crates/slot-gfx/src/kms.rs`). Il vecchio
  backend fbdev resta come ripiego (`SLOT_GFX=fbdev|kms` per forzarne uno).
- **Tasti**: mappa per il joypad "GO-Super Gamepad" (`crates/slot/src/input/evdev.rs`).
  FN = Menu, SELECT/START come scritti, croce e levetta sinistra muovono.
- **Avvio**: `card/System/r36s-launch.sh` e gli script in `card/tools/`.
- **Core GBA**: gpSP di default (`SLOT_GBA_CORE=gpsp` nel launcher), perché
  mGBA non tiene i 60 fps sull'RK3326. Il selettore del core di slot vale
  ancora per il singolo gioco.
- **Schermo 640×480**: il 3× del GBA non ci sta. Menu → Screen → GBA → Scale:
  *Integer* è il 2× nitido (480×320), *Fit* riempie la larghezza a 2,67× e
  va usato con lo shader *Shimmerless*, che tiene i pixel regolari.

## Build

```bash
r36s/build.sh          # crea dist-r36s/ (serve Docker o Colima)
r36s/build.sh test     # test della workspace in Linux
```

## Preparare la scheda

Con la SD inserita nel Mac:

```bash
python3 r36s/prepare_sd.py
```

Copia `slot/` e gli script in EASYROMS, estrae le ROM GB/GBC/GBA di ArkOS in
`slot/Games`, porta i salvataggi `.srm` in `slot/Saves` come `.sav` e crea le
etichette delle cartucce. Per ogni gioco usa la prima che trova:

1. la label vera della cartuccia dall'archivio di slot (art.slot-cfw.fyi, lo
   stesso di Cart Studio), per checksum della ROM o per nome per le ROM hack;
2. il logo del gioco, da quell'archivio o da quello scaricato da ArkOS, sopra
   la box art sfumata;
3. la box art, da ArkOS o da thumbnails.libretro.com.

Non cancella niente e si può rilanciare dopo aver aggiunto giochi: crea solo le
label che mancano, `--relabel` le rifà tutte. Se le cartelle di ArkOS non sono
più sulla scheda, `--arkos` indica dove sono (per esempio un backup). Per un
gioco senza niente basta mettere un `.png` con lo stesso nome in
`slot/Labels/GBA`.

Con `--clean`, dopo l'import, cancella tutto quello che slot non usa: le
cartelle delle altre console, le copie ArkOS di gb/gbc/gba, BIOS, musiche,
video e i tool di ArkOS. Restano `slot/`, i tool di Slot e i temi di
EmulationStation, che serve ancora come ripiego. Si ferma se qualche gioco
non è stato portato in `slot/Games`. Fai prima un backup della scheda:

```bash
diskutil list external   # trova la SD, per esempio disk4
sudo dd if=/dev/rdisk4 bs=4m status=progress | zstd -T0 -3 -o r36s-sd.img.zst
```

## Sulla console

ArkOS → Options → **Tools**:

| Voce | Cosa fa |
|---|---|
| **Slot** | Prova slot. Spegni/Riavvia dentro slot ti riportano ad ArkOS |
| **Slot - Installa** | La console si avvia direttamente in slot. Spegne anche Samba, SSH, Bluetooth, gli altri servizi di condivisione e gli aiutanti di ArkOS per EmulationStation (icona Wi-Fi, lettore video, sospensione automatica) |
| **Slot - Disinstalla** | Torna EmulationStation all'avvio e riaccende i servizi spenti |

Se slot si blocca o non parte, al riavvio torna da sola EmulationStation. Per
avviare ArkOS una volta senza disinstallare, crea dal PC un file vuoto
`EASYROMS/slot/ARKOS`.

Mentre slot gira, il mixer del chip audio (RK817) è portato sull'altoparlante
e al massimo, perché il volume lo regola slot; all'uscita tornano i livelli di
ArkOS. Valori prima e dopo sono in `slot-session.log`. Con una scheda audio USB-C collegata slot suona lì, e se la
stacchi torna all'altoparlante o alle cuffie senza riavviare il gioco.

Log e diagnostica sono in `EASYROMS/slot/Logs` (`slot.log`, `slot-session.log`,
`diag.txt`). `diag.txt` si rifà solo nelle prove da Tools. Per le tracce
dettagliate (ogni tasto, ritmo dell'audio, latenza) crea un file vuoto
`EASYROMS/slot/TRACE`: costano qualche frame, perché slot sincronizza la
scheda ogni secondo mentre il log cresce, quindi toglilo finita la prova.
