<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Popover from "$lib/components/ui/popover";
  import { app } from "$lib/state.svelte";
  import { LOADER_LABEL } from "$lib/types";
  import { ScrollText, Square } from "@lucide/svelte";
  import InstanceIcon from "./InstanceIcon.svelte";

  let open = $state(false);

  const runningInstances = $derived(
    app.runningIds
      .map((id) => app.instances.find((instance) => instance.id === id))
      .filter((instance) => instance !== undefined),
  );
</script>

<Popover.Root bind:open>
  <Popover.Trigger class="flex items-center gap-1.5 font-medium text-launch-foreground">
    <span class="size-1.5 rounded-full bg-launch-foreground"></span>
    {app.runningCount} running
  </Popover.Trigger>
  <Popover.Content align="end" class="w-80">
    <Popover.Header>
      <Popover.Title>Running instances</Popover.Title>
      <Popover.Description>Stop a game or inspect its live output.</Popover.Description>
    </Popover.Header>
    <div class="flex flex-col gap-1">
      {#each runningInstances as instance (instance.id)}
        <div class="flex items-center gap-2 rounded-md p-2 hover:bg-accent/40">
          <InstanceIcon {instance} class="size-6 shrink-0" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-xs font-medium">{instance.name}</p>
            <p class="truncate text-micro text-muted-foreground">
              MC {instance.mcVersion} · {LOADER_LABEL[instance.loader]}
            </p>
          </div>
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={`Logs for ${instance.name}`}
            onclick={() => {
              open = false;
              app.openLogs(instance.id);
            }}
          >
            <ScrollText />
          </Button>
          <Button
            variant="destructive"
            size="icon-sm"
            aria-label={`Stop ${instance.name}`}
            onclick={() => app.stopInstance(instance.id)}
          >
            <Square />
          </Button>
        </div>
      {/each}
    </div>
  </Popover.Content>
</Popover.Root>
