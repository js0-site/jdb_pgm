#!/usr/bin/env bun
import {
  unlinkSync,
  copyFileSync,
  openSync,
  readSync,
  writeSync,
  closeSync,
  existsSync,
  mkdirSync,
  renameSync,
  symlinkSync,
  rmSync,
} from "node:fs";
import { join } from "node:path";
import { readdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import { $ } from "zx";
import { SingleBar, Presets } from "cli-progress";

$.verbose = false;

const DIR = import.meta.dirname,
  DATA = join(DIR, "data"),
  TMP = tmpdir(),
  DL_DIR = join(TMP, "jdb_ftl_dl"),
  EXTRACT = join(DATA, "msrc_extracted"),
  TAR = join(DATA, "msrc.tar.gz"),
  TMP_GEN = join(TMP, "jdb_ftl"),
  URL = "https://github.com/js0-site/jdb_pgm/releases/download/v0.1.0",
  PARTS = ["MSRC-trace-003.tar.gz.part-aa", "MSRC-trace-003.tar.gz.part-ab"];

const TASKS = {
  quick: { file: join(DATA, "quick.bin"), limit: 1_000_000 },
  full: { file: join(DATA, "full.bin"), limit: 100_000_000 },
};

const io = {
  mkdir: (p) => {
    if (!existsSync(p)) mkdirSync(p, { recursive: true });
  },
  rm: (p) => {
    if (existsSync(p)) rmSync(p, { recursive: true, force: true });
  },
  mv: (src, dest) => {
    if (existsSync(src)) {
      try {
        renameSync(src, dest);
      } catch (e) {
        if (e.code === "EXDEV") {
          copyFileSync(src, dest);
          unlinkSync(src);
        } else {
          throw e;
        }
      }
    }
  },
  link: (src, dest) => {
    io.rm(dest);
    symlinkSync(src, dest);
  },
};

const net = {
  download: async (url, dest) => {
    console.log(`[NET] Downloading ${url} -> ${dest}`);
    await $`wget --no-check-certificate -c -O ${dest} ${url}`;
  },
};

const process_task = (files, taskKey) => {
  const { file: finalPath, limit } = TASKS[taskKey];
  const tmpFile = join(TMP_GEN, `${taskKey}.bin`);

  if (existsSync(finalPath)) {
    console.log(`[CHECK] ${finalPath} 已存在，跳过生成`);
    return 0;
  }

  console.log(`[GEN] 生成 ${taskKey.toUpperCase()} 数据 -> 临时文件...`);
  const fdOut = openSync(tmpFile, "w");
  const bar = new SingleBar(
    { format: `转换 ${taskKey} [{bar}] {percentage}% | {value}/{total}` },
    Presets.shades_classic,
  );
  bar.start(limit, 0);

  const CHUNK = 128 * 1024,
    buf = new Uint8Array(CHUNK),
    outBuf = new Uint8Array(16), // Changed to 16 bytes
    outView = new DataView(outBuf.buffer),
    dec = new TextDecoder("utf-8");

  let count = 0;
  let maxLba = 0n;

  // Simulation State for Golden Data
  let pbaCounter = 1n;
  const shadowMap = new Map(); // LBA (bigint) -> PBA (bigint)

  try {
    const parseAndWrite = (line) => {
      line = line.trim();
      if (!line) return false;

      const isCsv = line.includes(",");
      const parts = isCsv ? line.split(",") : line.split(/\s+/);

      let lba = 0n;
      let isWrite = false;
      let found = false;

      // 1. Parse LBA and Op
      if (!isCsv && parts.length >= 3) {
        const typeStr = parts[1];
        if (["RS", "WS", "R", "W"].includes(typeStr)) {
          try {
            lba = BigInt(parts[2]) / 8n; // 512B -> 4KB
            if (typeStr.includes("W")) isWrite = true;
            found = true;
          } catch { }
        }
      } else if (isCsv && parts.length >= 5) {
        try {
          const typeStr = parts[3]?.toLowerCase() || "";
          let offsetStr = parts[4].split(".")[0];
          lba = BigInt(offsetStr) / 4096n; // 4KB alignment assume bytes
          if (typeStr.includes("write")) isWrite = true;
          found = true;
        } catch { }
      }

      if (!found) return false;
      if (lba > maxLba) maxLba = lba;

      // 2. Simulate PBA (Golden Truth)
      let pba = 0n;
      let op = 0n; // 0=Read, 1=Write

      if (isWrite) {
        op = 1n;
        pba = ++pbaCounter;
        shadowMap.set(lba, pba);
      } else {
        // Read
        op = 0n;
        const existing = shadowMap.get(lba);
        if (existing === undefined) {
          pba = 0n;
        } else {
          pba = existing;
        }
      }

      // 3. Write 16 bytes: [LBA (u64)] [Op(4) + PBA(60)]
      outView.setBigUint64(0, lba, true);
      const meta = (op << 60n) | (pba & 0x0fffffffffffffffn);
      outView.setBigUint64(8, meta, true);

      writeSync(fdOut, outBuf);
      count++;
      if (count % 50000 === 0) bar.update(count);
      return true;
    };

    for (const file of files) {
      if (count >= limit) break;
      const fdIn = openSync(file, "r");
      let rest = "";

      try {
        while (count < limit) {
          const n = readSync(fdIn, buf, 0, CHUNK, null);
          const chunkStr =
            n > 0 ? dec.decode(buf.subarray(0, n), { stream: true }) : "";
          const str = rest + chunkStr;

          let last = 0,
            nl = str.indexOf("\n");
          while (nl !== -1 && count < limit) {
            const line = str.substring(last, nl).trim();
            last = nl + 1;
            nl = str.indexOf("\n", last);
            if (line) parseAndWrite(line);
          }
          rest = str.substring(last);

          if (n === 0) {
            if (rest.trim()) parseAndWrite(rest.trim());
            break;
          }
        }
      } finally {
        closeSync(fdIn);
      }
    }
  } finally {
    closeSync(fdOut);
  }

  bar.update(count);
  bar.stop();

  if (count > 0) {
    console.log(`[MOVE] ${tmpFile} -> ${finalPath}`);
    io.mv(tmpFile, finalPath);

    const configPath = join(DATA, `${taskKey}.json`);
    const config = { max_lba: maxLba.toString() };
    writeSync(openSync(configPath, "w"), JSON.stringify(config, null, 2));
    console.log(`[CONF] Config written to ${configPath}`);
  }
  return maxLba;
};

const main = async () => {
  console.log(">>> [INIT] JDB-FTL 数据初始化 (Replay Format)");

  io.mkdir(DATA);
  io.mkdir(TMP_GEN);

  if (!existsSync(EXTRACT)) {
    let src = TAR;
    if (!existsSync(TAR)) {
      io.mkdir(DL_DIR);
      await Promise.all(
        PARTS.map((p) => net.download(`${URL}/${p}`, join(DL_DIR, p))),
      );
      console.log(`[MERGE] Merging parts to ${src}...`);
      await $`cat ${PARTS.map((p) => join(DL_DIR, p))} > ${src}`;
      io.rm(DL_DIR);
    }

    io.mkdir(EXTRACT);
    console.log(`[EXTRACT] Extracting ${src}...`);
    await $`tar -xzf ${src} -C ${EXTRACT}`;
    io.rm(src);
  }

  const files = (await readdir(join(EXTRACT, "final-trace")))
    .filter((f) => f.endsWith(".revised"))
    .sort()
    .map((f) => join(EXTRACT, "final-trace", f));

  process_task(files, "quick");
  process_task(files, "full");

  if (existsSync(TASKS.quick.file)) {
    io.link(TASKS.quick.file, join(DATA, "trace.bin"));
  }

  console.log(">>> [CLEAN] 清理临时文件... (Skipped for debugging)");
  // io.rm(EXTRACT);
  // io.rm(TMP_GEN);
  const old = join(DATA, "MSRC-trace-003.tar.gz");
  // if (existsSync(old)) io.rm(old);

  console.log(">>> [SUCCESS] 任务全部完成。");
};

await main();
export default main;
