<script lang="ts">
  import { crystalFor, hash, random, RAMPS } from "$lib/instance-shapes";

  type CrystalCell = { tone: number; revealAt: number } | null;

  let {
    seed,
    progress,
  }: {
    seed: string;
    progress: number;
  } = $props();

  let host = $state<HTMLDivElement>();
  let width = $state(0);
  let height = $state(0);

  const CELL_SIZE = 6;
  const columns = $derived(Math.max(1, Math.round(width / CELL_SIZE)));
  const rows = $derived(Math.max(1, Math.round(height / CELL_SIZE)));
  const cells = $derived(buildBackdrop(seed, columns, rows));

  function buildBackdrop(seedValue: string, columnCount: number, rowCount: number): CrystalCell[] {
    const field: CrystalCell[] = new Array(columnCount * rowCount).fill(null);
    const next = random(hash(`${seedValue}|launch-backdrop`));
    const crystalCount = Math.max(2, Math.floor(columnCount / 18) + 1);

    for (let crystalIndex = 0; crystalIndex < crystalCount; crystalIndex += 1) {
      const shape = crystalFor(`${seedValue}|launch-crystal|${crystalIndex}`, {
        size: 9,
        maxPrisms: 3,
        taper: "gradual",
      });
      const spread = Math.max(0, columnCount - shape.size);
      const anchorX = Math.round((crystalIndex / Math.max(1, crystalCount - 1)) * spread);
      const x = Math.min(spread, Math.max(0, anchorX + Math.round((next() - 0.5) * 6)));
      let baseY = 0;
      for (let index = 0; index < shape.cells.length; index += 1) {
        if (shape.cells[index] !== null) {
          baseY = Math.max(baseY, Math.floor(index / shape.size));
        }
      }
      // Align the lowest occupied crystal pixel with the backdrop's bottom row.
      // This keeps every crystal grounded even when its generated silhouette is
      // shorter than the full 9 × 9 lattice.
      const y = Math.max(0, rowCount - 1 - baseY);

      for (let index = 0; index < shape.cells.length; index += 1) {
        const tone = shape.cells[index];
        if (tone === null) continue;
        const cellX = x + (index % shape.size);
        const cellY = y + Math.floor(index / shape.size);
        if (cellX < 0 || cellY < 0 || cellX >= columnCount || cellY >= rowCount) continue;

        const heightFromBase = baseY - Math.floor(index / shape.size);
        const revealAt = 8 + heightFromBase * 8 + Math.floor(next() * 7);
        field[cellY * columnCount + cellX] = { tone, revealAt };
      }
    }

    return field;
  }

  $effect(() => {
    if (!host) return;
    const observer = new ResizeObserver(([entry]) => {
      width = entry.contentRect.width;
      height = entry.contentRect.height;
    });
    observer.observe(host);
    return () => observer.disconnect();
  });
</script>

<div
  bind:this={host}
  class="pointer-events-none absolute inset-0 overflow-hidden"
  aria-hidden="true"
>
  <div
    class="launch-crystal-grid grid size-full gap-px"
    style={`grid-template-columns: repeat(${columns}, minmax(0, 1fr)); grid-template-rows: repeat(${rows}, minmax(0, 1fr))`}
  >
    {#each cells as cell, index (index)}
      <div
        class:visible={cell !== null && progress >= cell.revealAt}
        style={cell === null ? undefined : `background-color: ${RAMPS.vanilla[cell.tone]}`}
      ></div>
    {/each}
  </div>
</div>

<style>
  .launch-crystal-grid {
    background-color: oklch(0.62 0.13 155 / 16%);
  }

  .launch-crystal-grid > div {
    opacity: 0;
    transition: opacity 120ms linear;
  }

  .launch-crystal-grid > div.visible {
    opacity: 0.62;
  }

  @media (prefers-reduced-motion: reduce) {
    .launch-crystal-grid > div {
      transition: none;
    }
  }
</style>
