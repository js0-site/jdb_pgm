#!/usr/bin/env bun

import { i18nImport, benchJsonLi, readmeWrite } from "./conf.js";
import { formatTime } from "./lib/json.js";

const gen = async () => {
  const I18N = await i18nImport(import.meta);
  let data;
  try {
    data = benchJsonLi();
  } catch (e) {
    return;
  }

  const parsed = {};
  for (const item of data) {
    if (item.reason !== "benchmark-complete") continue;
    const parts = item.id.split("/");
    if (parts.length !== 3) continue;
    const [dataset, lib, metric] = parts;
    if (!parsed[dataset]) parsed[dataset] = {};
    if (!parsed[dataset][metric]) parsed[dataset][metric] = {};
    parsed[dataset][metric][lib] = item.mean.estimate;
  }

  const render = (I18N) => {
    let svgContent = "";
    // Simplified SVG generation logic
    // For each dataset and metric, draw a bar chart

    // This is a placeholder for actual SVG drawing logic
    // In a real implementation, we would calculate coordinates and draw <rect> elements
    // For now, we will output a simple text listing in SVG for demonstration

    let y = 20;
    const fh = 20;

    svgContent += `<svg xmlns="http://www.w3.org/2000/svg" width="800" height="1000" font-family="monospace">`;
    svgContent += `<style>text { font-size: 14px; }</style>`;

    for (const dataset of Object.keys(parsed)) {
      svgContent += `<text x="10" y="${y}" font-weight="bold">${dataset}</text>`;
      y += fh;

      for (const metric of Object.keys(parsed[dataset])) {
        svgContent += `<text x="20" y="${y}">${metric}</text>`;
        y += fh;

        const libs = parsed[dataset][metric];
        const maxVal = Math.max(...Object.values(libs));

        for (const [lib, val] of Object.entries(libs)) {
          const width = (val / maxVal) * 500;
          svgContent += `<text x="30" y="${y}" alignment-baseline="middle">${lib}</text>`;
          svgContent += `<rect x="100" y="${y - 10}" width="${width}" height="10" fill="#4caf50" />`;
          svgContent += `<text x="${110 + width}" y="${y}" alignment-baseline="middle">${formatTime(val)}</text>`;
          y += 20;
        }
        y += 10;
      }
      y += 20;
    }

    svgContent += `</svg>`;
    return svgContent;
  };

  // Writing to a specific file or just example
  // User instructions didn't specify filename for SVG, but likely bench.svg or similar
  await Promise.all(readmeWrite(import.meta, "bench.svg", render));
};

export default gen;

if (import.meta.main) {
  await gen();
}
