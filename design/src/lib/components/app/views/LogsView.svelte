<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import { app } from "$lib/state.svelte";
  import { Select as SelectPrimitive } from "bits-ui";
  import { ArrowLeft, FileText, Search, ScrollText, Terminal } from "@lucide/svelte";

  const instance = $derived(app.activeInstance);
  const running = $derived(instance ? app.runningIds.includes(instance.id) : false);
  type LogLevel = "all" | "info" | "warn" | "error" | "debug";
  type LogSource = "live" | "latest" | "debug";

  let source = $state<LogSource>("live");
  let level = $state<LogLevel>("all");
  let query = $state("");
  let outputCount = $state(0);

  const ARCHIVED_LOGS: Record<Exclude<LogSource, "live">, string[]> = {
    latest: [
      "[12:14:03] [Render thread/INFO]: Reloading ResourceManager: vanilla",
      "[12:14:06] [Server thread/INFO]: Local session started",
      "[12:14:08] [Render thread/WARN]: Missing optional resource pack metadata",
      "[12:15:17] [Server thread/INFO]: Saving chunks for level 'ServerLevel[world]'",
    ],
    debug: [
      "[12:14:02] [main/DEBUG]: Launch arguments resolved",
      "[12:14:03] [main/INFO]: Using Java runtime version 21",
      "[12:14:04] [main/DEBUG]: Fabric loader initialized",
      "[12:14:07] [Render thread/WARN]: Skipping unavailable GPU extension",
    ],
  };
  const LIVE_OUTPUT = [
    "[Render thread/INFO]: Chunk render batches rebuilt",
    "[Server thread/INFO]: Saving players",
    "[Render thread/INFO]: Sound engine tick",
    "[Server thread/INFO]: Autosave completed",
  ];

  $effect(() => {
    const inst = instance;
    if (!inst || !running) return;
    const timer = setInterval(() => {
      const message = LIVE_OUTPUT[outputCount++ % LIVE_OUTPUT.length];
      app.appendLog(inst.id, `[${new Date().toTimeString().slice(0, 8)}] ${message}`);
    }, 1200);
    return () => clearInterval(timer);
  });

  const rawLines = $derived(
    source === "live" ? (instance ? app.getLogs(instance.id) : []) : ARCHIVED_LOGS[source],
  );
  const lines = $derived(
    rawLines.filter((line) => {
      const matchesLevel = level === "all" || line.toLowerCase().includes(`/${level}]`);
      const matchesQuery = !query.trim() || line.toLowerCase().includes(query.trim().toLowerCase());
      return matchesLevel && matchesQuery;
    }),
  );

  function getLevel(line: string): Exclude<LogLevel, "all"> {
    if (line.includes("/ERROR]")) return "error";
    if (line.includes("/WARN]")) return "warn";
    if (line.includes("/DEBUG]")) return "debug";
    return "info";
  }

  function levelClass(line: string) {
    const classes = {
      info: "text-primary/80",
      warn: "text-yellow-500",
      error: "text-destructive",
      debug: "text-muted-foreground",
    };
    return classes[getLevel(line)];
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="shrink-0 border-b p-6 pb-4">
    <Button variant="ghost" size="sm" class="-ml-2" onclick={() => app.closeLogs()}>
      <ArrowLeft data-icon="inline-start" />
      Back
    </Button>
    <div class="mt-3 flex items-start justify-between gap-4">
      <div class="min-w-0">
        <h1 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
          <ScrollText class="size-4.5 text-primary" />
          Game logs
          {#if source === "live" && running}<Badge class="gap-1"
              ><span class="size-1.5 rounded-full bg-primary-foreground"></span>Live</Badge
            >{/if}
        </h1>
        <p class="mt-0.5 text-xs text-muted-foreground">
          {source === "live" && running
            ? "Streaming the game process stdout."
            : "Inspect output captured for this instance."}
        </p>
      </div>
    </div>
  </header>

  <div class="flex min-h-0 flex-1 flex-col gap-4 p-6">
    <div class="flex flex-wrap gap-3">
      <Select.Root bind:value={source} type="single">
        <Select.Trigger class="w-44 text-xs">
          <Terminal data-icon="inline-start" />
          <SelectPrimitive.Value />
        </Select.Trigger>
        <Select.Content>
          <Select.Group>
            <Select.Item value="live">Live output</Select.Item>
            <Select.Item value="latest">latest.log</Select.Item>
            <Select.Item value="debug">debug.log</Select.Item>
          </Select.Group>
        </Select.Content>
      </Select.Root>
      <Select.Root bind:value={level} type="single">
        <Select.Trigger class="w-32 text-xs">
          <SelectPrimitive.Value />
        </Select.Trigger>
        <Select.Content>
          <Select.Group>
            <Select.Item value="all">All levels</Select.Item>
            <Select.Item value="info">Info</Select.Item>
            <Select.Item value="warn">Warning</Select.Item>
            <Select.Item value="error">Error</Select.Item>
            <Select.Item value="debug">Debug</Select.Item>
          </Select.Group>
        </Select.Content>
      </Select.Root>
      <div class="relative min-w-48 flex-1">
        <Search
          class="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground"
        />
        <Input bind:value={query} placeholder="Search log entries…" class="pl-8 text-xs" />
      </div>
    </div>
    <div
      class="min-h-0 flex-1 overflow-auto rounded-md border bg-black/40 p-4 font-mono text-micro leading-relaxed"
    >
      {#if lines.length}
        {#each lines as line, index (`${index}-${line}`)}
          <div class={levelClass(line)}>{line}</div>
        {/each}
      {:else}
        <div class="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
          <FileText class="size-5" />
          <span>No matching log entries.</span>
        </div>
      {/if}
    </div>
  </div>
</div>
