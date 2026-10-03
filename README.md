# Nicotine Teams für Linux

Ein eigener Fork von [Nicotine](https://github.com/isomerc/nicotine), basierend auf Upstream `ebd8328`. Die MIT-Lizenz und die ursprünglichen Urheberhinweise bleiben erhalten.

Der erste Ausbau ergänzt den vorhandenen EVE-Clientmanager um eine deutsche Oberfläche, gespeicherte Teamprofile und Vorschau-Layouts. Ein Profil enthält Charakterreihenfolge, Hotkeys, Vorschaugröße, Deckkraft, FPS und Fensterpositionen. Profile lassen sich während des Betriebs laden sowie als JSON importieren und exportieren.

## Benutzen

```bash
cargo build --release --bin Nicotine
bash scripts/install-user.sh
nicotine-teams start
```

Das manuell startbare GitHub-Workflow **Build Linux package** erzeugt ein Linux-Archiv. Nach dem Entpacken: `bash scripts/install-user.sh "$PWD/Nicotine"`.

Der Starter erscheint auch im Anwendungsmenü unter **Nicotine Teams**. Rust wird zum Bauen benötigt. GNOME 50 nutzt für native Proton-Wayland-Clients die mitgelieferte Shell-Erweiterung; der XWayland-Fallback benötigt kein `wmctrl`.

```bash
bash scripts/install-gnome-bridge.sh --enable
```

`--enable` fügt ausschließlich diese Erweiterung zur Aktivierungsliste hinzu und sichert den vorherigen Stand; es benötigt python3-gi. Eine neue Erweiterung wird erst nach einer regulären Ab- und Anmeldung geladen. Danach Nicotine Teams neu starten. Das Installationsskript startet GNOME und EVE nicht neu. Eine bereits aktive Erweiterung erscheint in der Oberfläche als „GNOME-Anbindung aktiv“.

1. Unter **Charaktere** die angemeldeten Clients hinzufügen und die Reihenfolge einstellen.
2. Unter **Anzeige** Vorschauen einstellen und an ihre gewünschten Positionen ziehen.
3. Unter **Teamprofile** einen Namen eingeben und **Anlegen** wählen.
4. Ein anderes Profil auswählen und **Laden** drücken.
5. Nach Änderungen **Profil aktualisieren** drücken, um den gespeicherten Schnappschuss zu ersetzen.

**Anlegen** speichert den aktuellen Zustand. **Laden** ersetzt die aktuellen Team- und Vorschau-Einstellungen durch den gespeicherten Zustand. Ein leeres aktives Profil wählt keine Clients aus. Das Löschen eines Profils entfernt seinen Schnappschuss, keine EVE-Daten.

X11-Vorschauen laufen standardmäßig mit 15 FPS; unter Anzeige sind 5–60 FPS einstellbar. Native GNOME-Vorschauen sind Clutter-Klone und werden vom Compositor gezeichnet; diese FPS-Begrenzung gilt dort nicht. Globale Geräte-Hotkeys und Mauswechsel sind zunächst ausgeschaltet. Ohne Gerätefreigabe können Desktop-Tastenkürzel die Befehle `nicotine-teams forward`, `backward` oder `1`, `2` usw. aufrufen. Die Installation ändert keine Geräteberechtigungen.

## Kommandozeile

```bash
nicotine-teams profile list
nicotine-teams profile save Mining
nicotine-teams profile update Mining
nicotine-teams profile apply Mining
nicotine-teams profile export Mining /absoluter/pfad/mining.json
nicotine-teams profile import /absoluter/pfad/mining.json
nicotine-teams profile delete Mining
nicotine-teams list
nicotine-teams panel  # nur die Einstellungen öffnen
nicotine-teams stop
```

Profilbefehle funktionieren ohne geöffneten Client oder Display. Import und Export überschreiben vorhandene Profile bzw. Dateien nicht. Namen, Maße, FPS, doppelte Charaktere und kollidierende Hotkeys werden geprüft. Ein unbekanntes oder fehlerhaftes Profil ersetzt den aktuellen Zustand nicht.

## Speicherung und Grenzen

Der installierte Starter verwendet `~/.config/nicotine-teams/` und ein eigenes Laufzeitverzeichnis. Vorhandene Upstream-Einstellungen unter `~/.config/nicotine/` bleiben getrennt. Das native Programm berücksichtigt `NICOTINE_CONFIG_DIR`, `NICOTINE_RUNTIME_DIR` und `NICOTINE_SOCKET_PATH` für Tests oder eigene Starter.

Exportdateien enthalten Charakternamen und Tastenkürzel. Sie enthalten keine EVE-Zugangsdaten, lokalen Eingabegerätepfade oder den Zustand der gesamten Profilverwaltung. Der Fork sendet keine Upstream-Telemetrie und fragt nicht automatisch nach Upstream-Updates. Profilwechsel schreiben Konfiguration und Layout; fehlgeschlagene Konfigurationsspeicherung versucht das vorherige Layout wiederherzustellen.

Die GNOME-Erweiterung erkennt native Wayland- und XWayland-Clients anhand ihres Fenstertitels und des Prozessnamens `exefile.exe`. Sie erlaubt ausschließlich diese EVE-Fenster auszuwählen, zu minimieren und wiederherzustellen. Vorschauen und kompakte Listen unterstützen Profilreihenfolge, Größe, Deckkraft und gespeicherte Positionen. Vorschauen verschwinden automatisch, wenn Nicotine länger als 2,5 Sekunden keine Aktualisierung sendet.

Native GNOME-Vorschauen lassen sich in diesem Ausbaustand noch nicht direkt ziehen; sie übernehmen die Positionen aus dem gespeicherten Layout. Der Layout-Editor, Bildausschnitte und der Tausch der **EVE-Spielfenster** in einen Hauptbereich folgen. Windows-Kompatibilität der Fork-Erweiterungen ist noch nicht abgenommen.

## Prüfen

```bash
cargo test --locked
cargo fmt --check
cargo clippy --all-targets -- -D warnings
bash scripts/test-x11.sh
node scripts/test-gnome-model.mjs
bash scripts/test-gnome.sh  # benötigt GNOME 50, python3-gi und GTK 4
```

Die X11-Prüfung startet einen separaten Xvfb/Openbox-Desktop und benutzt künstliche EVE-Fenster. Tests, die systemweite Eingaben über `/dev/uinput` erzeugen, werden dabei ausdrücklich ausgelassen. Die GNOME-Prüfung verwendet einen eigenen D-Bus, eigene Einstellungen, einen headless Compositor und künstliche native GTK-Clients. Sie prüft Erkennung, Fokus, Minimize/Restore, Vorschauen, Layoutwechsel, leere Teams und Aufräumen ohne Heartbeat. Die echte Mehrclient-Abnahme im Spiel bleibt ein separater praktischer Schritt.

Für die GUI-Prüfung gibt es `scripts/gui_smoke.py` (Pillow und python-xlib); sie darf ausschließlich auf einem separaten Xvfb-Display laufen. Die Prüfung erzeugt Profile über die Oberfläche, übernimmt einen CLI-Profilwechsel und exportiert den ausgewählten Schnappschuss.

Die verbleibenden Ausbauschritte und ihr Prüfstatus stehen in [ROADMAP.md](ROADMAP.md).

## Zurückbauen

Die GNOME-Anbindung lässt sich mit `gnome-extensions disable nicotine-teams@hapsiellx` abschalten. `nicotine-teams stop` beendet den eigenen Daemon. Vorhandene Starter und Binärdateien werden bei einer erneuten Installation mit Zeitstempel gesichert; die Erweiterungsinstallation sichert ihren bisherigen Ordner. Zum Entfernen nur die separat installierten Dateien unter `~/.local/opt/nicotine-teams`, `~/.local/bin/nicotine-teams`, `~/.local/share/applications/nicotine-teams.desktop` und `~/.local/share/gnome-shell/extensions/nicotine-teams@hapsiellx` entfernen. Profil-Daten liegen unabhängig unter `~/.config/nicotine-teams`.

GNOME-Grundlagen: [Extension API](https://gjs.guide/extensions/topics/extension.html), [Meta.Window](https://gnome.pages.gitlab.gnome.org/mutter/meta/class.Window.html).
