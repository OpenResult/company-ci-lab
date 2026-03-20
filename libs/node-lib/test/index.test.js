import test from 'node:test';
import assert from 'node:assert/strict';

test('library greeting contract', () => {
  assert.equal(`hello, codex`, 'hello, codex');
});
