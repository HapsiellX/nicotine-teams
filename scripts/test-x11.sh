#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
xvfb-run -a -s '-screen 0 1600x1000x24' env -u WAYLAND_DISPLAY     XDG_SESSION_TYPE=x11 XDG_CURRENT_DESKTOP=OPENBOX bash -c '
    openbox >"${TMPDIR:-/tmp}/nicotine-teams-test-openbox.log" 2>&1 &
    wm_pid=$!
    trap "kill $wm_pid 2>/dev/null || true" EXIT
    sleep 1
    cargo test --test fake_eve -- --ignored --test-threads=1         --skip per_character_hotkey_activates_target_window         --skip modifier_combo_hotkey_activates_target_window         --skip mouse_side_button_cycles_forward         --skip hotkey_rebind_via_hot_reload_takes_effect
'
