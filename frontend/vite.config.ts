import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { readFileSync, existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import type { Plugin } from 'vite';

/**
 * Standalone build plugin: replaces the wasm-pack `new URL(*.wasm, import.meta.url)`
 * reference with an inlined base64 ArrayBuffer so the app works from file:// in Chrome
 * without any network requests.
 */
function inlineWasmPlugin(): Plugin {
    return {
        name: 'inline-wasm-standalone',
        enforce: 'pre',
        transform(code, id) {
            if (!id.endsWith('screening_wasm.js') || id.includes('node_modules')) return null;
            const wasmPath = resolve(dirname(id), 'screening_wasm_bg.wasm');
            if (!existsSync(wasmPath)) return null;
            const base64 = readFileSync(wasmPath).toString('base64');
            // Replace the URL-based WASM reference with an inline ArrayBuffer.
            // The wasm-pack init function accepts an ArrayBuffer directly and
            // skips fetch() when given one, so this is all that's needed.
            return {
                code: code.replace(
                    "module_or_path = new URL('screening_wasm_bg.wasm', import.meta.url);",
                    `{ const _b='${base64}',_s=atob(_b),_u=new Uint8Array(_s.length);` +
                    `for(let _i=0;_i<_s.length;_i++)_u[_i]=_s.charCodeAt(_i);` +
                    `module_or_path=_u.buffer; }`
                ),
                map: null,
            };
        },
    };
}

const isStandalone = process.env.VITE_STANDALONE === 'true';

export default defineConfig({
    plugins: [
        tailwindcss(),
        sveltekit(),
        ...(isStandalone ? [inlineWasmPlugin()] : []),
    ],
    server: {
        fs: { allow: ['..'] },
    },
});
