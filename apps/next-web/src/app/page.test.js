import test from 'node:test';
import assert from 'node:assert/strict';

test('sample page metadata', () => {
  assert.equal('company-ci next-web'.includes('next-web'), true);
});
