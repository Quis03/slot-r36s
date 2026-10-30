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
etichette delle cartucce dalle box art. Le prende da quelle già scaricate da
ArkOS oppure da thumbnails.libretro.com. Non cancella niente e si può rilanciare
dopo aver aggiunto giochi. Per una ROM hack senza box art basta mettere un
`.png` con lo stesso nome in `slot/Labels/GBA`.

## Sulla console

ArkOS → Options → **Tools**:

| Voce | Cosa fa |
|---|---|
| **Slot** | Prova slot. Spegni/Riavvia dentro slot ti riportano ad ArkOS |
| **Slot - Installa** | La console si avvia direttamente in slot |
| **Slot - Disinstalla** | Torna EmulationStation all'avvio |

Se slot si blocca o non parte, al riavvio torna da sola EmulationStation. Per
avviare ArkOS una volta senza disinstallare, crea dal PC un file vuoto
`EASYROMS/slot/ARKOS`.

Log e diagnostica sono in `EASYROMS/slot/Logs` (`slot.log`, `diag.txt`).
