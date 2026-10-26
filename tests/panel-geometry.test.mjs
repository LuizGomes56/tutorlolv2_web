import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const source = await readFile(new URL('../public/events.js', import.meta.url), 'utf8');
const api = await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);

test('wide tables shrink proportionally along the pointer diagonal', () => {
    assert.equal(typeof api.resize_geometry, 'function');
    assert.deepEqual(api.resize_geometry({handle:'se',left:100,top:200,width:820,height:120,baseWidth:800,baseHeight:100,scale:1}, -280, -35), {left:100,top:200,scale:.65});
});

test('all four corners preserve the opposite corner while shrinking', () => {
    assert.equal(typeof api.resize_geometry, 'function');
    const start={left:100,top:200,width:820,height:120,baseWidth:800,baseHeight:100,scale:1};
    assert.deepEqual(api.resize_geometry({...start,handle:'nw'}, 280, 35), {left:380,top:235,scale:.65});
    assert.deepEqual(api.resize_geometry({...start,handle:'ne'}, -280, 35), {left:100,top:235,scale:.65});
    assert.deepEqual(api.resize_geometry({...start,handle:'sw'}, 280, -35), {left:380,top:200,scale:.65});
});

test('minimum and maximum scale keep the anchored edge fixed', () => {
    assert.equal(typeof api.resize_geometry, 'function');
    const start={handle:'nw',left:100,top:200,width:820,height:120,baseWidth:800,baseHeight:100,scale:1};
    assert.deepEqual(api.resize_geometry(start, 8000, 1000), {left:620,top:265,scale:.35});
    assert.deepEqual(api.resize_geometry(start, -8000, -1000), {left:-700,top:100,scale:2});
});
