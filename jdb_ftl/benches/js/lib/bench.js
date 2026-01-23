#!/usr/bin/env bun

import { BENCH_JSON_PATH, ROOT } from "../conf.js";
import { $, cd } from "zx";

$.verbose = true;

process.env.BIN = "full";
cd(ROOT);

const main = async (features = "bench_all") => {
  const { stdout } =
    await $`cargo criterion --bench main --message-format=json --features ${features}`.quiet();
  await Bun.write(BENCH_JSON_PATH, stdout);
};

export default main;

if (import.meta.main) {
  await main();
}
