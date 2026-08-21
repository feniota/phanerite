<script lang="ts">
  import { cn } from "$lib/lib";
  import LabBlock from "./LabBlock.svelte";
  import LabCrystal from "./LabCrystal.svelte";
  import type { FillOrder, Loader, Taper } from "./shapes";

  let {
    seed,
    loader,
    order = "scatter",
    progress = null,
    ghost = true,
    palette = "ramp",
    failed = false,
    /** Once true the block dissolves and the instance's crystal takes its place. */
    crystallized = false,
    lattice = 9,
    maxPrisms = 4,
    taper = "sharp",
    gap = true,
    class: className,
  }: {
    seed: string;
    loader: Loader;
    order?: FillOrder;
    progress?: number | null;
    ghost?: boolean;
    palette?: "ramp" | "legacy";
    failed?: boolean;
    crystallized?: boolean;
    lattice?: number;
    maxPrisms?: number;
    taper?: Taper;
    gap?: boolean;
    class?: string;
  } = $props();
</script>

<!--
  A cross-fade with a touch of scale, not a real morph. A true lattice morph
  costs far more than the 220ms it would be visible for.
-->
<div class={cn("relative", className)}>
  <div
    class="absolute inset-0 transition-[opacity,transform] duration-200 ease-out motion-reduce:transition-none"
    style:opacity={crystallized ? "0" : "1"}
    style:transform={crystallized ? "scale(0.94)" : "scale(1)"}
  >
    <LabBlock {seed} {loader} {order} {progress} {ghost} {palette} {failed} class="size-full" />
  </div>
  <div
    class="absolute inset-0 transition-[opacity,transform] duration-200 ease-out motion-reduce:transition-none"
    style:opacity={crystallized ? "1" : "0"}
    style:transform={crystallized ? "scale(1)" : "scale(1.06)"}
  >
    <LabCrystal {seed} {loader} {lattice} {maxPrisms} {taper} {gap} class="size-full" />
  </div>
</div>
