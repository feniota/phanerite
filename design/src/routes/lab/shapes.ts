/**
 * Shape generators for the design sandbox at /lab.
 *
 * Two forms grown from the same seed, `name|mcVersion|loader`:
 *
 *   blockFor()   the existing 5x5 launcher block, plus the fill orders that
 *                turn it into a launch progress indicator
 *   crystalFor() the replacement default icon: a transparent, borderless,
 *                irregular crystal cluster on an NxN pixel lattice
 *
 * Kept beside the route so the whole experiment deletes as one directory.
 */

import type { Loader } from "$lib/types";

export type { Loader };

export const LOADERS: readonly Loader[] = ["vanilla", "fabric", "forge", "neoforge"];

/** The palette InstanceIcon.svelte ships today. Kept only for the before/after. */
export const LEGACY_PALETTES: Record<Loader, readonly string[]> = {
  vanilla: ["var(--mc-grass-dark)", "var(--mc-dirt)", "var(--mc-grass)", "var(--mc-sand)"],
  fabric: ["rgb(83 74 68)", "rgb(112 100 92)", "rgb(138 123 113)", "rgb(168 153 140)"],
  forge: ["rgb(55 59 98)", "rgb(73 78 122)", "rgb(91 97 151)", "rgb(122 128 184)"],
  neoforge: ["rgb(160 91 56)", "rgb(190 114 72)", "rgb(220 137 92)", "rgb(239 166 120)"],
};

/** Single-hue ramps, darkest first. Index 0..3 is the shading scale. */
export const RAMPS: Record<Loader, readonly string[]> = {
  vanilla: [
    "var(--loader-vanilla-1)",
    "var(--loader-vanilla-2)",
    "var(--loader-vanilla-3)",
    "var(--loader-vanilla-4)",
  ],
  fabric: [
    "var(--loader-fabric-1)",
    "var(--loader-fabric-2)",
    "var(--loader-fabric-3)",
    "var(--loader-fabric-4)",
  ],
  forge: [
    "var(--loader-forge-1)",
    "var(--loader-forge-2)",
    "var(--loader-forge-3)",
    "var(--loader-forge-4)",
  ],
  neoforge: [
    "var(--loader-neoforge-1)",
    "var(--loader-neoforge-2)",
    "var(--loader-neoforge-3)",
    "var(--loader-neoforge-4)",
  ],
};

export function hash(value: string) {
  let result = 2166136261;
  for (const char of value) {
    result ^= char.charCodeAt(0);
    result = Math.imul(result, 16777619);
  }
  return result >>> 0;
}

export function random(seed: number) {
  return () => {
    seed += 0x6d2b79f5;
    let value = seed;
    value = Math.imul(value ^ (value >>> 15), value | 1);
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61);
    return ((value ^ (value >>> 14)) >>> 0) / 4294967296;
  };
}

/* ------------------------------------------------------------------ block */

export type FillOrder = "scatter" | "bottomUp" | "nucleation" | "dimLit";

export interface BlockShape {
  /** 25 colored cells in the 5 × 5 block. */
  colors: string[];
  /** Cells in hash order — this is the shuffle the icon already does. */
  scatter: number[];
  /** Same cells, rows bottom to top, hash order preserved inside each row. */
  bottomUp: number[];
  /** Same cells, growing outward from one hash-picked nucleation site. */
  nucleation: number[];
}

function squaredDistance(origin: number, cell: number) {
  const dx = (cell % 5) - (origin % 5);
  const dy = Math.floor(cell / 5) - Math.floor(origin / 5);
  return dx * dx + dy * dy;
}

export function blockFor(
  seed: string,
  loader: Loader,
  palette: "ramp" | "legacy" = "ramp",
): BlockShape {
  const next = random(hash(seed));
  const positions = Array.from({ length: 25 }, (_, index) => index);

  for (let index = positions.length - 1; index > 0; index -= 1) {
    const swapWith = Math.floor(next() * (index + 1));
    [positions[index], positions[swapWith]] = [positions[swapWith], positions[index]];
  }

  const tones = palette === "ramp" ? RAMPS[loader] : LEGACY_PALETTES[loader];
  const colors = Array.from({ length: 25 }, () => tones[Math.floor(next() * tones.length)]);

  // The shuffle already sorted every cell into a per-instance order.
  // Today that ordering is thrown away; reusing it is the whole trick.
  const scatter = positions;
  const bottomUp = [...scatter].sort((a, b) => Math.floor(b / 5) - Math.floor(a / 5));
  const origin = scatter.length > 0 ? scatter[0] : 12;
  const nucleation = [...scatter].sort(
    (a, b) => squaredDistance(origin, a) - squaredDistance(origin, b),
  );

  return { colors, scatter, bottomUp, nucleation };
}

export function fillOrder(shape: BlockShape, order: FillOrder) {
  if (order === "bottomUp") return shape.bottomUp;
  if (order === "nucleation") return shape.nucleation;
  return shape.scatter;
}

/* ---------------------------------------------------------------- crystal */

export type Taper = "sharp" | "gradual";

export interface CrystalOptions {
  /** Lattice edge, in cells. */
  size?: number;
  /** Upper bound on cluster size; the generator picks 2..maxPrisms. */
  maxPrisms?: number;
  taper?: Taper;
}

export interface CrystalShape {
  size: number;
  /** size*size entries; null is transparent, otherwise a ramp index 0..3. */
  cells: (number | null)[];
  /** Prisms that survived the fit pass. */
  prisms: number;
}

interface Prism {
  w: number;
  h: number;
  /** -1, 0 or 1 cell of drift per three rows. */
  lean: number;
  /** Darkens the whole prism by one ramp step, so overlaps read as depth. */
  shade: number;
  gap: number;
  x: number;
}

interface Raster {
  tones: Map<string, number>;
  minX: number;
  minY: number;
  width: number;
  height: number;
}

function rasterize(prisms: Prism[], taper: Taper): Raster {
  let cursor = 0;
  for (const prism of prisms) {
    prism.x = cursor;
    cursor += Math.max(1, prism.w - 1 + prism.gap);
  }

  const tones = new Map<string, number>();
  // Shortest first, so the tallest prism reads as the front of the cluster.
  const ordered = [...prisms].sort((a, b) => a.h - b.h);

  for (const prism of ordered) {
    let previousStart = Number.NaN;
    let previousEnd = Number.NaN;

    for (let row = 0; row < prism.h; row += 1) {
      const fromTop = prism.h - 1 - row;
      const tapered =
        taper === "gradual"
          ? Math.max(1, prism.w - Math.max(0, 2 - fromTop))
          : fromTop === 0
            ? 1
            : fromTop === 1 && prism.w === 3
              ? 2
              : prism.w;
      const inset = Math.floor((prism.w - tapered) / 2);
      const shift = Math.round((prism.lean * row) / 4);

      let start = prism.x + inset + shift;
      let end = start + tapered - 1;

      // Every row must share at least one column with the row below it.
      // Without this a leaning prism sheds its tip as a diagonally-attached
      // floating pixel, and leaves one-cell holes inside the silhouette.
      if (!Number.isNaN(previousStart)) {
        if (start > previousEnd) start = previousEnd;
        if (end < previousStart) end = previousStart;
      }

      const width = end - start + 1;
      for (let column = 0; column < width; column += 1) {
        // Fixed light from the upper left: left face lit, right face in shadow.
        let tone =
          width === 1
            ? 2
            : width === 2
              ? column === 0
                ? 2
                : 1
              : column === 0
                ? 3
                : column === width - 1
                  ? 1
                  : 2;
        if (fromTop <= 1) tone = Math.min(3, tone + 1);
        tone = Math.max(0, Math.min(3, tone - prism.shade));
        tones.set(`${start + column},${-row}`, tone);
      }

      previousStart = start;
      previousEnd = end;
    }
  }

  if (tones.size === 0) return { tones, minX: 0, minY: 0, width: 0, height: 0 };

  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;
  for (const key of tones.keys()) {
    const comma = key.indexOf(",");
    const x = Number(key.slice(0, comma));
    const y = Number(key.slice(comma + 1));
    if (x < minX) minX = x;
    if (x > maxX) maxX = x;
    if (y < minY) minY = y;
    if (y > maxY) maxY = y;
  }

  return { tones, minX, minY, width: maxX - minX + 1, height: maxY - minY + 1 };
}

/**
 * Fills pockets that no path reaches from outside the lattice. Two prisms
 * overlapping at different heights can leave a one-cell gap enclosed on all
 * four sides, which reads as a rendering bug rather than as texture. Gaps that
 * are open to the outside are the shape and are left alone.
 */
function closeHoles(cells: (number | null)[], size: number) {
  const outside = new Set<number>();
  const queue: number[] = [];

  for (let index = 0; index < size * size; index += 1) {
    const x = index % size;
    const y = Math.floor(index / size);
    const onBorder = x === 0 || y === 0 || x === size - 1 || y === size - 1;
    if (onBorder && cells[index] === null) {
      outside.add(index);
      queue.push(index);
    }
  }

  while (queue.length > 0) {
    const index = queue.pop() as number;
    const x = index % size;
    const y = Math.floor(index / size);
    for (const [dx, dy] of [
      [1, 0],
      [-1, 0],
      [0, 1],
      [0, -1],
    ]) {
      const nx = x + dx;
      const ny = y + dy;
      if (nx < 0 || ny < 0 || nx >= size || ny >= size) continue;
      const neighbour = ny * size + nx;
      if (cells[neighbour] !== null || outside.has(neighbour)) continue;
      outside.add(neighbour);
      queue.push(neighbour);
    }
  }

  for (let index = 0; index < size * size; index += 1) {
    if (cells[index] !== null || outside.has(index)) continue;
    const x = index % size;
    const y = Math.floor(index / size);
    let tone = 1;
    for (const [dx, dy] of [
      [1, 0],
      [-1, 0],
      [0, 1],
      [0, -1],
    ]) {
      const nx = x + dx;
      const ny = y + dy;
      if (nx < 0 || ny < 0 || nx >= size || ny >= size) continue;
      const neighbour = cells[ny * size + nx];
      // A crevice sits in shadow, so take the darkest tone around it.
      if (neighbour !== null && neighbour < tone) tone = neighbour;
    }
    cells[index] = tone;
  }
}

export function crystalFor(seed: string, options: CrystalOptions = {}): CrystalShape {
  const size = options.size ?? 9;
  const maxPrisms = options.maxPrisms ?? 4;
  const taper = options.taper ?? "sharp";
  const next = random(hash(seed));

  const count = 2 + Math.floor(next() * Math.max(1, maxPrisms - 1));
  const minHeight = Math.max(3, Math.round(size * 0.34));
  const prisms: Prism[] = [];
  for (let index = 0; index < count; index += 1) {
    const roll = next();
    prisms.push({
      w: roll < 0.28 ? 1 : roll < 0.78 ? 2 : 3,
      h: minHeight + Math.floor(next() * (size - minHeight + 1)),
      lean: Math.floor(next() * 3) - 1,
      shade: next() < 0.45 ? 1 : 0,
      gap: next() < 0.65 ? 0 : 1,
      x: 0,
    });
  }

  // Without a guaranteed peak some seeds produce a cluster of stubs, which at
  // 36px is a couple of specks rather than an icon.
  const peakHeight = Math.round(size * 0.66);
  let peak = prisms[0];
  for (const prism of prisms) if (prism.h > peak.h) peak = prism;
  if (peak.h < peakHeight) peak.h = peakHeight;

  // Drop the shortest prism until the cluster fits the lattice, rather than
  // clipping it — a clipped crystal reads as a rendering bug, a smaller
  // cluster reads as a different crystal.
  let active = prisms;
  let drawn = rasterize(active, taper);
  while (active.length > 1 && (drawn.width > size || drawn.height > size)) {
    let shortest = active[0];
    for (const prism of active) if (prism.h < shortest.h) shortest = prism;
    active = active.filter((prism) => prism !== shortest);
    drawn = rasterize(active, taper);
  }

  const cells: (number | null)[] = new Array(size * size).fill(null);
  const offsetX = Math.floor((size - drawn.width) / 2) - drawn.minX;
  // Bias the odd row down: crystals grow off a surface, they do not float.
  const offsetY = Math.ceil((size - drawn.height) / 2) - drawn.minY;

  for (const [key, tone] of drawn.tones) {
    const comma = key.indexOf(",");
    const x = Number(key.slice(0, comma)) + offsetX;
    const y = Number(key.slice(comma + 1)) + offsetY;
    if (x < 0 || y < 0 || x >= size || y >= size) continue;
    cells[y * size + x] = tone;
  }

  closeHoles(cells, size);

  return { size, cells, prisms: active.length };
}
