# Nicotine Teams: Arbeits- und Prüfliste

Stand: 3. Oktober 2026. Ein Häkchen bezeichnet die angegebene geprüfte Teilfunktion, nicht vollständige ISBoxer-Parität.

## Erster Ausbaustand

- [x] Deutsche Einstellungsoberfläche mit Teamprofilen.
- [x] Team, Reihenfolge, Hotkeys und Vorschau-Einstellungen als Schnappschuss speichern.
- [x] Profile im laufenden Daemon wechseln; direkt folgende CLI-Wechsel nutzen die neue Reihenfolge.
- [x] Leere aktive Teams wechseln zu keinem Client.
- [x] Offline-CLI sowie JSON-Import/-Export mit Schema- und Konfliktprüfung.
- [x] CLI-Profilwechsel in der geöffneten Oberfläche übernehmen.
- [x] Vorschau-FPS global begrenzen und je Profil speichern.
- [x] Bestehende Konfigurationen ohne Profilfelder weiterhin lesen.
- [x] Einstellungen und Vorschaupositionen in isolierten Verzeichnissen atomar schreiben.
- [x] Beenden auf die eigene Instanz begrenzen.
- [x] GUI: Profile per Klick anlegen, extern wechseln und per Klick exportieren.
- [x] Gespeicherte Vorschaupositionen und Größen zur Laufzeit wiederherstellen (XComposite-Test).
- [x] GNOME-50-Anbindung für native Wayland- und XWayland-Clients in eigener GNOME-Testsession prüfen.
- [x] Native Vorschauen, Profilreihenfolge, Layoutwechsel, Minimize/Restore und Heartbeat-Abschaltung prüfen.
- [ ] Echtes Mehrclient-Spiel prüfen: neue Shell-Erweiterung benötigt zuerst eine reguläre Neuanmeldung.
- [x] Lokale Installation und Start der Einstellungsoberfläche unter GNOME Wayland prüfen.
- [x] Eigene GNOME-Erweiterung installieren und mit gesicherter Aktivierungsliste für die nächste Anmeldung vorbereiten.

## Weiterer Ausbau

| Funktion | Status | Nächster überprüfbarer Schritt |
|---|---|---|
| Komfortable Teamprofile | Grundfunktionen geprüft | Umbenennen, Profil-Duplikate und Profile für verschiedene Tätigkeiten |
| Layout-Editor für mehrere Monitore | Offen | Bildschirmgeometrie erfassen, Plätze bearbeiten, Vorschau der Anordnung |
| Haupt-/Nebenfenster-Tausch | Offen | GNOME-Shell-Integration für ausschließlich freigegebene EVE-Fenster |
| Gespeicherte Bildausschnitte | Offen | Crop-Editor mit passender XRender-Transformation |
| Vorschau-FPS pro Client | Teilweise | Globale Begrenzung vorhanden; eigene Werte für einzelne Clients ergänzen |
| Verbindungsstatus und Wiedererkennung | Teilweise | Vorhandene Erkennung erhalten; Wiederanmeldung desselben Charakters und explizite Statusanzeige testen |
| EVE-Tools-Anbindung | Offen | Authentifizierten Lesezugriff und veraltete Daten sauber anzeigen |

## Abnahmegrenzen

Die X11-Tests und die native GNOME-50-Prüfung verwenden künstliche Clients in separaten Desktops. Die echte EVE-Mehrclient-Abnahme bleibt offen. Im laufenden Desktop ist EVE mit `PROTON_ENABLE_WAYLAND=1` erkannt worden; die neue Shell-Erweiterung kann GNOME 50 erst nach der nächsten Anmeldung laden. EVE wurde nicht neu gestartet. Eine gestartete Oberfläche allein bestätigt keinen vollständigen Mehrclient-Ablauf.

Native GNOME-Vorschauen übernehmen gespeicherte Positionen; direktes Ziehen und eine FPS-Begrenzung der Clutter-Klone fehlen noch.

EVE-Spielaktionen bleiben individuelle manuelle Eingaben. Die erste Erweiterung verwaltet Fenster, Vorschauen und Profile.
