import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';

const config = JSON.parse(readFileSync(new URL('../src/config/site.json', import.meta.url), 'utf8'));
const outDir = new URL('../dist/', import.meta.url);
mkdirSync(outDir, { recursive: true });

const html = `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>${config.title}</title>
    <meta name="description" content="${config.description}" />
  </head>
  <body>
    <main>
      <h1>${config.title}</h1>
      <p>${config.subtitle}</p>
    </main>
  </body>
</html>
`;

writeFileSync(new URL('../dist/index.html', import.meta.url), html);
writeFileSync(new URL('../dist/build-manifest.json', import.meta.url), JSON.stringify({ generatedBy: 'company-ci scaffold', route: '/', title: config.title }, null, 2));
console.log('next-web build produced dist/index.html and dist/build-manifest.json');
