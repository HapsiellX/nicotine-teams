#!/usr/bin/env bash
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
binary_path=${1:-"$repo_dir/target/release/Nicotine"}
test -x "$binary_path" || { echo 'Bitte zuerst cargo build --release --bin Nicotine ausführen.' >&2; exit 1; }
install_dir="$HOME/.local/opt/nicotine-teams"
bin_dir="$HOME/.local/bin"
apps_dir="$HOME/.local/share/applications"
mkdir -p "$install_dir" "$bin_dir" "$apps_dir"
stamp=$(date +%Y%m%d-%H%M%S)
for file in "$install_dir/Nicotine" "$bin_dir/nicotine-teams" "$apps_dir/nicotine-teams.desktop"; do
    if test -f "$file"; then cp -p -- "$file" "$file.$stamp.backup"; fi
done
# Rename after copying so an existing running binary keeps its old inode.
install -m 755 "$binary_path" "$install_dir/Nicotine.new"
mv -f -- "$install_dir/Nicotine.new" "$install_dir/Nicotine"
install -m 644 "$repo_dir/assets/icon.png" "$install_dir/icon.png"
cat > "$bin_dir/nicotine-teams" <<'WRAPPER'
#!/usr/bin/env bash
set -euo pipefail
export NICOTINE_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nicotine-teams"
export NICOTINE_RUNTIME_DIR="${XDG_RUNTIME_DIR:-$HOME/.cache}/nicotine-teams"
mkdir -p "$NICOTINE_RUNTIME_DIR"
chmod 700 "$NICOTINE_RUNTIME_DIR"
export NICOTINE_SOCKET_PATH="$NICOTINE_RUNTIME_DIR/daemon.sock"
# The settings panel uses CPU rendering; XRender handles the game previews.
export ICED_BACKEND="${ICED_BACKEND:-tiny-skia}"
exec "$HOME/.local/opt/nicotine-teams/Nicotine" "$@"
WRAPPER
chmod 755 "$bin_dir/nicotine-teams"
cat > "$apps_dir/nicotine-teams.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Nicotine Teams
Comment=EVE-Clients, Teamprofile und Live-Vorschauen
Exec="$bin_dir/nicotine-teams" start
Icon=$install_dir/icon.png
Terminal=false
Categories=Game;Utility;
StartupWMClass=nicotine-teams
DESKTOP
if command -v update-desktop-database >/dev/null; then
    update-desktop-database "$apps_dir"
fi
printf 'Installiert: %s\n' "$bin_dir/nicotine-teams"
printf 'Start: nicotine-teams start\n'
