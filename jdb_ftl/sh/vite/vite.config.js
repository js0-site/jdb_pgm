import { defineConfig } from "vite";
import { resolve } from "path";
import { readFileSync, writeFileSync, mkdirSync, readdirSync } from "fs";

export default defineConfig({
  build: {
    outDir: "dist",
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
        entryFileNames: "assets.js",
        chunkFileNames: "assets.js",
        assetFileNames: "assets.[ext]",
      },
    },
    minify: "terser",
    terserOptions: {
      compress: {
        drop_console: true,
        pure_funcs: ["console.log"],
      },
    },
    cssMinify: true,
  },
  assetsInclude: ["**/*.svg"],
  css: {
    preprocessorOptions: {
      stylus: {
        imports: [resolve(__dirname, "src/styles/main.styl")],
      },
    },
  },
  plugins: [
    {
      name: "inline-resources",
      closeBundle() {
        const assets = [];
        const distDir = resolve(__dirname, "dist");
        const files = readdirSync(distDir);

        for (const file of files) {
          if (file.endsWith(".css")) {
            const content = readFileSync(resolve(distDir, file), "utf-8");
            assets.push({
              type: "css",
              content: content,
            });
          } else if (file.endsWith(".js")) {
            const content = readFileSync(resolve(distDir, file), "utf-8");
            assets.push({
              type: "js",
              content: content,
            });
          }
        }

        // Write assets to a JSON file for readmeMerge.js to use
        const outputPath = resolve(distDir, "assets.json");
        writeFileSync(outputPath, JSON.stringify(assets, null, 2));
      },
    },
  ],
});
