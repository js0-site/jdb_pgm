#!/usr/bin/env bun

import { mkdirSync, readFileSync, writeFileSync } from "fs";
import { dirname, join, relative, resolve } from "path";
import MarkdownIt from "markdown-it";
import texmath from "markdown-it-texmath";
import katex from "katex";
import { launch } from "puppeteer";
import { $ } from "zx";

const DIR = resolve(import.meta.dirname),
  GEN = join(DIR, "gen"),
  VITE_DATA_DIR = join(DIR, "sh", "vite", "dist", "data"),
  md = MarkdownIt({
    html: true,
    linkify: true,
    typographer: true,
    breaks: true,
  }).use(texmath, { engine: katex, delimiters: "dollars" });

mkdirSync(GEN, { recursive: true });
mkdirSync(VITE_DATA_DIR, { recursive: true });

md.core.ruler.after("block", "github_alerts", (state) => {
  for (let i = 0; i < state.tokens.length; i++) {
    if (state.tokens[i].type === "blockquote_open") {
      let inline_token = null;
      for (
        let j = i + 1;
        j < state.tokens.length && state.tokens[j].type !== "blockquote_close";
        j++
      ) {
        if (state.tokens[j].type === "inline") {
          inline_token = state.tokens[j];
          break;
        }
      }
      if (inline_token) {
        const match = inline_token.content.match(
          /^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]/i,
        );
        if (match) {
          const type = match[1].toUpperCase();
          inline_token.content = inline_token.content.replace(
            /^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*/i,
            "",
          );
          state.tokens[i].attrJoin("class", "markdown-alert");
          state.tokens[i].attrJoin(
            "class",
            `markdown-alert-${type.toLowerCase()}`,
          );
        }
      }
    }
  }
});

const original_fence = md.renderer.rules.fence;
const escapeHtml = (unsafe) => {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
};

md.renderer.rules.fence = (tokens, idx, options, env, slf) => {
  const token = tokens[idx];
  if (token.info.trim() === "mermaid") {
    return `<pre class="mermaid">${escapeHtml(token.content)}</pre>`;
  }
  return original_fence(tokens, idx, options, env, slf);
};

const pathToId = (abs_path) => {
  const rel = relative(DIR, abs_path);
  return (
    rel
      .replace(/\.[^/.]+$/, "")
      .replace(/[^a-zA-Z0-9]/g, "-")
      .replace(/-+/g, "-")
      .replace(/^-|-$/g, "") || "root"
  );
};

const fixLinks = (content, base_dir) => {
  return content.replace(
    /\[((?:\[[^\]]*\]|[^\]])+)\]\(([^)]+\.md)(#[^)]*)?\)/g,
    (match, text, path, hash) => {
      if (path.startsWith("http")) return match;
      const abs_path = resolve(base_dir, path),
        id = pathToId(abs_path);
      return `[${text}](#${id}${hash || ""})`;
    },
  );
};

const fixImages = (content, base_dir) => {
  return content.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, (match, alt, path) => {
    if (path.startsWith("http") || path.startsWith("data:")) return match;

    const abs_path = resolve(base_dir, path);

    if (path.toLowerCase().endsWith(".svg")) {
      try {
        const svg_content = readFileSync(abs_path, "utf-8");
        return `\n\n<div class="svg-inline-marker">${svg_content}</div>\n\n`;
      } catch (e) {
        console.warn(`Warning: Could not read SVG image at ${abs_path}`);
        return match;
      }
    }

    const rel_to_root = relative(DIR, abs_path);
    return `![${alt}](../${rel_to_root})`;
  });
};

const collectLinks = (content, base_dir) => {
  const link_re = /\[((?:\[[^\]]*\]|[^\]])+)\]\(([^)]+\.md)(#[^)]*)?\)/g,
    links = [];
  let match;
  while ((match = link_re.exec(content)) !== null) {
    const path = match[2];
    if (!path.startsWith("http")) {
      // Normalize path to absolute and resolve relative dots
      const abs_path = resolve(base_dir, path.split("#")[0]);
      if (!links.includes(abs_path)) {
        links.push(abs_path);
      }
    }
  }
  return links;
};

const loadAndMerge = (paths, visited) => {
  let merged = "";
  for (const p of paths) {
    if (visited.has(p)) continue;
    visited.add(p);

    console.log(`  Merge: ${p}`);
    let content_raw;
    try {
      content_raw = readFileSync(p, "utf-8");
    } catch (e) {
      console.warn(`  Warning: Could not read file ${p}, skipping.`);
      continue;
    }

    const base = dirname(p),
      nested_links = collectLinks(content_raw, base);

    let content = fixLinks(content_raw, base);
    content = fixImages(content, base);

    const id = pathToId(p);
    merged += `\n\n---\n\n<div id="${id}"></div>\n\n${content}`;

    // Process nested links
    if (nested_links.length > 0) {
      merged += loadAndMerge(nested_links, visited);
    }
  }
  return merged;
};

const processLang = async (lang) => {
  console.log(`\n正在处理语言: ${lang}...`);

  const intro_path = resolve(DIR, "readme", `${lang}.md`),
    toc_path = resolve(DIR, "readme", `${lang}.toc.md`),
    output_json_path = join(VITE_DATA_DIR, `${lang}.json`),
    intro_raw = readFileSync(intro_path, "utf-8"),
    toc_raw = readFileSync(toc_path, "utf-8"),
    toc_links = collectLinks(toc_raw, DIR); // Use DIR as base for TOC links

  // Explicitly track visited files using normalized absolute paths
  const visited = new Set([intro_path, toc_path]);

  let intro = fixLinks(intro_raw, dirname(intro_path));
  intro = fixImages(intro, dirname(intro_path));

  let toc = fixLinks(toc_raw, DIR);
  toc = fixImages(toc, dirname(toc_path));

  const embedded_docs = loadAndMerge(toc_links, visited);

  let merged = `\n<div id="${pathToId(intro_path)}"></div>\n\n${intro}\n\n---\n\n<div id="${pathToId(toc_path)}"></div>\n\n${toc}${embedded_docs}`;
  merged = merged.replace(/\$ (.*?) \$/g, "$$$1$$");

  console.log(`  Total files merged for ${lang}: ${visited.size}`);

  let html_body = md.render(merged);
  html_body = html_body.replace(
    /<p>\s*<div class="svg-inline-marker">([\s\S]*?)<\/div>\s*<\/p>/g,
    "$1",
  );
  html_body = html_body.replace(
    /<div class="svg-inline-marker">([\s\S]*?)<\/div>/g,
    "$1",
  );

  const title =
    lang === "zh" ? "JDB-FTL 技术文档" : "JDB-FTL Technical Documentation",
    data = {
      lang,
      title,
      body: html_body,
    };

  writeFileSync(output_json_path, JSON.stringify(data));
  console.log(
    `✓ JSON 已生成: ${output_json_path} (Size: ${Math.round(JSON.stringify(data).length / 1024)} KB)`,
  );
};

const main = async () => {
  console.log("正在生成 JSON 数据...");
  await processLang("zh");
  await processLang("en");

  console.log("\n正在构建 vite 项目 (并运行 build.js)...");
  await $({ cwd: join(DIR, "sh", "vite") })`bun build.js`;
  console.log("✓ vite/build.js 执行完成");

  console.log("\n正在生成 PDF...");
  const browser = await launch({
    headless: "new",
    args: ["--no-sandbox", "--disable-setuid-sandbox"],
  });

  for (const lang of ["zh", "en"]) {
    const html_path = join(GEN, `${lang}.html`),
      pdf_path = join(GEN, `${lang}.pdf`),
      html = readFileSync(html_path, "utf-8"),
      page = await browser.newPage();

    await page.setContent(html, { timeout: 60000 });

    // Wait for mermaid rendering to complete (signaled by window.mermaidDone in bundle.js)
    try {
      await page.waitForFunction("window.mermaidDone === true", {
        timeout: 30000,
      });
    } catch (e) {
      console.warn("Wait for mermaidDone timed out, continuing...");
    }

    try {
      await page.waitForNetworkIdle({ timeout: 5000 });
    } catch {
      console.warn("网络闲置等待超时，继续处理...");
    }

    await page.pdf({
      path: pdf_path,
      format: "A4",
      margin: { top: "2cm", right: "2cm", bottom: "2cm", left: "2cm" },
      printBackground: true,
      timeout: 120000,
    });
    await page.close();
    console.log(`✓ PDF 已生成: ${pdf_path}`);
  }

  await browser.close();
  console.log("\n全部任务完成！");
  process.exit(0);
};

main();
export default main;
