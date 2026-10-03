import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Clutter from 'gi://Clutter';
import St from 'gi://St';
import Meta from 'gi://Meta';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {parsePreviewOptions, teamWindows} from './model.js';

const PATH = '/org/gnome/Shell/Extensions/NicotineTeams';
const XML = `<node><interface name="org.gnome.Shell.Extensions.NicotineTeams">
<method name="Version"><arg type="u" direction="out"/></method>
<method name="List"><arg type="s" direction="out"/></method>
<method name="Active"><arg type="u" direction="out"/></method>
<method name="Activate"><arg type="u" direction="in"/><arg type="b" direction="out"/></method>
<method name="Minimize"><arg type="u" direction="in"/><arg type="b" direction="out"/></method>
<method name="Restore"><arg type="u" direction="in"/><arg type="b" direction="out"/></method>
<method name="Previews"><arg type="s" direction="in"/><arg type="b" direction="out"/></method>
<method name="Status"><arg type="s" direction="out"/></method>
</interface></node>`;

export default class NicotineTeamsBridge extends Extension {
    enable() {
        this._cards = new Map();
        this._options = null;
        this._heartbeat = 0;
        this._signature = '';
        this._error = null;
        this._dbus = Gio.DBusExportedObject.wrapJSObject(XML, this);
        this._dbus.export(Gio.DBus.session, PATH);
        this._timer = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
            try {
                if (GLib.get_monotonic_time() - this._heartbeat > 2500000)
                    this._clear();
                else
                    this._render();
            } catch (error) {
                this._error = String(error);
                console.error(`Nicotine Teams preview: ${error}`);
                this._clear();
            }
            return GLib.SOURCE_CONTINUE;
        });
    }

    disable() {
        if (this._timer) {
            GLib.Source.remove(this._timer);
            this._timer = null;
        }
        this._clear();
        this._dbus?.unexport();
        this._dbus = null;
        this._options = null;
    }

    _windows() {
        return global.get_window_actors().map(actor => actor.meta_window).filter(window => {
            const title = window.get_title() ?? '';
            if (!title.startsWith('EVE - ') || window.get_window_type() !== Meta.WindowType.NORMAL)
                return false;
            // Only read the short executable name: no launch arguments, tokens or memory access.
            try {
                const [, bytes] = GLib.file_get_contents(`/proc/${window.get_pid()}/comm`);
                return new TextDecoder().decode(bytes).trim() === 'exefile.exe';
            } catch (_) {
                return false;
            }
        });
    }

    _find(id) { return this._windows().find(w => w.get_stable_sequence() === id); }
    Version() { return 1; }
    List() {
        return JSON.stringify(this._windows().map(window => ({
            id: window.get_stable_sequence(),
            title: window.get_title(),
            pid: window.get_pid(),
            native: window.get_client_type() === Meta.WindowClientType.WAYLAND,
            minimized: window.minimized,
        })));
    }
    Active() {
        const window = this._windows().find(w => w.has_focus());
        return window ? window.get_stable_sequence() : 0;
    }
    Activate(id) {
        const window = this._find(id);
        if (!window)
            return false;
        window.unminimize();
        Main.activateWindow(window, global.get_current_time());
        return true;
    }
    Minimize(id) {
        const window = this._find(id);
        if (!window || !window.can_minimize())
            return false;
        window.minimize();
        return true;
    }
    Restore(id) {
        const window = this._find(id);
        if (!window)
            return false;
        window.unminimize();
        return true;
    }
    Previews(json) {
        this._options = parsePreviewOptions(json);
        this._heartbeat = GLib.get_monotonic_time();
        this._render();
        return true;
    }
    Status() {
        return JSON.stringify({clients: this._windows().length, previews: this._cards.size,
            error: this._error, renderer: 'Clutter compositor clone',
            layout: [...this._cards.entries()].map(([id, card]) => ({id,
                x: card.x, y: card.y, width: card.width, height: card.height}))});
    }
    _clear() {
        if (!this._cards)
            return;
        for (const card of this._cards.values()) {
            Main.layoutManager.removeChrome(card);
            card.destroy();
        }
        this._cards.clear();
        this._signature = '';
    }
    _render() {
        if (!this._options?.show || Main.sessionMode.isLocked || Main.sessionMode.isGreeter) {
            this._clear();
            return;
        }
        const windows = this._windows();
        const records = windows.map(w => ({id: w.get_stable_sequence(), title: w.get_title(),
            width: w.get_compositor_private()?.width ?? 0,
            height: w.get_compositor_private()?.height ?? 0}));
        const selected = teamWindows(records, this._options.characters)
            .filter(record => !this._options.hide_active || record.id !== this.Active());
        const signature = JSON.stringify([selected, this._options]);
        if (signature === this._signature)
            return;
        this._clear();
        this._signature = signature;
        const options = this._options;
        for (const [index, record] of selected.entries()) {
            const window = windows.find(w => w.get_stable_sequence() === record.id);
            const actor = window?.get_compositor_private();
            if (!actor || actor.width <= 0 || actor.height <= 0)
                continue;
            const name = record.title.slice(6);
            const height = options.mode === 'List' ? 34 : options.height;
            const card = new St.Widget({reactive: true, width: options.width, height: height + 26,
                style: 'background-color: #181818; border: 1px solid #b49b69;'});
            const label = new St.Label({text: name, width: options.width - 12, height: 24,
                style: 'color: #fcfaf2; padding-left: 6px; padding-top: 3px; font-size: 13px;'});
            card.add_child(label);
            if (options.mode !== 'List') {
                const clone = new Clutter.Clone({source: actor});
                const scale = Math.min(options.width / actor.width, height / actor.height);
                clone.set_scale(scale, scale);
                clone.set_position((options.width - actor.width * scale) / 2, 26);
                card.add_child(clone);
            }
            card.opacity = Math.round(options.opacity * 255 / 100);
            const pos = Object.hasOwn(options.positions, name)
                ? options.positions[name] : [20, 80 + index * (height + 36)];
            card.set_position(pos[0], pos[1]);
            // A single click focuses one client. No input duplication or game commands.
            card.connect('button-release-event', (_, event) => {
                if (event.get_button() === 1)
                    this.Activate(record.id);
                return Clutter.EVENT_STOP;
            });
            this._cards.set(record.id, card);
            Main.layoutManager.addChrome(card, {trackFullscreen: false});
        }
    }
}
