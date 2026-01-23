import helper from "../svg_helper.js";
import loadStats from "../lib/data.js";

const I18N = {
  zh: {
    ht: "LBA 访问热度分布 (100K 桶)",
    rd: "模型残差位宽分布 (Residual Density)",
  },
  en: {
    ht: "LBA Access Heatmap (100K Buckets)",
    rd: "Residual Bit-width Distribution",
  },
},
  run = async (lang = "en", save) => {
    const stats = loadStats(),
      t = I18N[lang];

    // 1. LBA Heatmap
    // keys are bucket indices "0", "1", etc.
    // We need to convert to a dense array for visualization
    const hm_dist = stats.lba_heatmap_100k || {},
      max_bucket = Math.max(0, ...Object.keys(hm_dist).map(Number)),
      heatmap_data = Array.from(
        { length: max_bucket + 1 },
        (_, i) => hm_dist[i] || 0,
      );

    save(
      lang,
      "lba_heatmap",
      helper.generateHeatmap(t.ht, heatmap_data),
    );

    // 2. Residual Distribution (Recycled from bit_width_dist)
    const bw_keys = Object.keys(stats.bit_width_dist || {})
      .map(Number)
      .sort((a, b) => a - b),
      // Take top 10 or meaningful range
      residual_data = bw_keys.slice(0, 8).map((k) => ({
        label: k === 0 ? "0 (Perfect)" : `${k}`,
        value: stats.bit_width_dist[k],
        suffix: " Segs",
      }));

    save(
      lang,
      "residual_dist",
      helper.generateBarChart(t.rd, residual_data, { color: "#8b5cf6" }),
    );
  };

export default run;
