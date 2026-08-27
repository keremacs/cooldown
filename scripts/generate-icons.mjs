#!/usr/bin/env node
/**
 * Cross-platform Cooldown icon generator (PNG + ICO + ICNS).
 * Works on Windows, macOS, and Linux — no PowerShell required.
 */
import fs from "node:fs";
import path from "node:path";
import { deflateSync } from "node:zlib";
import { fileURLToPath } from "node:url";
import pngToIco from "png-to-ico";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const iconsDir = path.join(__dirname, "..", "src-tauri", "icons");

function drawCooldownIcon(size) {
  const rgba = Buffer.alloc(size * size * 4, 0);
  const cx = (size - 1) / 2;
  const cy = (size - 1) / 2;
  const radius = size * 0.42;

  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const dx = x - cx;
      const dy = y - cy;
      const dist = Math.hypot(dx, dy);
      const i = (y * size + x) * 4;

      if (dist <= radius) {
        const t = (dx + dy) / (radius * 2) + 0.5;
        rgba[i] = Math.round(79 + t * 20);
        rgba[i + 1] = Math.round(70 + t * 32);
        rgba[i + 2] = Math.round(229 + t * 12);
        rgba[i + 3] = 255;
      } else if (dist <= radius + Math.max(1, size * 0.04)) {
        rgba[i + 3] = Math.round(255 * (1 - (dist - radius) / Math.max(1, size * 0.04)));
      }
    }
  }

  // Simple snowflake mark
  const mark = Math.max(2, Math.round(size * 0.06));
  for (let a = 0; a < 3; a++) {
    const angle = (a * Math.PI * 2) / 3 - Math.PI / 2;
    for (let r = size * 0.08; r <= size * 0.22; r += 0.5) {
      const px = Math.round(cx + Math.cos(angle) * r);
      const py = Math.round(cy + Math.sin(angle) * r);
      for (let oy = -mark; oy <= mark; oy++) {
        for (let ox = -mark; ox <= mark; ox++) {
          const x = px + ox;
          const y = py + oy;
          if (x < 0 || y < 0 || x >= size || y >= size) continue;
          const idx = (y * size + x) * 4;
          rgba[idx] = 255;
          rgba[idx + 1] = 255;
          rgba[idx + 2] = 255;
          rgba[idx + 3] = 220;
        }
      }
    }
  }

  return rgba;
}

function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
  }
  return ~c >>> 0;
}

function pngChunk(type, data) {
  const typeBuf = Buffer.from(type, "ascii");
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([len, typeBuf, data, crc]);
}

function encodePng(size, rgba) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;

  const stride = size * 4 + 1;
  const raw = Buffer.alloc(stride * size);
  for (let y = 0; y < size; y++) {
    raw[y * stride] = 0;
    rgba.copy(raw, y * stride + 1, y * size * 4, (y + 1) * size * 4);
  }

  const compressed = deflateSync(raw);

  return Buffer.concat([
    signature,
    pngChunk("IHDR", ihdr),
    pngChunk("IDAT", compressed),
    pngChunk("IEND", Buffer.alloc(0)),
  ]);
}

async function writePng(filePath, size) {
  const png = encodePng(size, drawCooldownIcon(size));
  fs.writeFileSync(filePath, png);
  console.log(`Created ${path.basename(filePath)}`);
}

async function writeIcns(png512Path, icnsPath) {
  try {
    const icongen = (await import("icon-gen")).default;
    await icongen(png512Path, path.dirname(icnsPath), {
      report: false,
      ico: false,
      icns: {
        name: path.basename(icnsPath, ".icns"),
        sizes: [16, 32, 64, 128, 256, 512, 1024],
      },
    });
    console.log(`Created ${path.basename(icnsPath)}`);
  } catch (err) {
    console.warn("icon-gen unavailable, trying: npx tauri icon");
    console.warn(String(err));
  }
}

fs.mkdirSync(iconsDir, { recursive: true });

const sizes = {
  "32x32.png": 32,
  "128x128.png": 128,
  "128x128@2x.png": 256,
  "icon.png": 512,
  "tray-icon.png": 32,
};

for (const [name, size] of Object.entries(sizes)) {
  await writePng(path.join(iconsDir, name), size);
}

const ico = await pngToIco([
  path.join(iconsDir, "32x32.png"),
  path.join(iconsDir, "128x128.png"),
  path.join(iconsDir, "128x128@2x.png"),
]);
fs.writeFileSync(path.join(iconsDir, "icon.ico"), ico);
console.log("Created icon.ico");

await writeIcns(path.join(iconsDir, "icon.png"), path.join(iconsDir, "icon.icns"));

console.log("Done.");
