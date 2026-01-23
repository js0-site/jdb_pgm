import { writeFileSync, mkdirSync } from "fs";
import { join } from "path";
import { optimize } from "svgo";

const DIR_ROOT = join(import.meta.dirname, "../../../"),
  SAVE_PATHS = {
    zh: join(DIR_ROOT, "readme/zh/svg"),
    en: join(DIR_ROOT, "readme/en/svg"),
  };

const ensureDir = (dir) => {
  mkdirSync(dir, { recursive: true });
};

const save = (lang, name, content) => {
  const dir = SAVE_PATHS[lang],
    path = join(dir, `${name}.svg`),
    { data: optimized } = optimize(content, {
      path,
      multipass: true,
      plugins: [
        "preset-default",
        { name: "removeViewBox", active: false },
        { name: "removeDimensions", active: false },
      ],
    });

  ensureDir(dir);
  writeFileSync(path, optimized);
  console.log(`[SVG] Optimized & Saved ${lang}: ${name}`);
};

export default { save };
