// Run from the workspace root:
// deno run --allow-read --allow-write --allow-env --allow-ffi phanerite-app/scripts/generate-app-icons.js

import { Buffer } from "node:buffer";
import sharp from "npm:sharp@0.35.4";

const root = new URL("../", import.meta.url);
const output = new URL("assets/app-icons/", root);
const sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024];

function ico(images) {
  const frames = [...images].filter(([size]) => size <= 256);
  const directory = Buffer.alloc(6 + frames.length * 16);
  directory.writeUInt16LE(1, 2); // ICONDIR: type 1 is an icon, not a cursor.
  directory.writeUInt16LE(frames.length, 4);
  let offset = directory.length;
  for (const [index, [size, png]] of frames.entries()) {
    const entry = 6 + index * 16;
    directory[entry] = directory[entry + 1] = size === 256 ? 0 : size;
    directory.writeUInt16LE(1, entry + 4);
    directory.writeUInt16LE(32, entry + 6);
    directory.writeUInt32LE(png.length, entry + 8);
    directory.writeUInt32LE(offset, entry + 12);
    offset += png.length;
  }
  // PNG-backed ICO frames are supported on every Windows version GPUI targets.
  return Buffer.concat([directory, ...frames.map(([, png]) => png)]);
}

function icns(images) {
  const types = [
    ["icp4", 16],
    ["icp5", 32],
    ["icp6", 64],
    ["ic07", 128],
    ["ic08", 256],
    ["ic09", 512],
    ["ic10", 1024],
    ["ic11", 32], // 16 pt @2x
    ["ic12", 64], // 32 pt @2x
    ["ic13", 256], // 128 pt @2x
    ["ic14", 512], // 256 pt @2x
  ];
  const chunks = types.map(([type, size]) => {
    const png = images.get(size);
    const header = Buffer.alloc(8);
    header.write(type, 0, 4, "ascii");
    header.writeUInt32BE(8 + png.length, 4);
    return Buffer.concat([header, png]);
  });
  const header = Buffer.alloc(8);
  header.write("icns", 0, 4, "ascii");
  header.writeUInt32BE(
    8 + chunks.reduce((length, chunk) => length + chunk.length, 0),
    4,
  );
  return Buffer.concat([header, ...chunks]);
}

const svg = await Deno.readFile(new URL("assets/phanerite-logo.svg", root));
const { width } = await sharp(svg).metadata();
const images = new Map();
await Deno.mkdir(output, { recursive: true });
for (const size of sizes) {
  // Rasterize the SVG at each target density instead of enlarging a small PNG.
  const png = await sharp(svg, { density: 72 * size / width })
    .resize(size, size)
    .png()
    .toBuffer();
  images.set(size, png);
  await Deno.writeFile(new URL(`${size}.png`, output), png);
}
await Deno.writeFile(new URL("phanerite.ico", output), ico(images));
await Deno.writeFile(new URL("phanerite.icns", output), icns(images));
console.log("Generated Phanerite PNG, ICO and ICNS icons from the SVG logo.");
