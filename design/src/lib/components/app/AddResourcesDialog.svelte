<script lang="ts">
  import { app } from "$lib/state.svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Empty from "$lib/components/ui/empty";
  import { importableMods } from "$lib/seed";
  import type { Mod } from "$lib/types";
  import { FileUp, FolderOpen } from "@lucide/svelte";
  import { toast } from "svelte-sonner";

  let { mode, open = $bindable(false) }: { mode: "mod" | "pack" | "shader"; open?: boolean } =
    $props();

  const COPY = $derived(
    {
      mod: { title: "Add mods", ext: ".jar", sample: "mods" },
      pack: { title: "Import resource packs", ext: ".zip", sample: "resource packs" },
      shader: { title: "Import shader packs", ext: ".zip", sample: "shader packs" },
    }[mode],
  );

  const PACK_POOL = [
    {
      name: "Vanilla Tweaks",
      author: "xisumavoid",
      version: "1.21.4",
      description: "Subtle quality-of-life texture tweaks.",
      size: "2.3 MB",
    },
    {
      name: "Programmer Art",
      author: "Mojang",
      version: "1.21.4",
      description: "The classic pixel look, restored.",
      size: "1.8 MB",
    },
  ];
  const SHADER_POOL = [
    { name: "Complementary Unbound", author: "EminGT", version: "r5.3", gpu: "GTX 1060 6GB" },
    { name: "MakeUp Ultra Fast", author: "Capt Tatsu", version: "v9.1c", gpu: "Intel UHD 630" },
  ];

  function uid(prefix: string) {
    return `${prefix}-${crypto.randomUUID().slice(0, 8)}`;
  }

  function importSamples() {
    if (mode === "mod") {
      const picks = [...importableMods].sort(() => Math.random() - 0.5).slice(0, 3);
      const mods: Mod[] = picks.map((m) => ({
        id: uid("mod"),
        name: m.name,
        version: m.version,
        fileName: m.fileName,
        loader: m.loader,
        enabled: true,
      }));
      app.addMods(app.activeInstanceId, mods);
      toast.success(`Imported ${mods.length} mods`, {
        description: mods.map((m) => m.name).join(", "),
      });
    } else if (mode === "pack") {
      const picks = [...PACK_POOL].sort(() => Math.random() - 0.5).slice(0, 2);
      const packs = picks.map((p) => ({
        id: uid("pack"),
        name: p.name,
        author: p.author,
        version: p.version,
        description: p.description,
        size: p.size,
        enabled: true,
      }));
      app.addResourcePacks(app.activeInstanceId, packs);
      toast.success(`Imported ${packs.length} resource packs`, {
        description: packs.map((p) => p.name).join(", "),
      });
    } else {
      const picks = [...SHADER_POOL].sort(() => Math.random() - 0.5).slice(0, 2);
      const shaders = picks.map((s) => ({
        id: uid("shader"),
        name: s.name,
        author: s.author,
        version: s.version,
        gpu: s.gpu,
        enabled: true,
      }));
      app.addShaderPacks(app.activeInstanceId, shaders);
      toast.success(`Imported ${shaders.length} shader packs`, {
        description: shaders.map((s) => s.name).join(", "),
      });
    }
    open = false;
  }
</script>

<Dialog.Root bind:open>
  <Button onclick={() => (open = true)}>
    <FileUp data-icon="inline-start" />
    {COPY.title}
  </Button>
  <Dialog.Content class="max-w-md">
    <Dialog.Header>
      <Dialog.Title>{COPY.title}</Dialog.Title>
      <Dialog.Description>
        Files are copied into the selected instance's folder. This preview imports sample items
        instead.
      </Dialog.Description>
    </Dialog.Header>

    <div class="px-6 pb-2">
      <div
        role="button"
        tabindex="0"
        onclick={importSamples}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") importSamples();
        }}
        class="cursor-pointer"
      >
        <Empty.Root>
          <Empty.Media>
            <FileUp class="size-6 text-muted-foreground" />
          </Empty.Media>
          <Empty.Title>Drop {COPY.ext} files here</Empty.Title>
          <Empty.Description>
            or click to browse. In the real launcher this opens a native file picker.
          </Empty.Description>
        </Empty.Root>
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button onclick={importSamples}>
        <FolderOpen data-icon="inline-start" />
        Browse…
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
