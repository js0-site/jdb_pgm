#!/usr/bin/env bun

import { join } from "node:path";
import read from "@3-/read";
import { Eta } from "eta";
import { readmeWrite, DIR_I18N } from "../conf.js";

const ETA = new Eta({ autoEscape: false, varName: "_" });

export default (table) =>
  Promise.all(
    readmeWrite(import.meta, "bench.md", (I18N, lang) =>
      ETA.renderString(read(join(DIR_I18N, lang, "tpl.md")), {
        I18N,
        ...table(I18N),
      }),
    ),
  );
