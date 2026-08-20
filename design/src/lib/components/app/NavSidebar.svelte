<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { app } from "$lib/state.svelte";
  import { cn } from "$lib/lib";
  import MinecraftAvatar from "./MinecraftAvatar.svelte";
  import InstanceIcon from "./InstanceIcon.svelte";
  import {
    ChevronDown,
    Flame,
    KeyRound,
    Layers,
    Monitor,
    Play,
    Server,
    Settings,
    Star,
    Users,
  } from "@lucide/svelte";

  const activeType = $derived(app.activeAccount?.type ?? "offline");
  const favorites = $derived(app.instances.filter((instance) => instance.favorite));
  const localInstances = $derived(
    app.instances.filter((instance) => !instance.aphanite && !instance.favorite),
  );
  const aphaniteInstances = $derived(
    app.instances.filter((instance) => instance.aphanite && !instance.favorite),
  );

  let instancesExpanded = $state(true);
  let aphaniteExpanded = $state(true);

  function openInstance(id: string) {
    app.openInstanceDetail(id);
  }

  function instanceIsActive(id: string) {
    return app.view === "instance-detail" && app.activeInstanceId === id;
  }
</script>

<aside class="flex h-full w-52 shrink-0 flex-col border-r bg-sidebar">
  <nav class="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-2 py-3">
    <button
      type="button"
      onclick={() => app.setView("play")}
      class={cn(
        "flex h-8 items-center gap-2.5 rounded-md px-2.5 text-xs font-medium transition-colors",
        app.view === "play"
          ? "bg-accent text-accent-foreground"
          : "text-muted-foreground hover:bg-muted hover:text-foreground",
      )}
    >
      <Play class="size-4 shrink-0" />
      <span>Quick Play</span>
      {#if app.runningIds.length > 0}
        <span class="ml-auto flex items-center gap-1 text-micro text-primary">
          <span class="size-1.5 rounded-full bg-primary"></span>
          {app.runningIds.length}
        </span>
      {/if}
    </button>

    <section class="mt-3">
      <p
        class="flex h-7 items-center gap-2 px-2.5 text-micro font-semibold uppercase tracking-widest text-muted-foreground-subtle"
      >
        <Star class="size-3.5" />
        Favorites
        <span class="ml-auto text-muted-foreground">{favorites.length}</span>
      </p>
      {#if favorites.length > 0}
        <div class="mt-0.5 flex flex-col gap-0.5">
          {#each favorites as instance (instance.id)}
            <button
              type="button"
              onclick={() => openInstance(instance.id)}
              class={cn(
                "flex h-7 items-center gap-2 rounded-md px-2.5 text-left text-xs transition-colors",
                instanceIsActive(instance.id)
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground",
              )}
            >
              <InstanceIcon {instance} class="size-4 shrink-0" />
              <span class="min-w-0 flex-1 truncate">{instance.name}</span>
              {#if app.runningIds.includes(instance.id)}
                <span class="size-1.5 shrink-0 rounded-full bg-primary" title="Running"></span>
              {/if}
            </button>
          {/each}
        </div>
      {:else}
        <p class="px-2.5 py-1 text-micro text-muted-foreground-subtle">No favorites yet</p>
      {/if}
    </section>

    <section class="mt-3">
      <div
        class={cn(
          "flex items-center rounded-md transition-colors",
          app.view === "instances" && "bg-accent text-accent-foreground",
        )}
      >
        <button
          type="button"
          onclick={() => app.setView("instances")}
          class={cn(
            "flex h-8 min-w-0 flex-1 items-center gap-2.5 rounded-l-md px-2.5 text-xs font-medium",
            app.view === "instances"
              ? "text-accent-foreground"
              : "text-muted-foreground hover:bg-muted hover:text-foreground",
          )}
        >
          <Layers class="size-4 shrink-0" />
          <span>Instances</span>
          <span class="ml-auto text-micro text-muted-foreground-subtle"
            >{localInstances.length}</span
          >
        </button>
        <button
          type="button"
          class="flex size-8 shrink-0 items-center justify-center rounded-r-md text-muted-foreground hover:bg-muted hover:text-foreground"
          aria-label={`${instancesExpanded ? "Collapse" : "Expand"} local instances`}
          aria-expanded={instancesExpanded}
          onclick={() => (instancesExpanded = !instancesExpanded)}
        >
          <ChevronDown
            class={cn("size-3.5 transition-transform", !instancesExpanded && "-rotate-90")}
          />
        </button>
      </div>
      {#if instancesExpanded}
        <div class="mt-0.5 flex flex-col gap-0.5">
          {#each localInstances as instance (instance.id)}
            <button
              type="button"
              onclick={() => openInstance(instance.id)}
              class={cn(
                "flex h-7 items-center gap-2 rounded-md px-2.5 text-left text-xs transition-colors",
                instanceIsActive(instance.id)
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground",
              )}
            >
              <InstanceIcon {instance} class="size-4 shrink-0" />
              <span class="min-w-0 flex-1 truncate">{instance.name}</span>
              {#if app.runningIds.includes(instance.id)}
                <span class="size-1.5 shrink-0 rounded-full bg-primary" title="Running"></span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </section>

    <section class="mt-2">
      <div
        class={cn(
          "flex items-center rounded-md transition-colors",
          app.view === "aphanite" && "bg-accent text-accent-foreground",
        )}
      >
        <button
          type="button"
          onclick={() => app.setView("aphanite")}
          class={cn(
            "flex h-8 min-w-0 flex-1 items-center gap-2.5 rounded-l-md px-2.5 text-xs font-medium",
            app.view === "aphanite"
              ? "text-accent-foreground"
              : "text-muted-foreground hover:bg-muted hover:text-foreground",
          )}
        >
          <Flame class="size-4 shrink-0 text-red-500" />
          <span>Aphanite</span>
          <span class="ml-auto text-micro text-muted-foreground-subtle"
            >{aphaniteInstances.length}</span
          >
        </button>
        <button
          type="button"
          class="flex size-8 shrink-0 items-center justify-center rounded-r-md text-muted-foreground hover:bg-muted hover:text-foreground"
          aria-label={`${aphaniteExpanded ? "Collapse" : "Expand"} Aphanite instances`}
          aria-expanded={aphaniteExpanded}
          onclick={() => (aphaniteExpanded = !aphaniteExpanded)}
        >
          <ChevronDown
            class={cn("size-3.5 transition-transform", !aphaniteExpanded && "-rotate-90")}
          />
        </button>
      </div>
      {#if aphaniteExpanded}
        <div class="mt-0.5 flex flex-col gap-0.5">
          {#each aphaniteInstances as instance (instance.id)}
            <button
              type="button"
              onclick={() => openInstance(instance.id)}
              class={cn(
                "flex h-7 items-center gap-2 rounded-md px-2.5 text-left text-xs transition-colors",
                instanceIsActive(instance.id)
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground",
              )}
            >
              <InstanceIcon {instance} class="size-4 shrink-0" />
              <span class="min-w-0 flex-1 truncate">{instance.name}</span>
              {#if app.runningIds.includes(instance.id)}
                <span class="size-1.5 shrink-0 rounded-full bg-primary" title="Running"></span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </section>
  </nav>

  <DropdownMenu.Root>
    <DropdownMenu.Trigger
      type="button"
      class="m-2 flex w-[calc(100%-1rem)] items-center gap-2.5 rounded-md border bg-card px-2.5 py-2 text-left transition-colors hover:bg-accent"
    >
      {#if app.activeProfile}
        <MinecraftAvatar
          skinUrl={app.activeProfile.skinUrl}
          alt={app.activeProfile.name}
          class="size-7 shrink-0"
        />
      {/if}
      <span class="min-w-0 flex-1">
        <span class="block truncate text-xs font-medium">
          {app.activeAccount?.username ?? "Offline"}
        </span>
        <span class="block truncate text-micro capitalize text-muted-foreground">
          {activeType} account
        </span>
      </span>
      <ChevronDown class="size-3.5 shrink-0 text-muted-foreground" />
    </DropdownMenu.Trigger>
    <DropdownMenu.Content side="top" align="start" class="w-56">
      <DropdownMenu.Label>Switch account</DropdownMenu.Label>
      <DropdownMenu.RadioGroup
        value={app.activeAccountId}
        onValueChange={(id) => {
          if (id) app.setActiveAccount(id);
        }}
      >
        {#each app.accounts as account (account.id)}
          <DropdownMenu.RadioItem value={account.id}>
            {#if account.profiles.find((profile) => profile.id === account.activeProfileId)}
              {@const profile = account.profiles.find(
                (profile) => profile.id === account.activeProfileId,
              )!}
              <MinecraftAvatar
                skinUrl={profile.skinUrl}
                alt={profile.name}
                class="size-5 shrink-0"
              />
            {/if}
            <span class="min-w-0 flex-1">
              <span class="block truncate text-xs">{account.username}</span>
              <span class="block truncate text-micro capitalize text-muted-foreground">
                {account.type}
              </span>
            </span>
            {#if account.type === "microsoft"}
              <KeyRound />
            {:else if account.type === "aphanite"}
              <Flame class="text-red-500" />
            {:else if account.type === "yggdrasil"}
              <Server />
            {:else}
              <Monitor />
            {/if}
          </DropdownMenu.RadioItem>
        {/each}
      </DropdownMenu.RadioGroup>
      <DropdownMenu.Separator />
      <DropdownMenu.Item onclick={() => app.setView("accounts")}>
        <Users data-icon="inline-start" />
        Manage accounts
      </DropdownMenu.Item>
    </DropdownMenu.Content>
  </DropdownMenu.Root>
  {#if app.activeAccountId === ""}
    <p class="px-3 pb-2 text-micro text-destructive">No account selected</p>
  {/if}
  <button
    type="button"
    onclick={() => app.setView("settings")}
    class={cn(
      "mx-2 mb-2 flex h-8 items-center gap-2.5 rounded-md px-2.5 text-xs font-medium transition-colors",
      app.view === "settings"
        ? "bg-accent text-accent-foreground"
        : "text-muted-foreground hover:bg-muted hover:text-foreground",
    )}
  >
    <Settings class="size-4 shrink-0" />
    <span>Settings</span>
  </button>
</aside>
