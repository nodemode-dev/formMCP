import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { extensionsDev } from "./extensions-dev";

const FRONTEND_PORT = 5620;
const pkg = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf-8")) as { version: string };
const demoRoot = fileURLToPath(new URL("../demo/src", import.meta.url));
const demoOut = fileURLToPath(new URL("../demo", import.meta.url));

export default defineConfig({
  root: demoRoot,
  base: "./",
  publicDir: false,
  clearScreen: false,
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  plugins: [
    extensionsDev(),
    {
      name: "classic-script-for-file-urls",
      enforce: "post",
      transformIndexHtml: (html: string) => html.replace(/<script type="module" crossorigin/g, "<script defer").replace(/ crossorigin/g, ""),
    },
  ],
  server: {
    port: FRONTEND_PORT,
    host: false,
    strictPort: true,
    fs: { allow: [fileURLToPath(new URL("..", import.meta.url))] },
  },
  preview: {
    port: FRONTEND_PORT + 1,
    host: false,
    strictPort: true,
  },
  build: {
    outDir: demoOut,
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
});
