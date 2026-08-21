<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { app } from "$lib/state.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Empty from "$lib/components/ui/empty";
  import { Switch } from "$lib/components/ui/switch";
  import { ArrowLeft, Ellipsis, Sparkles, Trash2 } from "@lucide/svelte";
  import AddResourcesDialog from "../AddResourcesDialog.svelte";

  let pendingDelete = $state<string | null>(null);

  function confirmDelete() {
    if (!pendingDelete || !app.activeInstance) return;
    app.deleteShaderPack(app.activeInstance.id, pendingDelete);
    pendingDelete = null;
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="shrink-0 border-b p-6 pb-4">
    <Button variant="ghost" size="sm" class="-ml-2" onclick={() => app.setView("instance-detail")}>
      <ArrowLeft data-icon="inline-start" />
      {app.activeInstance?.name ?? "Instance"}
    </Button>
    <div class="mt-3 flex items-center justify-between gap-4">
      <div>
        <h1 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
          <Sparkles class="size-4.5 text-primary" />
          Shader Packs
          {#if app.activeInstance}
            <Badge variant="secondary">{app.activeInstance.shaderPacks.length}</Badge>
          {/if}
        </h1>
        <p class="mt-0.5 text-xs text-muted-foreground">
          Shader packs for the instance you opened.
        </p>
      </div>
      <AddResourcesDialog mode="shader" />
    </div>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto p-6">
    {#if app.activeInstance?.shaderPacks.length === 0}
      <Empty.Root>
        <Empty.Title>No shader packs</Empty.Title>
        <Empty.Description
          >Shaders need Iris or OptiFine in the instance's mod loader.</Empty.Description
        >
      </Empty.Root>
    {:else if app.activeInstance}
      {@const inst = app.activeInstance}
      <div class="flex flex-col gap-2">
        {#each inst.shaderPacks as pack (pack.id)}
          <div class="flex items-center gap-3 rounded-md border bg-card p-2.5">
            <div class="flex size-8 shrink-0 items-center justify-center rounded-md bg-secondary">
              <Sparkles class="size-4 text-muted-foreground" />
            </div>
            <div class="min-w-0 flex-1">
              <p class="truncate text-xs font-medium">{pack.name}</p>
              <p class="mt-0.5 truncate text-micro text-muted-foreground">
                {pack.author} · {pack.version}
              </p>
            </div>
            <Badge variant="outline">{pack.gpu}</Badge>
            <Switch
              checked={pack.enabled}
              onCheckedChange={() => app.toggleShaderPack(inst.id, pack.id)}
              aria-label={`Toggle ${pack.name}`}
            />
            <DropdownMenu.Root>
              <DropdownMenu.Trigger>
                <Button variant="ghost" size="icon-sm" aria-label="Shader actions"
                  ><Ellipsis /></Button
                >
              </DropdownMenu.Trigger>
              <DropdownMenu.Content align="end" class="w-40">
                <DropdownMenu.Item onclick={() => app.toggleShaderPack(inst.id, pack.id)}>
                  {pack.enabled ? "Disable" : "Enable"}
                </DropdownMenu.Item>
                <DropdownMenu.Separator />
                <DropdownMenu.Item variant="destructive" onclick={() => (pendingDelete = pack.id)}>
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
  onOpenChange={(open) => {
    if (!open) pendingDelete = null;
  }}
>
  <AlertDialog.Content class="max-w-sm">
    <AlertDialog.Header>
      <AlertDialog.Title>Delete this shader pack?</AlertDialog.Title>
      <AlertDialog.Description>The file is removed from the instance.</AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel><Button variant="outline">Cancel</Button></AlertDialog.Cancel>
      <AlertDialog.Action onclick={confirmDelete}
        ><Button variant="destructive">Delete</Button></AlertDialog.Action
      >
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
