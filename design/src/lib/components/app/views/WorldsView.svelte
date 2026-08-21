<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { app } from "$lib/state.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Empty from "$lib/components/ui/empty";
  import { ArrowLeft, Ellipsis, Globe, Play, Trash2 } from "@lucide/svelte";
  import { toast } from "svelte-sonner";

  let pendingDelete = $state<string | null>(null);

  function confirmDelete() {
    if (!pendingDelete || !app.activeInstance) return;
    const index = app.activeInstance.worlds.findIndex((world) => world.id === pendingDelete);
    if (index !== -1) app.activeInstance.worlds.splice(index, 1);
    pendingDelete = null;
  }

  function playWorld(name: string) {
    toast.info(`Play world: ${name}`, {
      description: "Launches the instance directly into this world in the real launcher.",
    });
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="shrink-0 border-b p-6 pb-4">
    <Button variant="ghost" size="sm" class="-ml-2" onclick={() => app.setView("instance-detail")}>
      <ArrowLeft data-icon="inline-start" />
      {app.activeInstance?.name ?? "Instance"}
    </Button>
    <div class="mt-3">
      <h1 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
        <Globe class="size-4.5 text-primary" />
        Worlds
        {#if app.activeInstance}
          <Badge variant="secondary">{app.activeInstance.worlds.length}</Badge>
        {/if}
      </h1>
      <p class="mt-0.5 text-xs text-muted-foreground">Saved worlds for the instance you opened.</p>
    </div>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto p-6">
    {#if app.activeInstance?.worlds.length === 0}
      <Empty.Root>
        <Empty.Title>No worlds yet</Empty.Title>
        <Empty.Description>Saved games appear here once you create them.</Empty.Description>
      </Empty.Root>
    {:else if app.activeInstance}
      {@const inst = app.activeInstance}
      <div class="flex flex-col gap-2">
        {#each inst.worlds as world (world.id)}
          <div class="flex items-center gap-3 rounded-md border bg-card p-2.5">
            <div class="flex size-8 shrink-0 items-center justify-center rounded-md bg-secondary">
              <Globe class="size-4 text-muted-foreground" />
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <p class="truncate text-xs font-medium">{world.name}</p>
                <Badge variant="outline">MC {world.version}</Badge>
              </div>
              <p class="mt-0.5 truncate font-mono text-micro text-muted-foreground">
                seed {world.seed} · {world.lastPlayed} · {world.players} player{world.players === 1
                  ? ""
                  : "s"}
              </p>
            </div>
            <Button variant="outline" size="sm" onclick={() => playWorld(world.name)}>
              <Play data-icon="inline-start" />
              Play
            </Button>
            <DropdownMenu.Root>
              <DropdownMenu.Trigger>
                <Button variant="ghost" size="icon-sm" aria-label="World actions"
                  ><Ellipsis /></Button
                >
              </DropdownMenu.Trigger>
              <DropdownMenu.Content align="end" class="w-40">
                <DropdownMenu.Item variant="destructive" onclick={() => (pendingDelete = world.id)}>
                  <Trash2 data-icon="inline-start" />
                  Delete world
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<AlertDialog.Root
  open={pendingDelete !== null}
  onOpenChange={(open) => {
    if (!open) pendingDelete = null;
  }}
>
  <AlertDialog.Content class="max-w-sm">
    <AlertDialog.Header>
      <AlertDialog.Title>Delete this world?</AlertDialog.Title>
      <AlertDialog.Description
        >The world and all its files are removed from disk.</AlertDialog.Description
      >
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel><Button variant="outline">Cancel</Button></AlertDialog.Cancel>
      <AlertDialog.Action onclick={confirmDelete}
        ><Button variant="destructive">Delete</Button></AlertDialog.Action
      >
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
