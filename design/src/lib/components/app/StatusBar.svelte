<script lang="ts">
  import { app } from "$lib/state.svelte";
  import { cn } from "$lib/lib.js";
  import { Separator } from "$lib/components/ui/separator";
  import RunningInstancesPopover from "./RunningInstancesPopover.svelte";
</script>

<footer
  class={cn(
    "flex h-7 shrink-0 items-center gap-3 border-t px-3 text-micro transition-colors",
    app.runningCount > 0 ? "border-launch bg-launch" : "bg-background",
  )}
  style:color={app.runningCount > 0 ? "var(--launch-foreground)" : "var(--muted-foreground)"}
>
  <span
    class="font-medium"
    style:color={app.runningCount > 0 ? "var(--launch-foreground)" : "var(--foreground)"}
    >Phanerite</span
  >
  <span class="font-mono text-micro">0.1.0-pre</span>
  <Separator
    orientation="vertical"
    class={cn("h-3.5", app.runningCount > 0 && "bg-launch-foreground/30")}
  />
  <span>{app.instances.length} instances</span>
  {#if app.runningCount > 0}
    <Separator orientation="vertical" class="h-3.5 bg-launch-foreground/30" />
    <RunningInstancesPopover />
  {/if}
  {#if app.activeAccount}
    <Separator
      orientation="vertical"
      class={cn("h-3.5", app.runningCount > 0 && "bg-launch-foreground/30")}
    />
    <span class="hidden 2xl:inline">Signed in as {app.activeAccount.username}</span>
  {/if}
</footer>
