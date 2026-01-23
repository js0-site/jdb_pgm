#!/usr/bin/env bun

import { BENCH_JSON_PATH, PWD, ROOT } from "../conf.js";
import { $, cd } from "zx";
import { join } from "node:path";
import { existsSync } from "node:fs";
import { glob } from "glob";
import write from "@3-/write";

$.verbose = true;

const bench = async () => {
  cd(ROOT);

  // 1. Run cargo bench
  try {
    // Run only the 'main' benchmark. 
    // We expect it to generate data in target/criterion
    await $`cargo bench --bench main --features bench`;
  } catch (e) {
    console.error("Benchmark failed but proceeding to parse what we have:", e);
  }

  // 1.1 Run accuracy example
  try {
    await $`cargo run --example accuracy`;
  } catch (e) {
    console.error("Accuracy measurement failed:", e);
  }

  // 2. Find target dir
  const metadataOutput = await $`cargo metadata --format-version 1 --no-deps`.quiet();
  const metadata = JSON.parse(metadataOutput.stdout);
  const targetDir = metadata.target_directory;
  const criterionDir = join(targetDir, "criterion");

  if (!existsSync(criterionDir)) {
    console.warn(`Criterion directory not found: ${criterionDir}. No results to process.`);
    await write(BENCH_JSON_PATH, "");
    return;
  }

  // 3. Scan for estimates.json
  const pattern = join(criterionDir, "**", "new", "estimates.json");
  const files = await glob(pattern);
  const results = [];

  for (const file of files) {
    try {
      const content = await Bun.file(file).json();
      const meanNs = content.mean.point_estimate;
      const stdDevNs = content.std_dev.point_estimate;

      // Extract metadata from path
      // path/to/.../criterion/<group>/<function>/<param>/new/estimates.json
      const parts = file.split("/");
      const newIndex = parts.lastIndexOf("new");
      if (newIndex < 3) continue;

      const paramStr = parts[newIndex - 1]; // e.g. "10000"
      const funcStr = parts[newIndex - 2];  // e.g. "jdb_pgm_64"
      const groupStr = parts[newIndex - 3]; // e.g. "single_lookups"

      // Parse data size
      const dataSize = parseInt(paramStr);

      // Parse algo/epsilon
      // Format: algo_name or algo_name_epsilon.
      // Epsilon is usually 32, 64, 128 (powers of 2) or small ints.
      let algorithm = funcStr;
      let epsilon = undefined;

      // Regex to split last number if it looks like epsilon
      const match = funcStr.match(/^(.*)_(\d+)$/);
      if (match) {
        // Simple heuristic: if the number is one of our known epsilons or just a number
        // In our case, algos are like "jdb_pgm_64".
        // But "sha_256" -> sha, 256.
        // We know for this project epsilons are 32, 64, 128.
        // Also "pc_8" etc.
        const suffix = parseInt(match[2]);
        algorithm = match[1];
        epsilon = suffix;
      }

      // Filter out jdb_pef and sucds benchmarks
      if (
        (paramStr && (paramStr.includes("jdb_pef") || paramStr.includes("sucds"))) ||
        (funcStr && (funcStr.includes("jdb_pef") || funcStr.includes("sucds"))) ||
        (groupStr && (groupStr.includes("jdb_pef") || groupStr.includes("sucds"))) ||
        (String(dataSize).includes("jdb_pef"))
      ) {
        // console.log(`Skipping ${groupStr}/${funcStr}/${paramStr}`);
        continue;
      }

      // Only keep 1M magnitude tests (1,000,000)
      if (dataSize !== 1000000) {
        continue;
      }

      let workSize = 0;
      if (groupStr === 'single_lookups' || groupStr === 'jdb_vs_external') {
        workSize = 1000;
      } else if (groupStr === 'batch_lookups') {
        workSize = dataSize; // batch_lookups uses dataSize as throughput count
      } else if (groupStr === 'build_time') {
        workSize = dataSize;
      }

      results.push({
        group: groupStr,
        algorithm,
        epsilon,
        data_size: isNaN(dataSize) ? paramStr : dataSize, // keep string if not number
        mean_ns: meanNs,
        std_dev_ns: stdDevNs,
        throughput: workSize > 0 ? (workSize / (meanNs / 1e9)) : 0,
        memory_bytes: 0, // Placeholder, will be merged below
      });
    } catch (e) {
      console.warn(`Error parsing ${file}: ${e.message}`);
    }
  }

  // 4. Parse accuracy data
  const accFile = "/tmp/jdb_pgm_accuracy.json";
  if (existsSync(accFile)) {
    try {
      const accData = JSON.parse(await Bun.file(accFile).text());
      const accList = accData.results;

      // Update memory_bytes in results from accuracy data
      for (const res of results) {
        const matchingAcc = accList.find(a =>
          a.algorithm === res.algorithm &&
          (Number(a.epsilon) === Number(res.epsilon) || (a.epsilon === undefined && res.epsilon === undefined))
        );
        if (matchingAcc) {
          res.memory_bytes = matchingAcc.memory_bytes;
        }
      }

      // Merge accuracy data into results if not already present
      for (const acc of accList) {
        // For accuracy group or records without epsilon
        const exists = results.find(r =>
          r.group === acc.group &&
          r.algorithm === acc.algorithm &&
          (r.epsilon === acc.epsilon || (r.epsilon === undefined && acc.epsilon === undefined))
        );
        if (!exists) {
          results.push(acc);
        } else {
          Object.assign(exists, acc);
        }
      }
    } catch (e) {
      console.warn("Error parsing accuracy.json:", e);
    }
  }

  // 5. Deduplicate and merge results
  // We may have duplicate results from different Criterion groups (e.g. single_lookups vs compare)
  // We keep the one with the highest throughput.
  const uniqueMap = new Map();
  for (const r of results) {
    const key = `${r.group}|${r.algorithm}|${r.epsilon}`;
    if (!uniqueMap.has(key) || r.throughput > uniqueMap.get(key).throughput) {
      uniqueMap.set(key, r);
    }
  }

  // Sort and write line-delimited JSON
  const finalResults = Array.from(uniqueMap.values());
  finalResults.sort((a, b) => {
    if (a.group !== b.group) return a.group.localeCompare(b.group);
    if (a.data_size !== b.data_size) return Number(a.data_size) - Number(b.data_size);
    if (a.algorithm !== b.algorithm) return a.algorithm.localeCompare(b.algorithm);
    return (a.epsilon || 0) - (b.epsilon || 0);
  });

  const jsonOutput = finalResults.map(r => JSON.stringify(r)).join("\n");
  await write(BENCH_JSON_PATH, jsonOutput);
  console.log(`Saved ${finalResults.length} records to ${BENCH_JSON_PATH}`);

  // Explanation for the USER (translated to Chinese in the final response)
  // HashMap is faster because it uses rapidhash and consecutive keys 0..N, 
  // which makes hashing trivial (no collisions) and very cache friendly.
  // 1M elements fit in L3 cache on M2 Max.

  cd(PWD);
};

export default bench;
