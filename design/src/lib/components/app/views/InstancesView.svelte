<script lang="ts">
  import { app } from "$lib/state.svelte";
  import { cn } from "$lib/lib.js";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Empty from "$lib/components/ui/empty";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import { LOADER_LABEL, type Loader } from "$lib/types";
  import {
    Copy,
    Ellipsis,
    Flame,
    Layers,
    LoaderCircle,
    Package,
    Play,
    Square,
    Star,
    Trash2,
  } from "@lucide/svelte";
  import { Select as SelectPrimitive } from "bits-ui";
  import { toast } from "svelte-sonner";
  import InstanceCreateDialog from "../InstanceCreateDialog.svelte";
  import ConfirmDestructive from "../ConfirmDestructive.svelte";
  import InstanceIcon from "../InstanceIcon.svelte";

  let q = $state("");
  let loaderFilter = $state("all");
  let pendingDelete = $state<string | null>(null);
  let confirmDeleteOpen = $state(false);

  const LOADER_FILTERS: { value: string; label: string }[] = [
    { value: "all", label: "All loaders" },
    { value: "vanilla", label: "Vanilla" },
    { value: "fabric", label: "Fabric" },
    { value: "forge", label: "Forge" },
    { value: "neoforge", label: "NeoForge" },
  ];

  const filtered = $derived(
    app.instances.filter((i) => {
      const matchQ = !q.trim() || i.name.toLowerCase().includes(q.trim().toLowerCase());
      const matchLoader = loaderFilter === "all" || i.loader === (loaderFilter as Loader);
      return matchQ && matchLoader;
    }),
  );

  function openDetail(id: string) {
    app.openInstanceDetail(id);
  }

  function duplicate(id: string) {
    app.duplicateInstance(id);
    toast.success("Instance duplicated");
  }

  function confirmDelete() {
    if (!pendingDelete) return;
    const inst = app.instances.find((i) => i.id === pendingDelete);
    const name = inst?.name ?? "";
    app.deleteInstance(pendingDelete);
    toast.success(`Deleted “${name}”`);
    pendingDelete = null;
    confirmDeleteOpen = false;
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="shrink-0 border-b p-6 pb-4">
    <div class="flex items-center justify-between gap-4">
      <div class="min-w-0">
        <h1 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
          <Layers class="size-4.5 text-primary" />
          Instances
          <Badge variant="secondary" class="ml-1">{app.instances.length}</Badge>
        </h1>
        <p class="mt-0.5 text-xs text-muted-foreground">
          Each instance is an isolated game installation with its own mods, packs and settings.
          Launch several side by side.
        </p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <Input bind:value={q} placeholder="Search instances…" class="h-7 w-48 text-xs" />
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
        <InstanceCreateDialog />
      </div>
    </div>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto p-6">
    {#if filtered.length === 0}
      <Empty.Root>
        <Empty.Title>No instances found</Empty.Title>
        <Empty.Description>
          {app.instances.length === 0
            ? "Create your first instance to get started."
            : "Nothing matches the current search or loader filter."}
        </Empty.Description>
      </Empty.Root>
    {:else}
      <div class="flex flex-col gap-2">
        {#each filtered as inst (inst.id)}
          <div
            role="button"
            tabindex="0"
            class="group flex cursor-pointer items-center gap-3 rounded-md border bg-card p-3 transition-colors hover:bg-accent/40"
            onclick={() => openDetail(inst.id)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                openDetail(inst.id);
              }
            }}
          >
            <InstanceIcon instance={inst} class="size-9 shrink-0" />
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <p class="truncate text-sm font-medium">{inst.name}</p>
                {#if inst.aphanite}
                  <Flame class="size-3.5 shrink-0 text-red-500" />
                {/if}
                {#if app.runningIds.includes(inst.id)}
                  <Badge class="gap-1">
                    <span class="size-1.5 rounded-full bg-primary-foreground"></span>
                    Running
                  </Badge>
                {:else if app.getLaunch(inst.id)}
                  <Badge variant="secondary" class="gap-1">
                    <LoaderCircle class="size-3 animate-spin" />
                    Starting
                  </Badge>
                {/if}
              </div>
              <p class="mt-0.5 truncate text-micro text-muted-foreground">
                {LOADER_LABEL[inst.loader]} · MC {inst.mcVersion} ·{" "}
                {inst.lastPlayed ?? "never played"} · {inst.playCount} launches
              </p>
            </div>

            <div class="flex shrink-0 items-center gap-1">
              {#if app.getLaunch(inst.id)}
                <Button
                  size="icon-sm"
                  variant="secondary"
                  aria-label={`Cancel launch of ${inst.name}`}
                  onclick={(e) => {
                    e.stopPropagation();
                    app.cancelLaunch(inst.id);
                  }}
                >
                  <LoaderCircle class="animate-spin" />
                </Button>
              {:else if app.runningIds.includes(inst.id)}
                <Button
                  size="icon-sm"
                  variant="destructive"
                  aria-label={`Stop ${inst.name}`}
                  onclick={(e) => {
                    e.stopPropagation();
                    app.stopInstance(inst.id);
                  }}
                >
                  <Square />
                </Button>
              {:else}
                <Button
                  size="icon-sm"
                  variant="outline"
                  aria-label={`Play ${inst.name}`}
                  onclick={(e) => {
                    e.stopPropagation();
                    app.startLaunch(inst.id);
                  }}
                >
                  <Play />
                </Button>
              {/if}
              <Button
                size="icon-sm"
                variant={inst.favorite ? "secondary" : "ghost"}
                aria-label={inst.favorite
                  ? `Remove ${inst.name} from favorites`
                  : `Add ${inst.name} to favorites`}
                onclick={(e) => {
                  e.stopPropagation();
                  app.toggleFavorite(inst.id);
                }}
              >
                <Star class={cn("size-3.5", inst.favorite && "fill-current")} />
              </Button>
              <DropdownMenu.Root>
                <DropdownMenu.Trigger onclick={(e) => e.stopPropagation()}>
                  <Button variant="ghost" size="icon-sm" aria-label="Instance actions">
                    <Ellipsis />
                  </Button>
                </DropdownMenu.Trigger>
                <DropdownMenu.Content align="end" class="w-44">
                  {#if app.getLaunch(inst.id)}
                    <DropdownMenu.Item onclick={() => app.cancelLaunch(inst.id)}>
                      <LoaderCircle data-icon="inline-start" />
                      Cancel launch
                    </DropdownMenu.Item>
                  {:else}
                    <DropdownMenu.Item
                      onclick={() => {
                        if (!app.runningIds.includes(inst.id)) app.startLaunch(inst.id);
                      }}
                    >
                      <Play data-icon="inline-start" />
                      Launch
                    </DropdownMenu.Item>
                  {/if}
                  <DropdownMenu.Item onclick={() => app.openInstanceDetail(inst.id)}>
                    <Layers data-icon="inline-start" />
                    Instance details
                  </DropdownMenu.Item>
                  <DropdownMenu.Item onclick={() => duplicate(inst.id)}>
                    <Copy data-icon="inline-start" />
                    Duplicate
                  </DropdownMenu.Item>
                  <DropdownMenu.Item
                    onclick={() =>
                      toast.info("Export", {
                        description: "Exports a portable .zip of this instance.",
                      })}
                  >
                    <Package data-icon="inline-start" />
                    Export…
                  </DropdownMenu.Item>
                  <DropdownMenu.Separator />
                  <DropdownMenu.Item
                    variant="destructive"
                    onclick={() => {
                      pendingDelete = inst.id;
                      confirmDeleteOpen = true;
                    }}
                  >
                    <Trash2 data-icon="inline-start" />
                    Delete
                  </DropdownMenu.Item>
                </DropdownMenu.Content>
              </DropdownMenu.Root>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<ConfirmDestructive
  bind:open={confirmDeleteOpen}
  title="Delete this instance?"
  consequence="This removes the instance and its files permanently."
  onconfirm={confirmDelete}
/>
