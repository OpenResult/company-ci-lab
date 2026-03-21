import { createReadStream, existsSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, join } from 'node:path';

const port = Number.parseInt(process.env.PORT ?? '3000', 10);
const distDir = new URL('../dist/', import.meta.url);
const indexPath = new URL('../dist/index.html', import.meta.url);

const contentTypes = {
  '.html': 'text/html; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
};

const server = createServer((request, response) => {
  if (!request.url) {
    response.writeHead(400);
    response.end('missing request url');
    return;
  }

  const url = new URL(request.url, `http://${request.headers.host ?? '127.0.0.1'}`);
  if (url.pathname === '/healthz') {
    response.writeHead(200, { 'content-type': 'text/plain; charset=utf-8' });
    response.end('ok');
    return;
  }

  const targetPath = url.pathname === '/' ? indexPath.pathname : join(distDir.pathname, url.pathname);
  if (!existsSync(targetPath)) {
    response.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
    response.end('not found');
    return;
  }

  response.writeHead(200, {
    'content-type': contentTypes[extname(targetPath)] ?? 'application/octet-stream',
    'cache-control': 'no-store',
  });
  createReadStream(targetPath).pipe(response);
});

server.listen(port, '0.0.0.0', () => {
  console.log(`next-web static server listening on ${port}`);
});
