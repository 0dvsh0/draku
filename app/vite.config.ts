import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The app is on-device by design; keep the bundle lean and static.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "127.0.0.1",
  },
  build: {
    target: "es2022",
    minify: "esbuild",
    sourcemap: false,
    outDir: "dist",
  },
});
