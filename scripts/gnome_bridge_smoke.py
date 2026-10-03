#!/usr/bin/env python3
"""Only run inside scripts/test-gnome.sh's private D-Bus/display session."""
import json
import os
import time
import subprocess
import pathlib
import gi
from gi.repository import Gio, GLib
assert os.environ.get('NICOTINE_ISOLATED_GNOME') == '1'
connection = Gio.bus_get_sync(Gio.BusType.SESSION, None)
path = '/org/gnome/Shell/Extensions/NicotineTeams'
interface = 'org.gnome.Shell.Extensions.NicotineTeams'
def call(method, value=None):
    return connection.call_sync('org.gnome.Shell', path, interface, method, value, None,
                                Gio.DBusCallFlags.NONE, 2000, None).unpack()[0]
def windows():
    return json.loads(call('List'))
def wait_for(predicate, message):
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(0.1)
    raise AssertionError(message)
wait_for(lambda: len(windows()) == 2, 'Native client detection failed')
clients = windows()
assert {w['title'] for w in clients} == {'EVE - Test A', 'EVE - Test B'}
assert all(w['native'] for w in clients), 'Test clients are not native Wayland'
a, b = sorted(clients, key=lambda w: w['title'])
assert call('Activate', GLib.Variant('(u)', [a['id']]))
wait_for(lambda: call('Active') == a['id'], 'Activation did not focus A')
assert call('Activate', GLib.Variant('(u)', [b['id']]))
wait_for(lambda: call('Active') == b['id'], 'Activation did not focus B')
assert call('Minimize', GLib.Variant('(u)', [a['id']]))
wait_for(lambda: next(w for w in windows() if w['id'] == a['id'])['minimized'], 'Minimize failed')
assert call('Restore', GLib.Variant('(u)', [a['id']]))
wait_for(lambda: not next(w for w in windows() if w['id'] == a['id'])['minimized'], 'Restore failed')
assert not call('Activate', GLib.Variant('(u)', [4294967295])), 'Non-EVE target accepted'
options = dict(show=True, hide_active=False, characters=['Test B', 'Test A'], width=320, height=180,
               opacity=90, revision=1, positions={'Test A': [50,80], 'Test B': [410,80]}, mode='Previews')
assert call('Previews', GLib.Variant('(s)', [json.dumps(options)]))
status = json.loads(call('Status'))
assert status['previews'] == 2 and status['error'] is None, status
layout = {card['id']: card for card in status['layout']}
assert (layout[a['id']]['x'], layout[a['id']]['y'], layout[a['id']]['width']) == (50,80,320)
options['revision'] = 2
options['width'] = 500
options['positions']['Test A'] = [600,120]
call('Previews', GLib.Variant('(s)', [json.dumps(options)]))
layout = {card['id']: card for card in json.loads(call('Status'))['layout']}
assert (layout[a['id']]['x'], layout[a['id']]['y'], layout[a['id']]['width']) == (600,120,500)
options['hide_active'] = True
call('Previews', GLib.Variant('(s)', [json.dumps(options)]))
assert json.loads(call('Status'))['previews'] == 1
options['hide_active'] = False
options['characters'] = []
assert call('Previews', GLib.Variant('(s)', [json.dumps(options)]))
assert json.loads(call('Status'))['previews'] == 0
options['characters'] = ['Test A']
options['mode'] = 'List'
assert call('Previews', GLib.Variant('(s)', [json.dumps(options)]))
assert json.loads(call('Status'))['previews'] == 1
options['width'] = 0
try:
    call('Previews', GLib.Variant('(s)', [json.dumps(options)]))
except GLib.Error:
    pass
else:
    raise AssertionError('Invalid dimensions accepted')
wait_for(lambda: json.loads(call('Status'))['previews'] == 0, 'Orphaned previews after missing heartbeat')
repo = pathlib.Path(__file__).resolve().parents[1]
result = subprocess.run([str(repo/'target/debug/Nicotine'), 'list'], capture_output=True, text=True, check=True)
assert 'Using GNOME Shell bridge' in result.stdout and 'Test A' in result.stdout and 'Test B' in result.stdout
print(json.dumps({'verified': ['native discovery', 'process filtering', 'focus A/B', 'minimize/restore',
    'non-EVE rejection', 'compositor preview creation', 'empty team', 'list mode', 'invalid size',
    'heartbeat cleanup', 'Rust backend detection', 'layout reload', 'hide active']}))
