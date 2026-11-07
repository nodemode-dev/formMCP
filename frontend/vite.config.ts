import { readFileSync } from "node:fs";
import { defineConfig } from "vite";
import { extensionsDev } from "./extensions-dev";

const FRONTEND_PORT = 5620;
const pkg = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf-8")) as { version: string };

export default defineConfig({
  root: ".",
  publicDir: "public",
  clearScreen: false,
  plugins: [extensionsDev()],
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  server: {
    port: FRONTEND_PORT,
    host: false,
    strictPort: true,
    fs: { allow: [".", "../core/data"] },
  },
  preview: {
    port: FRONTEND_PORT + 1,
    host: false,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    sourcemap: false,
    target: "es2022",
    assetsInlineLimit: 0,
  },
});
