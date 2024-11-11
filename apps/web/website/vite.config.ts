import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [sveltekit()],
  build: {
    target: ["chrome124", "safari18", "firefox125", "esnext"],
  },
  server: {
    fs: {
      strict: true,
      // fixes error when running under ibazel.
      allow: ["/tmp", "/home"],
    },
  },
  resolve: {
    preserveSymlinks: true,
  },
});
