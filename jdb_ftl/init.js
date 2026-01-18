#!/usr/bin/env bun
import { existsSync, mkdirSync, createWriteStream } from "node:fs";
import { join } from "node:path";
import { writeFile } from "node:fs/promises";
import { Readable } from "node:stream";
import { parse } from "csv-parse";

// 全局常量
const DATA_DIR = join(import.meta.dirname, "data"),
  TRACE_FILE = join(DATA_DIR, "trace_msrc.bin"),
  META_FILE = join(DATA_DIR, "meta.json"),
  MSRC_URL =
    "https://raw.githubusercontent.com/foxandxss/msr-cambridge-traces/master/src1_0.csv";

// 确保目录存在
const ensureDir = (dir) => {
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
};

// 合并生成模拟数据
const genSynthetic = async (file_path) => {
  const write_stream = createWriteStream(file_path),
    n = 100000,
    alpha = 1.2,
    alpha_inv = -1.0 / (alpha - 1.0);

  for (let i = 0; i < n; i++) {
    const r = Math.random(),
      lba = BigInt(Math.floor(Math.pow(r, alpha_inv) % 1000000)),
      buf = Buffer.alloc(8);
    buf.writeBigUInt64LE(lba);
    write_stream.write(buf);
  }
  write_stream.end();
  return n;
};

// 入口函数
const run = async () => {
  ensureDir(DATA_DIR);

  if (existsSync(TRACE_FILE)) {
    console.log("数据已存在");
    return;
  }

  const res = await fetch(MSRC_URL);
  let n = 0;

  if (res.ok) {
    console.log("流式下载并解析 MSRC 数据...");
    const write_stream = createWriteStream(TRACE_FILE),
      parser = Readable.fromWeb(res.body).pipe(
        parse({ columns: true, skip_empty_lines: true }),
      );

    for await (const row of parser) {
      // 解析 Offset 并转换为 LBA
      const lba = BigInt(Math.floor(parseInt(row.Offset) / 512)),
        buf = Buffer.alloc(8);
      buf.writeBigUInt64LE(lba);
      write_stream.write(buf);
      n++;
    }
    write_stream.end();
  } else {
    console.log("远程数据不可用，生成模拟数据...");
    n = await genSynthetic(TRACE_FILE);
  }

  const meta = {
    source: res.ok ? MSRC_URL : "synthetic",
    count: n,
    date: new Date().toISOString(),
  };
  await writeFile(META_FILE, JSON.stringify(meta, null, 2));
  console.log(`完成: ${n} 条记录`);
};

await run();

export default run;
