<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import { app } from "$lib/state.svelte";
  import type { LaunchJob } from "$lib/types";
  import { onDestroy, untrack } from "svelte";
  import { X } from "@lucide/svelte";
  import { toast } from "svelte-sonner";
  import LabBlock from "../../../routes/lab/LabBlock.svelte";

  let { job }: { job: LaunchJob } = $props();

  const instance = $derived(app.instances.find((item) => item.id === job.instanceId) ?? null);

  const PHASES = [
    "Preparing environment",
    "Resolving versions",
    "Downloading assets",
    "Downloading libraries",
    "Extracting natives",
    "Launching game",
  ];
  const PHASE_AT = [10, 22, 62, 84, 94, 100];

  let timer: ReturnType<typeof setInterval> | undefined;
  let finishTimer: ReturnType<typeof setTimeout> | undefined;
  let finished = false;

  function ts() {
    return new Date().toTimeString().slice(0, 8);
  }

  function line(message: string) {
    // Each row owns its timer. Keep log writes outside the effect dependency
    // graph so appending a line cannot restart progress simulation.
    untrack(() => {
      app.appendLog(job.instanceId, `[${ts()}] [Phanerite/INFO]: ${message}`);
    });
  }

  function cancel() {
    if (!instance) return;
    app.cancelLaunch(job.instanceId);
    toast.info(`Launch of “${instance.name}” cancelled`, {
      description: "Downloaded files are kept for the next attempt.",
    });
  }

  $effect(() => {
    const inst = instance;
    if (!inst || job.status !== "starting") return;

    line(`Preparing instance '${inst.name}' (${inst.mcVersion})`);
    line(`Loader: ${inst.loader} ${inst.loaderVersion}`);

    timer = setInterval(() => {
      job.progress = Math.min(job.progress + 0.4 + Math.random() * 1.4, 100);
      const next = PHASE_AT.findIndex((point) => job.progress < point);
      const nextPhase = next === -1 ? 5 : next;
      if (nextPhase !== job.phase) {
        job.phase = nextPhase;
        line(
          PHASES[nextPhase] +
            (nextPhase === 2
              ? ` — ${app.settings.downloadThreads} threads · ${(12 + Math.random() * 40).toFixed(1)} MB/s`
              : nextPhase === 3
                ? ` — ${80 + Math.floor(Math.random() * 220)} libraries`
                : ""),
        );
      }
      if (job.progress >= 100 && !finished) {
        finished = true;
        if (timer) clearInterval(timer);
        line("Game process spawned, session started.");
        finishTimer = setTimeout(() => {
          app.finishLaunch(inst.id);
          toast.success(`“${inst.name}” is running`, {
            action: { label: "View logs", onClick: () => app.openLogs(inst.id) },
          });
        }, 250);
      }
    }, 35);

    return () => {
      if (timer) clearInterval(timer);
      if (finishTimer) clearTimeout(finishTimer);
    };
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
    if (finishTimer) clearTimeout(finishTimer);
  });
</script>

{#if instance && job.status === "starting"}
  <div class="flex items-center gap-3 rounded-md border bg-card px-3 py-2">
    <LabBlock
      seed={`${instance.name}|${instance.mcVersion}|${instance.loader}`}
      loader={instance.loader}
      order="nucleation"
      palette="ramp"
      progress={job.progress}
      ghost={false}
      failed={false}
      class="size-6 shrink-0"
    />
    <div class="min-w-0 w-44 shrink-0">
      <p class="truncate text-xs font-medium">{instance.name}</p>
      <p class="truncate text-micro text-muted-foreground">{PHASES[job.phase]}</p>
    </div>
    <Progress value={job.progress} class="min-w-24 flex-1" />
    <span class="w-9 text-right font-mono text-micro text-muted-foreground">
      {Math.floor(job.progress)}%
    </span>
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label={`Cancel launch of ${instance.name}`}
      onclick={cancel}
    >
      <X />
    </Button>
  </div>
{/if}
