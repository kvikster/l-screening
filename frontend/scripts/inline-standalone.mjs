#!/usr/bin/env node
/**
 * Post-build script: takes the multi-file standalone/ output and produces
 * a single self-contained index.html that works from file:// in Chrome.
 *
 * Usage: node scripts/inline-standalone.mjs [standalone-dir]
 */

import { build } from 'esbuild';
import { readFileSync, writeFileSync, existsSync, readdirSync } from 'node:fs';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
// script lives at frontend/scripts/ → go up two levels to reach project root
const root = resolve(__dirname, '../..');
const standaloneDir = process.argv[2]
    ? resolve(process.cwd(), process.argv[2])
    : join(root, 'standalone');

const htmlPath = join(standaloneDir, 'index.html');
if (!existsSync(htmlPath)) {
    console.error(`Error: ${htmlPath} not found. Run build:standalone first.`);
    process.exit(1);
}

let html = readFileSync(htmlPath, 'utf8');

// ── Extract entry JS file paths from the bootstrap <script> ──────────────────
const importRe = /import\("(\/_app\/[^"]+)"\)/g;
const entryAbsPaths = [];
let m;
while ((m = importRe.exec(html)) !== null) {
    const absPath = join(standaloneDir, m[1].replace(/^\//, ''));
    if (!entryAbsPaths.includes(absPath)) entryAbsPaths.push(absPath);
}
if (entryAbsPaths.length === 0) {
    console.error('Could not find SvelteKit entry JS files in the bootstrap script.');
    process.exit(1);
}
console.log(`Found ${entryAbsPaths.length} entry point(s).`);

// ── Extract the SvelteKit global variable name ────────────────────────────────
// e.g. __sveltekit_15uzwra
const varMatch = html.match(/(__sveltekit_\w+)\s*=/);
const skVar = varMatch ? varMatch[1] : '__sveltekit_app';

// ── Gather CSS from assets dir ────────────────────────────────────────────────
const assetsDir = join(standaloneDir, '_app', 'immutable', 'assets');
let inlinedCss = '';
if (existsSync(assetsDir)) {
    for (const file of readdirSync(assetsDir)) {
        if (file.endsWith('.css')) {
            inlinedCss += readFileSync(join(assetsDir, file), 'utf8');
        }
    }
}

// ── Create synthetic entry that imports both SK entry files ───────────────────
// start.js exports `start`; app.js is the app module passed to start().
// We must set the SK global before start() reads it.
// Element: SvelteKit mounts into the <div style="display: contents"> wrapper.
// We give it id="sk-root" in the HTML below so we can reference it in ESM.
const [startPath, appPath] = entryAbsPaths;
const syntheticEntry = `
import * as kit from ${JSON.stringify(startPath)};
import * as app from ${JSON.stringify(appPath)};

// Replicate what the original bootstrap script does
window[${JSON.stringify(skVar)}] = {
    base: new URL('.', location.href).pathname.slice(0, -1),
};

const element = document.getElementById('sk-root');
kit.start(app, element);
`.trim();

// ── esbuild: bundle everything into one ESM file ──────────────────────────────
// Plugin: resolve /_app/... absolute imports to the standalone dir
const absoluteAppResolver = {
    name: 'absolute-app-resolver',
    setup(b) {
        b.onResolve({ filter: /^\/_app\// }, args => ({
            path: join(standaloneDir, args.path.replace(/^\//, '')),
        }));
    },
};

console.log('Bundling with esbuild...');
const result = await build({
    stdin: {
        contents: syntheticEntry,
        resolveDir: standaloneDir,
        sourcefile: 'entry.js',
    },
    bundle: true,
    format: 'esm',
    write: false,
    minify: true,
    plugins: [absoluteAppResolver],
});

if (result.errors.length) {
    console.error('esbuild errors:', result.errors);
    process.exit(1);
}

const bundledJs = result.outputFiles[0].text;
console.log(`Bundled JS size: ${(bundledJs.length / 1024).toFixed(0)} kB`);

// ── Patch the HTML ────────────────────────────────────────────────────────────

// 1. Strip all modulepreload links
html = html.replace(/<link[^>]+rel="modulepreload"[^>]*\/?>\s*/g, '');

// 2. Strip existing stylesheet links
html = html.replace(/<link[^>]+rel="stylesheet"[^>]*\/?>\s*/g, '');

// 3. Inject CSS before </head>
if (inlinedCss) {
    html = html.replace('</head>', `<style>${inlinedCss}</style>\n</head>`);
}

// 4. Add id="sk-root" to the SvelteKit container div
html = html.replace(
    '<div style="display: contents">',
    '<div id="sk-root" style="display: contents">',
);

// 5. Replace the inner SvelteKit bootstrap <script> block with our inlined bundle.
//    The inner script is the Promise.all([import(...)]) block.
const innerScriptRe = /(<script>\s*\{[\s\S]*?Promise\.all\(\[[\s\S]*?\]\)[\s\S]*?\}\s*<\/script>)/;
const moduleScriptTag = `<script type="module">${bundledJs}</script>`;

if (innerScriptRe.test(html)) {
    html = html.replace(innerScriptRe, moduleScriptTag);
} else {
    // Fallback: append before </body>
    html = html.replace('</body>', `${moduleScriptTag}\n</body>`);
}

writeFileSync(htmlPath, html, 'utf8');
console.log(`✅  Single-file bundle written to ${htmlPath}`);
