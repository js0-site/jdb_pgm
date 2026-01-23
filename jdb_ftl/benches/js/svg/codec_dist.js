import helper from "../svg_helper.js";
import loadStats from "../lib/data.js";

const I18N = {
  zh: {
    md: "存储组模式分布 (Real-Time)",
    bd: "PGM 残差位宽分布 (Bit-Width Efficiency)",
    sl: "L1 分段长度分布 (Segment Length)",
    m: {
      p: "PGM 压缩组",
      d: "Direct 直写组",
      e: "Empty 空闲组",
    },
    s: {
      b: "Bits",
      e: "Entries",
    },
  },
  en: {
    md: "Storage Group Mode Distribution (Real-Time)",
    bd: "PGM Residual Bit-Width Distribution",
    sl: "L1 Segment Length Distribution",
    m: {
      p: "PGM Compressed",
      d: "Direct Mode",
      e: "Empty Groups",
    },
    s: {
      b: "Bits",
      e: "Entries",
    },
  },
},
  run = async (lang = "en", save) => {
    const stats = loadStats(),
      t = I18N[lang];

    // 1. Group Mode Distribution (Donut)
    const mode_data = [
      { label: t.m.p, value: stats.groups.pgm },
      { label: t.m.d, value: stats.groups.direct },
      { label: t.m.e, value: stats.groups.empty },
    ]; // Removed filter to show all modes even if 0

    save(
      lang,
      "codec_modes_real",
      helper.generateDonutChart(t.md, mode_data),
    );

    // 2. Bit Width Distribution (Bar)
    const bw_keys = Object.keys(stats.bit_width_dist)
      .map(Number)
      .sort((a, b) => a - b),
      bw_data = bw_keys.slice(0, 10).map((k) => ({
        label: k === 0 ? "Perfect (0)" : `${k} bits`,
        value: stats.bit_width_dist[k],
      }));

    save(
      lang,
      "codec_bit_width",
      helper.generateBarChart(t.bd, bw_data, { color: "#8b5cf6" }),
    );

    // 3. Segment Length Distribution (Heatmap/Bar)
    // We use a simplified bar chart view for the 10 most common dense buckets
    const sl_keys = Object.keys(stats.seg_len_dist)
      .map(Number)
      .sort((a, b) => a - b),
      sl_data = sl_keys.slice(0, 8).map((k) => ({
        label: `${k}-${k + 10}`,
        value: stats.seg_len_dist[k],
        // suffix removed as requested
      }));

    save(
      lang,
      "codec_seg_len",
      helper.generateBarChart(t.sl, sl_data, { color: "#3b82f6" }),
    );
  };

export default run;
