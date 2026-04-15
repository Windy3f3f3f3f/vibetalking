import { defineConfig } from "vite";
import { resolve } from "path";

export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    rollupOptions: {
      input: {
        popover: resolve(__dirname, "src/popover.html"),
        settings: resolve(__dirname, "src/settings.html"),
      },
    },
    outDir: "../dist",
    emptyOutDir: true,
  },
  root: "src",
});
