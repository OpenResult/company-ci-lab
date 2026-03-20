import { readFileSync } from 'node:fs';

const source = readFileSync(new URL('../src/index.ts', import.meta.url), 'utf8');

if (!source.includes('formatReleaseTag')) {
  throw new Error('Expected formatReleaseTag export in src/index.ts');
}

if (!source.includes('ReleaseInfo')) {
  throw new Error('Expected ReleaseInfo interface in src/index.ts');
}

console.log('node-lib lint passed');
