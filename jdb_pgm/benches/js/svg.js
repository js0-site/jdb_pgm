#!/usr/bin/env bun

// SVG chart generator for Pgm-Index benchmark (no external chart library)
// Pgm-Index 基准测试 SVG 图表生成器（不依赖外部图表库）

import CONV from "./CONV.js";
import { DIR_README, i18nLi } from "./conf.js";
import { formatDataSize, formatMemory } from "./lib/fmt.js";
import { ALGORITHM_COLORS, getColor } from "./lib/style.js";
import write from "@3-/write";
import { join, basename } from "node:path";

// Chart dimensions / 图表尺寸
const W = 520;
const CHART_H = 180;
const M = { t: 90, r: 30, b: 90, l: 70 };
const BAR_W = 50;
const BAR_GAP = 20;

// Escape XML special characters / 转义 XML 特殊字符
const escXml = (s) =>
  String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

// Generate Y-axis with grid lines / 生成 Y 轴和网格线
const genYAxis = (baseY, h, maxVal, labelX = 60) => {
  const ticks = 4;
  let svg = "";
  for (let i = 0; i <= ticks; i++) {
    const y = baseY + h - (i / ticks) * h;
    const val = ((i / ticks) * maxVal).toFixed(1);
    svg += `<path stroke="#e5e7eb" d="M${M.l} ${y}h${W - M.l - M.r}"/>`;
    svg += `<text x="${labelX}" y="${y + 4}" fill="#9ca3af" font-size="11" text-anchor="end">${val}</text>`;
  }
  return svg;
};

// Generate vertical axis label / 生成垂直轴标签
const genYLabel = (baseY, h, label) => {
  const cy = baseY + h / 2;
  return `<text x="18" y="${cy}" fill="#6b7280" font-size="12" font-weight="500" text-anchor="middle" transform="rotate(-90 18 ${cy})">${escXml(label)}</text>`;
};

// Generate bar with value label / 生成带数值标签的柱状图
const genBar = (x, baseY, h, val, maxVal, color, label, subLabel = null) => {
  const barH = maxVal > 0 ? (val / maxVal) * h : 0;
  const y = baseY + h - barH;
  let svg = "";
  if (barH > 0) {
    svg += `<rect fill="${color}" x="${x}" y="${y}" width="${BAR_W}" height="${barH}"/>`;
  }
  svg += `<text x="${x + BAR_W / 2}" y="${y - 6}" fill="#374151" font-size="12" font-weight="600" text-anchor="middle">${val.toFixed(2)}</text>`;
  svg += `<text x="${x + BAR_W / 2}" y="${baseY + h + 16}" fill="#4b5563" font-size="11" font-weight="500" text-anchor="end" transform="rotate(-40 ${x + BAR_W / 2} ${baseY + h + 16})">${escXml(label)}</text>`;
  if (subLabel) {
    svg += `<text x="${x + BAR_W / 2}" y="${baseY + h + 32}" fill="#9ca3af" font-size="10" text-anchor="end" transform="rotate(-40 ${x + BAR_W / 2} ${baseY + h + 32})">${escXml(subLabel)}</text>`;
  }
  return svg;
};

// Generate star for best performer / 为最佳性能添加红色五角星
const genStar = (x, y) => {
  const size = 12;
  return `<path fill="#ef4444" d="M${x} ${y - size - 5}l${size * 0.22} ${size * 0.7}h${size * 0.7}l-${size * 0.57} ${size * 0.42}l${size * 0.22} ${size * 0.7}l-${size * 0.57} -${size * 0.42}l-${size * 0.57} ${size * 0.42}l${size * 0.22} -${size * 0.7}l-${size * 0.57} -${size * 0.42}h${size * 0.7}z"/>`;
};

// Generate legend / 生成图例
const genLegend = (y, names) => {
  const items = [
    { key: "jdb_pgm", color: ALGORITHM_COLORS.jdb_pgm },
    { key: "pgm_index", color: ALGORITHM_COLORS.external_pgm },
    { key: "binary_search", color: ALGORITHM_COLORS.binary_search },
    { key: "btreemap", color: ALGORITHM_COLORS.btreemap },
  ];
  let svg = "";
  const cols = 2;
  const colW = 140;
  const startX = (W - cols * colW) / 2;
  items.forEach((item, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    const x = startX + col * colW;
    const ly = y + row * 24;
    svg += `<rect fill="${item.color}" x="${x}" y="${ly}" width="14" height="14" rx="2"/>`;
    svg += `<text x="${x + 20}" y="${ly + 11}" fill="#6b7280" font-size="12">${escXml(names[item.key] || item.key)}</text>`;
  });
  return svg;
};

// Generate chart title
const genChartTitle = (x, y, text) => {
  return `<text x="${x}" y="${y}" fill="#111827" font-size="16" font-weight="700" text-anchor="middle">${escXml(text)}</text>`;
};

// Deduplicate helper - keeps first occurrence after sorting
const dedupe = (list) => {
  const seen = new Set();
  return list.filter((r) => {
    const key = `${r.algorithm}_${r.epsilon}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
};

// Generate full SVG / 生成完整 SVG
const genSvg = (I18N, lang) => {
  const names = I18N.ALGORITHM_NAMES;
  const dataSize = 1000000;
  const targetEpsilon = 64;

  // Filter for Epsilon 64 (or undefined for non-eps algos)
  const isTarget = (r) =>
    r.data_size === dataSize &&
    (r.epsilon === targetEpsilon || r.epsilon === undefined);

  let perfData = CONV.perf
    .filter(isTarget)
    .filter((r) => r.algorithm !== "hashmap" && r.throughput > 0);

  // Sort by throughput DESC so dedupe picks the best one
  perfData.sort((a, b) => b.throughput - a.throughput);
  perfData = dedupe(perfData);
  // Keep sorted by throughput for display
  perfData.sort((a, b) => b.throughput - a.throughput);

  // Error chart: ONLY JdbPgm and PgmIndex
  let accuracyData = CONV.accuracy
    .filter(isTarget)
    .filter((r) => r.algorithm === "jdb_pgm" || r.algorithm === "external_pgm");
  accuracyData.sort((a, b) => (a.avg_error || 0) - (b.avg_error || 0)); // Sort by error ASC (best first)
  accuracyData = dedupe(accuracyData);

  const title = lang === "en" ? "Pgm-Index Benchmark" : "Pgm 索引评测";
  const subtitle =
    lang === "en"
      ? `ε=${targetEpsilon}, ${formatDataSize(dataSize)} elements`
      : `ε=${targetEpsilon}, ${formatDataSize(dataSize)} 元素`;

  const chartSpacing = 130;
  const hasAccuracy = accuracyData.length > 0;
  const totalH =
    M.t +
    CHART_H +
    chartSpacing +
    CHART_H +
    (hasAccuracy ? chartSpacing + CHART_H : 0) +
    M.b +
    60;

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${totalH}" viewBox="0 0 ${W} ${totalH}">`;
  svg += `<rect fill="#f9fafb" width="${W}" height="${totalH}"/>`;
  svg += `<rect fill="#fff" x="10" y="10" width="${W - 20}" height="${totalH - 20}" rx="12" filter="drop-shadow(0 2px 4px rgba(0,0,0,0.05))"/>`;

  // Header
  svg += `<text x="${W / 2}" y="50" fill="#111827" font-size="22" font-weight="800" text-anchor="middle">${escXml(title)}</text>`;
  svg += `<text x="${W / 2}" y="75" fill="#6b7280" font-size="13" font-weight="500" text-anchor="middle">${escXml(subtitle)}</text>`;

  let currentY = M.t + 30;

  // 1. Throughput Chart (sorted by throughput DESC)
  svg += genChartTitle(
    W / 2,
    currentY - 20,
    lang === "en" ? "Throughput (M/s)" : "吞吐量 (M/s)",
  );
  const maxThroughput =
    Math.max(...perfData.map((r) => r.throughput / 1e6), 1) * 1.12;
  svg += genYAxis(currentY, CHART_H, maxThroughput);
  svg += genYLabel(currentY, CHART_H, "M/s");

  let x =
    M.l +
    (W -
      M.l -
      M.r -
      (perfData.length * BAR_W + (perfData.length - 1) * BAR_GAP)) /
      2;

  perfData.forEach((r, i) => {
    const val = r.throughput / 1e6;
    svg += genBar(
      x,
      currentY,
      CHART_H,
      val,
      maxThroughput,
      getColor(r.algorithm),
      names[r.algorithm] || r.algorithm,
      r.epsilon ? `ε=${r.epsilon}` : null,
    );
    if (i === 0)
      svg += genStar(
        x + BAR_W / 2,
        currentY + CHART_H - (val / maxThroughput) * CHART_H - 24,
      );
    x += BAR_W + BAR_GAP;
  });

  currentY += CHART_H + chartSpacing;

  // 2. Memory Chart (sorted by memory ASC - lower is better)
  let memData = [...perfData].sort(
    (a, b) => (a.memory_bytes || 0) - (b.memory_bytes || 0),
  );
  svg += genChartTitle(
    W / 2,
    currentY - 20,
    lang === "en" ? "Memory (MB)" : "内存 (MB)",
  );
  const maxMem =
    Math.max(...memData.map((r) => (r.memory_bytes || 0) / (1024 * 1024)), 1) *
    1.12;
  svg += genYAxis(currentY, CHART_H, maxMem);
  svg += genYLabel(currentY, CHART_H, "MB");

  x =
    M.l +
    (W -
      M.l -
      M.r -
      (memData.length * BAR_W + (memData.length - 1) * BAR_GAP)) /
      2;
  memData.forEach((r, i) => {
    const val = (r.memory_bytes || 0) / (1024 * 1024);
    svg += genBar(
      x,
      currentY,
      CHART_H,
      val,
      maxMem,
      getColor(r.algorithm),
      names[r.algorithm] || r.algorithm,
      r.epsilon ? `ε=${r.epsilon}` : null,
    );
    if (i === 0)
      svg += genStar(
        x + BAR_W / 2,
        currentY + CHART_H - (val / maxMem) * CHART_H - 24,
      );
    x += BAR_W + BAR_GAP;
  });

  if (hasAccuracy) {
    currentY += CHART_H + chartSpacing;

    // 3. Average Error Chart (sorted by error ASC - lower is better)
    svg += genChartTitle(
      W / 2,
      currentY - 20,
      lang === "en" ? "Avg Error" : "平均误差",
    );
    const maxErr =
      Math.max(...accuracyData.map((r) => r.avg_error || 0), 1) * 1.12;
    svg += genYAxis(currentY, CHART_H, maxErr);
    svg += genYLabel(currentY, CHART_H, lang === "en" ? "Error" : "误差");

    x =
      M.l +
      (W -
        M.l -
        M.r -
        (accuracyData.length * BAR_W + (accuracyData.length - 1) * BAR_GAP)) /
        2;
    accuracyData.forEach((r, i) => {
      const val = r.avg_error || 0;
      svg += genBar(
        x,
        currentY,
        CHART_H,
        val,
        maxErr,
        getColor(r.algorithm),
        names[r.algorithm] || r.algorithm,
        r.epsilon ? `ε=${r.epsilon}` : null,
      );
      if (i === 0)
        svg += genStar(
          x + BAR_W / 2,
          currentY + CHART_H - (val / maxErr) * CHART_H - 24,
        );
      x += BAR_W + BAR_GAP;
    });
  }

  // Legend at bottom
  const legendY = currentY + CHART_H + 70;
  svg += genLegend(legendY, names);

  svg += "</svg>";
  return svg;
};

const gen = async () => {
  const langs = i18nLi("svg.js");
  for (const [lang, fp] of langs) {
    const I18N = await import(fp);
    const svgStr = genSvg(I18N, lang);
    const outPath = join(DIR_README, lang, "bench.svg");
    write(outPath, svgStr);
    console.log(`Written: ${outPath}`);
  }
};

export default gen;

if (import.meta.main) {
  await gen();
}
