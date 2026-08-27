import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import pngToIco from "png-to-ico";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const iconsDir = path.join(__dirname, "..", "src-tauri", "icons");

const pngs = ["32x32.png", "128x128.png", "128x128@2x.png"].map((f) =>
  path.join(iconsDir, f),
);

const ico = await pngToIco(pngs);
fs.writeFileSync(path.join(iconsDir, "icon.ico"), ico);
console.log("Created icon.ico");
