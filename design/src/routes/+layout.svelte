<script lang="ts">
  import "./layout.css";
  import favicon from "$lib/assets/favicon.svg";
  import PhaneriteAppIcon from "$lib/components/app/PhaneriteAppIcon.svelte";
  import LaunchTray from "$lib/components/app/LaunchTray.svelte";
  import NavSidebar from "$lib/components/app/NavSidebar.svelte";
  import StatusBar from "$lib/components/app/StatusBar.svelte";
  import { Toaster } from "$lib/components/ui/sonner";
  import { Minus, Square, X } from "@lucide/svelte";
  import { page } from "$app/state";
  import { toast } from "svelte-sonner";

  let { children } = $props();

  // /lab is a design sandbox, not part of the launcher. Render it bare so it
  // is not boxed into the 1200x760 window frame. Delete with src/routes/lab/.
  const bare = $derived(page.url.pathname.startsWith("/lab"));

  function windowControl(kind: string) {
    toast.info("Window controls are decorative here", {
      description: `${kind} would be handled by the native window in the GPUI app.`,
    });
  }
</script>

<svelte:head>
  <title>Phanerite — Minecraft Java Launcher</title>
  <link rel="icon" href={favicon} />
</svelte:head>

{#if bare}
  {@render children()}
{:else}
  <div class="flex min-h-svh items-center justify-center bg-background p-4">
    <div
      class="flex h-[760px] max-h-[calc(100svh-2rem)] w-[1200px] max-w-[calc(100vw-2rem)] flex-col overflow-hidden rounded-md border bg-card shadow-2xl"
    >
      <!-- title bar -->
      <div class="flex h-10 shrink-0 items-center border-b bg-card px-3 select-none">
        <div class="flex items-center gap-2">
          <PhaneriteAppIcon class="size-5" />
          <span class="text-xs font-semibold tracking-tight">Phanerite</span>
          <span class="font-mono text-micro text-muted-foreground">v0.1.0-pre</span>
        </div>
        <div class="flex-1"></div>
        <div class="flex h-full items-stretch">
          <button
            type="button"
            aria-label="Minimize"
            onclick={() => windowControl("Minimize")}
            class="flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <Minus class="size-4" />
          </button>
          <button
            type="button"
            aria-label="Maximize"
            onclick={() => windowControl("Maximize")}
            class="flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <Square class="size-3.5" />
          </button>
          <button
            type="button"
            aria-label="Close"
            onclick={() => windowControl("Close")}
            class="flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-destructive hover:text-white"
          >
            <X class="size-4" />
          </button>
        </div>
      </div>

      <!-- body: sidebar + app.view -->
      <div class="flex min-h-0 flex-1">
        <NavSidebar />
        <main class="min-h-0 min-w-0 flex-1 overflow-hidden">
          {@render children()}
        </main>
      </div>

      <LaunchTray />
      <StatusBar />
    </div>
  </div>
{/if}

<Toaster position="bottom-right" closeButton />
