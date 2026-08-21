<script lang="ts">
  import { app, ACCENT_OPTIONS } from "$lib/state.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import { Slider } from "$lib/components/ui/slider";
  import { Switch } from "$lib/components/ui/switch";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import type { AccentKey } from "$lib/types";
  import { Flame, Plus, RefreshCw, Settings as SettingsIcon } from "@lucide/svelte";
  import { Select as SelectPrimitive } from "bits-ui";
  import { toast } from "svelte-sonner";

  const ACCENT_DOT: Record<AccentKey, string> = {
    emerald: "var(--swatch-emerald)",
    gold: "var(--gold)",
    slate: "var(--swatch-slate)",
    teal: "var(--swatch-teal)",
  };

  const LANGUAGES = [
    { value: "en", label: "English" },
    { value: "zh", label: "简体中文" },
    { value: "de", label: "Deutsch" },
    { value: "ja", label: "日本語" },
  ];
  const FONT_SIZES = [
    { value: "sm", label: "Small" },
    { value: "md", label: "Medium" },
    { value: "lg", label: "Large" },
  ];

  const LAUNCH_TOGGLES = [
    { key: "closeAfterLaunch", label: "Close launcher after game starts" },
    { key: "hideOnLaunch", label: "Minimize to tray while playing" },
    { key: "multiInstance", label: "Allow multiple instances at once" },
    { key: "checkUpdates", label: "Check for updates on startup" },
  ] as const;
  const aphaniteConfigurations = $derived(app.instances.filter((instance) => instance.aphanite));
  const aphaniteInstalled = $derived(
    aphaniteConfigurations.filter((instance) => instance.lastPlayed !== null).length,
  );
  let runtimeName = $state("");
  let runtimeVersion = $state(21);
  let runtimePath = $state("");
  let addRuntimeOpen = $state(false);

  function addRuntime() {
    if (!runtimeName.trim() || !runtimePath.trim()) return;
    app.addJavaRuntime(runtimeName.trim(), runtimeVersion, runtimePath.trim());
    runtimeName = "";
    runtimePath = "";
    addRuntimeOpen = false;
    toast.success("Java runtime added");
  }

  function checkUpdates() {
    toast.success("Phanerite is up to date", {
      description: "v0.1.0-pre · checked just now.",
    });
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-y-auto">
  <header class="shrink-0 border-b p-6 pb-4">
    <h1 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
      <SettingsIcon class="size-4.5 text-primary" />
      Settings
    </h1>
    <p class="mt-0.5 text-xs text-muted-foreground">
      Global preferences. Instance-specific options live in each instance's detail panel.
    </p>
  </header>

  <div class="flex flex-1 flex-col gap-4 p-6">
    <Card>
      <CardHeader class="flex flex-row items-start gap-4 p-4 pb-2">
        <div>
          <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Java & runtime
          </CardTitle>
          <CardDescription class="text-micro">
            Phanerite manages Java runtimes for your instances.
          </CardDescription>
        </div>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <div class="flex flex-row items-center justify-between">
          <span class="text-xs/relaxed font-medium">Available Java runtimes</span>
          <Dialog.Root bind:open={addRuntimeOpen}>
            <Dialog.Trigger>
              <Button variant="outline" size="sm">
                <Plus data-icon="inline-start" />
                Add custom
              </Button>
            </Dialog.Trigger>
            <Dialog.Content class="max-w-md">
              <Dialog.Header>
                <Dialog.Title>Add Java runtime</Dialog.Title>
                <Dialog.Description>
                  Add a Java executable installed outside Phanerite’s managed runtimes.
                </Dialog.Description>
              </Dialog.Header>
              <form
                onsubmit={(event) => {
                  event.preventDefault();
                  addRuntime();
                }}
              >
                <Field.FieldGroup>
                  <Field.Field>
                    <Field.FieldLabel for="runtime-name">Name</Field.FieldLabel>
                    <Input
                      id="runtime-name"
                      bind:value={runtimeName}
                      placeholder="e.g. Oracle JDK 21"
                    />
                  </Field.Field>
                  <div class="grid grid-cols-[7rem_1fr] gap-3">
                    <Field.Field>
                      <Field.FieldLabel for="runtime-version">Version</Field.FieldLabel>
                      <Input
                        id="runtime-version"
                        type="number"
                        bind:value={runtimeVersion}
                        min={8}
                      />
                    </Field.Field>
                    <Field.Field>
                      <Field.FieldLabel for="runtime-path">Java executable</Field.FieldLabel>
                      <Input
                        id="runtime-path"
                        bind:value={runtimePath}
                        placeholder="/path/to/bin/java"
                        class="font-mono text-micro"
                      />
                    </Field.Field>
                  </div>
                </Field.FieldGroup>
                <Dialog.Footer class="mt-5">
                  <Button type="button" variant="outline" onclick={() => (addRuntimeOpen = false)}>
                    Cancel
                  </Button>
                  <Button type="submit" disabled={!runtimeName.trim() || !runtimePath.trim()}>
                    Add runtime
                  </Button>
                </Dialog.Footer>
              </form>
            </Dialog.Content>
          </Dialog.Root>
        </div>
        <div class="flex flex-col gap-2">
          {#each app.javaRuntimes as runtime (runtime.id)}
            <div class="flex items-center gap-3 rounded-md border bg-muted/20 p-3">
              <div class="min-w-0 flex-1">
                <p class="text-xs font-medium">{runtime.name}</p>
                <p class="mt-0.5 truncate font-mono text-micro text-muted-foreground">
                  {runtime.path}
                </p>
              </div>
              <Badge variant={runtime.managed ? "secondary" : "outline"}
                >Java {runtime.version}</Badge
              >
              {#if runtime.managed}<Badge>Managed</Badge>{/if}
            </div>
          {/each}
        </div>
        <Field.Field>
          <div class="flex items-center justify-between gap-3">
            <Field.FieldLabel>Memory allocation</Field.FieldLabel>
            <ToggleGroup.Root
              type="single"
              bind:value={app.launchSettings.memoryMode}
              variant="outline"
              size="sm"
              class="w-48"
              aria-label="Memory allocation mode"
            >
              <ToggleGroup.Item value="auto" class="flex-1">Automatic</ToggleGroup.Item>
              <ToggleGroup.Item value="manual" class="flex-1">Manual</ToggleGroup.Item>
            </ToggleGroup.Root>
          </div>
          <Slider
            type="single"
            value={app.launchSettings.memory}
            min={1}
            max={16}
            step={1}
            disabled={app.launchSettings.memoryMode === "auto"}
            onValueChange={(value) => (app.launchSettings.memory = value)}
          />
          <Field.FieldDescription>
            {app.launchSettings.memoryMode === "auto"
              ? "Phanerite chooses a safe allocation for the machine."
              : `${app.launchSettings.memory} GB maximum heap`}
          </Field.FieldDescription>
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel>JVM arguments</Field.FieldLabel>
          <Input bind:value={app.launchSettings.javaArgs} class="font-mono text-micro" />
        </Field.Field>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="p-4 pb-2">
        <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Game launch defaults
        </CardTitle>
        <CardDescription class="text-micro">
          Default values inherited by every instance.
        </CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <div class="grid grid-cols-2 gap-3">
          <Field.Field>
            <Field.FieldLabel>Window mode</Field.FieldLabel>
            <Select.Root bind:value={app.launchSettings.windowMode} type="single">
              <Select.Trigger class="w-full text-xs"><SelectPrimitive.Value /></Select.Trigger>
              <Select.Content
                ><Select.Group
                  ><Select.Item value="windowed">Windowed</Select.Item><Select.Item
                    value="maximized">Maximized</Select.Item
                  ><Select.Item value="fullscreen">Fullscreen</Select.Item></Select.Group
                ></Select.Content
              >
            </Select.Root>
          </Field.Field>
          <Field.Field>
            <Field.FieldLabel>Process priority</Field.FieldLabel>
            <Select.Root bind:value={app.settings.processPriority} type="single">
              <Select.Trigger class="w-full text-xs"><SelectPrimitive.Value /></Select.Trigger>
              <Select.Content
                ><Select.Group
                  ><Select.Item value="low">Low</Select.Item><Select.Item value="normal"
                    >Normal</Select.Item
                  ><Select.Item value="high">High</Select.Item></Select.Group
                ></Select.Content
              >
            </Select.Root>
          </Field.Field>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <Field.Field>
            <Field.FieldLabel>Window width</Field.FieldLabel>
            <Input type="number" bind:value={app.launchSettings.windowWidth} />
          </Field.Field>
          <Field.Field>
            <Field.FieldLabel>Window height</Field.FieldLabel>
            <Input type="number" bind:value={app.launchSettings.windowHeight} />
          </Field.Field>
        </div>
        {#each [["Show game logs", "showGameLogs"], ["Write debug logs", "debugLogs"], ["Generate game options", "generateGameOptions"], ["Use default JVM arguments", "useDefaultJvmArgs"], ["Use optimized JVM arguments", "useOptimizedJvmArgs"]] as [label, key] (key)}
          <Field.Field orientation="horizontal" class="justify-between">
            <Field.FieldLabel class="text-xs font-normal">{label}</Field.FieldLabel>
            <Switch
              checked={key in app.launchSettings
                ? (app.launchSettings[key as keyof typeof app.launchSettings] as boolean)
                : (app.settings[key as keyof typeof app.settings] as boolean)}
              onCheckedChange={(value) => {
                if (key in app.launchSettings)
                  app.launchSettings[key as keyof typeof app.launchSettings] = value as never;
                else app.settings[key as keyof typeof app.settings] = value as never;
              }}
            />
          </Field.Field>
        {/each}
        <Field.Field>
          <Field.FieldLabel>Environment variables</Field.FieldLabel>
          <Input
            bind:value={app.launchSettings.environmentVariables}
            class="font-mono text-micro"
            placeholder="KEY=value KEY2=value"
          />
        </Field.Field>
        <div class="grid grid-cols-3 gap-3">
          <Field.Field>
            <Field.FieldLabel>Pre-launch command</Field.FieldLabel>
            <Input bind:value={app.launchSettings.preLaunchCommand} class="font-mono text-micro" />
          </Field.Field>
          <Field.Field>
            <Field.FieldLabel>Command wrapper</Field.FieldLabel>
            <Input bind:value={app.launchSettings.commandWrapper} class="font-mono text-micro" />
          </Field.Field>
          <Field.Field>
            <Field.FieldLabel>Post-exit command</Field.FieldLabel>
            <Input bind:value={app.launchSettings.postExitCommand} class="font-mono text-micro" />
          </Field.Field>
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="p-4 pb-2">
        <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Graphics & native libraries
        </CardTitle>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <div class="grid grid-cols-2 gap-3">
          <Field.Field>
            <Field.FieldLabel>OpenGL renderer</Field.FieldLabel>
            <Input bind:value={app.settings.openglRenderer} />
          </Field.Field>
          <Field.Field>
            <Field.FieldLabel>Vulkan renderer</Field.FieldLabel>
            <Input bind:value={app.settings.vulkanRenderer} />
          </Field.Field>
        </div>
        {#each [["Prefer high-performance GPU", "preferHighPerformanceGpu"], ["Use custom natives", "useCustomNatives"], ["Skip native patching", "skipNativePatching"]] as [label, key] (key)}
          <Field.Field orientation="horizontal" class="justify-between">
            <Field.FieldLabel class="text-xs font-normal">{label}</Field.FieldLabel>
            <Switch
              checked={key in app.launchSettings
                ? (app.launchSettings[key as keyof typeof app.launchSettings] as boolean)
                : (app.settings[key as keyof typeof app.settings] as boolean)}
              onCheckedChange={(value) => {
                if (key in app.launchSettings)
                  app.launchSettings[key as keyof typeof app.launchSettings] = value as never;
                else app.settings[key as keyof typeof app.settings] = value as never;
              }}
            />
          </Field.Field>
        {/each}
        {#each [["Native library path", "nativesPath"], ["GLFW path", "glfwPath"], ["OpenAL path", "openalPath"]] as [label, key] (key)}
          <Field.Field>
            <Field.FieldLabel>{label}</Field.FieldLabel>
            <Input
              bind:value={app.launchSettings[key as keyof typeof app.launchSettings] as string}
              class="font-mono text-micro"
            />
          </Field.Field>
        {/each}
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="p-4 pb-2">
        <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Launcher
        </CardTitle>
        <CardDescription class="text-micro">Behaviour around launching.</CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-3">
        {#each LAUNCH_TOGGLES as row (row.key)}
          <Field.Field orientation="horizontal" class="justify-between">
            <Field.FieldLabel class="text-xs font-normal">
              {row.label}
            </Field.FieldLabel>
            <Switch
              checked={app.settings[row.key]}
              onCheckedChange={(c) => (app.settings[row.key] = c)}
            />
          </Field.Field>
        {/each}
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="p-4 pb-2">
        <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Appearance
        </CardTitle>
        <CardDescription class="text-micro">
          Phanerite is dark-only, like most launchers.
        </CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <Field.Field>
          <Field.FieldLabel>Accent color</Field.FieldLabel>
          <ToggleGroup.Root
            type="single"
            value={app.accent}
            spacing={2}
            onValueChange={(v) => {
              if (v) app.setAccent(v as AccentKey);
            }}
          >
            {#each ACCENT_OPTIONS as key (key)}
              <ToggleGroup.Item
                value={key}
                aria-label={`${key} accent`}
                class="size-8 rounded-full p-0"
              >
                <span class="size-4 rounded-full" style={`background-color: ${ACCENT_DOT[key]}`}
                ></span>
              </ToggleGroup.Item>
            {/each}
          </ToggleGroup.Root>
          <Field.FieldDescription>
            Switches the primary color across the whole UI.
          </Field.FieldDescription>
        </Field.Field>
        <div class="grid grid-cols-2 gap-4">
          <Field.Field>
            <Field.FieldLabel>Language</Field.FieldLabel>
            <Select.Root bind:value={app.settings.language} type="single">
              <Select.Trigger class="w-full text-xs">
                <SelectPrimitive.Value placeholder="Language" />
              </Select.Trigger>
              <Select.Content>
                {#each LANGUAGES as l (l.value)}
                  <Select.Item value={l.value}>{l.label}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </Field.Field>
          <Field.Field>
            <Field.FieldLabel>UI font size</Field.FieldLabel>
            <Select.Root bind:value={app.settings.fontSize} type="single">
              <Select.Trigger class="w-full text-xs">
                <SelectPrimitive.Value placeholder="Font size" />
              </Select.Trigger>
              <Select.Content>
                {#each FONT_SIZES as f (f.value)}
                  <Select.Item value={f.value}>{f.label}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </Field.Field>
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="p-4 pb-2">
        <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Network
        </CardTitle>
        <CardDescription class="text-micro">
          Download behaviour for assets and libraries.
        </CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <Field.Field>
          <Field.FieldLabel>
            Concurrent downloads — {app.settings.downloadThreads}
          </Field.FieldLabel>
          <Slider
            type="single"
            bind:value={app.settings.downloadThreads}
            min={1}
            max={16}
            step={1}
          />
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel>Connection timeout (s)</Field.FieldLabel>
          <Input
            type="number"
            bind:value={app.settings.connectionTimeout}
            class="h-7 w-28 text-xs"
          />
          <Field.FieldDescription>
            Seconds before a mirror connection is retried.
          </Field.FieldDescription>
        </Field.Field>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="p-4 pb-2">
        <CardTitle
          class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground"
        >
          <Flame class="size-3.5 text-red-500" />
          Connected Aphanite server
        </CardTitle>
        <CardDescription class="text-micro">
          Fetch server-managed modpack configurations.
        </CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-2">
        <Field.Field>
          <Field.FieldLabel>Server URL</Field.FieldLabel>
          <Input type="url" bind:value={app.settings.aphaniteServer} class="font-mono text-micro" />
          <Field.FieldDescription>
            Contains {aphaniteConfigurations.length} modpack configurations, {aphaniteInstalled} installed.
          </Field.FieldDescription>
        </Field.Field>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="p-4 pb-2">
        <CardTitle class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          About
        </CardTitle>
      </CardHeader>
      <CardContent class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-6 text-micro">
          <div>
            <p class="text-muted-foreground">Version</p>
            <p class="font-mono font-medium">0.1.0-pre</p>
          </div>
          <div>
            <p class="text-muted-foreground">Runtime</p>
            <p class="font-mono font-medium">Java 21.0.2</p>
          </div>
          <div>
            <p class="text-muted-foreground">License</p>
            <p class="font-medium">MIT</p>
          </div>
          <Badge variant="outline">Rust · GPUI · GPUI Components</Badge>
        </div>
        <Button variant="outline" onclick={checkUpdates}>
          <RefreshCw data-icon="inline-start" />
          Check for updates
        </Button>
      </CardContent>
    </Card>
  </div>
</div>
