#!/usr/bin/env bun

import { i18nImport, benchJsonLi } from "./conf.js";
import { createTable } from "@visulima/tabular";
import { NO_BORDER } from "@visulima/tabular/style";

// Number of elements in dataset (1000 MiB of u64s)
const N_ELEMENTS = 131_072_000;
// Number of random queries (from common.rs)
const N_QUERIES = 5_000_000;

const gen = async () => {
  const I18N = await i18nImport(import.meta);

  let data;
  try {
    data = benchJsonLi();
  } catch (e) {
    console.error("No benchmark data found. Run bench.js first.");
    return;
  }

  const parsed = {}; // Dataset -> Metric -> Library -> Value (mean ns)

  for (const item of data) {
    if (item.reason !== "benchmark-complete" && item.reason !== "custom-metric")
      continue;

    // id format: "Dataset/Library/Metric"
    const parts = item.id.split("/");
    if (parts.length !== 3) continue;

    const [dataset, lib, metric] = parts;

    if (!parsed[dataset]) parsed[dataset] = {};
    if (!parsed[dataset][metric]) parsed[dataset][metric] = {};

    if (item.reason === "custom-metric") {
      parsed[dataset][metric][lib] = item.estimate;
    } else {
      parsed[dataset][metric][lib] = item.mean.estimate;
    }
  }

  const table = createTable({
    showHeader: true,
    style: {
      paddingLeft: 0,
      border: NO_BORDER,
    },
  });

  table.setHeaders([I18N.METRIC, "Lib", "Value", "Ratio"]);

  const libs = ["Pc", "Sucds", "Array"];
  // Map internal lib name to display name
  const libDisplay = { Pc: "Pc", Sucds: "EF", Array: "Array" };
  const baselineLib = "Sucds";

  const metrics = [
    { key: "Size", zh: "占用 (MB)", en: "Size (MB)", higherBetter: false },
    { key: "Ratio", zh: "压缩率 (%)", en: "Ratio (%)", higherBetter: false },
    {
      key: "CompRatio",
      zh: "压缩比 (x)",
      en: "Comp Ratio (x)",
      higherBetter: true,
    },
    { key: "Build", zh: "构建 (MB/s)", en: "Build (MB/s)", higherBetter: true },
    {
      key: "RandomGet",
      zh: "随机 (MB/s)",
      en: "Get (MB/s)",
      higherBetter: true,
    },
    { key: "Iter", zh: "顺序 (MB/s)", en: "Iter (MB/s)", higherBetter: true },
    { key: "RevIter", zh: "迭代 (MB/s)", en: "Rev (MB/s)", higherBetter: true },
    { key: "Latency", zh: "延迟 (ns)", en: "Lat (ns)", higherBetter: false },
  ];

  for (const dataset of Object.keys(parsed)) {
    table.addRow([`--- ${dataset} ---`, "", "", ""]);

    // Calculate array size in bytes: N * 8
    const arraySizeInBytes = N_ELEMENTS * 8;

    for (const mDef of metrics) {
      const metricValues = {};
      let hasData = false;

      // First pass: collect values
      for (const lib of libs) {
        let val = null;
        const raw = parsed[dataset];

        if (mDef.key === "Size") {
          val = raw["Size"]?.[lib] / (1024 * 1024);
        } else if (mDef.key === "Ratio") {
          // Compression ratio relative to raw array size
          const size = raw["Size"]?.[lib];
          if (size) val = (size / arraySizeInBytes) * 100;
        } else if (mDef.key === "CompRatio") {
          const size = raw["Size"]?.[lib];
          if (size) val = arraySizeInBytes / size;
        } else if (mDef.key === "Build") {
          const ns = raw["Build"]?.[lib];
          // Throughput: Total bytes / Time in seconds
          // arraySizeInBytes is the data volume processed
          if (ns) val = arraySizeInBytes / (ns / 1e9) / (1024 * 1024);
        } else if (mDef.key === "RandomGet") {
          const ns = raw["RandomGet"]?.[lib];
          // Throughput: (N_QUERIES * 8 bytes) / Time
          // Note: RandomGet bench runs N_QUERIES iterations of 1 get per iter?
          // Check common.rs: `for &idx in &indices { index.get(idx) }` is inside b.iter(|| ... )
          // NO, in common.rs: b.iter(|| { for ... { ... } }) -> This means ONE iteration of benchmark is N_QUERIES ops.
          // Correct calculation: (N_QUERIES * 8) / (ns_per_batch / 1e9) / MB
          // Wait, 'ns' from criterion is per-iteration time (so per batch of N_QUERIES).
          if (ns) val = (N_QUERIES * 8) / (ns / 1e9) / (1024 * 1024);
        } else if (mDef.key === "Iter" || mDef.key === "RevIter") {
          const ns = raw[mDef.key]?.[lib];
          // Throughput: (N_ELEMENTS * 8) / Time
          // In common.rs: b.iter(|| { for val in index.iter() ... })
          // So 'ns' is time to iterate ALL elements.
          if (ns) val = arraySizeInBytes / (ns / 1e9) / (1024 * 1024);
        } else if (mDef.key === "Latency") {
          const ns = raw["RandomGet"]?.[lib];
          // Avg latency per query
          if (ns) val = ns / N_QUERIES;
        }

        if (val !== null && !isNaN(val)) {
          metricValues[lib] = val;
          hasData = true;
        }
      }

      if (!hasData) continue;

      // Find baseline value (EF/Sucds)
      const baselineVal = metricValues[baselineLib];

      const metricName =
        I18N[import.meta.env?.LANG?.includes("zh") ? mDef.zh : mDef.en] ||
        mDef.zh;

      let first = true;
      for (const lib of libs) {
        if (metricValues[lib] === undefined) continue;

        const val = metricValues[lib];
        let ratioStr = "";

        // Calculate ratio based on EF baseline
        if (baselineVal && lib !== baselineLib) {
          let ratio;
          // User request: Compression Rate ratio should be inverse
          if (mDef.key === "Ratio") {
            ratio = baselineVal / val;
          } else if (mDef.higherBetter) {
            // e.g. Build MB/s: Higher is better.
            ratio = val / baselineVal;
          } else {
            // e.g. Latency: Lower is better.
            ratio = val / baselineVal;
          }
          ratioStr = ratio.toFixed(2) + "x";
        }

        const libName = libDisplay[lib] || lib;
        // Fix formatting: if value < 0.01 and not 0, show more decimals or scientific?
        // simple toFixed(2) is requested.

        table.addRow([
          first ? metricName : "",
          libName,
          val.toFixed(2),
          ratioStr,
        ]);
        first = false;
      }
    }
    table.addRow(["", "", "", ""]);
  }

  console.log(table.toString());
};

export default gen;

if (import.meta.main) {
  await gen();
}
