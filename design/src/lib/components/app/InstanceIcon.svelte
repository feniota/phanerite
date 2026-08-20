<script lang="ts">
  import { cn } from "$lib/lib";
  import { crystalFor, RAMPS } from "$lib/instance-shapes";
  import type { MCInstance } from "$lib/types";

  let { instance, class: className }: { instance: MCInstance; class?: string } = $props();

  const seed = $derived(`${instance.name}|${instance.mcVersion}|${instance.loader}`);
  const shape = $derived(
    crystalFor(seed, {
      size: 9,
      maxPrisms: 3,
      taper: "gradual",
    }),
  );
  const ramp = $derived(RAMPS[instance.loader]);
</script>

<div class={cn("instance-crystal shrink-0", className)} style="container-type: inline-size">
  <div
    class="instance-crystal-grid grid size-full auto-rows-fr gap-px"
    style={`grid-template-columns: repeat(${shape.size}, minmax(0, 1fr))`}
  >
    {#each shape.cells as tone, index (index)}
      <div style={tone === null ? undefined : `background-color: ${ramp[tone]}`}></div>
    {/each}
  </div>
</div>

<style>
  @container (max-width: 23px) {
    .instance-crystal-grid {
      gap: 0;
    }
  }
</style>
