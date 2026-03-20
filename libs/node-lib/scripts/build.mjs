import { mkdirSync, writeFileSync } from 'node:fs';
mkdirSync(new URL('../dist/', import.meta.url), { recursive: true });
writeFileSync(new URL('../dist/index.js', import.meta.url), `export function greet(name) { return \`hello, \${name}\`; }\n`);
writeFileSync(new URL('../dist/index.d.ts', import.meta.url), `export declare function greet(name: string): string;\n`);
console.log('node-lib build placeholder passed');
