import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('.', import.meta.url)));
const port = Number(process.env.SURFER_SPIKE_PORT || 4173);
const host = process.env.SURFER_SPIKE_HOST || '127.0.0.1';

const server = createServer(async (request, response) => {
  const pathname = new URL(request.url || '/', 'http://127.0.0.1').pathname;
  const relativePath = pathname === '/' ? 'index.html' : decodeURIComponent(pathname.slice(1));
  const filePath = resolve(root, relativePath);
  if (filePath !== root && !filePath.startsWith(root + sep)) {
    response.writeHead(403).end('forbidden');
    return;
  }
  try {
    const bytes = await readFile(filePath);
    const contentType = new Map([
      ['.html', 'text/html; charset=utf-8'],
      ['.js', 'text/javascript; charset=utf-8'],
      ['.json', 'application/json; charset=utf-8'],
      ['.vcd', 'text/plain; charset=utf-8'],
      ['.wasm', 'application/wasm'],
    ]).get(extname(filePath)) || 'application/octet-stream';
    response.writeHead(200, {
      'Content-Type': contentType,
      'Cache-Control': 'no-store',
    });
    response.end(bytes);
  } catch {
    response.writeHead(404).end('not found');
  }
});

server.listen(port, host, () => {
  console.log('Surfer spike harness listening on http://' + host + ':' + port + '/');
});
