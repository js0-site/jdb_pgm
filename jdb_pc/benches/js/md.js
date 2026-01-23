#!/usr/bin/env bun

import { i18nImport, benchJsonLi, readmeWrite } from "./conf.js";
import { formatTime } from "./lib/json.js";
import { markdownTable } from "markdown-table";

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
    const lines = [];
    const libs = ["Pc", "Sucds", "Array"];
    const metrics = ["Build", "RandomGet", "Iter"];

    for (const dataset of Object.keys(parsed)) {
      lines.push(`### ${dataset}`);
      const headers = [I18N.METRIC, "Lib", I18N.TIME, "Ratio"];
      const rows = [];

      for (const metric of metrics) {
        if (!parsed[dataset][metric]) continue;

        let minTime = Infinity;
        for (const lib of libs) {
          if (parsed[dataset][metric][lib]) {
            minTime = Math.min(minTime, parsed[dataset][metric][lib]);
          }
        }

        const metricName = I18N[metric.toUpperCase()] || metric;
        let first = true;
        for (const lib of libs) {
          const val = parsed[dataset][metric][lib];
          if (!val) continue;

          const ratio = (val / minTime).toFixed(2) + "x";
          const timeStr = formatTime(val);
          const libName = I18N[lib.toUpperCase()] || lib;

          rows.push([first ? metricName : "", libName, timeStr, ratio]);
          first = false;
        }
      }
      if (rows.length > 0) {
        lines.push(markdownTable([headers, ...rows]));
        lines.push("");
      }
    }
    return lines.join("\n");
  };

  await Promise.all(readmeWrite(import.meta, "bench.md", render));
};

export default gen;

if (import.meta.main) {
  await gen();
}
