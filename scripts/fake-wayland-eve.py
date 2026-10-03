#!/usr/bin/env python3
"""Harmless GTK client; only use on the disposable GNOME test session."""
import ctypes
import sys
import gi
gi.require_version('Gtk', '4.0')
from gi.repository import Gtk
if '--not-eve' not in sys.argv:
    ctypes.CDLL(None).prctl(15, b'exefile.exe', 0, 0, 0)
app = Gtk.Application(application_id='local.nicotine.TestEve' + sys.argv[1])
def activate(app):
    window = Gtk.ApplicationWindow(application=app, title='EVE - Test ' + sys.argv[1])
    window.set_default_size(900, 600)
    window.set_child(Gtk.Label(label='NICOTINE TEST CLIENT ' + sys.argv[1]))
    window.present()
app.connect('activate', activate)
app.run([])
