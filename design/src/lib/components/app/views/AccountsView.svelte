<script lang="ts">
  import { app } from "$lib/state.svelte";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { cn } from "$lib/lib";
  import type { Account, AccountType } from "$lib/types";
  import {
    CircleCheck,
    Flame,
    Info,
    KeyRound,
    Monitor,
    Server,
    Trash2,
    Users,
  } from "@lucide/svelte";
  import { toast } from "svelte-sonner";
  import AccountAddDialog from "../AccountAddDialog.svelte";
  import MinecraftAvatar from "../MinecraftAvatar.svelte";
  import { SkinViewer } from "tiny-skin-viewer";

  let pendingRemove = $state<string | null>(null);
  let previewAccountId = $state(app.activeAccountId);

  const removing = $derived(app.accounts.find((a) => a.id === pendingRemove) ?? null);
  const previewAccount = $derived(
    app.accounts.find((account) => account.id === previewAccountId) ?? app.activeAccount,
  );
  const previewProfile = $derived(
    previewAccount?.profiles.find((profile) => profile.id === previewAccount.activeProfileId) ??
      null,
  );
  const ACCOUNT_LABEL: Record<AccountType, string> = {
    microsoft: "Microsoft",
    offline: "Offline",
    aphanite: "Aphanite",
    yggdrasil: "Yggdrasil",
  };

  function accountIcon(type: AccountType) {
    return type === "aphanite"
      ? Flame
      : type === "yggdrasil"
        ? Server
        : type === "microsoft"
          ? KeyRound
          : Monitor;
  }

  function accountDetail(account: Account) {
    return account.authServer ? account.authServer : `last used ${account.lastUsed}`;
  }

  function confirmRemove() {
    if (!pendingRemove) return;
    const name = removing?.username ?? "";
    app.removeAccount(pendingRemove);
    toast.success(`Removed account “${name}”`);
    pendingRemove = null;
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="shrink-0 border-b p-6 pb-4">
    <div class="flex items-center justify-between gap-4">
      <div class="min-w-0">
        <h1 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
          <Users class="size-4.5 text-primary" />
          Accounts
        </h1>
        <p class="mt-0.5 text-xs text-muted-foreground">
          One active account per launch; offline accounts work for single-player only.
        </p>
      </div>
      <AccountAddDialog />
    </div>
  </header>

  <div class="flex min-h-0 flex-1 gap-6 overflow-y-auto p-6">
    <div class="flex min-w-0 flex-1 flex-col gap-4">
      <Alert>
        <AlertTitle class="flex items-center gap-1.5 text-xs">
          <Info class="size-3.5" />
          Account types
        </AlertTitle>
        <AlertDescription class="text-xs">
          Microsoft, Aphanite, and custom Yggdrasil accounts support online play. Offline accounts
          skip login entirely.
        </AlertDescription>
      </Alert>

      {#each app.accounts as acc (acc.id)}
        <div
          role="button"
          tabindex="0"
          class={cn(
            "flex cursor-pointer items-center gap-3 rounded-md border bg-card p-3 transition-colors hover:bg-accent/30",
            acc.id === app.activeAccountId && "border-primary/40",
            acc.id === previewAccountId && "bg-accent/20",
          )}
          onclick={() => {
            previewAccountId = acc.id;
            if (acc.id !== app.activeAccountId) {
              app.setActiveAccount(acc.id);
              toast.success(`Now playing as ${acc.username}`);
            }
          }}
          onkeydown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              previewAccountId = acc.id;
              if (acc.id !== app.activeAccountId) {
                app.setActiveAccount(acc.id);
                toast.success(`Now playing as ${acc.username}`);
              }
            }
          }}
        >
          {#if acc.profiles.find((profile) => profile.id === acc.activeProfileId)}
            {@const profile = acc.profiles.find((profile) => profile.id === acc.activeProfileId)!}
            <MinecraftAvatar skinUrl={profile.skinUrl} alt={profile.name} class="size-9 shrink-0" />
          {/if}
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <p class="truncate text-xs font-medium">{acc.username}</p>
              <Badge variant={acc.type === "microsoft" ? "secondary" : "outline"} class="shrink-0">
                {@const Icon = accountIcon(acc.type)}
                <Icon data-icon="inline-start" />
                {ACCOUNT_LABEL[acc.type]}
              </Badge>
              {#if acc.id === app.activeAccountId}
                <Badge class="gap-1 shrink-0">
                  <CircleCheck class="size-3" />
                  Active
                </Badge>
              {/if}
            </div>
            <p class="mt-0.5 text-micro text-muted-foreground">
              {accountDetail(acc)}
            </p>
          </div>
          {#if app.accounts.length > 1}
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={`Remove ${acc.username}`}
              onclick={(event) => {
                event.stopPropagation();
                pendingRemove = acc.id;
              }}
            >
              <Trash2 class="size-3.5" />
            </Button>
          {/if}
        </div>
      {/each}
    </div>
    {#if previewAccount && previewProfile}
      <aside class="sticky top-0 h-fit w-72 shrink-0">
        <Card class="overflow-hidden">
          <CardHeader class="p-4 pb-2">
            <CardTitle class="flex items-center gap-2">
              <MinecraftAvatar
                skinUrl={previewProfile.skinUrl}
                alt={previewProfile.name}
                class="size-8"
              />
              <span class="truncate">{previewProfile.name}</span>
            </CardTitle>
            <CardDescription
              >{ACCOUNT_LABEL[previewAccount.type]} player profile preview</CardDescription
            >
          </CardHeader>
          <CardContent class="p-0">
            <SkinViewer
              skinUrl={previewProfile.skinUrl}
              isSlim={previewProfile.isSlim}
              width={280}
              height={320}
              scale={1.2}
            />
          </CardContent>
        </Card>
      </aside>
    {/if}
  </div>
</div>

<AlertDialog.Root
  open={pendingRemove !== null}
  onOpenChange={(o) => {
    if (!o) pendingRemove = null;
  }}
>
  <AlertDialog.Content class="max-w-sm">
    <AlertDialog.Header>
      <AlertDialog.Title>Remove “{removing?.username}”?</AlertDialog.Title>
      <AlertDialog.Description>
        The stored credentials are deleted. You can add the account again later.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>
        <Button variant="outline">Keep account</Button>
      </AlertDialog.Cancel>
      <AlertDialog.Action onclick={confirmRemove}>
        <Button variant="destructive">Remove</Button>
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
