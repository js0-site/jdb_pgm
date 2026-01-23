import { readFileSync } from "fs";
import { join, dirname } from "path";
import { renderFile } from "pug";
import { fileURLToPath } from "url";
import { promisify } from "util";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const templatePath = join(__dirname, "template.pug");
const renderFileAsync = promisify(renderFile);

export const renderHtml = async (lang) => {
  const dataPath = join(__dirname, "..", "dist", "data", `${lang}.json`);
  const data = JSON.parse(readFileSync(dataPath, "utf-8"));
  return renderFileAsync(templatePath, data);
};

export const renderZh = async () => renderHtml("zh");
export const renderEn = async () => renderHtml("en");
