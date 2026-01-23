import { readFileSync } from "fs";
import { join } from "path";

// Central data loader for SVG generation
// Reads the Rust-generated JSON stats from benches/js/svg/json/stats.json
const loadStats = () => {
  // Relative path from this file (benches/js/lib/data.js) to the json file
  // benches/js/lib/../../benches/js/svg/json/stats.json -> benches/js/svg/json/stats.json
  // Actually simpler: join(import.meta.dirname, '../svg/json/stats.json')
  const path = join(import.meta.dirname, "../svg/json/stats.json");
  try {
    return JSON.parse(readFileSync(path, "utf-8"));
  } catch (e) {
    console.warn(
      `[SVG] Data Warning: Could not read ${path}. Defaulting to empty stats.`,
    );
    return {
      total_groups: 0,
      groups: { pgm: 0, direct: 0, empty: 0 },
      bit_width_dist: {},
      seg_len_dist: {},
      lba_heatmap_100k: {},
    };
  }
};

export default loadStats;
