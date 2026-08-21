<script lang="ts">
  import { app } from "$lib/state.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent } from "$lib/components/ui/card";
  import * as Empty from "$lib/components/ui/empty";
  import { ArrowLeft, ArrowRight, Flame, PackageCheck, Star } from "@lucide/svelte";
  import { cn } from "$lib/lib.js";
  import InstanceIcon from "../InstanceIcon.svelte";

  const aphaniteInstances = $derived(app.instances.filter((instance) => instance.aphanite));
  const installedCount = $derived(
    aphaniteInstances.filter((instance) => instance.lastPlayed !== null).length,
  );
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="shrink-0 border-b p-6 pb-4">
    <h1 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
      <Flame class="size-4.5 text-red-500" />
      Aphanite configurations
      <Badge variant="secondary">{aphaniteInstances.length}</Badge>
    </h1>
    <p class="mt-0.5 text-xs text-muted-foreground">
      Modpack configurations provided by your connected Aphanite server.
    </p>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto p-6">
    <div class="flex flex-col gap-4">
      <Card>
        <CardContent class="flex items-center justify-between">
          <div class="min-w-0">
            <span class="text-xs font-medium">Currently connected to</span>
            <!-- 实际使用中应该替换成 Aphanite 的服务器名称字段(暂定为 authlib-injector 元数据结构，理论上应该实现专属的) -->
            <span class="truncate text-primary font-bold"> Enita's Aphanite Server </span>
          </div>
          <div class="flex shrink-0 items-center gap-2 text-right text-xs">
            <PackageCheck class="size-4 text-primary" />
            <span>
              <strong>{aphaniteInstances.length}</strong> configurations,
              <strong> {installedCount}</strong> installed
            </span>
          </div>
        </CardContent>
      </Card>

      {#if aphaniteInstances.length === 0}
        <Empty.Root>
          <Empty.Title>No Aphanite configurations</Empty.Title>
          <Empty.Description>
            Connect a server in Settings to discover modpack configurations.
          </Empty.Description>
        </Empty.Root>
      {:else}
        <div class="flex flex-col gap-2">
          {#each aphaniteInstances as inst (inst.id)}
            <div
              class="flex items-center rounded-md border bg-card transition-colors hover:bg-accent/40"
            >
              <button
                type="button"
                onclick={() => app.openInstanceDetail(inst.id)}
                class="flex min-w-0 flex-1 items-center gap-3 p-3 text-left focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
              >
                <InstanceIcon instance={inst} class="size-10 shrink-0" />
                <span class="min-w-0 flex-1">
                  <span class="block truncate text-sm font-medium">{inst.name}</span>
                  <span class="mt-0.5 block truncate text-micro text-muted-foreground">
                    <span class="capitalize">{inst.loader}</span> · MC {inst.mcVersion} ·
                    {inst.mods.filter((mod) => mod.enabled).length} mods
                  </span>
                </span>
                <span class="flex shrink-0 items-center gap-2">
                  {#if inst.lastPlayed}
                    <Badge variant="outline">Installed</Badge>
                  {:else}
                    <Badge variant="secondary">Not installed</Badge>
                  {/if}
                  <ArrowRight class="size-4 text-muted-foreground" />
                </span>
              </button>
              <Button
                variant={inst.favorite ? "secondary" : "ghost"}
                size="icon-sm"
                class="mr-2"
                aria-label={inst.favorite
                  ? `Remove ${inst.name} from favorites`
                  : `Add ${inst.name} to favorites`}
                onclick={() => app.toggleFavorite(inst.id)}
              >
                <Star class={cn("size-3.5", inst.favorite && "fill-current")} />
              </Button>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
