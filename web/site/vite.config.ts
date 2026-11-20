import { defineConfig } from "vite";
import { resolve } from "node:path";

export default defineConfig({
  // The word list lives in ../../data/wordlist and is imported with ?url, so
  // the build emits it as its own hashed asset (not inside the JS bundle).
  server: { fs: { allow: [__dirname, resolve(__dirname, "../../data/wordlist")] } },
  build: {
    target: "es2022",
    assetsInlineLimit: 0,
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        privacy: resolve(__dirname, "privacy.html"),
        cookies: resolve(__dirname, "cookies.html"),
        help: resolve(__dirname, "help.html"),
        notFound: resolve(__dirname, "404.html"),
      },
    },
  },
});
