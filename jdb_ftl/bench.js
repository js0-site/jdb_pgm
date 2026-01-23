#!/usr/bin/env bun
import bench from "./benches/js/lib/bench.js";
import { $ } from "zx";
console.log("🚀 Running Full Benchmark & Generating Reports...");

// 1. Run Benchmark & Generate Data (using lib/bench.js directly)
console.log("-> Executing Cargo Benchmark...");
// Run full workload
process.env.BIN = "full";
await bench("bench_all");

// 2. Generate Markdown Documentation
console.log("-> Generating README Metrics...");
// Dynamic import to ensure fresh data loading (CONV relies on file existence)
const { default: md } = await import("./benches/js/md.js");
await md();

// 3. Print Table to Console for Immediate Feedback
console.log("-> Benchmark Summary:");
// Dynamic import for consistency, though less critical here
const { default: table } = await import("./benches/js/table.js");
await table("full");
await import("./readmeMerge.js");
console.log("✅ All Done.");
