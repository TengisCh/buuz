import { defineConfig } from "vite";
import { resolve } from "node:path";

export default defineConfig({
  build: {
    target: "es2022",
    assetsInlineLimit: 0,
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        privacy: resolve(__dirname, "privacy.html"),
        cookies: resolve(__dirname, "cookies.html"),
        notFound: resolve(__dirname, "404.html"),
      },
    },
  },
});
