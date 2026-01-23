#!/usr/bin/env bun

import { i18nImport, benchJsonLi } from "./conf.js";
import table from "./table.js";
import bench from "./lib/bench.js";

// Run benchmarks only for Pc
await bench("bench-pc");

const I18N = await i18nImport(import.meta);

// TODO: table.js is currently designed for cross-lib comparison (bench.js).
// Regression history support needs to be added or separated.
// For now, we reuse table, which will show Pc results (and missing baselines).
console.log(table());
