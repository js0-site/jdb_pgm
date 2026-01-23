#! /usr/bin/env bun

import run from "./benches/js/regress.js";

const kind = process.env.BIN || "full";
await run(kind);
