const blocked = (): never => {
  throw new DOMException("this demo keeps its state in memory only", "SecurityError");
};

try {
  Object.defineProperty(window, "localStorage", { configurable: true, get: blocked });
} catch {
  Object.defineProperty(window, "__isolated", { value: false });
}
