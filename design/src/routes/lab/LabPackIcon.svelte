<script lang="ts">
  import { cn } from "$lib/lib";

  let { variant, class: className }: { variant: number; class?: string } = $props();

  const PACK_ICONS: Record<number, string> = {
    0: "/lab/modpack-icons/prominence-2-hasturian-era-4.0.webp",
    1: "/lab/modpack-icons/cottage-witch.png",
    2: "/lab/modpack-icons/all-the-mods-10.png",
    3: "/lab/modpack-icons/fabulously-optimized.png",
  };
</script>

<!--
  Real third-party modpack artwork. These are intentionally kept outside the
  theme system because Phanerite does not control their branding.
-->
<img
  src={PACK_ICONS[variant] ?? PACK_ICONS[0]}
  alt=""
  aria-hidden="true"
  class={cn("shrink-0 object-contain", className)}
/>
