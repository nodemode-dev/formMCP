import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const pkg = JSON.parse(readFileSync(new URL("../../frontend/package.json", import.meta.url), "utf-8"));
const root = fileURLToPath(new URL("./src", import.meta.url));
const out = fileURLToPath(new URL("./", import.meta.url));
const repo = fileURLToPath(new URL("../../", import.meta.url));

export default {
  root,
  base: "./",
  publicDir: false,
  clearScreen: false,
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  plugins: [
    {
      name: "classic-script-for-file-urls",
      enforce: "post",
      transformIndexHtml: (html) => html.replace(/<script type="module" crossorigin/g, "<script defer").replace(/ crossorigin/g, ""),
    },
  ],
  server: {
    port: 5623,
    host: false,
    strictPort: true,
    fs: { allow: [repo] },
  },
  build: {
    outDir: out,
    emptyOutDir: false,
    assetsDir: "assets",
    cssCodeSplit: false,
    sourcemap: false,
    target: "es2022",
    assetsInlineLimit: 0,
    rolldownOptions: {
      output: { format: "iife", codeSplitting: false },
    },
  },
};
