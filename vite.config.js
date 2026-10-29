import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  clearScreen: false,
  resolve: {
    conditions: ["browser", "import", "module", "default"],
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Tell Vite to ignore watching `src-tauri` and build outputs
      ignored: [
        "**/src-tauri/**",
        "**/legacy_wpf/**",
        "**/.agents/**",
        "**/release/**",
      ],
    },
  },
  build: {
    target: "esnext",
  },
});
