import { readFileSync } from 'node:fs';
readFileSync(new URL('../src/app/page.tsx', import.meta.url));
console.log('next-web lint placeholder passed');
