import test from 'node:test';
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const moduleUrl = pathToFileURL(resolve(process.cwd(), 'dist/index.js')).href;

await test('build output exposes the greeting contract', async () => {
  const lib = await import(moduleUrl);
  assert.equal(lib.greet('codex'), 'hello, codex');
  assert.equal(lib.formatReleaseTag({ component: 'node-lib', pipeline: 'verify' }), 'node-lib:verify');
});
