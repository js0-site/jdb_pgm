import helper from "../svg_helper.js";
import loadStats from "../lib/data.js";

const I18N = {
  zh: {
    dt: "分段编码分布 (Trace 真实重放)",
    st: "各策略存储节省量 (Bytes)",
    m: {
      l: "线性拟合 (Type A)",
      c: "常数优化 (Type B)",
      d: "直接模式 (Direct)",
      p: "PGM 模式",
      e: "空组 (Empty)",
    },
    s: { l: "线性拟合", d: "直接模式", e: "异常表" },
    sv: "已节省",
  },
  en: {
    dt: "Segment Encoding Distribution (Trace Analysis)",
    st: "Storage Savings by Strategy (Bytes)",
    m: {
      l: "Type A (Linear)",
      c: "Type B (Constant)",
      d: "Direct Mode",
      p: "PGM Mode",
      e: "Empty Groups",
    },
    s: { l: "Linear Model", d: "Direct Mode", e: "Exception Table" },
    sv: "Saved",
  },
},
  run = async (lang = "en", save) => {
    const stats = loadStats(),
      t = I18N[lang],
      // Compare Metadata Entries (Segments/Groups)
      // Standard PGM = Total PGM Segments - Type A - Type B
      std_pgm =
        (stats.total_pgm_segments || 0) -
        (stats.sim_type_a || 0) -
        (stats.sim_type_b || 0),
      dist_data = [
        { label: t.m.l, value: stats.sim_type_a || 0 },
        { label: t.m.c, value: stats.sim_type_b || 0 },
        { label: t.m.p, value: std_pgm > 0 ? std_pgm : 0 }, // Standard PGM
        { label: t.m.d, value: stats.groups.direct || 0 }, // 1 Direct Group ~= 1 Entry
        { label: t.m.e, value: stats.groups.empty || 0 }, // 1 Empty Group ~= 1 Entry
      ].filter((d) => d.value > 0),
      savings_data = [
        {
          label: t.s.l,
          value: stats.saved_linear || 0, // renamed from linear_model_bytes_saved
          suffix: t.sv,
        },
        { label: t.s.d, value: stats.saved_payload || 0, suffix: t.sv },
        {
          label: t.s.e,
          value: stats.saved_exception || 0,
          suffix: t.sv,
        },
      ];

    save(
      lang,
      "codec_modes",
      helper.generateDonutChart(t.dt, dist_data),
    );
    save(
      lang,
      "codec_efficiency",
      helper.generateBarChart(t.st, savings_data, { color: "#10b981" }),
    );
  };

export default run;
