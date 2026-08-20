<script lang="ts">
  import AccountsView from "$lib/components/app/views/AccountsView.svelte";
  import AphaniteView from "$lib/components/app/views/AphaniteView.svelte";
  import CrashReportView from "$lib/components/app/views/CrashReportView.svelte";
  import InstanceDetailView from "$lib/components/app/views/InstanceDetailView.svelte";
  import InstanceLaunchSettingsView from "$lib/components/app/views/InstanceLaunchSettingsView.svelte";
  import LogsView from "$lib/components/app/views/LogsView.svelte";
  import InstancesView from "$lib/components/app/views/InstancesView.svelte";
  import ModsView from "$lib/components/app/views/ModsView.svelte";
  import PacksView from "$lib/components/app/views/PacksView.svelte";
  import PlayView from "$lib/components/app/views/PlayView.svelte";
  import SettingsView from "$lib/components/app/views/SettingsView.svelte";
  import ShaderPacksView from "$lib/components/app/views/ShaderPacksView.svelte";
  import WorldsView from "$lib/components/app/views/WorldsView.svelte";
  import { app } from "$lib/state.svelte";
</script>

{#if app.view === "play"}
  <PlayView />
{:else if app.view === "instances"}
  <InstancesView />
{:else if app.view === "aphanite"}
  <AphaniteView />
{:else if app.view === "instance-detail"}
  <InstanceDetailView />
{:else if app.view === "mods"}
  <ModsView />
{:else if app.view === "packs"}
  <PacksView />
{:else if app.view === "shaders"}
  <ShaderPacksView />
{:else if app.view === "worlds"}
  <WorldsView />
{:else if app.view === "logs"}
  <LogsView />
{:else if app.view === "crash"}
  <CrashReportView />
{:else if app.view === "launch-settings"}
  <InstanceLaunchSettingsView />
{:else if app.view === "accounts"}
  <AccountsView />
{:else}
  <SettingsView />
{/if}
