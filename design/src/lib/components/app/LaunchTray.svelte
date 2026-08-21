<script lang="ts">
  import { app } from "$lib/state.svelte";
  import LaunchProgressRow from "./LaunchProgressRow.svelte";

  const jobs = $derived(app.launches.filter((job) => job.status === "starting"));
</script>

{#if jobs.length > 0}
  <div class="flex shrink-0 flex-col gap-1 border-t bg-background p-2">
    {#each jobs as job (job.instanceId)}
      <LaunchProgressRow {job} />
    {/each}
  </div>
{/if}
