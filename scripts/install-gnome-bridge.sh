#!/usr/bin/env bash
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
uuid=nicotine-teams@hapsiellx
source_dir="$repo_dir/gnome-extension/$uuid"
dest_dir="${XDG_DATA_HOME:-$HOME/.local/share}/gnome-shell/extensions/$uuid"
if test -d "$dest_dir"; then cp -a -- "$dest_dir" "$dest_dir.$(date +%Y%m%d-%H%M%S).backup"; fi
mkdir -p -- "$dest_dir"
install -m 644 "$source_dir/metadata.json" "$source_dir/extension.js" "$source_dir/model.js" "$dest_dir/"
if test "${1:-}" = --enable; then
    python3 - <<'PY'
import datetime
import json
import os
import pathlib
from gi.repository import Gio
settings = Gio.Settings.new('org.gnome.shell')
enabled = list(settings.get_strv('enabled-extensions'))
uuid = 'nicotine-teams@hapsiellx'
if uuid not in enabled:
    folder = pathlib.Path(os.environ.get('XDG_CONFIG_HOME', str(pathlib.Path.home()/'.config'))) / 'nicotine-teams'
    folder.mkdir(parents=True, exist_ok=True)
    backup = folder / ('gnome-enabled-extensions-' + datetime.datetime.now().strftime('%Y%m%d-%H%M%S') + '.json.backup')
    descriptor = os.open(backup, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, 'w') as file:
        json.dump(enabled, file)
    if not settings.set_strv('enabled-extensions', enabled + [uuid]):
        raise RuntimeError('GNOME could not enable the extension')
    Gio.Settings.sync()
print('GNOME-Anbindung für die nächste Anmeldung aktiviert.')
PY
fi
printf 'GNOME-Erweiterung installiert: %s\n' "$uuid"
printf 'Aktivieren: gnome-extensions enable %s\n' "$uuid"
printf 'Falls GNOME sie noch nicht kennt: bei nächster Anmeldung aktivieren.\n'
