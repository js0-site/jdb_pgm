import loadStats from "../lib/data.js";
import helper from "../svg_helper.js";

const I18N = {
  zh: {
    t: "核心算法复杂性概况",
    m: {
      "O(1)": "常数级 O(1)",
      "O(log n)": "对数级 O(log n)",
      "O(1) - O(n)": "区间 O(1)-O(n)",
      "O(N)": "线性级 O(N)",
      "O(N + U log U)": "复合级 O(N+UlogU)",
    },
  },
  en: {
    t: "Algorithm Complexity Profile",
    m: {
      "O(1)": "O(1)",
      "O(log n)": "O(log n)",
      "O(1) - O(n)": "O(1) - O(n)",
      "O(N)": "O(N)",
      "O(N + U log U)": "O(N + U log U)",
    },
  },
},
  SCORE_MAP = {
    "O(1)": 1,
    "O(log n)": 3,
    "O(1) - O(n)": 5,
    "O(N)": 4,
    "O(N + U log U)": 6,
  },
  run = async (lang = "en", save) => {
    const stats = loadStats(),
      t = I18N[lang],
      t_suffix = t.m,
      data = (stats.algorithms || []).map((a) => ({
        label: (a.name || "").split(" (")[0],
        value: SCORE_MAP[a.complexity] || 2,
        suffix: t_suffix[a.complexity] || a.complexity,
      }));

    save(
      lang,
      "algorithms",
      helper.generateBarChart(t.t, data, { color: "#f59e0b" }),
    );
  };

export default run;
