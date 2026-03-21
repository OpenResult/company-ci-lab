import { readFileSync } from 'node:fs';
<<<<<<< ours
<<<<<<< ours
<<<<<<< ours
readFileSync(new URL('../src/app/page.tsx', import.meta.url));
console.log('next-web lint placeholder passed');
=======
=======
>>>>>>> theirs
=======
>>>>>>> theirs

const pageSource = readFileSync(new URL('../src/app/page.tsx', import.meta.url), 'utf8');
const siteConfig = JSON.parse(readFileSync(new URL('../src/config/site.json', import.meta.url), 'utf8'));

if (!pageSource.includes('site.title') || !pageSource.includes('site.subtitle')) {
  throw new Error('Page source should render site.title and site.subtitle');
}

if (!siteConfig.subtitle.includes('CLI orchestration')) {
  throw new Error('Unexpected site subtitle content');
}

console.log(`next-web lint passed for ${siteConfig.title}`);
<<<<<<< ours
<<<<<<< ours
>>>>>>> theirs
=======
>>>>>>> theirs
=======
>>>>>>> theirs
