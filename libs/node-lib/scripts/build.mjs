import { mkdirSync, writeFileSync } from 'node:fs';

mkdirSync(new URL('../dist/', import.meta.url), { recursive: true });

writeFileSync(
  new URL('../dist/index.js', import.meta.url),
  `export function greet(name) { return \`hello, \${name}\`; }
export function formatReleaseTag(info) { return \`\${info.component}:\${info.pipeline}\`; }
`
);

writeFileSync(
  new URL('../dist/index.d.ts', import.meta.url),
  `export interface ReleaseInfo {
  readonly component: string;
  readonly pipeline: 'verify' | 'package' | 'publish';
}
export declare function greet(name: string): string;
export declare function formatReleaseTag(info: ReleaseInfo): string;
`
);

console.log('node-lib build produced dist/index.js and dist/index.d.ts');
