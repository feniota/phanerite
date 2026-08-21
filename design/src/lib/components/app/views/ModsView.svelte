<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { app } from "$lib/state.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Empty from "$lib/components/ui/empty";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import { Switch } from "$lib/components/ui/switch";
  import { type Loader } from "$lib/types";
  import { ArrowLeft, Box, Ellipsis, FolderOpen, Puzzle, Search, Trash2 } from "@lucide/svelte";
  import { Select as SelectPrimitive } from "bits-ui";
  import { toast } from "svelte-sonner";
  import AddResourcesDialog from "../AddResourcesDialog.svelte";

  const LOADER_FILTERS: { value: string; label: string }[] = [
    { value: "all", label: "All loaders" },
    { value: "fabric", label: "Fabric" },
    { value: "forge", label: "Forge" },
    { value: "neoforge", label: "NeoForge" },
  ];

  let q = $state("");
  let loaderFilter = $state("all");
  let pendingDelete = $state<string | null>(null);
  let addModsOpen = $state(false);

  const enabledCount = $derived(app.activeInstance?.mods.filter((m) => m.enabled).length ?? 0);

  const filtered = $derived(
    app.activeInstance?.mods.filter((m) => {
      const query = q.trim().toLowerCase();
      const matchQ =
        !query || m.name?.toLowerCase().includes(query) || m.fileName.toLowerCase().includes(query);
      const matchLoader = loaderFilter === "all" || m.loader === (loaderFilter as Loader);
      return matchQ && matchLoader;
    }) ?? [],
  );

  function confirmDelete() {
    if (!pendingDelete || !app.activeInstance) return;
    const m = app.activeInstance.mods.find((x) => x.id === pendingDelete);
    app.deleteMod(app.activeInstance.id, pendingDelete);
    toast.success(`Deleted “${m?.name ?? m?.fileName ?? "mod"}”`);
    pendingDelete = null;
  }

  function openFile(fileName: string) {
    toast.info(`Open file: ${fileName}`, {
      description: "Reveals the .jar in the mods folder.",
    });
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="shrink-0 border-b p-6 pb-4">
    <Button variant="ghost" size="sm" class="-ml-2" onclick={() => app.setView("instance-detail")}>
      <ArrowLeft data-icon="inline-start" />
      {app.activeInstance?.name ?? "Instance"}
    </Button>
    <div class="mt-3 flex items-center justify-between gap-4">
      <div class="min-w-0">
        <h1 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
          <Puzzle class="size-4.5 text-primary" />
          Mods
          {#if app.activeInstance}
            <Badge variant="secondary" class="ml-1">
              {app.activeInstance.name} · {enabledCount}/{app.activeInstance.mods.length} enabled
            </Badge>
          {/if}
        </h1>
        <p class="mt-0.5 text-xs text-muted-foreground">
          Manage the isolated mods folder for the instance you opened.
        </p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <div class="relative">
          <Search class="absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input bind:value={q} placeholder="Search mods…" class="h-7 w-44 pl-7 text-xs" />
        </div>
        <Select.Root bind:value={loaderFilter} type="single">
          <Select.Trigger class="h-7 w-32 text-xs">
            <SelectPrimitive.Value placeholder="All loaders" />
          </Select.Trigger>
          <Select.Content>
            {#each LOADER_FILTERS as f (f.value)}
              <Select.Item value={f.value}>{f.label}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
        <AddResourcesDialog mode="mod" bind:open={addModsOpen} />
      </div>
    </div>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto p-6">
    {#if !app.activeInstance}
      <Empty.Root>
        <Empty.Title>No instance selected</Empty.Title>
        <Empty.Description>
          Pick an instance from the dropdown above to see its mods.
        </Empty.Description>
      </Empty.Root>
    {:else if filtered.length === 0}
      <Empty.Root>
        <Empty.Title>
          {app.activeInstance.mods.length === 0 ? "No mods installed" : "No mods match"}
        </Empty.Title>
        <Empty.Description>
          {#if app.activeInstance.mods.length === 0}
            {app.activeInstance.name} is a clean installation. Drop some .jar files to make it interesting.
          {:else}
            Nothing matches the current search or loader filter.
          {/if}
        </Empty.Description>
        {#if app.activeInstance.mods.length === 0}
          <Button class="mt-1" onclick={() => (addModsOpen = true)}>Add mods</Button>
        {/if}
      </Empty.Root>
    {:else if app.activeInstance}
      {@const inst = app.activeInstance}
      <div class="flex flex-col gap-2">
        {#each filtered as mod (mod.id)}
          <div
            class="flex items-center gap-3 rounded-md border bg-card p-2.5 transition-colors hover:bg-accent/30"
          >
            <div class="flex size-8 shrink-0 items-center justify-center rounded-md bg-secondary">
              <Box class="size-4 text-muted-foreground" />
            </div>
            <div class="min-w-0 flex-1">
              {#if mod.name}
                <p class="truncate text-xs font-medium">{mod.name}</p>
                <p class="mt-0.5 truncate text-micro text-muted-foreground">
                  {mod.version ? `${mod.version} · ${mod.fileName}` : mod.fileName}
                </p>
              {:else}
                <p class="truncate text-xs font-medium">{mod.fileName}</p>
              {/if}
            </div>
            <Switch
              checked={mod.enabled}
              onCheckedChange={(c) => app.setModEnabled(inst.id, mod.id, c)}
              aria-label={`Toggle ${mod.name ?? mod.fileName}`}
            />
            <DropdownMenu.Root>
              <DropdownMenu.Trigger>
                <Button variant="ghost" size="icon-sm" aria-label="Mod actions">
                  <Ellipsis />
                </Button>
              </DropdownMenu.Trigger>
              <DropdownMenu.Content align="end" class="w-44">
                <DropdownMenu.Item onclick={() => app.setModEnabled(inst.id, mod.id, !mod.enabled)}>
                  {mod.enabled ? "Disable" : "Enable"}
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => openFile(mod.fileName)}>
                  <FolderOpen data-icon="inline-start" />
                  Open file
                </DropdownMenu.Item>
                <DropdownMenu.Separator />
                <DropdownMenu.Item variant="destructive" onclick={() => (pendingDelete = mod.id)}>
                  <Trash2 data-icon="inline-start" />
                  Delete
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
  onOpenChange={(o) => {
    if (!o) pendingDelete = null;
  }}
>
  <AlertDialog.Content class="max-w-sm">
    <AlertDialog.Header>
      <AlertDialog.Title>Delete this mod?</AlertDialog.Title>
      <AlertDialog.Description>
        The .jar file is removed from the instance. Some worlds may fail to load without it.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>
        <Button variant="outline">Keep mod</Button>
      </AlertDialog.Cancel>
      <AlertDialog.Action onclick={confirmDelete}>
        <Button variant="destructive">Delete</Button>
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
