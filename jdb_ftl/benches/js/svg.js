import { readdirSync, writeFileSync, mkdirSync } from "fs";
import { join } from "path";
import { optimize } from "svgo";

const DIR_ROOT = join(import.meta.dirname, "../../"),
  SVG_DIR_SRC = join(import.meta.dirname, "svg");

const ensureDir = (dir) => {
  mkdirSync(dir, { recursive: true });
};

const save = (lang, name, content) => {
  const dir = join(DIR_ROOT, `svg/${lang}`),
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

const run_all = async () => {
  const files = readdirSync(SVG_DIR_SRC).filter((f) => f.endsWith(".js"));
  console.log(`[SVG] Scanning ${SVG_DIR_SRC}, found ${files.length} scripts.`);

  for (const file of files) {
    const path = join(SVG_DIR_SRC, file),
      module = await import(path);
    if (module.default) {
      await module.default("en", save);
      await module.default("zh", save);
    }
  }
};

run_all();
