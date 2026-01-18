#!/usr/bin/env bun

/*
格式，BENCH_LI
用于展示性能测试结果，按性能优劣排序(不同字段可能顺序不一样)
以最低为基准 1
输出格式如：
指标（单位，如果性能比较高，用百万作为单位，保留2位小数）
  库1: 实测性能 ( 倍数x )
  库2: 实测性能 ( 倍数x )
  库3: 实测性能 ( 1x )
*/

import { i18nImport } from "./conf.js";
import { createTable } from "@visulima/tabular";
import { NO_BORDER } from "@visulima/tabular/style";
import CONV from "./CONV.js";

const gen = async () => {
  const I18N = await i18nImport(import.meta),
    table = createTable({
      showHeader: true,
      style: {
        paddingLeft: 0,
        border: NO_BORDER,
      },
    });
  table.setHeaders([I18N.XX, I18N.XX]);

  // Group by Data Size
  const perfByMsg = {};
  CONV.perf.forEach((r) => {
    if (!perfByMsg[r.data_size]) perfByMsg[r.data_size] = [];
    perfByMsg[r.data_size].push(r);
  });

  const sizes = Object.keys(perfByMsg).sort(
    (a, b) => parseInt(a) - parseInt(b),
  );

  for (const size of sizes) {
    const subset = perfByMsg[size].filter(
      (r) => r.throughput > 0 && r.algorithm !== "hashmap",
    );
    if (subset.length === 0) continue;

    // Sort by throughput ascending (baseline first)
    subset.sort((a, b) => a.throughput - b.throughput);

    const baseline = subset[0];
    const baselineVal = baseline.throughput;

    console.log(`\n${I18N.THROUGHPUT} (${I18N.DATA_SIZE}: ${size})`);

    // Print in descending order for display, but calc relative to baseline (min)
    const displaySet = [...subset].sort((a, b) => b.throughput - a.throughput);

    for (const item of displaySet) {
      const name = I18N.ALGORITHM_NAMES[item.algorithm] || item.algorithm;
      const label = item.epsilon ? `${name} (e=${item.epsilon})` : name;
      // Million ops per second
      const valStr = (item.throughput / 1e6).toFixed(2) + " M/s";
      const multiple = (item.throughput / baselineVal).toFixed(1) + "x";

      console.log(`  ${label.padEnd(30)}: ${valStr.padEnd(15)} (${multiple})`);
    }
  }
};

export default gen;

if (import.meta.main) {
  await gen();
}
