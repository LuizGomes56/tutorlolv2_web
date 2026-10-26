import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const source = await readFile(new URL('../public/invoke.js', import.meta.url), 'utf8');
const tick = () => new Promise(resolve => setImmediate(resolve));
async function bridge(invoke, listen) {
    globalThis.window = Object.assign(new EventTarget(), { __TAURI_INTERNALS__: { invoke }, __TAURI__: { event: { listen } } });
    return import(`data:text/javascript;base64,${Buffer.from(source + `\n//${Math.random()}`).toString('base64')}`);
}

test('a newer focus event wins over a delayed initial snapshot', async () => {
    let deliver;
    let resolveSnapshot;
    const api = await bridge(() => new Promise(resolve => { resolveSnapshot = resolve; }), async (_, callback) => { deliver = callback; return () => {}; });
    assert.equal(typeof api.watch_overlay_focus, 'function');
    const values = [];
    const stop = api.watch_overlay_focus(value => values.push(value), error => { throw error; });
    await tick();
    deliver({ payload: true });
    resolveSnapshot(false);
    await tick();
    assert.deepEqual(values, [true]);
    stop();
});

test('disposing before async registration resolves still removes the listener', async () => {
    let registered;
    let callback;
    let removed = 0;
    const api = await bridge(async () => false, (_, deliver) => { callback = deliver; return new Promise(resolve => { registered = resolve; }); });
    assert.equal(typeof api.watch_overlay_focus, 'function');
    const values = [];
    const stop = api.watch_overlay_focus(value => values.push(value), error => { throw error; });
    stop();
    registered(() => removed++);
    await tick();
    callback({ payload: true });
    assert.equal(removed, 1);
    assert.deepEqual(values, []);
});

test('shortcut save preserves native validation and storage errors', async () => {
    const api = await bridge(async command => { assert.equal(command, 'set_overlay_shortcut'); throw new Error('Disk is full'); }, async () => () => {});
    assert.equal(typeof api.save_overlay_shortcut, 'function');
    await assert.rejects(api.save_overlay_shortcut({code:'F8',ctrl:false,alt:false,shift:false,meta:false}), /Disk is full/);
});

test('focus loss cancels recording even while native startup is pending', async () => {
    let acknowledge;
    const calls = [];
    const api = await bridge(async (command, args) => {
        calls.push(args.recording);
        if (args.recording) await new Promise(resolve => { acknowledge = resolve; });
    }, async () => () => {});
    const target = new EventTarget();
    window.addEventListener = target.addEventListener.bind(target);
    window.removeEventListener = target.removeEventListener.bind(target);
    let finished = 0;
    const captured = [];
    const stop = api.start_shortcut_recording(value => captured.push(value), error => { throw error; }, () => finished++);
    try {
        target.dispatchEvent(new Event('blur'));
        acknowledge();
        await tick();
        assert.equal(finished, 1);
        assert.deepEqual(calls, [true, false]);
        target.dispatchEvent(new Event('keydown'));
        assert.deepEqual(captured, []);
    } finally { stop(); }
});

test('Escape in the focused webview closes interaction and cleanup removes the handler', async () => {
    const calls = [];
    const values = [];
    const api = await bridge(async command => {
        calls.push(command);
        if (command === 'get_overlay_focus') return true;
    }, async () => () => {});
    const target = new EventTarget();
    window.addEventListener = target.addEventListener.bind(target);
    window.removeEventListener = target.removeEventListener.bind(target);
    const stop = api.watch_overlay_focus(value => values.push(value), error => { throw error; });
    await tick();
    const escape = new Event('keydown', { cancelable: true });
    Object.defineProperty(escape, 'code', { value: 'Escape' });
    target.dispatchEvent(escape);
    await tick();
    assert.equal(calls.filter(command => command === 'blur_overlay').length, 1);
    assert.equal(values.at(-1), false);
    assert.equal(escape.defaultPrevented, true);
    stop();
    target.dispatchEvent(escape);
    await tick();
    assert.equal(calls.filter(command => command === 'blur_overlay').length, 1);
});

test('a delayed Escape response cannot overwrite a newer native reopen', async () => {
    let deliver;
    let closed;
    const api = await bridge(async command => {
        if (command === 'get_overlay_focus') return true;
        if (command === 'blur_overlay') await new Promise(resolve => { closed = resolve; });
    }, async (_, callback) => { deliver = callback; return () => {}; });
    const values = [];
    const stop = api.watch_overlay_focus(value => values.push(value), error => { throw error; });
    await tick();
    const escape = new Event('keydown', { cancelable: true });
    Object.defineProperty(escape, 'code', { value: 'Escape' });
    window.dispatchEvent(escape);
    deliver({payload: false});
    deliver({payload: true});
    closed();
    await tick();
    assert.deepEqual(values, [true, false, true]);
    stop();
});
