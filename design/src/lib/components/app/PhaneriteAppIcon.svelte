<script lang="ts">
  import { cn } from "$lib/lib";

  let { class: className }: { class?: string } = $props();

  const COLORS: Record<string, string> = {
    G: "var(--mc-grass)",
    D: "var(--mc-dirt)",
    E: "transparent",
  };
  const cells = ["GGGGG", "GGGDG", "DDDDD", "DEDDD", "EDEDE"].flatMap((row) => [...row]);
</script>

<div
  class={cn(
    "grid aspect-square auto-rows-fr grid-cols-5 gap-px rounded-[3px] bg-black/25 p-px ring-1 ring-inset ring-black/40",
    className,
  )}
>
  {#each cells as color, index (index)}
    <div class="rounded-[1px]" style={`background-color: ${COLORS[color]}`}></div>
  {/each}
</div>
