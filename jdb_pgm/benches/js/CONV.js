#!/usr/bin/env bun

import { benchJsonLi } from "./conf.js";
import conv from "./lib/conv.js";

const raw = benchJsonLi();
const cooked = conv(raw);

export default cooked;
