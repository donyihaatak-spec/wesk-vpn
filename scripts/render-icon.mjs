/**
 * Рендер app-icon.svg → app-icon.png (1024) для `tauri icon`.
 * SVG совпадает с Logo.tsx в приложении.
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import sharp from "sharp";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const iconsDir = join(root, "src-tauri", "icons");
const svgPath = join(iconsDir, "app-icon.svg");
const pngPath = join(iconsDir, "app-icon.png");

const svg = readFileSync(svgPath);

await sharp(Buffer.from(svg), { density: 384 })
  .resize(1024, 1024)
  .png()
  .toFile(pngPath);

console.log(`Saved: ${pngPath}`);
