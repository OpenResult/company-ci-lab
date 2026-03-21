import test from 'node:test';
import assert from 'node:assert/strict';
<<<<<<< ours
<<<<<<< ours
<<<<<<< ours

test('sample page metadata', () => {
  assert.equal('company-ci next-web'.includes('next-web'), true);
=======
=======
>>>>>>> theirs
=======
>>>>>>> theirs
import { readFileSync } from 'node:fs';

const config = JSON.parse(readFileSync(new URL('../config/site.json', import.meta.url), 'utf8'));

test('site config exposes expected homepage copy', () => {
  assert.equal(config.title, 'company-ci next-web');
  assert.match(config.subtitle, /CLI orchestration/);
<<<<<<< ours
<<<<<<< ours
>>>>>>> theirs
=======
>>>>>>> theirs
=======
>>>>>>> theirs
});
