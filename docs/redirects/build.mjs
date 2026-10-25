import { mkdir, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

const root = path.resolve(process.argv[2] ?? 'docs/book');
const guide = path.join(root, 'docs');
const escape = (value) => value.replace(/[&<>"']/g, (character) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[character]);

async function visit(directory, prefix = '') {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const relative = path.posix.join(prefix, entry.name);
    if (entry.isDirectory()) {
      await visit(path.join(directory, entry.name), relative);
      continue;
    }
    if (!entry.isFile() || !relative.endsWith('.html') || relative === 'index.html' || relative === '404.html') continue;
    const destination = path.join(root, relative);
    const target = path.posix.relative(path.posix.dirname(relative), `docs/${relative}`);
    const url = `https://writerslogic.github.io/bluepencil/docs/${relative}`;
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, `<!doctype html>\n<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta http-equiv="refresh" content="0; url=${escape(target)}"><link rel="canonical" href="${escape(url)}"><title>Bluepencil guide moved</title></head><body><p>The guide page moved to <a href="${escape(target)}">${escape(target)}</a>.</p></body></html>\n`);
  }
}

await visit(guide);
