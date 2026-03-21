import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const config = JSON.parse(readFileSync(new URL('../config/site.json', import.meta.url), 'utf8'));

test('site config exposes expected homepage copy', () => {
  assert.equal(config.title, 'company-ci next-web');
  assert.match(config.subtitle, /CLI orchestration/);
});
