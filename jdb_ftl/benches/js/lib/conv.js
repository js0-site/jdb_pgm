#!/usr/bin/env bun

/*
  解析转换 ./benches/main.rs 生成的 json 为更加方便的格式，给上游使用
*/

export default (json_li) => {
  const data = {};
  const memory = {};
  const summary = {};

  for (const row of json_li) {
    if (row.type === "memory_usage") {
      memory[row.name] = row.mem_mb;
      continue;
    }

    if (row.type === "replay_summary") {
      summary[row.name] = row;
      continue;
    }

    if (row.type === "op_stat") {
      const { name, op, avg_ns, p99_ns, ops_per_sec, count_pct, time_pct } =
        row;
      const func = op; // get, set, rm
      const impl = name; // Ftl, Base

      if (!data[func]) {
        data[func] = {};
      }

      data[func][impl] = {
        avg_ns,
        p99_ns,
        ops_per_sec,
        count_pct,
        time_pct,
      };
    }
  }

  // 映射到统一的指标列表
  const ops_list = ["get", "set"];
  const rows = [];
  const ops_detail = {};

  for (const op of ops_list) {
    const impls = Object.keys(data[op] || {});
    const ftlImpl = impls.find((name) => name !== "[u8]");
    const baseImpl = "[u8]";

    const targetImpl = ftlImpl || baseImpl;
    if (data[op]?.[targetImpl]) {
      ops_detail[op] = {
        count:
          json_li.find(
            (r) => r.type === "op_stat" && r.name === targetImpl && r.op === op,
          )?.count || 0,
        count_pct: data[op][targetImpl].count_pct,
      };
    }

    rows.push({
      metric: op,
      base: data[op]?.[baseImpl]?.ops_per_sec || 0,
      ftl: ftlImpl ? data[op][ftlImpl].ops_per_sec : 0,
      unit: "ops/s",
      p99_base: data[op]?.[baseImpl]?.p99_ns || 0,
      p99_ftl: ftlImpl ? data[op][ftlImpl].p99_ns : 0,
      count_pct: ftlImpl ? data[op][ftlImpl].count_pct : 0,
      time_pct: ftlImpl ? data[op][ftlImpl].time_pct : 0,
    });
  }

  if (Object.keys(memory).length > 0) {
    const impls = Object.keys(memory);
    const ftlImpl = impls.find((name) => name !== "[u8]");
    rows.push({
      metric: "memory",
      base: memory["[u8]"] || 0,
      ftl: ftlImpl ? memory[ftlImpl] : 0,
      unit: "MB",
    });
  }

  return { rows, summary, ops: ops_detail };
};
