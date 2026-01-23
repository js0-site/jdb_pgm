import loadStats from "../lib/data.js";
import helper from "../svg_helper.js";

const I18N = {
  zh: {
    t: "核心模块代码量分析 (LoC)",
    m: {
      c: "核心统计 (stats.rs)",
      l: "L1 管理 (l1.rs)",
      p: "PGM 引擎 (pgm.rs)",
      b: "位图系统 (bitmap.rs)",
      f: "刷新策略 (bg.rs)",
    },
  },
  en: {
    t: "Critical Module LoC Analysis",
    m: {
      c: "Core (stats.rs)",
      l: "L1 Manager (l1.rs)",
      p: "PGM Engine (pgm.rs)",
      b: "Bitmap (bitmap.rs)",
      f: "Flush (bg.rs)",
    },
  },
},
  run = async (lang = "en", save) => {
    const stats = loadStats(),
      t = I18N[lang],
      loc = stats.loc || {},
      data = [
        { label: t.m.c, value: loc["src/ftl/stats.rs"] || 0 },
        { label: t.m.l, value: loc["src/ftl/l1.rs"] || 0 },
        { label: t.m.p, value: loc["src/ftl/codec/encoder/pgm.rs"] || 0 },
        { label: t.m.b, value: loc["src/ftl/bitmap.rs"] || 0 },
        { label: t.m.f, value: loc["src/ftl/bg.rs"] || 0 },
      ];

    save(
      lang,
      "metrics",
      helper.generateBarChart(t.t, data, { color: "#8b5cf6" }),
    );
  };

export default run;
