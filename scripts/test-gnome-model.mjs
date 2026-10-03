import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
const source = await readFile(new URL('../gnome-extension/nicotine-teams@hapsiellx/model.js', import.meta.url), 'utf8');
const {parsePreviewOptions, teamWindows} = await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);
const good = {show: true, mode: 'Previews', width: 320, height: 180, opacity: 100, revision: 2, characters: null, positions: {Pilot: [100,-50]}};
assert.equal(parsePreviewOptions(JSON.stringify(good)).width, 320);
for (const changes of [{width: 0}, {height: 10000}, {opacity: 0}, {characters: ['Pilot','Pilot']}, {characters: 'Pilot'}, {revision: -1}, {positions: {Pilot: [0,'bad']}}, {mode: 'Unknown'}, {hide_active: 'yes'}])
    assert.throws(() => parsePreviewOptions(JSON.stringify({...good, ...changes})));
assert.throws(() => parsePreviewOptions('x'.repeat(65537)));
const windows = [{title: 'EVE - A'}, {title: 'EVE - B'}, {title: 'EVE - Unselected'}];
assert.deepEqual(teamWindows(windows, []), []);
assert.deepEqual(teamWindows(windows, ['B','A']), [windows[1],windows[0]]);
assert.deepEqual(teamWindows(windows, null), windows);
console.log('GNOME bridge validation and team selection passed');
