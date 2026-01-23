import { readFileSync } from "fs";
import helper from "../svg_helper.js";

const I18N = {
  zh: {
    t: "精度 (Epsilon) 灵敏度曲线 (实测数据)",
    xl: "精度 (Epsilon)",
    yl: "分段数量 (相对于 ε=1)",
  },
  en: {
    t: "Epsilon Sensitivity Curve (Measured Data)",
    xl: "Epsilon",
    yl: "Segment Count (relative to ε=1)",
  },
},
  run = async (lang = "en", save) => {
    const t = I18N[lang];

    // Load real measured data from epsilon_sweep.rs output
    let eps_data;
    const raw = JSON.parse(
      readFileSync("benches/js/svg/json/epsilon_sweep.json", "utf-8"),
    );
    // Normalize Y values: percentage relative to ε=1 (first value)
    const baseline = raw[0].y;
    eps_data = raw.map((d) => ({
      x: d.x,
      y: Math.round((d.y / baseline) * 100),
    }));

    save(
      lang,
      "tuning_epsilon",
      helper.generateLineChart(t.t, eps_data, {
        xLabel: t.xl,
        yLabel: t.yl,
        logX: true,
      }),
    );
  };

export default run;
