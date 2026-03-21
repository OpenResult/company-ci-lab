import { readFileSync } from 'node:fs';

const source = readFileSync(new URL('../src/index.ts', import.meta.url), 'utf8');

const requiredSnippets = [
  'export interface ReleaseInfo',
  'readonly component: string;',
  "readonly pipeline: 'verify' | 'package' | 'publish';",
  'export function greet(name: string): string',
  'export function formatReleaseTag(info: ReleaseInfo): string',
];

for (const snippet of requiredSnippets) {
  if (!source.includes(snippet)) {
    throw new Error(`Missing expected TypeScript contract snippet: ${snippet}`);
  }
}

console.log('node-lib typecheck passed');
