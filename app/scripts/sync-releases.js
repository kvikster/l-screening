#!/usr/bin/env bun
import { marked } from "marked";
import { readdir, readFile, writeFile, mkdir, copyFile } from "fs/promises";
import { join, basename, dirname } from "path";
import { fileURLToPath } from "url";

const __dir = dirname(fileURLToPath(import.meta.url));
const APP = join(__dir, "..");
const SRC = join(APP, "..", "releases");
const DEST = join(APP, "static", "releases");

const CSS = `
    :root {
      color-scheme: light;
      --bg: #f8fafc;
      --card: #ffffff;
      --text: #0f172a;
      --muted: #475569;
      --border: #dbe4f0;
      --accent: #2563eb;
      --accent-soft: #dbeafe;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: linear-gradient(180deg, #eef4ff 0%, var(--bg) 12rem);
      color: var(--text);
    }
    .wrap {
      max-width: 860px;
      margin: 0 auto;
      padding: 32px 20px 72px;
    }
    .card {
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 24px;
      padding: 28px;
      box-shadow: 0 18px 50px rgba(15, 23, 42, 0.08);
    }
    .eyebrow {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      border-radius: 999px;
      background: var(--accent-soft);
      color: var(--accent);
      padding: 6px 12px;
      font-size: 12px;
      font-weight: 700;
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }
    h1 {
      margin: 18px 0 10px;
      font-size: clamp(2rem, 4vw, 3rem);
      line-height: 1.05;
    }
    p, li {
      color: var(--muted);
      font-size: 1rem;
      line-height: 1.7;
    }
    h2 {
      margin: 28px 0 12px;
      font-size: 1.2rem;
    }
    ul {
      margin: 0;
      padding-left: 20px;
    }
    li + li { margin-top: 10px; }
    code {
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.95em;
      background: #eff6ff;
      color: #1d4ed8;
      padding: 0.1em 0.4em;
      border-radius: 6px;
    }
    .back {
      display: inline-block;
      margin-top: 24px;
      color: var(--accent);
      text-decoration: none;
      font-weight: 600;
    }
    .back:hover { text-decoration: underline; }
    hr { border: none; border-top: 1px solid var(--border); margin: 28px 0 0; }
    a { color: var(--accent); }
`;

function buildHtml(title, bodyHtml) {
  return `<!doctype html>
<html lang="uk">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${title}</title>
  <style>${CSS}
  </style>
</head>
<body>
  <main class="wrap">
    <article class="card">
      <div class="eyebrow">Release Notes</div>
      ${bodyHtml}
      <a class="back" href="../">Повернутися до застосунку</a>
    </article>
  </main>
</body>
</html>
`;
}

function extractTitle(html) {
  const m = html.match(/<h1[^>]*>(.*?)<\/h1>/s);
  return m ? m[1].replace(/<[^>]+>/g, "").trim() : "Release Notes";
}

async function main() {
  await mkdir(DEST, { recursive: true });

  const files = (await readdir(SRC)).filter((f) => f.endsWith(".md"));

  for (const file of files) {
    const src = join(SRC, file);
    const md = await readFile(src, "utf8");
    const bodyHtml = await marked(md);
    const title = extractTitle(bodyHtml) || "Список релізів";
    const html = buildHtml(title, bodyHtml);

    const base = basename(file, ".md");
    const htmlWithLinks = html.replace(/href="([^"]*?)\.md"/g, 'href="$1.html"');
    await writeFile(join(DEST, file), md);
    await writeFile(join(DEST, `${base}.html`), htmlWithLinks);
    console.log(`  synced ${file} → ${base}.md + ${base}.html`);
  }

  console.log(`\nDone — ${files.length} release(s) synced to app/static/releases/`);
}

main().catch((e) => { console.error(e); process.exit(1); });
