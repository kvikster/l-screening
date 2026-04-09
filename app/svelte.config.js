import adapter from "@sveltejs/adapter-static";
import { relative, sep } from "node:path";

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
    // adapter-static is required for PWA deployment
    adapter: adapter({
      pages: "build",
      assets: "build",
      fallback: "index.html",
      precompress: false,
      strict: true,
    }),

    // Single-page app mode with hash routing.
    router: {
      type: "hash",
    },

    // Asset URLs for PWA deployment
    paths: {
      base: process.env.PUBLIC_BASE_PATH ?? '',
    },
  },
};

export default config;
