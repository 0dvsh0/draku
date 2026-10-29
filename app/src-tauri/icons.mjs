// Generates the Tauri icon set entirely offline: a black fur disk with
// Draku's white-ringed, crimson-flamed eyes. Run: node src-tauri/icons.mjs
import zlib from "node:zlib";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const out = path.join(here, "icons");
fs.mkdirSync(out, { recursive: true });

// ---- tiny RGBA canvas -------------------------------------------------------
function canvas(n) {
  return { n, px: new Uint8Array(n * n * 4) };
}
function set(c, x, y, r, g, b, a = 255) {
  if (x < 0 || y < 0 || x >= c.n || y >= c.n) return;
  const i = (y * c.n + x) * 4;
  const A = a / 255;
  const j = c.px;
  j[i] = j[i] * (1 - A) + r * A;
  j[i + 1] = j[i + 1] * (1 - A) + g * A;
  j[i + 2] = j[i + 2] * (1 - A) + b * A;
  j[i + 3] = Math.max(j[i + 3], a);
}
function dist(x, y, cx, cy) {
  return Math.hypot(x - cx, y - cy);
}

// Palette (tokens.css)
const FUR = [8, 7, 8]; // --fur-1
const BONE = [234, 230, 230]; // --bone
const IRIS = [122, 29, 37]; // --iris
const WING = [132, 30, 40]; // --wing
const GLOW = [194, 65, 78]; // --wing-glow

function drawDraku(n) {
  const c = canvas(n);
  const S = n / 100; // design space 100×100
  const P = (v) => Math.round(v * S);
  for (let y = 0; y < n; y++) {
    for (let x = 0; x < n; x++) {
      const dx = (x + 0.5) / S, dy = (y + 0.5) / S;
      // fur mask (squircle-ish): rounded disc
      const d = dist(dx, dy, 50, 50);
      const rr = 47 + 2.4 * (Math.cos(((dx - 50) / 50) * 8) * Math.sin(((dy - 50) / 50) * 8));
      if (d < rr) set(c, x, y, FUR[0], FUR[1], FUR[2], 255);
      else if (d < rr + 1.6) set(c, x, y, WING[0], WING[1], WING[2], 180);
    }
  }
  // two eyes: white ring + crimson iris + black flames
  for (const ex of [33, 67]) {
    const ey = 45, re = 15, ri = 9;
    for (let y = P(ey - re); y < P(ey + re) + 1; y++) {
      for (let x = P(ex - re); x < P(ex + re) + 1; x++) {
        const dd = dist((x + 0.5) / S, (y + 0.5) / S, ex, ey);
        if (dd <= re) {
          // flame (black over crimson): two tongue shapes
          const fx = ((x + 0.5) / S - ex) / re;
          const fy = ((y + 0.5) / S - ey) / re;
          const flame =
            Math.abs(0.8 * fx + 0.25 * fy) < 0.34 * (fy + 0.55) ||
            Math.abs(0.5 * fx - 0.35 * fy) < 0.28 * (fy + 0.5);
          if (flame && fy < 0.75) set(c, x, y, 5, 4, 5, 255);
          else if (dd <= ri) set(c, x, y, IRIS[0], IRIS[1], IRIS[2], 255);
          else set(c, x, y, BONE[0], BONE[1], BONE[2], 255);
        }
        // soft outer ring
        if (dd > re && dd < re + 1.4) set(c, x, y, BONE[0], BONE[1], BONE[2], 90);
      }
    }
  }
  // wing hints at the bottom corners
  for (const wx of [22, 78]) {
    for (let t = 0; t < 26; t++) {
      const x = P(wx), y = P(86 - t * 0.9);
      set(c, x + (t % 2 ? 1 : -1) * t * 0.35, y, WING[0], WING[1], WING[2], 220);
      set(c, x + (t % 2 ? 1 : -1) * t * 0.35 - 1, y, GLOW[0], GLOW[1], GLOW[2], 150);
    }
  }
  return c;
}

// ---- PNG writer (RGBA, deflate) --------------------------------------------
function crc32(buf) {
  let c = 0xffffffff;
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n++) {
    let k = n;
    for (let m = 0; m < 8; m++) k = k & 1 ? 0xedb88320 ^ (k >>> 1) : k >>> 1;
    table[n] = k;
  }
  for (const b of buf) c = table[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}
function writePng(file, c) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(c.n, 0);
  ihdr.writeUInt32BE(c.n, 4);
  ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  const raw = Buffer.alloc(c.n * (c.n * 4 + 1));
  let o = 0;
  for (let y = 0; y < c.n; y++) {
    raw[o++] = 0;
    raw.set(c.px.subarray(y * c.n * 4, (y + 1) * c.n * 4), o);
    o += c.n * 4;
  }
  const png = Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", zlib.deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
  fs.writeFileSync(file, png);
}

// ---- .ico with PNG entries --------------------------------------------------
function writeIco(file, pngBuf) {
  // ICO header + one 256px PNG entry
  const h = Buffer.alloc(6);
  h.writeUInt16LE(0, 0); h.writeUInt16LE(1, 2); h.writeUInt16LE(1, 4);
  const e = Buffer.alloc(16);
  e[0] = 0; e[1] = 0; // 256 → 0
  e[2] = 0; e[3] = 0;
  e.writeUInt16LE(1, 4); e.writeUInt16LE(32, 6); // RGBA/32
  e.writeUInt32LE(pngBuf.length, 8);
  e.writeUInt32LE(22, 12); // offset
  fs.writeFileSync(file, Buffer.concat([h, e, pngBuf]));
}

for (const n of [32, 44, 88, 128, 256]) {
  writePng(path.join(out, `${n}x${n}.png`), drawDraku(n));
}
writePng(path.join(out, "128x128@2x.png"), drawDraku(256));
writeIco(path.join(out, "icon.ico"), fs.readFileSync(path.join(out, "256x256.png")));
console.log("icons written to", out);
