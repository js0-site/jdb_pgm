#!/usr/bin/env bun

/*
生成性能测试 markdown 图表，以方便嵌入 readme
*/

import md from "./lib/md.js";

import { cpus, totalmem, platform, release, arch } from "node:os";

const main = async () => {
  const { default: CONV } = await import("./CONV.js");
  const proc = Bun.spawn(["rustc", "--version"]);
  const rustVer = (await new Response(proc.stdout).text()).trim();

  const keyStats = await Bun.file(
    import.meta.dirname + "/svg/json/key_stats.json",
  ).json();


  await md((I18N) => {
    const { rows, summary } = CONV;

    // Headers
    const headers = [
      I18N.METRIC || "Metric",
      I18N.BASELINE || "Baseline",
      "JDB-FTL",
      I18N.DIFF || "Diff",
      I18N.NOTE || "Note",
    ];

    const tableRows = [];

    // 1. Total Time
    const baseSum = summary["[u8]"];
    const ftlSum = summary["Ftl"];
    if (baseSum && ftlSum) {
      const baseT = baseSum.total_time_ms;
      const ftlT = ftlSum.total_time_ms;
      const diffStr = `${(ftlT / baseT).toFixed(2)}x`;
      tableRows.push([
        I18N.TOTAL_TIME || "Total Time",
        `${baseT} ms`,
        `${ftlT} ms`,
        diffStr,
        "",
      ]);
    }

    const label_map = {
      get: I18N.GET || "GET",
      set: I18N.SET || "SET",
      memory: I18N.MEMORY || "Memory",
    };

    for (const row of rows) {
      const { metric, base, ftl } = row;
      const baseLabel = label_map[metric] || metric;

      if (metric === "memory") {
        const baseStr = `${base.toFixed(2)} MB`;
        const ftlStr = `${ftl.toFixed(2)} MB`;
        const ratio = (ftl / base) * 100;
        // Compression Ratio (Space Saving)
        // Ratio 7.30% means we saved 92.7%
        // Diff column: show the Ratio % itself (e.g. 7.30%)
        // Note column: show reduction (e.g. -92.70%)
        const diffStr = `${ratio.toFixed(2)}%`;
        const reduction = ratio - 100;
        const noteStr = `${reduction.toFixed(2)}%`;

        tableRows.push([
          I18N.MEMORY || "Memory",
          baseStr,
          ftlStr,
          diffStr,
          noteStr,
        ]);
      } else {
        // Throughput Row
        const ITEM_SIZE_BYTES = 8;
        const base_mb = (base * ITEM_SIZE_BYTES) / (1024 * 1024);
        const ftl_mb = (ftl * ITEM_SIZE_BYTES) / (1024 * 1024);

        const baseTp = `${base_mb.toFixed(2)} MB/s`;
        const ftlTp = `${ftl_mb.toFixed(2)} MB/s`;

        const ratioTp = ftl / base;
        const diffTp = (ratioTp - 1) * 100;
        const diffTpStr = `${diffTp > 0 ? "+" : ""}${diffTp.toFixed(1)}%`;

        tableRows.push([
          `${baseLabel} (${I18N.THROUGHPUT || "MB/s"})`,
          baseTp,
          ftlTp,
          diffTpStr,
          "",
        ]);

        // Latency Row
        const p99_base = row.p99_base || 0;
        const p99_ftl = row.p99_ftl || 0;

        const baseLat = `${p99_base} ns`;
        const ftlLat = `${p99_ftl} ns`;

        // Avoid division by zero
        let diffLatStr = "-";
        if (p99_base > 0) {
          const ratioLat = p99_ftl / p99_base;
          const diffLat = (ratioLat - 1) * 100;
          diffLatStr = `${diffLat > 0 ? "+" : ""}${diffLat.toFixed(1)}%`;
        }

        tableRows.push([
          `${baseLabel} (${I18N.LATENCY_P99 || "P99"})`,
          baseLat,
          ftlLat,
          diffLatStr,
          "",
        ]);
      }
    }

    // Generate Standard Markdown Table
    const colAligns = [":---", "---:", "---:", "---:", ":---"]; // Alignments
    const headerLine = `| ${headers.join(" | ")} |`;
    const separatorLine = `| ${colAligns.join(" | ")} |`;
    const bodyLines = tableRows
      .map((row) => `| ${row.join(" | ")} |`)
      .join("\n");

    const tableStr = `${headerLine}\n${separatorLine}\n${bodyLines}`;

    // Collect Environment Info
    const cpuList = cpus();
    const cpuModel =
      cpuList.length > 0 ? cpuList[0].model : I18N.UNKNOWN || "Unknown";
    const memGB = (totalmem() / 1024 / 1024 / 1024).toFixed(1);
    const osStr = `${platform()} ${release()} (${arch()})`;

    const env = [
      `- **${I18N.OS || "OS"}**: ${osStr}`,
      `- **${I18N.CPU || "CPU"}**: ${cpuModel} x ${cpuList.length}`,
      `- **${I18N.MEMORY || "Memory"}**: ${memGB} GB`,
      `- **${I18N.RUSTC || "Rust Version"}**: ${rustVer}`,
    ].join("\n");

    // Compute derived statistics for templates
    const memRow = rows.find((r) => r.metric === "memory");
    const compressionRatio = memRow ? (memRow.ftl / memRow.base) * 100 : 0;
    const memEstimate16TB = ((32 * compressionRatio) / 100).toFixed(1); // 32GB baseline for 16TB

    const overheadRatio =
      baseSum && ftlSum
        ? (ftlSum.total_time_ms / baseSum.total_time_ms - 1) * 100
        : 0;

    // GET throughput drop
    const getRow = rows.find((r) => r.metric === "get");
    const getThroughputDrop = getRow ? (1 - getRow.ftl / getRow.base) * 100 : 0;

    // SET throughput drop
    const setRow = rows.find((r) => r.metric === "set");
    const setThroughputDrop = setRow ? (1 - setRow.ftl / setRow.base) * 100 : 0;

    // GET P99 latency
    const p99GetNs = getRow?.p99_ftl || 0;
    const p99GetUs = (p99GetNs / 1000).toFixed(3); // ns → μs

    // HashMap Comparison Calculation
    const uniqueKeys = keyStats.unique_keys;
    // Rust HashMap (hashbrown) overhead: bucket_count * (16 bytes data + 1 byte control)
    // bucket_count is next power of 2 of (n / 0.875)
    const bucketCount = 1 << Math.ceil(Math.log2(uniqueKeys / 0.875));
    const hashMapMemBytes = bucketCount * 17;
    const hashMapMemMB = hashMapMemBytes / (1024 * 1024);

    const jdbMemMB = memRow ? memRow.ftl : 0;
    const hashMapCompressionRatio =
      jdbMemMB > 0 ? (jdbMemMB / hashMapMemMB) * 100 : 0;

    const stats = {
      compressionRatio: compressionRatio.toFixed(2), // e.g. "3.05"
      memEstimate16TB, // e.g. "1.0" GB
      overheadRatio: overheadRatio.toFixed(0), // e.g. "12"
      minThroughputDrop: Math.min(getThroughputDrop, setThroughputDrop).toFixed(
        0,
      ),
      maxThroughputDrop: Math.max(getThroughputDrop, setThroughputDrop).toFixed(
        0,
      ),
      p99GetNs, // e.g. 84
      p99GetUs, // e.g. "0.084"

      // HashMap comparison stats
      uniqueKeys: uniqueKeys.toLocaleString(),
      keyLocality: keyStats.locality.toFixed(2),
      hashMapMemMB: hashMapMemMB.toFixed(2),
      hashMapCompressionRatio: hashMapCompressionRatio.toFixed(2),
      ftlMemMB: jdbMemMB.toFixed(2),
    };

    return {
      table: tableStr,
      env,
      stats,
    };
  });
};

if (import.meta.main) {
  await main();
}

export default main;
