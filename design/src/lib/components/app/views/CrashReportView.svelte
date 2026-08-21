<script lang="ts">
  import type { CrashFinding, CrashReport } from "$lib/crash";
  import { redact } from "$lib/redact";
  import { app } from "$lib/state.svelte";
  import { LOADER_LABEL } from "$lib/types";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent } from "$lib/components/ui/card";
  import {
    ArrowLeft,
    CircleAlert,
    Clipboard,
    Copy,
    FileDown,
    FolderOpen,
    Play,
    Send,
  } from "@lucide/svelte";
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";

  const report = $derived(app.activeCrashReport);
  const instance = $derived(app.activeInstance);
  const hasReport = $derived(report?.lines !== null);
  const output = $derived(report ? (report.lines ?? report.stderrTail) : []);

  let selectedFindingIndex = $state(0);
  let reportPanel = $state<HTMLElement>();

  const selectedFinding = $derived(
    report?.findings.length ? (report.findings[selectedFindingIndex] ?? report.findings[0]) : null,
  );
  const evidenceLines = $derived(new Set(selectedFinding?.evidenceLines ?? []));

  $effect(() => {
    report?.id;
    selectedFindingIndex = 0;
  });

  $effect(() => {
    selectedFinding?.evidenceLines;
    void tick().then(() => {
      reportPanel?.querySelector("[data-evidence]")?.scrollIntoView({ block: "center" });
    });
  });

  function sourceText(crash: CrashReport) {
    return (crash.lines ?? crash.stderrTail).join("\n");
  }

  function findingSummary(crash: CrashReport) {
    if (!crash.findings.length) return "- No local crash signature matched.";
    return crash.findings.map((finding) => `- ${finding.title} (${finding.rule})`).join("\n");
  }

  function aiText(crash: CrashReport) {
    const environment = crash.environment;
    const overrides = environment.activeOverrides.join(", ") || "none";
    const mods = environment.enabledMods.map((mod) => `${mod.name} ${mod.version}`).join("\n");
    return `My Minecraft game crashed while using the Phanerite launcher.
Please help me work out why.

## What the launcher already knows
- The launcher captured the crash report and process output below.
- Local crash-signature matches:
${findingSummary(crash)}
- File integrity was verified before launch.
- The Java runtime matches the version this instance requires.

## Environment
Minecraft ${environment.mcVersion} · ${LOADER_LABEL[environment.loader]} ${environment.loaderVersion}
Java ${environment.javaVersion} (${environment.javaName})
Memory ${environment.memory} GB · ${environment.os} · ${environment.gpu}
Overridden launch settings: ${overrides}

## Enabled mods (${environment.enabledMods.length})
${mods || "None"}

## Crash report
\`\`\`
${redact(sourceText(crash))}
\`\`\`

Please tell me the most likely cause and explain your reasoning.
If you need more information, tell me where to find it.`;
  }

  function postText(crash: CrashReport) {
    const environment = crash.environment;
    const mods = environment.enabledMods.map((mod) => `- ${mod.name} ${mod.version}`).join("\n");
    return `## Minecraft crash report

## Local crash-signature matches
${findingSummary(crash)}

| Environment | Value |
| --- | --- |
| Minecraft | ${environment.mcVersion} |
| Loader | ${LOADER_LABEL[environment.loader]} ${environment.loaderVersion} |
| Java | ${environment.javaVersion} (${environment.javaName}) |
| Memory | ${environment.memory} GB |
| OS / GPU | ${environment.os} · ${environment.gpu} |
| Overridden launch settings | ${environment.activeOverrides.join(", ") || "none"} |

<details>
<summary>${environment.enabledMods.length} enabled mods</summary>

${mods || "None"}
</details>

## Crash report
\`\`\`text
${redact(sourceText(crash))}
\`\`\``;
  }

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(`Copied — ${text.length} characters, credentials removed`);
    } catch {
      toast.error("Could not copy the report");
    }
  }

  function retry() {
    if (report) app.startLaunch(report.instanceId);
  }

  function applyFinding(finding: CrashFinding) {
    if (!report) return;
    if (finding.implicatedModIds?.[0])
      app.setModEnabled(report.instanceId, finding.implicatedModIds[0], false);
    if (finding.suggestedMemory)
      app.setLaunchOverride(report.instanceId, "memory", finding.suggestedMemory);
    app.startLaunch(report.instanceId);
  }

  function implicatedModName(finding: CrashFinding) {
    const id = finding.implicatedModIds?.[0];
    return instance?.mods.find((mod) => mod.id === id)?.name ?? "mod";
  }
</script>

{#if report && instance}
  <div class="flex h-full min-h-0 flex-col">
    <header class="shrink-0 border-b p-6 pb-4">
      <Button variant="ghost" size="sm" class="-ml-2" onclick={() => app.closeCrashReport()}>
        <ArrowLeft data-icon="inline-start" />
        Back
      </Button>
      <div class="mt-3 flex items-start justify-between gap-4">
        <div class="flex min-w-0 items-start gap-3">
          <CircleAlert class="mt-0.5 size-5 shrink-0 text-destructive" />
          <div class="min-w-0">
            <h1 class="truncate text-lg font-semibold tracking-tight">{instance.name} crashed</h1>
            <p class="mt-0.5 text-xs text-muted-foreground">
              {report.when} · exit code {report.exitCode}
            </p>
          </div>
        </div>
        {#if report.environment.source === "aphanite"}
          <Badge variant="secondary" class="shrink-0">Aphanite</Badge>
        {/if}
      </div>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto p-6">
      <div class="mx-auto flex max-w-5xl flex-col gap-6">
        {#if report.findings.length}
          <Card>
            <CardContent>
              <h2 class="text-sm font-semibold">Known crash patterns</h2>
              <p class="mt-1.5 text-sm text-muted-foreground">
                These are literal matches from local crash rules, not an AI diagnosis.
              </p>
              <div class="mt-4 flex flex-col gap-3">
                {#each report.findings as finding, index (finding.rule)}
                  <div
                    class="rounded-md border p-4"
                    class:border-primary={index === selectedFindingIndex}
                  >
                    <button
                      type="button"
                      class="flex w-full items-start justify-between gap-3 text-left"
                      onclick={() => (selectedFindingIndex = index)}
                      aria-pressed={index === selectedFindingIndex}
                    >
                      <span>
                        <span class="block text-sm font-medium">{finding.title}</span>
                        <span class="mt-1 block text-sm text-muted-foreground">
                          {finding.explanation}
                        </span>
                      </span>
                      <Badge variant="secondary" class="shrink-0">Matched</Badge>
                    </button>
                    {#if index === selectedFindingIndex && (finding.implicatedModIds?.length || finding.suggestedMemory)}
                      <div class="mt-4">
                        <Button size="sm" onclick={() => applyFinding(finding)}>
                          <Play data-icon="inline-start" />
                          {finding.implicatedModIds?.length
                            ? `Disable ${implicatedModName(finding)} and retry`
                            : `Raise memory to ${finding.suggestedMemory} GB and retry`}
                        </Button>
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            </CardContent>
          </Card>

          {#if selectedFinding && hasReport}
            <Card>
              <CardContent>
                <h2 class="text-sm font-semibold">Evidence</h2>
                <pre
                  class="mt-3 overflow-x-auto whitespace-pre-wrap font-mono text-micro leading-relaxed text-muted-foreground">{selectedFinding.evidenceLines
                    .map((line) => report.lines?.[line])
                    .filter(Boolean)
                    .join("\n")}</pre>
              </CardContent>
            </Card>
          {/if}
        {:else}
          <Card>
            <CardContent class="flex flex-wrap items-center gap-4">
              <div class="min-w-0 flex-1">
                <h2 class="text-sm font-semibold">No known crash pattern matched</h2>
                <p class="mt-1.5 max-w-3xl text-sm text-muted-foreground">
                  Phanerite could not match this output to one of its local rules. Copy the prepared
                  prompt at the bottom to ask an AI assistant to inspect the full report and
                  environment.
                </p>
              </div>
              <Button variant="outline" size="sm" onclick={retry}>
                <Play data-icon="inline-start" />
                Retry
              </Button>
            </CardContent>
          </Card>
        {/if}

        {#if hasReport}
          <Card>
            <CardContent class="p-0">
              <div class="flex items-center justify-between gap-3 border-b px-5 py-3">
                <h2 class="text-sm font-semibold">Crash report</h2>
                <Button variant="ghost" size="sm" onclick={() => copy(redact(sourceText(report)))}>
                  <Copy data-icon="inline-start" />
                  Copy
                </Button>
              </div>
              <div
                bind:this={reportPanel}
                class="h-96 overflow-auto bg-black/40 p-4 font-mono text-micro leading-relaxed"
              >
                {#each output as line, index (`${index}-${line}`)}
                  <div
                    data-evidence={evidenceLines.has(index) ? "" : undefined}
                    class={`whitespace-pre-wrap break-all ${evidenceLines.has(index) ? "bg-primary/15 text-primary" : ""}`}
                  >
                    {line}
                  </div>
                {/each}
              </div>
            </CardContent>
          </Card>
        {:else}
          <Card>
            <CardContent>
              <div class="flex items-start gap-3">
                <CircleAlert class="mt-0.5 size-4 shrink-0 text-destructive" />
                <div>
                  <h2 class="text-sm font-semibold">No crash report was written</h2>
                  <p class="mt-1.5 text-sm text-muted-foreground">
                    Minecraft did not write a crash report. This usually means the Java VM itself
                    terminated, rather than the game code failing.
                  </p>
                </div>
              </div>
              <div
                class="mt-5 rounded-md border bg-black/40 p-4 font-mono text-micro leading-relaxed"
              >
                <h3 class="mb-3 font-sans text-xs font-semibold text-foreground">
                  Process output (last 20 lines)
                </h3>
                {#each report.stderrTail as line, index (`${index}-${line}`)}
                  <div class="whitespace-pre-wrap break-all text-muted-foreground">{line}</div>
                {/each}
              </div>
              {#if report.hsErrPath}
                <div class="mt-4 flex flex-wrap items-center gap-3">
                  <p class="break-all text-xs text-muted-foreground">
                    A JVM error file was written to {report.hsErrPath}
                  </p>
                  <Button
                    variant="outline"
                    size="sm"
                    onclick={() =>
                      toast.info("Reveal in file manager", {
                        description: "Reveals the JVM error file in your file manager.",
                      })}
                  >
                    <FolderOpen data-icon="inline-start" />
                    Reveal in file manager
                  </Button>
                </div>
              {/if}
            </CardContent>
          </Card>
        {/if}

        <Card>
          <CardContent>
            <h2 class="text-sm font-semibold">Environment</h2>
            <dl class="mt-4 grid gap-3 text-sm sm:grid-cols-2">
              <div class="flex justify-between gap-3">
                <dt class="text-muted-foreground">Minecraft</dt>
                <dd>{report.environment.mcVersion}</dd>
              </div>
              <div class="flex justify-between gap-3">
                <dt class="text-muted-foreground">Java</dt>
                <dd>{report.environment.javaName} {report.environment.javaVersion}</dd>
              </div>
              <div class="flex justify-between gap-3">
                <dt class="text-muted-foreground">Loader</dt>
                <dd>
                  {LOADER_LABEL[report.environment.loader]}
                  {report.environment.loaderVersion}
                </dd>
              </div>
              <div class="flex justify-between gap-3">
                <dt class="text-muted-foreground">Memory</dt>
                <dd>{report.environment.memory} GB</dd>
              </div>
              <div class="sm:col-span-2 flex justify-between gap-3">
                <dt class="text-muted-foreground">OS / GPU</dt>
                <dd class="text-right">{report.environment.os} · {report.environment.gpu}</dd>
              </div>
            </dl>
            {#if report.environment.activeOverrides.length}
              <p class="mt-4 text-xs text-yellow-500">
                Launch settings overridden: {report.environment.activeOverrides.join(", ")}
              </p>
            {/if}
            <details class="mt-4">
              <summary class="cursor-pointer text-xs font-medium"
                >{report.environment.enabledMods.length} enabled mods</summary
              >
              <ul class="mt-3 grid gap-1 text-xs text-muted-foreground sm:grid-cols-2">
                {#each report.environment.enabledMods as mod (`${mod.name}-${mod.version}`)}
                  <li>{mod.name} {mod.version}</li>
                {/each}
              </ul>
            </details>
          </CardContent>
        </Card>
      </div>
    </div>

    <footer class="flex shrink-0 flex-wrap items-center gap-2 border-t p-4">
      <Button size="sm" onclick={() => copy(aiText(report))}>
        <Clipboard data-icon="inline-start" />
        Copy for AI
      </Button>
      <Button variant="outline" size="sm" onclick={() => copy(postText(report))}>
        <Copy data-icon="inline-start" />
        Copy as post
      </Button>
      <Button
        variant="outline"
        size="sm"
        onclick={() => toast.info("Save report", { description: "Saves a portable report file." })}
      >
        <FileDown data-icon="inline-start" />
        Save report…
      </Button>
      {#if report.environment.source === "aphanite"}
        <Button
          variant="outline"
          size="sm"
          class="ml-auto"
          onclick={() =>
            toast.info("Report to server owner", {
              description: `Prepares this report for ${report.environment.aphaniteServer ?? "the server owner"}.`,
            })}
        >
          <Send data-icon="inline-start" />
          Report to server owner
        </Button>
      {/if}
    </footer>
  </div>
{/if}
