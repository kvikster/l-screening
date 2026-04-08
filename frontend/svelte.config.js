import adapter from "@sveltejs/adapter-static";
import { relative, sep } from "node:path";

const isStandalone = process.env.VITE_STANDALONE === "true";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  compilerOptions: {
    // defaults to rune mode for the project, except for `node_modules`. Can be removed in svelte 6.
    runes: ({ filename }) => {
      const relativePath = relative(import.meta.dirname, filename);
      const pathSegments = relativePath.toLowerCase().split(sep);
      const isExternalLibrary = pathSegments.includes("node_modules");

      return isExternalLibrary ? undefined : true;
    },
  },
  kit: {
    // adapter-static is required for Tauri apps to package as a standalone app.
    adapter: adapter({
      pages: isStandalone ? "../standalone" : "build",
      assets: isStandalone ? "../standalone" : "build",
      fallback: "index.html",
      precompress: false,
      strict: true,
    }),

    // Single-page app mode with hash routing.
    router: {
      type: "hash",
    },

    // Keep asset URLs relative for standalone/file-based usage.
    paths: {
      relative: isStandalone,
      base: isStandalone ? '' : (process.env.PUBLIC_BASE_PATH ?? ''),
    },
  },
};

export default config;
