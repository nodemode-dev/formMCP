const memory = new Map<string, string>();

const store = {
  getItem: (key: string): string | null => memory.get(key) ?? null,
  setItem: (key: string, value: string): void => {
    memory.set(key, String(value));
  },
  removeItem: (key: string): void => {
    memory.delete(key);
  },
  clear: (): void => memory.clear(),
};

try {
  Object.defineProperty(window, "localStorage", { value: store, configurable: true });
} catch {
  memory.clear();
}
