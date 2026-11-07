declare module "*.css";
declare const __APP_VERSION__: string;
interface ImportMeta {
  glob(pattern: string, options: { eager: true; query: string; import: string }): Record<string, unknown>;
}
