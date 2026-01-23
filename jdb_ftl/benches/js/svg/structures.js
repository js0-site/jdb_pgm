import loadStats from "../lib/data.js";
import helper from "../svg_helper.js";

const I18N = {
  zh: { t: "数据结构复杂度 (字段数)", s: "个字段" },
  en: { t: "Data Structure Complexity (Field Count)", s: "fields" },
},
  run = async (lang = "en", save) => {
    const stats = loadStats(),
      t = I18N[lang],
      data = (stats.structs || [])
        .filter((s) => s.fields > 2)
        .map((s) => ({ label: s.name, value: s.fields, suffix: t.s }))
        .slice(0, 8);

    save(
      lang,
      "structures",
      helper.generateBarChart(t.t, data, { color: "#f43f5e" }),
    );
  };

export default run;
