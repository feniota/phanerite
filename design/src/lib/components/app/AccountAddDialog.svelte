<script lang="ts">
  import { app } from "$lib/state.svelte";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import * as Tabs from "$lib/components/ui/tabs";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import type { PlayerProfile } from "$lib/types";
  import { Flame, KeyRound, Loader2, Monitor, Server, UserPlus } from "@lucide/svelte";
  import { toast } from "svelte-sonner";

  let open = $state(false);
  let tab = $state("microsoft");
  let offlineName = $state("");
  let yggdrasilName = $state("");
  let yggdrasilServer = $state("");
  let aphaniteProfile = $state("aphanite-enita");
  let yggdrasilProfile = $state("ygg-player");
  let signingIn = $state(false);
  let signingInWith = $state<"microsoft" | "aphanite">("microsoft");
  let cancelled = false;

  const MS_USERNAME = "enita";
  const APHANITE_USERNAME = "enita";
  let signInTimer: ReturnType<typeof setTimeout> | undefined;
  const APHANITE_PROFILES: PlayerProfile[] = [
    { id: "aphanite-enita", name: "enita", skinUrl: "https://mc-heads.net/skin/enita" },
    { id: "aphanite-builder", name: "Builder", skinUrl: "https://mc-heads.net/skin/Builder" },
  ];
  const YGGDRASIL_PROFILES: PlayerProfile[] = [
    { id: "ygg-player", name: "Player", skinUrl: "https://mc-heads.net/skin/Player" },
    { id: "ygg-alex", name: "Alex", skinUrl: "https://mc-heads.net/skin/Alex", isSlim: true },
  ];

  function beginBrowserSignIn(provider: "microsoft" | "aphanite") {
    cancelled = false;
    signingIn = true;
    signingInWith = provider;
    signInTimer = setTimeout(() => {
      if (cancelled) return;
      signingIn = false;
      const isAphanite = provider === "aphanite";
      const profiles = isAphanite ? APHANITE_PROFILES : undefined;
      const profile = profiles?.find((item) => item.id === aphaniteProfile);
      app.addAccount(
        profile?.name ?? (isAphanite ? APHANITE_USERNAME : MS_USERNAME),
        provider,
        isAphanite ? app.settings.aphaniteServer : undefined,
        profiles,
        profile?.id,
      );
      toast.success(`Signed in as ${isAphanite ? APHANITE_USERNAME : MS_USERNAME}`, {
        description: isAphanite
          ? "Aphanite Yggdrasil account linked."
          : "Microsoft account linked.",
      });
      open = false;
    }, 1800);
  }

  function cancelSignIn() {
    cancelled = true;
    signingIn = false;
    if (signInTimer) clearTimeout(signInTimer);
  }

  function addOffline() {
    if (!offlineName.trim()) return;
    app.addAccount(offlineName.trim(), "offline");
    toast.success(`Offline account “${offlineName.trim()}” added`);
    offlineName = "";
    open = false;
  }

  function addYggdrasil() {
    if (!yggdrasilName.trim() || !yggdrasilServer.trim()) return;
    const profile = YGGDRASIL_PROFILES.find((item) => item.id === yggdrasilProfile)!;
    app.addAccount(
      profile.name,
      "yggdrasil",
      yggdrasilServer.trim(),
      YGGDRASIL_PROFILES,
      profile.id,
    );
    toast.success(`Yggdrasil account “${yggdrasilName.trim()}” linked`);
    yggdrasilName = "";
    yggdrasilServer = "";
    open = false;
  }

  function onOpenChange(nextOpen: boolean) {
    if (!nextOpen) cancelSignIn();
    open = nextOpen;
  }
</script>

<Dialog.Root bind:open {onOpenChange}>
  <Dialog.Trigger>
    <Button>
      <UserPlus data-icon="inline-start" />
      Add account
    </Button>
  </Dialog.Trigger>
  <Dialog.Content class="max-w-lg">
    <Dialog.Header>
      <Dialog.Title>Add account</Dialog.Title>
      <Dialog.Description>
        Sign in with Microsoft or a Yggdrasil provider, or add an offline profile.
      </Dialog.Description>
    </Dialog.Header>

    <Tabs.Root bind:value={tab} class="flex flex-col gap-4 px-6">
      <Tabs.List class="grid w-full grid-cols-4">
        <Tabs.Trigger value="microsoft">Microsoft</Tabs.Trigger>
        <Tabs.Trigger value="aphanite">Aphanite</Tabs.Trigger>
        <Tabs.Trigger value="yggdrasil">Yggdrasil</Tabs.Trigger>
        <Tabs.Trigger value="offline">Offline</Tabs.Trigger>
      </Tabs.List>

      <Tabs.Content value="microsoft" class="flex flex-col gap-4">
        <Alert>
          <AlertTitle class="flex items-center gap-1.5 text-xs">
            <Monitor class="size-3.5" />
            Microsoft account
          </AlertTitle>
          <AlertDescription class="text-xs">
            Sign in at login.live.com to use your Minecraft Java profile.
          </AlertDescription>
        </Alert>
        {@render BrowserSignIn(
          signingIn && signingInWith === "microsoft",
          "Microsoft",
          () => beginBrowserSignIn("microsoft"),
          cancelSignIn,
        )}
      </Tabs.Content>

      <Tabs.Content value="aphanite" class="flex flex-col gap-4">
        <Alert>
          <AlertTitle class="flex items-center gap-1.5 text-xs">
            <Flame class="size-3.5 text-red-500" />
            Aphanite Yggdrasil
          </AlertTitle>
          <AlertDescription class="text-xs">
            Authenticate against the Aphanite server configured in Settings.
          </AlertDescription>
        </Alert>
        <p
          class="rounded-md border bg-muted/30 px-3 py-2 font-mono text-micro text-muted-foreground"
        >
          {app.settings.aphaniteServer}
        </p>
        <Field.Field>
          <Field.FieldLabel>Player profile</Field.FieldLabel>
          <ToggleGroup.Root type="single" bind:value={aphaniteProfile}>
            {#each APHANITE_PROFILES as profile (profile.id)}
              <ToggleGroup.Item value={profile.id}>{profile.name}</ToggleGroup.Item>
            {/each}
          </ToggleGroup.Root>
        </Field.Field>
        {@render BrowserSignIn(
          signingIn && signingInWith === "aphanite",
          "Aphanite",
          () => beginBrowserSignIn("aphanite"),
          cancelSignIn,
        )}
      </Tabs.Content>

      <Tabs.Content value="yggdrasil" class="flex flex-col gap-4">
        <Alert>
          <AlertTitle class="flex items-center gap-1.5 text-xs">
            <Server class="size-3.5" />
            Custom Yggdrasil server
          </AlertTitle>
          <AlertDescription class="text-xs">
            Use a third-party Yggdrasil authentication endpoint.
          </AlertDescription>
        </Alert>
        <Field.FieldGroup>
          <Field.Field>
            <Field.FieldLabel for="yggdrasil-server">Server URL</Field.FieldLabel>
            <Input
              id="yggdrasil-server"
              type="url"
              bind:value={yggdrasilServer}
              placeholder="https://auth.example.com/"
            />
          </Field.Field>
          <Field.Field>
            <Field.FieldLabel for="yggdrasil-name">Player name</Field.FieldLabel>
            <Input
              id="yggdrasil-name"
              bind:value={yggdrasilName}
              placeholder="e.g. Player"
              maxlength={16}
            />
          </Field.Field>
        </Field.FieldGroup>
        <Field.Field>
          <Field.FieldLabel>Player profile</Field.FieldLabel>
          <ToggleGroup.Root type="single" bind:value={yggdrasilProfile}>
            {#each YGGDRASIL_PROFILES as profile (profile.id)}
              <ToggleGroup.Item value={profile.id}>{profile.name}</ToggleGroup.Item>
            {/each}
          </ToggleGroup.Root>
        </Field.Field>
        <Button onclick={addYggdrasil} disabled={!yggdrasilName.trim() || !yggdrasilServer.trim()}>
          <KeyRound data-icon="inline-start" />
          Sign in with Yggdrasil
        </Button>
      </Tabs.Content>

      <Tabs.Content value="offline" class="flex flex-col gap-4">
        <Alert>
          <AlertTitle class="flex items-center gap-1.5 text-xs">
            <Monitor class="size-3.5" />
            Single-player only
          </AlertTitle>
          <AlertDescription class="text-xs">
            Offline accounts skip authentication. You can join LAN worlds, but not online servers.
          </AlertDescription>
        </Alert>
        <Field.Field>
          <Field.FieldLabel for="offline-name">Player name</Field.FieldLabel>
          <Input
            id="offline-name"
            bind:value={offlineName}
            placeholder="e.g. Steve"
            maxlength={16}
          />
        </Field.Field>
        <Button onclick={addOffline} disabled={!offlineName.trim()}>Add offline account</Button>
      </Tabs.Content>
    </Tabs.Root>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Close</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

{#snippet BrowserSignIn(
  active: boolean,
  provider: "Microsoft" | "Aphanite",
  onStart: () => void,
  onCancel: () => void,
)}
  {#if active}
    <div class="flex flex-col gap-2 rounded-md border p-4">
      <div class="flex items-center gap-2 text-xs">
        <Loader2 class="size-4 animate-spin text-primary" />
        <span class="font-medium">Waiting for {provider} in the browser…</span>
      </div>
      <p class="text-micro text-muted-foreground">
        This prototype completes the sign-in flow automatically.
      </p>
      <Button variant="outline" size="sm" class="w-fit" onclick={onCancel}>Cancel sign-in</Button>
    </div>
  {:else}
    <Button class="w-full" onclick={onStart}>
      <KeyRound data-icon="inline-start" />
      Sign in with {provider}
    </Button>
  {/if}
{/snippet}
