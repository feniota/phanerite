<script lang="ts">
  import { cn } from "$lib/lib";

  let {
    skinUrl,
    alt,
    class: className,
  }: { skinUrl: string; alt: string; class?: string } = $props();
  let canvas = $state<HTMLCanvasElement | null>(null);
  let failed = $state(false);

  $effect(() => {
    if (!canvas) return;
    failed = false;
    canvas.getContext("2d")?.clearRect(0, 0, 16, 16);
    const image = new Image();
    image.crossOrigin = "anonymous";
    image.src = skinUrl;
    image.onload = () => {
      failed = false;
      const context = canvas?.getContext("2d");
      if (!context) return;
      context.imageSmoothingEnabled = false;
      context.clearRect(0, 0, 16, 16);
      context.drawImage(image, 8, 8, 8, 8, 0, 0, 16, 16);
      context.drawImage(image, 40, 8, 8, 8, 0, 0, 16, 16);
    };
    image.onerror = () => {
      failed = true;
    };
  });
</script>

{#if failed}
  <div
    class={cn(
      "flex aspect-square items-center justify-center rounded-md bg-secondary text-sm font-semibold text-secondary-foreground",
      className,
    )}
  >
    {alt.slice(0, 1).toUpperCase()}
  </div>
{:else}
  <canvas
    width="16"
    height="16"
    bind:this={canvas}
    class={cn("pixelated block aspect-square", className)}
  ></canvas>
{/if}
