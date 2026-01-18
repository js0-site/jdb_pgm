#!/usr/bin/env bun

/*
生成性能测试 markdown 图表，以方便嵌入 readme
*/

import md from "./lib/md.js";
import CONV from "./CONV.js";
import {
  formatDataSize,
  fmtTime,
  fmtThroughput,
  formatMemory,
} from "./lib/fmt.js";

// Helper to generate a markdown table string using custom padding
const generateMarkdownTable = (headers, rows) => {
  // 1. Calculate max width for each column
  const colWidths = headers.map((h) => String(h).length);
  rows.forEach((row) => {
    row.forEach((cell, i) => {
      const len = String(cell).length;
      if (len > colWidths[i]) {
        colWidths[i] = len;
      }
    });
  });

  // 2. Helper to pad cell
  const pad = (str, len) => String(str).padEnd(len, " ");

  // 3. Build table
  const buildRow = (r) =>
    "| " + r.map((c, i) => pad(c, colWidths[i])).join(" | ") + " |";
  const separator =
    "| " + colWidths.map((w) => "-".repeat(w)).join(" | ") + " |";

  const lines = [buildRow(headers), separator, ...rows.map(buildRow)];

  return lines.join("\n");
};

const gen = async () =>
  await md((I18N) => {
    // 1. Perf Tables
    // Sort all perf entries by Data Size -> Throughput
    const allPerf = CONV.perf.filter(
      (r) => r.throughput > 0 && r.algorithm !== "hashmap",
    );
    allPerf.sort((a, b) => {
      const da = parseInt(a.data_size);
      const db = parseInt(b.data_size);
      if (da !== db) return da - db;
      // Then by throughput descending
      return b.throughput - a.throughput;
    });

    const headers = [
      I18N.ALGORITHM,
      I18N.EPSILON,
      I18N.DATA_SIZE,
      I18N.MEMORY_MB,
      `${I18N.THROUGHPUT} (M/s)`,
    ];

    const rows = allPerf.map((r) => [
      I18N.ALGORITHM_NAMES[r.algorithm] || r.algorithm,
      r.epsilon || "-",
      formatDataSize(r.data_size),
      formatMemory(r.memory_bytes),
      (r.throughput / 1e6).toFixed(2), // Raw M/s value
    ]);

    const perf_tables_md = generateMarkdownTable(headers, rows);

    // 2. Build Time Table
    let build_time_table = "";
    if (CONV.build.length > 0) {
      // Simplify for now: just list
      build_time_table = generateMarkdownTable(
        [I18N.DATA_SIZE, I18N.ALGORITHM, I18N.EPSILON, I18N.MEAN_TIME],
        CONV.build.map((r) => [
          formatDataSize(r.data_size),
          I18N.ALGORITHM_NAMES[r.algorithm] || r.algorithm,
          r.epsilon,
          fmtTime(r.mean_ns),
        ]),
      );
    } else {
      build_time_table = "*No build time data available*";
    }

    // 3. Accuracy Table
    let accuracy_table = "";
    if (CONV.accuracy.length > 0) {
      // Group by Epsilon
      const accMap = {};
      CONV.accuracy.forEach((r) => {
        if (!accMap[r.epsilon]) accMap[r.epsilon] = {};
        accMap[r.epsilon][r.algorithm] = r;
      });

      const accHeaders = [
        I18N.DATA_SIZE,
        I18N.EPSILON,
        "jdb_pgm Max",
        "jdb_pgm Avg",
        "pgm_index Max",
        "pgm_index Avg",
      ];

      const accRows = Object.entries(accMap)
        .filter(([eps]) => eps !== "undefined")
        .sort((a, b) => parseInt(a[0]) - parseInt(b[0]))
        .map(([eps, libs]) => [
          formatDataSize(libs.jdb_pgm?.data_size || 1000000),
          eps,
          libs.jdb_pgm?.max_error ?? "-",
          libs.jdb_pgm?.avg_error?.toFixed(2) ?? "-",
          libs.external_pgm?.max_error ?? "-",
          libs.external_pgm?.avg_error?.toFixed(2) ?? "-",
        ]);

      accuracy_table = generateMarkdownTable(accHeaders, accRows);
    } else {
      accuracy_table = "*No accuracy data available*";
    }

    // Return the object expected by the template: _.perf_tables, _.accuracy_table, etc.
    return {
      perf_tables: perf_tables_md,
      accuracy_table: accuracy_table,
      build_time_table: build_time_table,
      lang: I18N, // To access I18N.config, etc.
      config: CONV.config,
      sys: CONV.sys,
    };
  });

export default gen;

if (import.meta.main) {
  await gen();
}
