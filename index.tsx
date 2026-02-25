import React from "react";
import { render } from "ink";
import App from "./src/components/App.tsx";

try {
  process.stdin.setRawMode?.(true);
} catch {
  // Raw mode is unavailable in some environments.
}

render(<App />);