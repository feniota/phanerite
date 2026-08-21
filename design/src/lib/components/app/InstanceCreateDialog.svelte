<script lang="ts">
  import { app } from "$lib/state.svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import { Slider } from "$lib/components/ui/slider";
  import { Textarea } from "$lib/components/ui/textarea";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import type { Loader } from "$lib/types";
  import { Plus } from "@lucide/svelte";
  import { Select as SelectPrimitive } from "bits-ui";
  import { toast } from "svelte-sonner";

  let open = $state(false);
  let name = $state("");
  let description = $state("");
  let loader = $state<Loader>("vanilla");
  let mcVersion = $state("1.21.4");
  let loaderVersion = $state("");
  let memory = $state(4);

  const MC_VERSIONS = ["1.21.4", "1.21.1", "1.20.1", "1.19.4", "1.12.2"];
  const LOADERS: { value: Loader; label: string }[] = [
    { value: "vanilla", label: "Vanilla" },
    { value: "fabric", label: "Fabric" },
    { value: "forge", label: "Forge" },
    { value: "neoforge", label: "NeoForge" },
  ];

  const loaderPlaceholder = $derived(
    loader === "vanilla"
      ? "No loader"
      : loader === "fabric"
        ? "0.115.1+1.21.4"
        : loader === "forge"
          ? "47.3.0"
          : loader === "neoforge"
            ? "21.1.181"
            : "0.32.2+1.21.4",
  );

  function create() {
    if (!name.trim()) return;
    const id = app.createInstance({
      name: name.trim(),
      description: description.trim(),
      loader,
      mcVersion,
      loaderVersion: loader === "vanilla" ? "—" : loaderVersion.trim() || loaderPlaceholder,
      memory,
    });
    app.setActiveInstance(id);
    open = false;
    name = "";
    description = "";
    loaderVersion = "";
    memory = 4;
    toast.success(`Instance “${name.trim()}” created`, {
      description: `${mcVersion} · ${loader} · ${memory} GB`,
    });
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Trigger>
    <Button>
      <Plus data-icon="inline-start" />
      New instance
    </Button>
  </Dialog.Trigger>
  <Dialog.Content class="max-w-lg">
    <Dialog.Header>
      <Dialog.Title>Create instance</Dialog.Title>
      <Dialog.Description>
        Set up a fresh game installation. Everything stays editable later.
      </Dialog.Description>
    </Dialog.Header>

    <Field.FieldGroup class="flex flex-col gap-4">
      <Field.Field>
        <Field.FieldLabel for="inst-name">Name</Field.FieldLabel>
        <Input id="inst-name" bind:value={name} placeholder="e.g. Modded Survival" required />
      </Field.Field>

      <div class="grid grid-cols-2 gap-4">
        <Field.Field>
          <Field.FieldLabel>Game version</Field.FieldLabel>
          <Select.Root bind:value={mcVersion} type="single">
            <Select.Trigger class="w-full">
              <SelectPrimitive.Value placeholder="Pick a version" />
            </Select.Trigger>
            <Select.Content>
              {#each MC_VERSIONS as v (v)}
                <Select.Item value={v}>{v}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </Field.Field>

        <Field.Field>
          <Field.FieldLabel>Mod loader</Field.FieldLabel>
          <ToggleGroup.Root
            type="single"
            value={loader}
            spacing={2}
            onValueChange={(v) => {
              if (v) loader = v as Loader;
            }}
          >
            {#each LOADERS as l (l.value)}
              <ToggleGroup.Item value={l.value}>{l.label}</ToggleGroup.Item>
            {/each}
          </ToggleGroup.Root>
        </Field.Field>
      </div>

      <Field.Field>
        <Field.FieldLabel>Loader version</Field.FieldLabel>
        <Input
          bind:value={loaderVersion}
          placeholder={loaderPlaceholder}
          disabled={loader === "vanilla"}
        />
        <Field.FieldDescription>
          Leave empty to use the latest {loader === "vanilla" ? "" : loader + " "}build.
        </Field.FieldDescription>
      </Field.Field>

      <Field.Field>
        <Field.FieldLabel>Maximum memory — {memory} GB</Field.FieldLabel>
        <Slider type="single" bind:value={memory} min={2} max={16} step={1} />
        <Field.FieldDescription>Given to the JVM via -Xmx at launch.</Field.FieldDescription>
      </Field.Field>

      <Field.Field>
        <Field.FieldLabel>Description</Field.FieldLabel>
        <Textarea bind:value={description} placeholder="What is this instance for?" />
      </Field.Field>
    </Field.FieldGroup>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button onclick={create} disabled={!name.trim()}>Create instance</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
