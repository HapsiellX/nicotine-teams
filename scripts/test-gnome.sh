#!/usr/bin/env bash
# The real desktop, D-Bus and GNOME configuration are never used by this test.
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
if test "${NICOTINE_ISOLATED_GNOME:-}" != 1; then
    qa_dir=$(mktemp -d "${TMPDIR:-/tmp}/nicotine-gnome-qa-XXXXXXXX")
    export NICOTINE_GNOME_QA_DIR="$qa_dir" NICOTINE_ISOLATED_GNOME=1
    printf 'GNOME test artifacts: %s\n' "$qa_dir"
    exec timeout 45 dbus-run-session -- bash "$0"
fi
qa_dir=$NICOTINE_GNOME_QA_DIR
export XDG_CONFIG_HOME="$qa_dir/config" XDG_DATA_HOME="$qa_dir/data" XDG_RUNTIME_DIR="$qa_dir/runtime"
mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME/gnome-shell/extensions" "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
cp -a "$repo_dir/gnome-extension/nicotine-teams@hapsiellx" "$XDG_DATA_HOME/gnome-shell/extensions/"
export GSETTINGS_BACKEND=keyfile LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe GIO_USE_VFS=local
export XDG_SESSION_TYPE=wayland XDG_CURRENT_DESKTOP=GNOME GDK_BACKEND=wayland
unset DISPLAY WAYLAND_DISPLAY XDG_SESSION_ID
export NICOTINE_CONFIG_DIR="$qa_dir/nicotine" NICOTINE_RUNTIME_DIR="$qa_dir/nicotine-runtime"
gsettings set org.gnome.shell enabled-extensions "['nicotine-teams@hapsiellx']"
gnome-shell --headless --wayland --virtual-monitor=1600x1000 --wayland-display=nicotine-test > "$qa_dir/shell.log" 2>&1 &
shell_pid=$!
trap 'kill "${client1:-}" "${client2:-}" "${browser:-}" "$shell_pid" 2>/dev/null || true' EXIT
for i in $(seq 1 30); do
    if gdbus call --session --dest org.gnome.Shell --object-path /org/gnome/Shell/Extensions/NicotineTeams --method org.gnome.Shell.Extensions.NicotineTeams.Version > "$qa_dir/version.log" 2>/dev/null; then break; fi
    sleep 0.5
done
grep -q 'uint32 1' "$qa_dir/version.log"
export WAYLAND_DISPLAY=nicotine-test
sleep 1
python3 "$repo_dir/scripts/fake-wayland-eve.py" A > "$qa_dir/client-A.log" 2>&1 & client1=$!
python3 "$repo_dir/scripts/fake-wayland-eve.py" B > "$qa_dir/client-B.log" 2>&1 & client2=$!
python3 "$repo_dir/scripts/fake-wayland-eve.py" Browser --not-eve > "$qa_dir/browser.log" 2>&1 & browser=$!
python3 "$repo_dir/scripts/gnome_bridge_smoke.py" > "$qa_dir/results.log" 2>&1
cat "$qa_dir/results.log"
