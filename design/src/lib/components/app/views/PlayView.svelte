<script lang="ts">
  import { app } from "$lib/state.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { cn } from "$lib/lib.js";
  import { ArrowRight, ChevronRight, Flame, LoaderCircle, Play, Square } from "@lucide/svelte";
  import InstanceIcon from "../InstanceIcon.svelte";
  import LaunchCrystalBackdrop from "../LaunchCrystalBackdrop.svelte";
  import MinecraftAvatar from "../MinecraftAvatar.svelte";

  const hour = new Date().getHours();
  const greeting = hour < 12 ? "Good morning." : hour < 18 ? "Good afternoon." : "Good evening.";
  let recommendedInstanceId = $state(
    app.instances[Math.floor(Math.random() * app.instances.length)]?.id ?? null,
  );

  const recommendedInstance = $derived(
    app.instances.find((instance) => instance.id === recommendedInstanceId) ??
      app.instances[0] ??
      null,
  );
  const recommendedIsRunning = $derived(
    recommendedInstance ? app.runningIds.includes(recommendedInstance.id) : false,
  );
  const recommendedLaunch = $derived(
    recommendedInstance ? app.getLaunch(recommendedInstance.id) : null,
  );
  const recommendedIsStarting = $derived(recommendedLaunch !== null);
  const serverOwnerRecommendations = $derived(
    app.instances.filter((instance) => instance.aphanite),
  );
</script>

<div class="flex h-full flex-col gap-6 overflow-y-auto p-6">
  <div class="flex shrink-0 items-center justify-between">
    <div>
      <h1 class="text-lg font-semibold tracking-tight">{greeting}</h1>
      <p class="mt-0.5 text-xs text-muted-foreground">
        {app.instances.length} instance{app.instances.length > 1 && "s"}
      </p>
    </div>
    <Button variant="outline" size="sm" onclick={() => app.setView("accounts")}>
      {#if app.activeProfile}
        <MinecraftAvatar
          skinUrl={app.activeProfile.skinUrl}
          alt={app.activeProfile.name}
          class="size-5"
        />
      {/if}
      <span class="max-w-28 truncate">{app.activeAccount?.username}</span>
      <ChevronRight class="size-3.5 text-muted-foreground" />
    </Button>
  </div>

  {#if recommendedInstance}
    {@const inst = recommendedInstance}
    <section>
      <div class="mb-2 flex items-center justify-between">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Recommended for you
        </h2>
      </div>
      <Card class="flex-row gap-0 py-0">
        <CardContent class="min-w-0 flex-1 p-0">
          <button
            type="button"
            class="flex h-full w-full items-start gap-4 px-(--card-spacing) py-(--card-spacing) text-left transition-colors hover:bg-accent/30 focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
            aria-label={`View details for ${inst.name}`}
            onclick={() => app.openInstanceDetail(inst.id)}
          >
            <InstanceIcon instance={inst} class="size-14 shrink-0" />
            <span class="min-w-0 flex-1">
              <span class="flex flex-wrap items-center gap-2">
                <span
                  class="flex items-center gap-2 truncate text-base font-semibold tracking-tight"
                >
                  <span class="truncate">{inst.name}</span>
                  {#if inst.aphanite}
                    <Flame class="size-4 shrink-0 text-red-500" />
                  {/if}
                </span>
                <Badge variant="outline">MC {inst.mcVersion}</Badge>
                <Badge variant="secondary" class="capitalize">{inst.loader}</Badge>
                {#if recommendedIsRunning}
                  <Badge class="gap-1">
                    <span class="size-1.5 rounded-full bg-primary-foreground"></span>
                    Running
                  </Badge>
                {:else if recommendedIsStarting}
                  <Badge variant="secondary" class="gap-1">
                    <LoaderCircle class="size-3 animate-spin" />
                    Starting
                  </Badge>
                {/if}
              </span>
              <span class="mt-1.5 block line-clamp-2 text-xs text-muted-foreground"
                >{inst.description}</span
              >
              <span class="mt-3 flex flex-wrap gap-x-5 gap-y-1 text-micro text-muted-foreground">
                {#if inst.loader !== "vanilla"}
                  <span
                    >{inst.mods.filter((mod) => mod.enabled).length}/{inst.mods.length} mods</span
                  >
                {/if}
                <span>{inst.resourcePacks.length} packs</span>
                <span>{inst.worlds.length} worlds</span>
                <span>Java {app.getJavaRuntime(inst.javaRuntimeId)?.version ?? inst.java}</span>
              </span>
            </span>
          </button>
        </CardContent>
        <button
          type="button"
          class={cn(
            "relative isolate flex w-44 shrink-0 items-center justify-center overflow-hidden border-l border-launch-foreground/15 transition-colors focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset",
            recommendedIsRunning
              ? "bg-card hover:bg-accent/30"
              : recommendedIsStarting
                ? "bg-black text-launch-foreground hover:bg-black/90"
                : "bg-launch text-launch-foreground hover:bg-launch/85",
          )}
          aria-label={`${recommendedIsRunning ? "Stop" : recommendedIsStarting ? "Cancel launch of" : "Play"} ${inst.name}`}
          onclick={() =>
            recommendedIsRunning
              ? app.stopInstance(inst.id)
              : recommendedIsStarting
                ? app.cancelLaunch(inst.id)
                : app.startLaunch(inst.id)}
        >
          {#if recommendedIsStarting}
            <LaunchCrystalBackdrop
              seed={`${inst.name}|${inst.mcVersion}|${inst.loader}`}
              progress={recommendedLaunch?.progress ?? 0}
            />
          {/if}
          {#if recommendedIsRunning}
            <div class="relative z-10 text-center">
              <div
                class="flex size-11 items-center justify-center text-destructive bg-destructive/10 rounded-full"
              >
                <Square class="fill-current size-5" />
              </div>
              <div>Stop</div>
            </div>
          {:else if recommendedIsStarting}
            <div class="relative z-10 flex flex-col items-center gap-2">
              <LoaderCircle class="size-5 animate-spin" />
              <span>Cancel</span>
            </div>
          {:else}
            <span class="relative z-10 flex size-11 items-center justify-center">
              <Play class="fill-current size-5" />
            </span>
          {/if}
        </button>
      </Card>
    </section>
  {/if}

  {#if serverOwnerRecommendations.length > 0}
    <section>
      <div class="mb-2 flex items-center justify-between">
        <h2
          class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground"
        >
          <Flame class="size-4 text-red-500" />
          Recommended by your server owner
        </h2>
        <Button variant="ghost" size="xs" onclick={() => app.setView("aphanite")}>
          Check out
          <ArrowRight class="size-3" />
        </Button>
      </div>
      <div class="flex flex-col gap-2">
        {#each serverOwnerRecommendations as inst (inst.id)}
          <div
            class="flex w-full items-center rounded-md border bg-card transition-colors hover:bg-accent/30"
          >
            <button
              type="button"
              onclick={() => app.openInstanceDetail(inst.id)}
              class="flex min-w-0 flex-1 items-center gap-3 p-3 text-left focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
            >
              <InstanceIcon instance={inst} class="size-9 shrink-0" />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-xs font-medium">{inst.name}</span>
                <span class="mt-0.5 block truncate text-micro text-muted-foreground">
                  MC {inst.mcVersion} · <span class="capitalize">{inst.loader}</span>
                  {inst.lastPlayed ? ` · ${inst.lastPlayed}` : ""}
                </span>
              </span>
              {#if app.runningIds.includes(inst.id)}
                <span class="size-2 shrink-0 rounded-full bg-primary" title="Running"></span>
              {:else if app.getLaunch(inst.id)}
                <LoaderCircle class="size-3.5 shrink-0 animate-spin text-muted-foreground" />
              {/if}
            </button>
            <button
              type="button"
              class={cn(
                "ml-1 mr-3 flex size-7 shrink-0 items-center justify-center rounded-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                app.runningIds.includes(inst.id)
                  ? "bg-destructive/10 text-destructive hover:bg-destructive/20"
                  : app.getLaunch(inst.id)
                    ? "bg-secondary text-secondary-foreground hover:bg-accent"
                    : "hover:bg-launch hover:text-launch-foreground",
              )}
              aria-label={`${app.runningIds.includes(inst.id) ? "Stop" : app.getLaunch(inst.id) ? "Cancel launch of" : "Play"} ${inst.name}`}
              onclick={() =>
                app.runningIds.includes(inst.id)
                  ? app.stopInstance(inst.id)
                  : app.getLaunch(inst.id)
                    ? app.cancelLaunch(inst.id)
                    : app.startLaunch(inst.id)}
            >
              {#if app.runningIds.includes(inst.id)}
                <span class="size-3 rounded-sm bg-current"></span>
              {:else if app.getLaunch(inst.id)}
                <LoaderCircle class="size-3.5 animate-spin" />
              {:else}
                <Play class="size-3.5" />
              {/if}
            </button>
          </div>
        {/each}
      </div>
    </section>
  {/if}

  <section>
    <div class="mb-2 flex items-center justify-between">
      <h2 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        All instances
      </h2>
      <Button variant="ghost" size="xs" onclick={() => app.setView("instances")}>
        Manage
        <ArrowRight class="size-3" />
      </Button>
    </div>
    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      {#each app.instances as inst (inst.id)}
        <div
          class="flex items-center rounded-md border bg-card transition-colors hover:bg-accent/30"
        >
          <button
            type="button"
            onclick={() => app.openInstanceDetail(inst.id)}
            class="flex min-w-0 flex-1 items-center gap-3 p-3 text-left focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
          >
            <InstanceIcon instance={inst} class="size-9 shrink-0" />
            <span class="min-w-0 flex-1">
              <span class="flex items-center gap-2">
                <span class="truncate text-xs font-medium">{inst.name}</span>
                {#if inst.aphanite}
                  <Flame class="size-3.5 shrink-0 text-red-500" />
                {/if}
              </span>
              <span class="mt-0.5 block truncate text-micro text-muted-foreground">
                MC {inst.mcVersion} · <span class="capitalize">{inst.loader}</span>
                {inst.lastPlayed ? ` · ${inst.lastPlayed}` : ""}
              </span>
            </span>
            {#if app.runningIds.includes(inst.id)}
              <span class="size-2 shrink-0 rounded-full bg-primary" title="Running"></span>
            {:else if app.getLaunch(inst.id)}
              <LoaderCircle class="size-3.5 shrink-0 animate-spin text-muted-foreground" />
            {/if}
          </button>
          <button
            type="button"
            class={cn(
              "ml-1 mr-3 flex size-7 shrink-0 items-center justify-center rounded-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              app.runningIds.includes(inst.id)
                ? "bg-destructive/10 text-destructive hover:bg-destructive/20"
                : app.getLaunch(inst.id)
                  ? "bg-secondary text-secondary-foreground hover:bg-accent"
                  : "hover:bg-launch hover:text-launch-foreground",
            )}
            aria-label={`${app.runningIds.includes(inst.id) ? "Stop" : app.getLaunch(inst.id) ? "Cancel launch of" : "Play"} ${inst.name}`}
            onclick={() =>
              app.runningIds.includes(inst.id)
                ? app.stopInstance(inst.id)
                : app.getLaunch(inst.id)
                  ? app.cancelLaunch(inst.id)
                  : app.startLaunch(inst.id)}
          >
            {#if app.runningIds.includes(inst.id)}
              <span class="size-3 rounded-sm bg-current"></span>
            {:else if app.getLaunch(inst.id)}
              <LoaderCircle class="size-3.5 animate-spin" />
            {:else}
              <Play class="size-3.5" />
            {/if}
          </button>
        </div>
      {/each}
    </div>
  </section>
</div>
