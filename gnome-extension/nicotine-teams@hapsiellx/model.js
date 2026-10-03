// Pure validation, also exercised by Node without a running Shell.
export function parsePreviewOptions(json) {
    if (json.length > 65536)
        throw new Error('Preview request too large');
    const value = JSON.parse(json);
    if (!value || typeof value !== 'object' || typeof value.show !== 'boolean')
        throw new Error('Invalid preview request');
    for (const [key, min, max] of [['width', 120, 800], ['height', 80, 600], ['opacity', 10, 100]]) {
        if (!Number.isInteger(value[key]) || value[key] < min || value[key] > max)
            throw new Error(`Invalid ${key}`);
    }
    if (!Number.isSafeInteger(value.revision) || value.revision < 0)
        throw new Error('Invalid revision');
    if (!['Previews', 'List'].includes(value.mode) ||
        (value.hide_active !== undefined && typeof value.hide_active !== 'boolean'))
        throw new Error('Invalid display mode');
    if (value.characters !== null && (!Array.isArray(value.characters) || value.characters.length > 100 ||
        value.characters.some(name => typeof name !== 'string' || !name.trim() || name.length > 100) ||
        new Set(value.characters).size !== value.characters.length))
        throw new Error('Invalid team');
    if (!value.positions || typeof value.positions !== 'object' || Array.isArray(value.positions))
        throw new Error('Invalid positions');
    for (const position of Object.values(value.positions)) {
        if (!Array.isArray(position) || position.length !== 2 ||
            position.some(n => !Number.isInteger(n) || n < -32768 || n > 32767))
            throw new Error('Invalid position');
    }
    return value;
}

export function teamWindows(windows, characters) {
    if (characters === null)
        return windows;
    return characters.flatMap(name => windows.filter(w => w.title === `EVE - ${name}`));
}
