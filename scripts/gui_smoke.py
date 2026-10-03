#!/usr/bin/env python3
"""Run only on a disposable Xvfb display; never inject into the user's desktop."""
import json
import os
import pathlib
import subprocess
import tempfile
import time
from PIL import ImageGrab
from Xlib import display, X, XK
from Xlib.ext import xtest
import tomllib

repo = pathlib.Path(__file__).resolve().parents[1]
out = pathlib.Path(os.environ['NICOTINE_QA_DIR'])
out.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='nicotine-gui-qa-') as directory:
    env = dict(os.environ, NICOTINE_CONFIG_DIR=directory, ICED_BACKEND='tiny-skia')
    binary = repo / 'target/debug/Nicotine'
    subprocess.run([str(binary), 'profile', 'list'], env=env, check=True, capture_output=True)
    logfile = open(out / 'gui.log', 'w')
    process = subprocess.Popen([str(binary), 'panel'], env=env, stdout=logfile, stderr=logfile)
    try:
        connection = display.Display()
        root = connection.screen().root
        atom = connection.intern_atom('_NET_CLIENT_LIST')
        deadline = time.monotonic() + 15
        window = None
        while time.monotonic() < deadline and process.poll() is None:
            clients = root.get_full_property(atom, 0)
            for wid in clients.value if clients else []:
                candidate = connection.create_resource_object('window', int(wid))
                if candidate.get_wm_name() == 'Nicotine Teams':
                    window = candidate
                    break
            if window:
                break
            time.sleep(0.1)
        assert window, 'Native config panel did not open'
        time.sleep(1)
        geometry = window.get_geometry()
        location = root.translate_coords(window, 0, 0)
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).save(out / 'profiles.png')
        def click(x, y):
            xtest.fake_input(connection, X.MotionNotify, x=location.x+x, y=location.y+y)
            xtest.fake_input(connection, X.ButtonPress, 1)
            xtest.fake_input(connection, X.ButtonRelease, 1)
            connection.sync()
            time.sleep(0.15)

        def type_text(value):
            shift = connection.keysym_to_keycode(XK.string_to_keysym('Shift_L'))
            for character in value:
                symbol = ord(character)
                code = connection.keysym_to_keycode(symbol)
                assert code, f'No keycode for {character!r}'
                mapping = connection.get_keyboard_mapping(code, 1)[0]
                use_shift = len(mapping) > 1 and mapping[1] == symbol
                if use_shift:
                    xtest.fake_input(connection, X.KeyPress, shift)
                xtest.fake_input(connection, X.KeyPress, code)
                xtest.fake_input(connection, X.KeyRelease, code)
                if use_shift:
                    xtest.fake_input(connection, X.KeyRelease, shift)
            connection.sync()
            time.sleep(0.15)

        def enter():
            code = connection.keysym_to_keycode(XK.string_to_keysym('Return'))
            xtest.fake_input(connection, X.KeyPress, code)
            xtest.fake_input(connection, X.KeyRelease, code)
            connection.sync()
            time.sleep(0.4)

        def config():
            with open(pathlib.Path(directory) / 'config.toml', 'rb') as file:
                return tomllib.load(file)

        click(260, 307)
        type_text('Mining')
        enter()
        assert config()['team']['active'] == 'Mining', 'GUI profile creation failed'
        click(260, 307)
        type_text('Missionen')
        enter()
        assert config()['team']['active'] == 'Missionen'
        subprocess.run([str(binary), 'profile', 'apply', 'Mining'], env=env, check=True)
        time.sleep(0.8)
        # GUI must adopt external changes before exporting its selected snapshot.
        click(290, 432)
        export = pathlib.Path(directory) / 'profil.json'
        type_text(str(export))
        click(215, 472)
        time.sleep(0.4)
        assert export.exists(), 'GUI export failed'
        assert json.loads(export.read_text())['name'] == 'Mining', 'GUI did not adopt CLI profile switch'
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).save(out / 'profiles-saved.png')
        click(70, 135)
        time.sleep(0.3)
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).save(out / 'display.png')
        click(70, 174)
        time.sleep(0.3)
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).save(out / 'characters.png')
        click(70, 210)
        time.sleep(0.3)
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).save(out / 'hotkeys.png')
        print(json.dumps({'verified': ['GUI create', 'CLI to GUI live switch', 'GUI export'], 'screenshots': str(out)}))

    finally:
        process.terminate()
        process.wait(timeout=5)
        logfile.close()
