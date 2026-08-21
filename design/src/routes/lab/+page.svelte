<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { cn } from "$lib/lib";
  import LabBlock from "./LabBlock.svelte";
  import LabCrystal from "./LabCrystal.svelte";
  import LabCrystallize from "./LabCrystallize.svelte";
  import LabPackIcon from "./LabPackIcon.svelte";
  import { LOADERS, type FillOrder, type Loader, type Taper } from "./shapes";

  /* ------------------------------------------------------------ crystal */

  let lattice = $state(9);
  let maxPrisms = $state(4);
  let taper = $state<Taper>("sharp");
  let gapMode = $state<"auto" | "on" | "off">("auto");
  let batch = $state(1);

  function gapAt(px: number) {
    if (gapMode === "auto") return px >= 24;
    return gapMode === "on";
  }

  const NAMES = [
    "Prominence II",
    "All the Mods 10",
    "Cottage Witch",
    "Vault Hunters",
    "Create: Above and Beyond",
    "Deep Dark Descent",
    "The Fog",
    "Old Faithful",
    "Better MC",
    "Nomifactory",
    "Enigmatica 9",
    "Stoneblock 3",
    "Terrafirma Rescue",
    "Divine Journey 2",
    "Skyfactory 5",
    "Vanilla Survival",
    "Crackpack III",
    "SevTech Ages",
    "Monifactory",
    "Fear Nightfall",
    "Cave Dweller Reborn",
    "Aged Oak",
    "NeoForge Server Test",
    "Lantern Rite",
  ];

  const gallery = $derived(
    LOADERS.map((loader, loaderIndex) => ({
      loader,
      items: Array.from({ length: 12 }, (_, index) => {
        const name = NAMES[(index + loaderIndex * 12) % NAMES.length];
        return { name, seed: `${name}|1.21.${batch}|${loader}` };
      }),
    })),
  );

  const SIZES = [
    { px: 16, class: "size-4", label: "16 侧栏" },
    { px: 20, class: "size-5", label: "20" },
    { px: 24, class: "size-6", label: "24 托盘" },
    { px: 32, class: "size-8", label: "32" },
    { px: 36, class: "size-9", label: "36 列表" },
    { px: 48, class: "size-12", label: "48" },
    { px: 56, class: "size-14", label: "56 推荐位" },
  ];

  const LADDER = $derived(
    [
      { name: "Prominence II", loader: "forge" as Loader },
      { name: "The Fog", loader: "fabric" as Loader },
      { name: "All the Mods 10", loader: "neoforge" as Loader },
      { name: "Vanilla Survival", loader: "vanilla" as Loader },
      { name: "Vault Hunters", loader: "forge" as Loader },
    ].map((entry) => ({ ...entry, seed: `${entry.name}|1.21.${batch}|${entry.loader}` })),
  );

  /* ----------------------------------------------------------- progress */

  const PHASES = [
    "Preparing environment",
    "Resolving versions",
    "Downloading assets",
    "Downloading libraries",
    "Extracting natives",
    "Launching game",
  ];
  const PHASE_AT = [10, 22, 62, 84, 94, 100];

  const VARIANTS: { key: FillOrder; title: string; note: string }[] = [
    {
      key: "scatter",
      title: "V1 散布",
      note: "纯哈希顺序，格子全盘浮现。唯一性最强，进度最难估。",
    },
    {
      key: "bottomUp",
      title: "V2 自下而上",
      note: "逐行上涨，行内仍是哈希顺序。进度最好读，唯一性只剩配色。",
    },
    {
      key: "nucleation",
      title: "V3 成核生长",
      note: "从一个哈希选定的成核点向外长。和「结晶」这个叙事最贴。",
    },
    {
      key: "dimLit",
      title: "V4 由暗到亮",
      note: "格子从头就在，只是暗着。最柔和，但把 100% 的成型感提前花掉了。",
    },
  ];

  let playing = $state(true);
  let speed = $state(1.2);
  let progress = $state(0);
  let stage = $state<"filling" | "settling" | "crystallized">("filling");
  let ghost = $state(true);
  let blockPalette = $state<"ramp" | "legacy">("ramp");
  let failed = $state(false);
  let demoLoader = $state<Loader>("fabric");
  let demoName = $state("Prominence II");

  const demoSeed = $derived(`${demoName}|1.21.1|${demoLoader}`);
  const phaseIndex = $derived(
    PHASE_AT.findIndex((point) => progress < point) === -1
      ? 5
      : PHASE_AT.findIndex((point) => progress < point),
  );

  $effect(() => {
    if (!playing) return;
    const step = speed;
    let hold = 0;
    const timer = setInterval(() => {
      if (failed) return;
      if (stage === "filling") {
        progress = Math.min(progress + step, 100);
        if (progress >= 100) {
          stage = "settling";
          hold = 0;
        }
        return;
      }
      hold += 1;
      if (stage === "settling" && hold > 8) {
        stage = "crystallized";
        hold = 0;
        return;
      }
      if (stage === "crystallized" && hold > 34) {
        progress = 0;
        stage = "filling";
      }
    }, 40);
    return () => clearInterval(timer);
  });

  function replay() {
    failed = false;
    progress = 0;
    stage = "filling";
    playing = true;
  }

  function simulateFailure() {
    playing = false;
    failed = true;
    stage = "filling";
    if (progress <= 0 || progress >= 100) progress = 47;
  }

  function scrub(event: Event & { currentTarget: HTMLInputElement }) {
    playing = false;
    failed = false;
    stage = "filling";
    progress = Number(event.currentTarget.value);
  }

  /* ------------------------------------------------------------- mixing */

  const MIXED: { name: string; version: string; loader: Loader; pack: number | null }[] = [
    { name: "Prominence II", version: "1.20.1", loader: "fabric", pack: 0 },
    { name: "The Fog", version: "1.21.1", loader: "fabric", pack: null },
    { name: "All the Mods 10", version: "1.21.1", loader: "neoforge", pack: 2 },
    { name: "Old Faithful", version: "1.7.10", loader: "forge", pack: null },
    { name: "Cottage Witch", version: "1.20.1", loader: "fabric", pack: 1 },
    { name: "Fabulously Optimized", version: "26.2", loader: "fabric", pack: 3 },
    { name: "Vanilla Survival", version: "1.21.4", loader: "vanilla", pack: null },
  ];

  const CONTROL =
    "h-8 rounded-md border border-input bg-secondary px-2 text-xs text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring";
</script>

<svelte:head>
  <title>Lab — 方块与晶簇</title>
</svelte:head>

<div class="min-h-svh bg-background text-foreground">
  <div class="mx-auto flex max-w-[1400px] flex-col gap-12 p-8">
    <header class="flex items-start justify-between gap-6 border-b pb-6">
      <div>
        <h1 class="text-lg font-semibold tracking-tight">方块与晶簇 · 设计沙盒</h1>
        <p class="mt-1 max-w-3xl text-xs text-muted-foreground">
          晶簇替代默认图标（透明底、不规则、无边界），方块退到启动过程里。两者由同一个
          <code class="rounded bg-secondary px-1 py-0.5 font-mono">name|mcVersion|loader</code>
          哈希生成 —— 同一颗矿的两种形态。这个路由和
          <code class="rounded bg-secondary px-1 py-0.5 font-mono">src/routes/lab/</code> 可以整个删掉。
        </p>
      </div>
      <Button variant="outline" size="sm" href="/">回到 App</Button>
    </header>

    <!-- ============================================================ 晶簇 -->
    <section class="flex flex-col gap-5">
      <div>
        <h2 class="text-sm font-semibold">一 · 晶簇（默认图标的替代范式）</h2>
        <p class="mt-1 text-xs text-muted-foreground">
          先看分布，不要只看单个 —— 生成式图形的成败在平均水平，不在挑出来的那一个。
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-4 rounded-md border bg-card p-3">
        <label class="flex items-center gap-2 text-xs">
          <span class="text-muted-foreground">点阵</span>
          <select
            class={CONTROL}
            value={lattice}
            onchange={(event) => (lattice = Number(event.currentTarget.value))}
          >
            <option value={7}>7 × 7</option>
            <option value={9}>9 × 9</option>
            <option value={11}>11 × 11</option>
            <option value={13}>13 × 13</option>
          </select>
        </label>
        <label class="flex items-center gap-2 text-xs">
          <span class="text-muted-foreground">最多几根柱</span>
          <select
            class={CONTROL}
            value={maxPrisms}
            onchange={(event) => (maxPrisms = Number(event.currentTarget.value))}
          >
            <option value={2}>2</option>
            <option value={3}>3</option>
            <option value={4}>4</option>
            <option value={5}>5</option>
          </select>
        </label>
        <label class="flex items-center gap-2 text-xs">
          <span class="text-muted-foreground">收尖</span>
          <select
            class={CONTROL}
            value={taper}
            onchange={(event) => (taper = event.currentTarget.value as Taper)}
          >
            <option value="sharp">陡（1 行收完）</option>
            <option value="gradual">缓（2 行收完）</option>
          </select>
        </label>
        <label class="flex items-center gap-2 text-xs">
          <span class="text-muted-foreground">格间隙</span>
          <select
            class={CONTROL}
            value={gapMode}
            onchange={(event) => (gapMode = event.currentTarget.value as "auto" | "on" | "off")}
          >
            <option value="auto">自动（≥24px 才留）</option>
            <option value="on">一律留</option>
            <option value="off">一律不留</option>
          </select>
        </label>
        <Button variant="outline" size="sm" onclick={() => (batch += 1)}>换一批种子</Button>
      </div>

      <div class="flex flex-col gap-3 rounded-md border bg-card p-4">
        {#each gallery as row (row.loader)}
          <div class="flex items-center gap-4">
            <span class="w-20 shrink-0 text-micro uppercase tracking-wider text-muted-foreground">
              {row.loader}
            </span>
            <div class="flex flex-wrap items-end gap-3">
              {#each row.items as item (item.seed)}
                <LabCrystal
                  seed={item.seed}
                  loader={row.loader}
                  {lattice}
                  {maxPrisms}
                  {taper}
                  gap={gapAt(48)}
                  class="size-12"
                />
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <div class="rounded-md border bg-card p-4">
        <h3 class="text-xs font-semibold">尺寸阶梯 —— 16px 下轮廓还立得住吗</h3>
        <div class="mt-4 flex flex-col gap-4">
          <div class="flex items-end gap-6 pl-40">
            {#each SIZES as size (size.px)}
              <span class="w-14 text-center text-micro text-muted-foreground">{size.label}</span>
            {/each}
          </div>
          {#each LADDER as entry (entry.seed)}
            <div class="flex items-center gap-6">
              <span class="w-40 shrink-0 truncate text-xs text-muted-foreground">{entry.name}</span>
              {#each SIZES as size (size.px)}
                <span class="flex w-14 justify-center">
                  <LabCrystal
                    seed={entry.seed}
                    loader={entry.loader}
                    {lattice}
                    {maxPrisms}
                    {taper}
                    gap={gapAt(size.px)}
                    class={size.class}
                  />
                </span>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    </section>

    <!-- ======================================================== 加载动画 -->
    <section class="flex flex-col gap-5">
      <div>
        <h2 class="text-sm font-semibold">二 · 方块作为启动进度</h2>
        <p class="mt-1 text-xs text-muted-foreground">
          四种填充顺序共用一个进度值，方便直接对比。100% 之后方块结晶成该实例的晶簇。
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-4 rounded-md border bg-card p-3">
        <Button variant="outline" size="sm" onclick={() => (playing = !playing)}>
          {playing ? "暂停" : "播放"}
        </Button>
        <Button variant="outline" size="sm" onclick={replay}>重放</Button>
        <Button variant="outline" size="sm" onclick={simulateFailure}>模拟启动失败</Button>
        <label class="flex items-center gap-2 text-xs">
          <span class="text-muted-foreground">进度</span>
          <input
            type="range"
            min="0"
            max="100"
            step="1"
            value={progress}
            oninput={scrub}
            class="w-56 accent-primary"
          />
          <span class="w-10 text-right font-mono text-micro text-muted-foreground">
            {Math.floor(progress)}%
          </span>
        </label>
        <label class="flex items-center gap-2 text-xs">
          <span class="text-muted-foreground">速度</span>
          <input
            type="range"
            min="0.3"
            max="4"
            step="0.1"
            bind:value={speed}
            class="w-24 accent-primary"
          />
        </label>
        <label class="flex items-center gap-2 text-xs">
          <input type="checkbox" bind:checked={ghost} class="accent-primary" />
          <span class="text-muted-foreground">未填充格显示轮廓幽灵</span>
        </label>
        <label class="flex items-center gap-2 text-xs">
          <span class="text-muted-foreground">方块配色</span>
          <select
            class={CONTROL}
            value={blockPalette}
            onchange={(event) => (blockPalette = event.currentTarget.value as "ramp" | "legacy")}
          >
            <option value="ramp">新色阶（和晶簇同矿）</option>
            <option value="legacy">现状调色板</option>
          </select>
        </label>
        <label class="flex items-center gap-2 text-xs">
          <span class="text-muted-foreground">Loader</span>
          <select
            class={CONTROL}
            value={demoLoader}
            onchange={(event) => (demoLoader = event.currentTarget.value as Loader)}
          >
            {#each LOADERS as loader (loader)}
              <option value={loader}>{loader}</option>
            {/each}
          </select>
        </label>
        <label class="flex items-center gap-2 text-xs">
          <span class="text-muted-foreground">实例</span>
          <select
            class={CONTROL}
            value={demoName}
            onchange={(event) => (demoName = event.currentTarget.value)}
          >
            {#each NAMES.slice(0, 8) as name (name)}
              <option value={name}>{name}</option>
            {/each}
          </select>
        </label>
      </div>

      <div class="grid grid-cols-4 gap-4">
        {#each VARIANTS as variant (variant.key)}
          <div class="flex flex-col items-center gap-3 rounded-md border bg-card p-4">
            <LabCrystallize
              seed={demoSeed}
              loader={demoLoader}
              order={variant.key}
              {progress}
              {ghost}
              palette={blockPalette}
              {failed}
              crystallized={stage === "crystallized"}
              {lattice}
              {maxPrisms}
              {taper}
              class="size-20"
            />
            <div class="text-center">
              <p class="text-xs font-medium">{variant.title}</p>
              <p class="mt-1 text-micro leading-relaxed text-muted-foreground">{variant.note}</p>
            </div>
          </div>
        {/each}
      </div>

      <div class="flex flex-col gap-3 rounded-md border bg-card p-4">
        <h3 class="text-xs font-semibold">放回真实场景</h3>

        <!-- 启动托盘行：方块管一眼瞄，条和数字管精确 -->
        <div class="flex items-center gap-3 rounded-md border bg-card px-3 py-2">
          <LabCrystallize
            seed={demoSeed}
            loader={demoLoader}
            order="nucleation"
            {progress}
            {ghost}
            palette={blockPalette}
            {failed}
            crystallized={stage === "crystallized"}
            {lattice}
            {maxPrisms}
            {taper}
            class="size-10 shrink-0"
          />
          <div class="w-44 min-w-0 shrink-0">
            <p class="truncate text-xs font-medium">{demoName}</p>
            <p class="truncate text-micro text-muted-foreground">
              {failed ? "Launch failed" : PHASES[phaseIndex]}
            </p>
          </div>
          <div class="h-2 min-w-24 flex-1 overflow-hidden rounded-full bg-secondary">
            <div
              class={cn(
                "h-full transition-[width] duration-100",
                failed ? "bg-destructive" : "bg-primary",
              )}
              style={`width: ${progress}%`}
            ></div>
          </div>
          <span class="w-9 text-right font-mono text-micro text-muted-foreground">
            {Math.floor(progress)}%
          </span>
        </div>

        <!-- 列表卡片：图标槽自己就是进度，没有第二个进度元素 -->
        <div class="grid grid-cols-2 gap-3">
          <div class="flex items-center gap-3 rounded-md border bg-card p-3">
            <LabCrystallize
              seed={demoSeed}
              loader={demoLoader}
              order="nucleation"
              {progress}
              {ghost}
              palette={blockPalette}
              {failed}
              crystallized={stage === "crystallized"}
              {lattice}
              {maxPrisms}
              {taper}
              class="size-9 shrink-0"
            />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-xs font-medium">{demoName}</span>
              <span class="mt-0.5 block truncate text-micro text-muted-foreground">
                MC 1.21.1 · <span class="capitalize">{demoLoader}</span>
              </span>
            </span>
          </div>
          <div class="flex items-center gap-3 rounded-md border bg-card p-3">
            <LabPackIcon variant={0} class="size-9 shrink-0" />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-xs font-medium">带自带图标的整合包</span>
              <span class="mt-0.5 block truncate text-micro text-muted-foreground">
                启动时这个槽位也会被方块暂时顶替
              </span>
            </span>
          </div>
        </div>
      </div>
    </section>

    <!-- ======================================================== 混排对照 -->
    <section class="flex flex-col gap-5">
      <div>
        <h2 class="text-sm font-semibold">三 · 和整合包自带图标混排</h2>
        <p class="mt-1 text-xs text-muted-foreground">
          同一份列表，只换图标槽里的东西。左边是现状，右边是晶簇。设计师那条批评在左边应该一眼可见。
        </p>
      </div>

      <div class="grid grid-cols-2 gap-6">
        {#each [{ key: "now", title: "现状 · 方块", tone: "border-destructive/40" }, { key: "next", title: "提案 · 晶簇", tone: "border-primary/40" }] as column (column.key)}
          <div class={cn("flex flex-col gap-2 rounded-md border bg-card p-4", column.tone)}>
            <h3 class="mb-1 text-xs font-semibold">{column.title}</h3>
            {#each MIXED as row (row.name)}
              <div class="flex items-center gap-3 rounded-md border bg-card p-3">
                {#if row.pack !== null}
                  <LabPackIcon variant={row.pack} class="size-9 shrink-0" />
                {:else if column.key === "now"}
                  <LabBlock
                    seed={`${row.name}|${row.version}|${row.loader}`}
                    loader={row.loader}
                    palette="legacy"
                    class="size-9 shrink-0"
                  />
                {:else}
                  <LabCrystal
                    seed={`${row.name}|${row.version}|${row.loader}`}
                    loader={row.loader}
                    {lattice}
                    {maxPrisms}
                    {taper}
                    gap={gapAt(36)}
                    class="size-9 shrink-0"
                  />
                {/if}
                <span class="min-w-0 flex-1">
                  <span class="block truncate text-xs font-medium">{row.name}</span>
                  <span class="mt-0.5 block truncate text-micro text-muted-foreground">
                    MC {row.version} · <span class="capitalize">{row.loader}</span>
                    {row.pack !== null ? " · 自带图标" : " · 无图标"}
                  </span>
                </span>
              </div>
            {/each}

            <div class="mt-2 flex items-center gap-2 rounded-md bg-secondary/50 p-2">
              <span class="text-micro text-muted-foreground">16px 侧栏：</span>
              {#each MIXED as row (row.name)}
                {#if row.pack !== null}
                  <LabPackIcon variant={row.pack} class="size-4 shrink-0" />
                {:else if column.key === "now"}
                  <LabBlock
                    seed={`${row.name}|${row.version}|${row.loader}`}
                    loader={row.loader}
                    palette="legacy"
                    class="size-4 shrink-0"
                  />
                {:else}
                  <LabCrystal
                    seed={`${row.name}|${row.version}|${row.loader}`}
                    loader={row.loader}
                    {lattice}
                    {maxPrisms}
                    {taper}
                    gap={gapAt(16)}
                    class="size-4 shrink-0"
                  />
                {/if}
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </section>
  </div>
</div>
