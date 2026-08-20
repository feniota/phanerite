<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import { Slider } from "$lib/components/ui/slider";
  import { Switch } from "$lib/components/ui/switch";
  import { app } from "$lib/state.svelte";
  import type { LaunchSettings } from "$lib/types";
  import { Select as SelectPrimitive } from "bits-ui";
  import { ArrowLeft, Rocket } from "@lucide/svelte";

  const instance = $derived(app.activeInstance);
  const effective = $derived(instance ? app.getLaunchSettings(instance) : null);

  function overridden(key: keyof LaunchSettings) {
    return !!instance?.launchOverrides && key in instance.launchOverrides;
  }

  function useOverride<K extends keyof LaunchSettings>(key: K, enabled: boolean) {
    if (!instance || !effective) return;
    if (enabled) app.setLaunchOverride(instance.id, key, effective[key]);
    else app.clearLaunchOverride(instance.id, key);
  }

  function set<K extends keyof LaunchSettings>(key: K, value: LaunchSettings[K]) {
    if (instance) app.setLaunchOverride(instance.id, key, value);
  }
</script>

{#if instance && effective}
  <div class="flex h-full min-h-0 flex-col">
    <header class="shrink-0 border-b p-6 pb-4">
      <Button
        variant="ghost"
        size="sm"
        class="-ml-2"
        onclick={() => app.setView("instance-detail")}
      >
        <ArrowLeft data-icon="inline-start" />
        {instance.name}
      </Button>
      <div class="mt-3">
        <h1 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
          <Rocket class="size-4.5 text-primary" />
          Launch settings
        </h1>
        <p class="mt-0.5 text-xs text-muted-foreground">
          Values inherit global defaults until you override them for this instance.
        </p>
      </div>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto p-6">
      <div class="flex flex-col gap-4">
        <Card>
          <CardHeader class="p-4 pb-2">
            <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground"
              >Memory & window</CardTitle
            >
          </CardHeader>
          <CardContent class="flex flex-col gap-4">
            <Field.Field>
              <div class="flex items-center justify-between">
                <Field.FieldLabel>Memory allocation</Field.FieldLabel>
                <Switch
                  checked={overridden("memory")}
                  onCheckedChange={(enabled) => useOverride("memory", enabled)}
                />
              </div>
              <div class="flex items-center justify-between gap-3">
                <Field.FieldLabel>Allocation strategy</Field.FieldLabel>
                <Switch
                  checked={overridden("memoryMode")}
                  onCheckedChange={(enabled) => useOverride("memoryMode", enabled)}
                />
              </div>
              <Select.Root
                type="single"
                value={effective.memoryMode}
                onValueChange={(value) => set("memoryMode", value as LaunchSettings["memoryMode"])}
              >
                <Select.Trigger class="w-full text-xs" disabled={!overridden("memoryMode")}>
                  <SelectPrimitive.Value />
                </Select.Trigger>
                <Select.Content>
                  <Select.Group>
                    <Select.Item value="auto">Automatic</Select.Item>
                    <Select.Item value="manual">Manual</Select.Item>
                  </Select.Group>
                </Select.Content>
              </Select.Root>
              <Slider
                type="single"
                value={effective.memory}
                min={1}
                max={16}
                step={1}
                disabled={!overridden("memory") || effective.memoryMode === "auto"}
                onValueChange={(value) => set("memory", value)}
              />
              <Field.FieldDescription>
                {overridden("memory")
                  ? `${effective.memory} GB instance override.`
                  : "Inherited from global launch settings."}
              </Field.FieldDescription>
              >
            </Field.Field>
            <Field.Field>
              <div class="flex items-center justify-between">
                <Field.FieldLabel>Window mode</Field.FieldLabel>
                <Switch
                  checked={overridden("windowMode")}
                  onCheckedChange={(enabled) => useOverride("windowMode", enabled)}
                />
              </div>
              <Select.Root
                type="single"
                value={effective.windowMode}
                onValueChange={(value) => set("windowMode", value as LaunchSettings["windowMode"])}
              >
                <Select.Trigger class="w-full text-xs" disabled={!overridden("windowMode")}
                  ><SelectPrimitive.Value /></Select.Trigger
                >
                <Select.Content
                  ><Select.Group
                    ><Select.Item value="windowed">Windowed</Select.Item><Select.Item
                      value="maximized">Maximized</Select.Item
                    ><Select.Item value="fullscreen">Fullscreen</Select.Item></Select.Group
                  ></Select.Content
                >
              </Select.Root>
            </Field.Field>
            <div class="grid grid-cols-2 gap-3">
              <Field.Field>
                <Field.FieldLabel>Width</Field.FieldLabel>
                <Input
                  type="number"
                  value={effective.windowWidth}
                  disabled={!overridden("windowWidth")}
                  oninput={(event) => set("windowWidth", Number(event.currentTarget.value))}
                />
              </Field.Field>
              <Field.Field>
                <Field.FieldLabel>Height</Field.FieldLabel>
                <Input
                  type="number"
                  value={effective.windowHeight}
                  disabled={!overridden("windowHeight")}
                  oninput={(event) => set("windowHeight", Number(event.currentTarget.value))}
                />
              </Field.Field>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader class="p-4 pb-2">
            <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground"
              >Quick play</CardTitle
            >
            <CardDescription class="text-micro"
              >Open a destination as soon as the game starts.</CardDescription
            >
          </CardHeader>
          <CardContent class="flex flex-col gap-3">
            <Select.Root
              type="single"
              value={effective.quickPlayMode}
              onValueChange={(value) =>
                set("quickPlayMode", value as LaunchSettings["quickPlayMode"])}
            >
              <Select.Trigger class="w-full text-xs"><SelectPrimitive.Value /></Select.Trigger>
              <Select.Content
                ><Select.Group
                  ><Select.Item value="none">No quick play</Select.Item><Select.Item value="server"
                    >Multiplayer server</Select.Item
                  ><Select.Item value="world">Single-player world</Select.Item><Select.Item
                    value="realms">Realms ID</Select.Item
                  ></Select.Group
                ></Select.Content
              >
            </Select.Root>
            {#if effective.quickPlayMode !== "none"}
              <Input
                value={effective.quickPlayTarget}
                placeholder={effective.quickPlayMode === "server"
                  ? "Server address"
                  : effective.quickPlayMode === "world"
                    ? "World name"
                    : "Realms ID"}
                oninput={(event) => set("quickPlayTarget", event.currentTarget.value)}
              />
            {/if}
          </CardContent>
        </Card>

        <Card>
          <CardHeader class="p-4 pb-2">
            <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground"
              >Advanced</CardTitle
            >
            <CardDescription class="text-micro"
              >Commands, JVM options, and native library overrides.</CardDescription
            >
          </CardHeader>
          <CardContent class="flex flex-col gap-4">
            {#each [["Game arguments", "gameArgs", "Additional arguments passed to Minecraft."], ["Environment variables", "environmentVariables", "KEY=value pairs, separated by spaces."], ["JVM arguments", "javaArgs", "Additional arguments passed to Java."], ["Pre-launch command", "preLaunchCommand", "Runs before the game starts."], ["Command wrapper", "commandWrapper", "Wraps the game launch command."], ["Post-exit command", "postExitCommand", "Runs after the game exits."], ["Custom natives path", "nativesPath", "Overrides the generated natives directory."], ["Custom GLFW path", "glfwPath", "Optional GLFW native library."], ["Custom OpenAL path", "openalPath", "Optional OpenAL native library."]] as [label, key, description] (key)}
              {@const settingKey = key as keyof LaunchSettings}
              <Field.Field>
                <div class="flex items-center justify-between">
                  <Field.FieldLabel>{label}</Field.FieldLabel>
                  <Switch
                    checked={overridden(settingKey)}
                    onCheckedChange={(enabled) => useOverride(settingKey, enabled)}
                  />
                </div>
                <Input
                  value={effective[settingKey] as string}
                  disabled={!overridden(settingKey)}
                  class="font-mono text-micro"
                  oninput={(event) => set(settingKey, event.currentTarget.value)}
                />
                <Field.FieldDescription>{description}</Field.FieldDescription>
              </Field.Field>
            {/each}
            {#each [["Use default JVM arguments", "useDefaultJvmArgs"], ["Use optimized JVM arguments", "useOptimizedJvmArgs"], ["Skip JVM validation", "skipJvmValidation"], ["Allow automatic Java Agent", "allowAutoAgent"], ["Skip integrity check", "skipIntegrityCheck"], ["Use custom natives", "useCustomNatives"], ["Skip native patching", "skipNativePatching"]] as [label, key] (key)}
              {@const settingKey = key as keyof LaunchSettings}
              <Field.Field orientation="horizontal" class="justify-between">
                <Field.FieldLabel class="text-xs font-normal">{label}</Field.FieldLabel>
                <Switch
                  checked={effective[settingKey] as boolean}
                  onCheckedChange={(value) => set(settingKey, value)}
                />
              </Field.Field>
            {/each}
          </CardContent>
        </Card>
      </div>
    </div>
  </div>
{/if}
