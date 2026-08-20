<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent } from "$lib/components/ui/card";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Field from "$lib/components/ui/field";
  import { Separator } from "$lib/components/ui/separator";
  import * as Select from "$lib/components/ui/select";
  import { Slider } from "$lib/components/ui/slider";
  import { app } from "$lib/state.svelte";
  import { cn } from "$lib/lib.js";
  import { LOADER_LABEL } from "$lib/types";
  import { Select as SelectPrimitive } from "bits-ui";
  import {
    ArrowLeft,
    ChevronRight,
    Copy,
    CircleAlert,
    Ellipsis,
    Flame,
    FolderOpen,
    Globe,
    LoaderCircle,
    ScrollText,
    Package,
    Palette,
    Play,
    Puzzle,
    Settings,
    Sparkles,
    Star,
    Square,
    Trash2,
  } from "@lucide/svelte";
  import { toast } from "svelte-sonner";
  import InstanceIcon from "../InstanceIcon.svelte";
  import ConfirmDestructive from "../ConfirmDestructive.svelte";
  import LaunchCrystalBackdrop from "../LaunchCrystalBackdrop.svelte";

  const instance = $derived(app.activeInstance);
  const running = $derived(instance ? app.runningIds.includes(instance.id) : false);
  const launch = $derived(instance ? app.getLaunch(instance.id) : null);
  const starting = $derived(launch !== null);
  const javaRuntime = $derived(instance ? app.getJavaRuntime(instance.javaRuntimeId) : null);

  let confirmDelete = $state(false);

  function openFolder() {
    toast.info("Open folder", {
      description: "Reveals the instance directory in your file manager.",
    });
  }

  function duplicate() {
    if (!instance) return;
    app.duplicateInstance(instance.id);
    toast.success(`Duplicated “${instance.name}”`);
  }

  function confirmDeleteInstance() {
    if (!instance) return;
    const name = instance.name;
    app.deleteInstance(instance.id);
    toast.success(`Deleted “${name}”`);
  }

  function openConfiguration(id: string, view: "mods" | "packs" | "shaders" | "worlds") {
    app.setActiveInstance(id);
    app.setView(view);
  }

  function selectJavaRuntime(id: string, runtimeId: string) {
    const inst = app.instances.find((item) => item.id === id);
    const runtime = app.getJavaRuntime(runtimeId);
    if (!inst || !runtime) return;
    inst.javaRuntimeId = runtimeId;
    inst.java = `${runtime.version}`;
  }
</script>

{#if instance}
  {@const inst = instance}
  <div class="flex h-full min-h-0 flex-col">
    <header class="shrink-0 border-b p-6 pb-4">
      <Button variant="ghost" size="sm" class="-ml-2" onclick={() => app.closeInstanceDetail()}>
        <ArrowLeft data-icon="inline-start" />
        Back
      </Button>
      <div class="mt-3 flex items-center justify-between gap-4">
        <div class="flex min-w-0 items-center gap-3">
          <InstanceIcon instance={inst} class="size-11 shrink-0" />
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <h1 class="truncate text-lg font-semibold tracking-tight">{inst.name}</h1>
              {#if inst.aphanite}
                <Flame class="size-4 shrink-0 text-red-500" />
              {/if}
              {#if running}
                <Badge class="gap-1">
                  <span class="size-1.5 rounded-full bg-primary-foreground"></span>
                  Running
                </Badge>
              {:else if starting}
                <Badge variant="secondary" class="gap-1">
                  <LoaderCircle class="size-3 animate-spin" />
                  Starting
                </Badge>
              {/if}
            </div>
            <p class="mt-0.5 truncate text-xs text-muted-foreground">{inst.description}</p>
          </div>
        </div>
        <Button
          variant={inst.favorite ? "secondary" : "outline"}
          size="icon"
          class="shrink-0"
          aria-label={inst.favorite
            ? `Remove ${inst.name} from favorites`
            : `Add ${inst.name} to favorites`}
          onclick={() => app.toggleFavorite(inst.id)}
        >
          <Star class={cn("size-4", inst.favorite && "fill-current")} />
        </Button>
      </div>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto p-6">
      <div class="flex flex-col gap-6">
        <div class="flex flex-wrap gap-1.5">
          <Badge variant="outline">MC {inst.mcVersion}</Badge>
          <Badge variant="secondary">
            {LOADER_LABEL[inst.loader]}{inst.loader === "vanilla" ? "" : ` ${inst.loaderVersion}`}
          </Badge>
          <Badge variant="outline">Java {javaRuntime?.version ?? inst.java}</Badge>
        </div>

        {#if inst.lastCrashId}
          {@const crashReport = app.crashReports.find((report) => report.id === inst.lastCrashId)}
          {#if crashReport}
            <Card class="border-destructive/40">
              <CardContent class="flex items-center gap-3">
                <CircleAlert class="size-4 shrink-0 text-destructive" />
                <div class="min-w-0 flex-1">
                  <p class="text-sm font-medium text-destructive">
                    Last launch crashed · {crashReport.when}
                  </p>
                  <p class="mt-0.5 text-xs text-muted-foreground">
                    Exit code {crashReport.exitCode}
                  </p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onclick={() => app.openCrashReport(crashReport.id)}
                >
                  View report
                  <ChevronRight data-icon="inline-end" />
                </Button>
              </CardContent>
            </Card>
          {/if}
        {/if}

        <Card>
          <CardContent class="p-0">
            <button
              type="button"
              onclick={() => openConfiguration(inst.id, "mods")}
              class="flex w-full items-center gap-3 p-4 text-left transition-colors hover:bg-accent/40"
            >
              <span
                class="flex size-9 shrink-0 items-center justify-center rounded-md bg-secondary"
              >
                <Puzzle class="size-4 text-muted-foreground" />
              </span>
              <span class="min-w-0 flex-1">
                <span class="block text-sm font-medium">Mods</span>
                <span class="mt-0.5 block text-xs text-muted-foreground">
                  {inst.mods.filter((mod) => mod.enabled).length}/{inst.mods.length} enabled
                </span>
              </span>
              <ChevronRight class="size-4 shrink-0 text-muted-foreground" />
            </button>
            <Separator />
            <button
              type="button"
              onclick={() => openConfiguration(inst.id, "packs")}
              class="flex w-full items-center gap-3 p-4 text-left transition-colors hover:bg-accent/40"
            >
              <span
                class="flex size-9 shrink-0 items-center justify-center rounded-md bg-secondary"
              >
                <Palette class="size-4 text-muted-foreground" />
              </span>
              <span class="min-w-0 flex-1">
                <span class="block text-sm font-medium">Resource packs</span>
                <span class="mt-0.5 block text-xs text-muted-foreground">
                  {inst.resourcePacks.length} installed
                </span>
              </span>
              <ChevronRight class="size-4 shrink-0 text-muted-foreground" />
            </button>
            <Separator />
            <button
              type="button"
              onclick={() => openConfiguration(inst.id, "shaders")}
              class="flex w-full items-center gap-3 p-4 text-left transition-colors hover:bg-accent/40"
            >
              <span
                class="flex size-9 shrink-0 items-center justify-center rounded-md bg-secondary"
              >
                <Sparkles class="size-4 text-muted-foreground" />
              </span>
              <span class="min-w-0 flex-1">
                <span class="block text-sm font-medium">Shader packs</span>
                <span class="mt-0.5 block text-xs text-muted-foreground">
                  {inst.shaderPacks.length} installed
                </span>
              </span>
              <ChevronRight class="size-4 shrink-0 text-muted-foreground" />
            </button>
            <Separator />
            <button
              type="button"
              onclick={() => openConfiguration(inst.id, "worlds")}
              class="flex w-full items-center gap-3 p-4 text-left transition-colors hover:bg-accent/40"
            >
              <span
                class="flex size-9 shrink-0 items-center justify-center rounded-md bg-secondary"
              >
                <Globe class="size-4 text-muted-foreground" />
              </span>
              <span class="min-w-0 flex-1">
                <span class="block text-sm font-medium">Worlds</span>
                <span class="mt-0.5 block text-xs text-muted-foreground">
                  {inst.worlds.length} saved
                </span>
              </span>
              <ChevronRight class="size-4 shrink-0 text-muted-foreground" />
            </button>
          </CardContent>
        </Card>

        <Card>
          <CardContent>
            <h2 class="text-sm font-semibold">Instance information</h2>
            <dl class="mt-4 flex flex-col gap-3 text-sm">
              {#each [["Game version", inst.mcVersion], ["Mod loader", `${LOADER_LABEL[inst.loader]} ${inst.loader === "vanilla" ? "" : inst.loaderVersion}`], ["Created", inst.createdAt], ["Last played", inst.lastPlayed ?? "Never"], ["Launches", `${inst.playCount}`]] as [label, value] (label)}
                <div class="flex items-center justify-between gap-3">
                  <dt class="text-muted-foreground">{label}</dt>
                  <dd class="truncate text-right font-medium">{value}</dd>
                </div>
              {/each}
            </dl>
            <Field.Field class="mt-5">
              <Field.FieldLabel>Java runtime</Field.FieldLabel>
              <Select.Root
                type="single"
                value={inst.javaRuntimeId}
                onValueChange={(runtimeId) => selectJavaRuntime(inst.id, runtimeId)}
              >
                <Select.Trigger class="w-full text-xs">
                  <SelectPrimitive.Value placeholder="Select a Java runtime" />
                </Select.Trigger>
                <Select.Content>
                  {#each app.javaRuntimes as runtime (runtime.id)}
                    <Select.Item value={runtime.id}>
                      {runtime.name} (Java {runtime.version}){runtime.managed ? " · Managed" : ""}
                    </Select.Item>
                  {/each}
                </Select.Content>
              </Select.Root>
              <Field.FieldDescription>{javaRuntime?.path}</Field.FieldDescription>
            </Field.Field>
          </CardContent>
        </Card>

        <Card>
          <CardContent>
            <Field.Field>
              {@const launchSettings = app.getLaunchSettings(inst)}
              <div class="flex items-center justify-between gap-3">
                <Field.FieldLabel>
                  Maximum memory, {launchSettings.memory} GB
                </Field.FieldLabel>
                {#if inst.launchOverrides?.memory !== undefined}
                  <Badge variant="secondary">Overridden</Badge>
                {:else}
                  <span class="text-micro text-muted-foreground">Inherited from global</span>
                {/if}
              </div>
              <Slider
                type="single"
                value={launchSettings.memory}
                min={2}
                max={16}
                step={1}
                onValueChange={(value) => app.setLaunchOverride(inst.id, "memory", value)}
              />
              {#if inst.launchOverrides?.memory !== undefined}
                <Button
                  variant="ghost"
                  size="xs"
                  class="self-start"
                  onclick={() => app.clearLaunchOverride(inst.id, "memory")}
                >
                  Reset to global
                </Button>
              {/if}
            </Field.Field>
          </CardContent>
        </Card>
      </div>
    </div>
    <footer class="shrink-0 border-t bg-card p-4">
      <div class="flex items-center gap-2">
        <button
          type="button"
          class={cn(
            "relative isolate flex h-14 min-w-0 flex-1 items-center justify-center gap-3 overflow-hidden rounded-md text-sm font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-card",
            running
              ? "border border-destructive/20 bg-destructive/10 text-destructive hover:bg-destructive/20"
              : starting
                ? "bg-black text-launch-foreground hover:bg-black/90"
                : "bg-launch text-launch-foreground hover:bg-launch/85",
          )}
          aria-label={`${running ? "Stop" : starting ? "Cancel launch of" : "Play"} ${inst.name}`}
          onclick={() =>
            running
              ? app.stopInstance(inst.id)
              : starting
                ? app.cancelLaunch(inst.id)
                : app.startLaunch(inst.id)}
        >
          {#if starting}
            <LaunchCrystalBackdrop
              seed={`${inst.name}|${inst.mcVersion}|${inst.loader}`}
              progress={launch?.progress ?? 0}
            />
          {/if}
          {#if running}
            <Square class="relative z-10 size-5 fill-current" />
            <span class="relative z-10">Stop {inst.name}</span>
          {:else if starting}
            <Square class="relative z-10 size-4" />
            <span class="relative z-10">Cancel launch</span>
          {:else}
            <Play class="relative z-10 size-5 fill-current" />
            <span class="relative z-10">Play {inst.name}</span>
          {/if}
        </button>
        <Button
          variant="outline"
          size="icon-lg"
          class="size-14"
          aria-label="Instance launch settings"
          onclick={() => app.setView("launch-settings")}
        >
          <Settings />
        </Button>
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            <Button variant="outline" size="icon-lg" class="size-14" aria-label="Instance actions">
              <Ellipsis />
            </Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="end" class="w-44">
            <DropdownMenu.Item onclick={openFolder}>
              <FolderOpen data-icon="inline-start" />
              Open folder
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={duplicate}>
              <Copy data-icon="inline-start" />
              Duplicate
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={() => app.openLogs(inst.id)}>
              <ScrollText data-icon="inline-start" />
              Game logs
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
            <DropdownMenu.Item variant="destructive" onclick={() => (confirmDelete = true)}>
              <Trash2 data-icon="inline-start" />
              Delete
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </div>
    </footer>
  </div>

  <ConfirmDestructive
    bind:open={confirmDelete}
    title={`Delete “${inst.name}”?`}
    consequence="Worlds, mods and screenshots are lost."
    onconfirm={confirmDeleteInstance}
  />
{/if}
