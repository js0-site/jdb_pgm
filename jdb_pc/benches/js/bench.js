#!/usr/bin/env bun

import { i18nImport } from "./conf.js";
import table from "./table.js";
import bench from "./lib/bench.js";

// Run benchmarks for all libraries
await bench("bench");

const I18N = await i18nImport(import.meta);

// Show cross-library comparison table
console.log(table());
