#!/usr/bin/env bun

import { mkdirSync, readFileSync, writeFileSync, rmSync } from "fs";
import { resolve } from "path";
import { renderEn, renderZh } from "./src/render.js";
import { minify } from "html-minifier-terser";
import { $ } from "zx";

const DIR = import.meta.dirname,
  DIST = resolve(DIR, "dist"),
  GEN = resolve(DIR, "..", "..", "gen");

mkdirSync(DIST, { recursive: true });
mkdirSync(GEN, { recursive: true });

console.log("Building JS and CSS bundle with Vite...");
await $`bunx vite build --config vite.bundle.config.js`;

const js_bundle = readFileSync(resolve(DIST, "bundle.js"), "utf-8"),
  css_path = resolve(DIST, "bundle.css");

let css_content = "";
try {
  css_content = readFileSync(css_path, "utf-8");
} catch {
  console.log("bundle.css not found, trying assets.css...");
  try {
    css_content = readFileSync(resolve(DIST, "assets.css"), "utf-8");
  } catch {
    console.log("CSS file not found, building with stylus...");
    await $`bunx stylus src/main.styl -o dist/assets.css --compress`;
    css_content = readFileSync(resolve(DIST, "assets.css"), "utf-8");
  }
}

const katex_path = resolve(
  DIR,
  "node_modules",
  "katex",
  "dist",
  "katex.min.css",
);
let katex_css = readFileSync(katex_path, "utf-8");
// Fix font paths and use absolute CDN URLs
katex_css = katex_css.replace(
  /url\(fonts\//g,
  "url(https://cdn.jsdelivr.net/npm/katex@0.16.27/dist/fonts/",
);

console.log(
  `✓ Assets loaded: JS(${Math.round(js_bundle.length / 1024)}KB), CSS(${Math.round(css_content.length / 1024)}KB)`,
);

console.log("Rendering and minifying HTML...");
for (const [lang, render_fn] of [
  ["zh", renderZh],
  ["en", renderEn],
]) {
  const output_file = resolve(GEN, `${lang}.html`);

  // Clean start
  try {
    rmSync(output_file);
  } catch { }

  let html = await render_fn();
  console.log(`  [${lang}] Pug output: ${html.length} chars`);

  // Use function callback in .replace() to avoid '$' special sequence issues in assets
  html = html
    .replace(/<link[^>]*katex[^>]*>/g, "")
    .replace(/<script[^>]*>[\s\S]*?mermaid[\s\S]*?<\/script>/g, "");

  html = html.replace(
    "</head>",
    () => `<style>${katex_css}${css_content}</style></head>`,
  );
  html = html.replace("</body>", () => `<script>${js_bundle}</script></body>`);

  console.log(`  [${lang}] Combined: ${html.length} chars`);

  let minified = html;
  try {
    minified = await minify(html, {
      collapseWhitespace: true,
      removeComments: true,
      minifyCSS: true,
      minifyJS: false, // Vite already minified the bundle
      useShortDoctype: true,
      removeEmptyAttributes: true,
      removeRedundantAttributes: true,
      removeScriptTypeAttributes: true,
      removeStyleLinkTypeAttributes: true,
    });
    console.log(`  [${lang}] Minified: ${minified.length} chars`);
  } catch (e) {
    console.warn(`  [${lang}] HTML minification failed: ${e.message.slice(0, 500)}`);
  }

  writeFileSync(output_file, minified);
  console.log(`✓ ${lang}.html saved!`);
}

console.log("\nBuild completed successfully!");
export default {};
