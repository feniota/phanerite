<script lang="ts">
  import { cn } from "$lib/lib";
  import { blockFor, fillOrder, type FillOrder, type Loader } from "./shapes";

  let {
    seed,
    loader,
    palette = "ramp",
    order = "scatter",
    /** 0..100 while launching, or null for the finished block. */
    progress = null,
    ghost = true,
    failed = false,
    class: className,
  }: {
    seed: string;
    loader: Loader;
    palette?: "ramp" | "legacy";
    order?: FillOrder;
    progress?: number | null;
    ghost?: boolean;
    failed?: boolean;
    class?: string;
  } = $props();

  const shape = $derived(blockFor(seed, loader, palette));
  const sequence = $derived(fillOrder(shape, order));
  const filled = $derived(
    progress === null ? sequence.length : Math.round((progress / 100) * sequence.length),
  );
  const lit = $derived(new Set(sequence.slice(0, filled)));
  const leading = $derived(progress === null || filled === 0 ? -1 : sequence[filled - 1]);

  function cellStyle(color: string, index: number) {
    if (lit.has(index)) {
      if (failed && index === leading) return "background-color: var(--destructive)";
      return `background-color: ${color}`;
    }
    // Unlit: either pre-visible and dim, a neutral ghost of the silhouette, or nothing.
    if (order === "dimLit") return `background-color: ${color}; opacity: 0.22`;
    if (ghost) return "background-color: oklch(1 0 0 / 7%)";
    return undefined;
  }
</script>

<div
  class={cn(
    "grid aspect-square auto-rows-fr grid-cols-5 gap-px rounded-[3px] bg-black/25 p-px ring-1 ring-black/40 ring-inset",
    className,
  )}
  role="progressbar"
  aria-valuenow={progress === null ? 100 : Math.round(progress)}
  aria-valuemin={0}
  aria-valuemax={100}
>
  {#each shape.colors as color, index (index)}
    <div
      class="cell rounded-[1px]"
      class:flash={index === leading && !failed}
      style={cellStyle(color, index)}
      aria-hidden="true"
    ></div>
  {/each}
</div>

<style>
  .cell {
    transition:
      background-color 140ms linear,
      opacity 140ms linear,
      filter 280ms ease-out;
  }

  /* One frame of highlight on the cell that just landed, decaying on its own. */
  .flash {
    filter: brightness(1.65);
  }

  @media (prefers-reduced-motion: reduce) {
    .cell {
      transition: none;
    }
    .flash {
      filter: none;
    }
  }
</style>
