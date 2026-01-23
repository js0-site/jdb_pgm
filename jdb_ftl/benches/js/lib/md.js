#!/usr/bin/env bun

import { join } from "node:path";
import read from "@3-/read";
import { Eta } from "eta";
import { readmeWrite, DIR_I18N } from "../conf.js";

const ETA = new Eta({ autoEscape: false, varName: "_" }),
  BENCH_MD = "bench.md";

export default (generator) =>
  Promise.all(
    readmeWrite(import.meta, BENCH_MD, (I18N, lang) =>
      ETA.renderString(read(join(DIR_I18N, lang, BENCH_MD)), {
        I18N,
        ...generator(I18N),
      }),
    ),
  );
