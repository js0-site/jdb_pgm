#! /usr/bin/env bun

import conv from "./lib/conv.js";
import { join } from "node:path";
import { readFileSync } from "node:fs";
import { parseArgs } from "node:util";
import { i18nImport } from "./conf.js";
import { $ } from "zx";

const gen = async (kind) => {
  const I18N = await i18nImport(import.meta),
    reports_dir = join(import.meta.dirname, "../reports"),
    report_path = join(reports_dir, `${kind}_all.json`);

  $.verbose = false;
  console.log(`Running benchmark (Base + Ftl) for ${kind}...`);
  await $`mkdir -p ${reports_dir}`;
  await $`RUSTFALG="target-cpu=native" cargo bench --bench main --features bench_all -q > ${report_path}`;

  let json_li = readFileSync(report_path, "utf-8")
    .trim()
    .split("\n")
    .filter((line) => line.trim().startsWith("{"))
    .map((line) => JSON.parse(line));

  const { rows, summary, ops } = conv(json_li);

  console.log(`# ${I18N.TITLE || "Performance Baseline"} [${kind}]\n`);

  const ftl_name = "Ftl";
  const base_name = "[u8]";

  if (summary[base_name] && summary[ftl_name]) {
    const base_time = summary[base_name].total_time_ms;
    const ftl_time = summary[ftl_name].total_time_ms;
    console.log(`总耗时 (ms)`);
    console.log(`  基准: ${base_time}`);
    console.log(`  库: ${ftl_time} (${(ftl_time / base_time).toFixed(2)}x)\n`);
  }

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

  const label_map = {
    get: I18N.GET || "GET",
    set: I18N.SET || "SET",
    memory: I18N.MEMORY || "Memory",
  };

  for (const row of rows) {
    const ftl = row.ftl;
    const base = row.base;
    const metric_label = label_map[row.metric] || row.metric;

    if (row.metric === "memory") {
      console.log(`${metric_label} (MB)`);
      console.log(`  基准: ${base.toFixed(2)}`);
      console.log(
        `  库: ${ftl.toFixed(2)} (${((ftl / base) * 100).toFixed(2)}%)\n`,
      );

      console.log(`压缩率 (%)`);
      console.log(`  基准: 100.00%`);
      const ratio = base > 0 ? (ftl / base) * 100 : 0;
      const diff = ratio - 100;
      console.log(
        `  库: ${ratio.toFixed(8)}% (${diff >= 0 ? "+" : ""}${diff.toFixed(8)}%)\n`,
      );
    } else {
      const ITEM_SIZE_BYTES = 8;
      const ftl_mb = (ftl * ITEM_SIZE_BYTES) / (1024 * 1024);
      const base_mb = (base * ITEM_SIZE_BYTES) / (1024 * 1024);

      console.log(`${metric_label} (MB/s)`);
      console.log(`  基准: ${base_mb.toFixed(2)}`);
      console.log(`  库: ${ftl_mb.toFixed(2)} (${(ftl / base).toFixed(2)}x)`);

      const p99_ftl = row.p99_ftl || 0;
      const p99_base = row.p99_base || 0;
      console.log(
        `  P99延时: ${p99_ftl.toFixed(2)}ns (基准: ${p99_base.toFixed(2)}ns, ${((p99_ftl / p99_base) * 100).toFixed(2)}%)\n`,
      );
    }
  }
};

if (import.meta.main) {
  const { values } = parseArgs({
    options: { kind: { type: "string", short: "k" } },
  });
  await gen(values.kind || process.env.BIN || "quick");
}

export default gen;
