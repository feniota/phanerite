<script lang="ts">
  import { cn } from "$lib/lib";
  import { crystalFor, RAMPS, type Loader, type Taper } from "./shapes";

  let {
    seed,
    loader,
    lattice = 9,
    maxPrisms = 4,
    taper = "sharp",
    /** Cell gaps read as texture at icon sizes and as mush below ~24px. */
    gap = true,
    class: className,
  }: {
    seed: string;
    loader: Loader;
    lattice?: number;
    maxPrisms?: number;
    taper?: Taper;
    gap?: boolean;
    class?: string;
  } = $props();

  const shape = $derived(crystalFor(seed, { size: lattice, maxPrisms, taper }));
  const ramp = $derived(RAMPS[loader]);
</script>

<!--
  No plate, no ring, no padding: transparent background, irregular silhouette,
  no bounding box. The same three properties every real modpack icon has.
-->
<div
  class={cn("grid aspect-square auto-rows-fr", gap && "gap-px", className)}
  style={`grid-template-columns: repeat(${shape.size}, minmax(0, 1fr))`}
>
  {#each shape.cells as tone, index (index)}
    <div style={tone === null ? undefined : `background-color: ${ramp[tone]}`}></div>
  {/each}
</div>
