#! /usr/bin/env bun

import { join } from "node:path";
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { parseArgs } from "node:util";
import conv from "./lib/conv.js";
import { $ } from "zx";
import { i18nImport } from "./conf.js";

const HISTORY_FILE = join(import.meta.dirname, "../reports/perf_history.json"),
  MAX_HISTORY = 512,
  bench = async (kind) => {
    const reports_dir = join(import.meta.dirname, "../reports"),
      report_file = join(reports_dir, `${kind}.json`);

    await $`mkdir -p ${reports_dir}`;
    // Only run bench_ftl for regression
    process.env.BIN = kind;
    await $`cargo bench --bench main --features bench_ftl -q > ${report_file}`;
  },
  report = async (kind) => {
    const I18N = await i18nImport(import.meta),
      report_path = join(import.meta.dirname, "../reports", `${kind}.json`);

    let json_li = readFileSync(report_path, "utf-8")
      .trim()
      .split("\n")
      .filter((line) => line.trim().startsWith("{"))
      .map((line) => JSON.parse(line));

    const { rows, summary, ops } = conv(json_li),
      curr_metrics = {};

    const ftl_name = "Ftl";
    if (summary[ftl_name]) {
      curr_metrics.total_time = summary[ftl_name].total_time_ms;
      curr_metrics.total_ops = summary[ftl_name].total_ops;
    }

    for (const row of rows) {
      if (row.ftl >= 0) {
        curr_metrics[row.metric] = row.ftl;
        curr_metrics[`${row.metric}_p99`] = row.p99_ftl;
      }
    }

    let history = [];
    if (existsSync(HISTORY_FILE)) {
      try {
        const stored = JSON.parse(readFileSync(HISTORY_FILE, "utf-8"));
        if (Array.isArray(stored)) history = stored;
      } catch {}
    }

    const prev_record = history
      .slice()
      .reverse()
      .find((r) => r.kind === kind);
    const prev_metrics = prev_record ? prev_record.metrics : {};

    let commit = "unknown";
    try {
      const out = await $`git rev-parse --short HEAD`.quiet();
      commit = out.toString().trim();
    } catch {}

    const record = {
      kind,
      commit,
      timestamp: Date.now(),
      metrics: curr_metrics,
    };

    console.log(`# ${I18N.TITLE} [${kind}] (${commit})\n`);

    // Total Time
    if (curr_metrics.total_time !== undefined) {
      const curr = curr_metrics.total_time;
      const prev = prev_metrics.total_time;
      console.log(`总耗时 (ms)`);
      if (prev) console.log(`  上次: ${prev}`);
      console.log(`  本次: ${curr}`);
      if (prev) {
        const diff = (prev / curr - 1) * 100;
        const ratio = curr / prev;
        console.log(
          `  变动: ${diff >= 0 ? "+" : ""}${diff.toFixed(2)}% (${ratio.toFixed(2)}x)`,
        );
      }
      console.log("");
    }

    // Op breakdown (only for current)
    if (ops.get || ops.set) {
      console.log(`操作数 (${I18N.UNIT_MILLION || "百万"})`);
      if (ops.get)
        console.log(
          `  读: ${(ops.get.count / 1_000_000).toFixed(2)} (${ops.get.count_pct.toFixed(2)}%)`,
        );
      if (ops.set)
        console.log(
          `  写: ${(ops.set.count / 1_000_000).toFixed(2)} (${ops.set.count_pct.toFixed(2)}%)`,
        );
      console.log("");
    }

    let base_mem_mb = 0;
    const config_path = join(import.meta.dirname, "../../data", `${kind}.json`);
    if (existsSync(config_path)) {
      const config = JSON.parse(readFileSync(config_path, "utf-8"));
      if (config.max_lba) {
        const cap = BigInt(config.max_lba) + 1n;
        base_mem_mb = Number(cap * 8n) / 1024 / 1024;
      }
    }

    const metrics_order = [
      { key: "get", label: I18N.GET || "GET" },
      { key: "set", label: I18N.SET || "SET" },
      { key: "memory", label: I18N.MEMORY },
    ];

    if (base_mem_mb > 0) {
      metrics_order.push({ key: "ratio", label: "压缩率" });
      curr_metrics.ratio = (curr_metrics.memory / base_mem_mb) * 100;
      if (prev_metrics.memory) {
        prev_metrics.ratio = (prev_metrics.memory / base_mem_mb) * 100;
      }
    }

    for (const { key, label } of metrics_order) {
      const curr = curr_metrics[key] || 0;
      const prev = prev_metrics[key] || 0;

      if (key === "ratio") {
        console.log(`${label} (%)`);
        if (prev) console.log(`  上次: ${prev.toFixed(2)}%`);
        console.log(`  本次: ${curr.toFixed(2)}%`);
        if (prev) {
          const diff = (prev / curr - 1) * 100;
          console.log(`  变动: ${diff >= 0 ? "+" : ""}${diff.toFixed(2)}%`);
        }
        console.log("");
      } else if (key === "memory") {
        console.log(`${label} (MB)`);
        if (prev) console.log(`  上次: ${prev.toFixed(2)}`);
        console.log(`  本次: ${curr.toFixed(2)}`);
        if (prev) {
          const diff = (prev / curr - 1) * 100;
          console.log(`  变动: ${diff >= 0 ? "+" : ""}${diff.toFixed(2)}%`);
        }
        console.log("");
      } else {
        const ITEM_SIZE_BYTES = 8;
        const curr_mb = (curr * ITEM_SIZE_BYTES) / (1024 * 1024);
        const prev_mb = (prev * ITEM_SIZE_BYTES) / (1024 * 1024);

        console.log(`${label} (MB/s)`);
        if (prev) console.log(`  上次: ${prev_mb.toFixed(2)}`);
        console.log(`  本次: ${curr_mb.toFixed(2)}`);
        if (prev) {
          const diff = (curr / prev - 1) * 100;
          const ratio = curr / prev;
          console.log(
            `  变动: ${diff >= 0 ? "+" : ""}${diff.toFixed(2)}% (${ratio.toFixed(2)}x)`,
          );
        }

        const cp99 = curr_metrics[`${key}_p99`] || 0;
        const pp99 = prev_metrics[`${key}_p99`] || 0;
        console.log(`  P99延时: ${cp99.toFixed(2)}ns`);
        if (pp99) {
          const diff = (pp99 / cp99 - 1) * 100;
          console.log(
            `  变动: ${diff >= 0 ? "+" : ""}${diff.toFixed(2)}% (上次: ${pp99.toFixed(2)}ns)`,
          );
        }
        console.log("");
      }
    }

    history.push(record);
    if (history.length > MAX_HISTORY)
      history = history.slice(history.length - MAX_HISTORY);
    writeFileSync(HISTORY_FILE, JSON.stringify(history, null, 2));
  },
  run = async (kind) => {
    $.verbose = false;
    console.log(`Running regression for ${kind}...`);
    await bench(kind);
    await report(kind);
  };

if (import.meta.main) {
  const { values } = parseArgs({
    options: { kind: { type: "string", short: "k" } },
  });
  await run(values.kind || process.env.BIN || "quick");
}

export default run;
