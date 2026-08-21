import { seedAccounts, seedInstances, seedNews } from "./seed";
import { seedCrashReports } from "./crash";
import type {
  AccentKey,
  Account,
  JavaRuntime,
  MCInstance,
  Mod,
  NewsItem,
  PlayerProfile,
  LaunchSettings,
  LaunchJob,
  View,
} from "./types";
import type { CrashReport } from "./crash";

/**
 * Global prototype state (Svelte 5 runes). Nothing here is persisted —
 * this is a UI preview, but the shape mirrors what the real Rust/GPUI
 * launcher would expose from its core layer as an `AppState` struct.
 */

/** Views that require a valid active instance. */
const INSTANCE_SCOPED_VIEWS: View[] = [
  "instance-detail",
  "mods",
  "packs",
  "shaders",
  "worlds",
  "logs",
  "crash",
  "launch-settings",
];

class AppState {
  view = $state<View>("play");

  instances = $state<MCInstance[]>(structuredClone(seedInstances));
  accounts = $state<Account[]>(structuredClone(seedAccounts));
  news = $state<NewsItem[]>(structuredClone(seedNews));
  instanceLogs = $state<Record<string, string[]>>(
    Object.fromEntries(
      seedInstances.map((instance) => [
        instance.id,
        [
          "[12:14:03] [Render thread/INFO]: Reloading ResourceManager: vanilla",
          `[12:14:06] [Server thread/INFO]: ${instance.name} local session started`,
          "[12:14:08] [Render thread/INFO]: OpenAL initialized on device: Default",
          "[12:15:17] [Server thread/INFO]: Saving chunks for level 'ServerLevel[world]'",
        ],
      ]),
    ),
  );
  crashReports = $state<CrashReport[]>(structuredClone(seedCrashReports));
  javaRuntimes = $state<JavaRuntime[]>([
    {
      id: "zulu-21",
      name: "Azul Zulu JDK 21",
      version: 21,
      path: "Managed by Phanerite",
      managed: true,
    },
    {
      id: "temurin-17",
      name: "Eclipse Temurin JDK 17",
      version: 17,
      path: "/usr/lib/jvm/java-17-openjdk",
      managed: false,
    },
    {
      id: "legacy-8",
      name: "OpenJDK 8",
      version: 8,
      path: "/usr/lib/jvm/java-8-openjdk",
      managed: false,
    },
  ]);

  activeAccountId = $state<string>("acc-enita");
  activeInstanceId = $state<string>("inst-fog");
  detailReturnView = $state<Exclude<View, "instance-detail">>("instances");
  logsReturnView = $state<Exclude<View, "logs">>("instance-detail");
  crashReportId = $state<string | null>(null);
  crashReturnView = $state<Exclude<View, "crash">>("instance-detail");

  /** Instances currently running (multi-instance is first-class). */
  runningIds = $state<string[]>([]);
  /** Instances currently in the launch flow. */
  launches = $state<LaunchJob[]>([]);

  accent = $state<AccentKey>("emerald");

  settings = $state({
    closeAfterLaunch: false,
    hideOnLaunch: true,
    multiInstance: true,
    checkUpdates: true,
    downloadThreads: 4,
    connectionTimeout: 30,
    language: "en",
    fontSize: "md",
    showNews: true,
    aphaniteServer: "https://aphanite.enita.cn/",
    showGameLogs: true,
    debugLogs: false,
    generateGameOptions: true,
    processPriority: "normal",
    openglRenderer: "Default",
    vulkanRenderer: "Default",
    preferHighPerformanceGpu: true,
  });

  launchSettings = $state<LaunchSettings>({
    memoryMode: "manual",
    memory: 4,
    windowMode: "windowed",
    windowWidth: 1280,
    windowHeight: 720,
    quickPlayMode: "none",
    quickPlayTarget: "",
    gameArgs: "",
    environmentVariables: "",
    javaArgs: "-XX:+UseG1GC -XX:MaxGCPauseMillis=150",
    useDefaultJvmArgs: true,
    useOptimizedJvmArgs: true,
    skipJvmValidation: false,
    allowAutoAgent: false,
    skipIntegrityCheck: false,
    preLaunchCommand: "",
    commandWrapper: "",
    postExitCommand: "",
    useCustomNatives: false,
    nativesPath: "",
    skipNativePatching: false,
    glfwPath: "",
    openalPath: "",
  });

  // ---- derived ----

  get activeInstance(): MCInstance | null {
    return this.instances.find((i) => i.id === this.activeInstanceId) ?? null;
  }

  get activeAccount(): Account | null {
    return this.accounts.find((a) => a.id === this.activeAccountId) ?? null;
  }

  get activeCrashReport(): CrashReport | null {
    return this.crashReports.find((report) => report.id === this.crashReportId) ?? null;
  }

  get activeProfile(): PlayerProfile | null {
    const account = this.activeAccount;
    return account?.profiles.find((profile) => profile.id === account.activeProfileId) ?? null;
  }

  get runningCount(): number {
    return this.runningIds.length;
  }

  get isLaunching(): boolean {
    return this.launches.length > 0;
  }

  getJavaRuntime(id: string): JavaRuntime | null {
    return this.javaRuntimes.find((runtime) => runtime.id === id) ?? null;
  }

  getLaunchSettings(instance: MCInstance): LaunchSettings {
    return { ...this.launchSettings, ...instance.launchOverrides };
  }

  getLaunch(id: string): LaunchJob | null {
    return this.launches.find((job) => job.instanceId === id) ?? null;
  }

  setLaunchOverride<K extends keyof LaunchSettings>(
    instanceId: string,
    key: K,
    value: LaunchSettings[K],
  ) {
    const instance = this.instances.find((item) => item.id === instanceId);
    if (!instance) return;
    instance.launchOverrides ??= {};
    instance.launchOverrides[key] = value;
  }

  clearLaunchOverride(instanceId: string, key: keyof LaunchSettings) {
    const instance = this.instances.find((item) => item.id === instanceId);
    if (!instance?.launchOverrides) return;
    delete instance.launchOverrides[key];
  }

  // ---- navigation ----

  setView(next: View) {
    this.view = next;
  }

  // ---- instances ----

  openInstanceDetail(id: string) {
    if (!this.instances.some((instance) => instance.id === id)) return;
    if (this.view !== "instance-detail") this.detailReturnView = this.view;
    this.activeInstanceId = id;
    this.view = "instance-detail";
  }

  openLogs(id: string) {
    if (!this.instances.some((instance) => instance.id === id)) return;
    this.activeInstanceId = id;
    if (this.view !== "logs") this.logsReturnView = this.view;
    this.view = "logs";
  }

  closeLogs() {
    this.view = this.logsReturnView;
  }

  openCrashReport(id: string) {
    const report = this.crashReports.find((item) => item.id === id);
    if (!report) return;
    this.activeInstanceId = report.instanceId;
    this.crashReportId = id;
    if (this.view !== "crash") this.crashReturnView = this.view;
    this.view = "crash";
  }

  closeCrashReport() {
    this.view = this.crashReturnView;
  }

  getLogs(id: string): string[] {
    return this.instanceLogs[id] ?? [];
  }

  appendLog(id: string, line: string) {
    if (!this.instanceLogs[id]) this.instanceLogs[id] = [];
    this.instanceLogs[id].push(line);
  }

  closeInstanceDetail() {
    this.view = this.detailReturnView;
  }

  setActiveInstance(id: string) {
    this.activeInstanceId = id;
  }

  toggleFavorite(id: string) {
    const instance = this.instances.find((item) => item.id === id);
    if (instance) instance.favorite = !instance.favorite;
  }

  createInstance(input: {
    name: string;
    description: string;
    mcVersion: string;
    loader: MCInstance["loader"];
    loaderVersion: string;
    memory: number;
  }) {
    const inst: MCInstance = {
      id: this.#uid("inst"),
      name: input.name,
      aphanite: false,
      description: input.description || "A new Phanerite instance.",
      loader: input.loader,
      mcVersion: input.mcVersion,
      loaderVersion: input.loaderVersion,
      java: input.mcVersion.startsWith("1.21") ? "21" : "17",
      javaRuntimeId: input.mcVersion.startsWith("1.21") ? "zulu-21" : "temurin-17",
      createdAt: "just now",
      lastPlayed: null,
      playCount: 0,
      mods: [],
      resourcePacks: [],
      shaderPacks: [],
      worlds: [],
      launchOverrides: { memory: input.memory },
    };
    this.instances.push(inst);
    this.activeInstanceId = inst.id;
    return inst.id;
  }

  duplicateInstance(id: string) {
    const src = this.instances.find((i) => i.id === id);
    if (!src) return;
    const copy = structuredClone(src);
    copy.id = this.#uid("inst");
    copy.name = `${src.name} (copy)`;
    copy.playCount = 0;
    copy.lastPlayed = null;
    this.instances.push(copy);
  }

  deleteInstance(id: string) {
    const idx = this.instances.findIndex((i) => i.id === id);
    if (idx === -1) return;
    this.instances.splice(idx, 1);
    this.runningIds = this.runningIds.filter((r) => r !== id);
    this.launches = this.launches.filter((job) => job.instanceId !== id);
    this.crashReports = this.crashReports.filter((report) => report.instanceId !== id);
    if (
      this.crashReportId &&
      !this.crashReports.some((report) => report.id === this.crashReportId)
    ) {
      this.crashReportId = null;
    }
    if (this.activeInstanceId === id) {
      this.activeInstanceId = this.instances[0]?.id ?? "";
      if (INSTANCE_SCOPED_VIEWS.includes(this.view)) this.view = "instances";
    }
  }

  // ---- mods ----

  toggleMod(instanceId: string, modId: string) {
    const m = this.instances.find((i) => i.id === instanceId)?.mods.find((mod) => mod.id === modId);
    if (m) m.enabled = !m.enabled;
  }

  setModEnabled(instanceId: string, modId: string, enabled: boolean) {
    const m = this.instances.find((i) => i.id === instanceId)?.mods.find((mod) => mod.id === modId);
    if (m) m.enabled = enabled;
  }

  deleteMod(instanceId: string, modId: string) {
    const inst = this.instances.find((i) => i.id === instanceId);
    if (!inst) return;
    const idx = inst.mods.findIndex((m) => m.id === modId);
    if (idx !== -1) inst.mods.splice(idx, 1);
  }

  addMods(instanceId: string, mods: Mod[]) {
    const inst = this.instances.find((i) => i.id === instanceId);
    if (!inst) return;
    inst.mods.push(...mods);
  }

  // ---- resource packs / shaders / worlds ----

  toggleResourcePack(instanceId: string, packId: string) {
    const p = this.instances
      .find((i) => i.id === instanceId)
      ?.resourcePacks.find((x) => x.id === packId);
    if (p) p.enabled = !p.enabled;
  }

  deleteResourcePack(instanceId: string, packId: string) {
    const inst = this.instances.find((i) => i.id === instanceId);
    if (!inst) return;
    const idx = inst.resourcePacks.findIndex((x) => x.id === packId);
    if (idx !== -1) inst.resourcePacks.splice(idx, 1);
  }

  addResourcePacks(instanceId: string, packs: MCInstance["resourcePacks"]) {
    const inst = this.instances.find((i) => i.id === instanceId);
    if (!inst) return;
    inst.resourcePacks.push(...packs);
  }

  toggleShaderPack(instanceId: string, packId: string) {
    const p = this.instances
      .find((i) => i.id === instanceId)
      ?.shaderPacks.find((x) => x.id === packId);
    if (p) p.enabled = !p.enabled;
  }

  deleteShaderPack(instanceId: string, packId: string) {
    const inst = this.instances.find((i) => i.id === instanceId);
    if (!inst) return;
    const idx = inst.shaderPacks.findIndex((x) => x.id === packId);
    if (idx !== -1) inst.shaderPacks.splice(idx, 1);
  }

  addShaderPacks(instanceId: string, packs: MCInstance["shaderPacks"]) {
    const inst = this.instances.find((i) => i.id === instanceId);
    if (!inst) return;
    inst.shaderPacks.push(...packs);
  }

  // ---- accounts ----

  addJavaRuntime(name: string, version: number, path: string) {
    this.javaRuntimes.push({ id: this.#uid("java"), name, version, path, managed: false });
  }

  addAccount(
    username: string,
    type: Account["type"],
    authServer?: string,
    profiles?: PlayerProfile[],
    activeProfileId?: string,
  ) {
    const accountProfiles = profiles ?? [
      {
        id: this.#uid("profile"),
        name: username,
        skinUrl: `https://mc-heads.net/skin/${encodeURIComponent(username)}`,
      },
    ];
    const acc: Account = {
      id: this.#uid("acc"),
      username,
      type,
      lastUsed: "just now",
      authServer,
      profiles: accountProfiles,
      activeProfileId: activeProfileId ?? accountProfiles[0].id,
    };
    this.accounts.push(acc);
    this.activeAccountId = acc.id;
    return acc.id;
  }

  setActiveProfile(accountId: string, profileId: string) {
    const account = this.accounts.find((item) => item.id === accountId);
    if (account?.profiles.some((profile) => profile.id === profileId))
      account.activeProfileId = profileId;
  }

  setActiveAccount(id: string) {
    const acc = this.accounts.find((a) => a.id === id);
    if (!acc) return;
    this.activeAccountId = id;
    acc.lastUsed = "just now";
  }

  removeAccount(id: string) {
    const idx = this.accounts.findIndex((a) => a.id === id);
    if (idx === -1) return;
    this.accounts.splice(idx, 1);
    if (this.activeAccountId === id) {
      this.activeAccountId = this.accounts[0]?.id ?? "";
    }
  }

  // ---- launch lifecycle ----

  startLaunch(id: string) {
    if (this.runningIds.includes(id) || this.getLaunch(id)) return;
    if (!this.settings.multiInstance && (this.launches.length > 0 || this.runningIds.length > 0)) {
      return;
    }
    this.activeInstanceId = id;
    this.launches.push({ instanceId: id, progress: 0, phase: 0, status: "starting" });
    this.appendLog(id, `[${this.#time()}] [Phanerite/INFO]: Launch requested.`);
  }

  finishLaunch(id: string) {
    if (!this.getLaunch(id)) return;
    this.launches = this.launches.filter((job) => job.instanceId !== id);
    if (!this.runningIds.includes(id)) this.runningIds.push(id);
    const inst = this.instances.find((i) => i.id === id);
    if (inst) {
      inst.lastPlayed = "just now";
      inst.playCount += 1;
    }
  }

  cancelLaunch(id: string) {
    this.launches = this.launches.filter((job) => job.instanceId !== id);
    this.appendLog(id, `[${this.#time()}] [Phanerite/INFO]: Launch cancelled by user.`);
  }

  stopInstance(id: string) {
    this.runningIds = this.runningIds.filter((r) => r !== id);
    this.appendLog(id, `[${this.#time()}] [Phanerite/INFO]: Game process stopped.`);
  }

  // ---- appearance ----

  setAccent(key: AccentKey) {
    this.accent = key;
    const a = ACCENTS[key];
    const root = document.documentElement;
    root.style.setProperty("--primary", a.primary);
    root.style.setProperty("--primary-foreground", a.foreground);
    root.style.setProperty("--ring", a.primary);
    root.style.setProperty("--sidebar-primary", a.primary);
    root.style.setProperty("--sidebar-ring", a.primary);
    root.style.setProperty("--chart-1", a.primary);
    root.style.setProperty("--chart-3", a.chart3);
    root.style.setProperty("--chart-5", a.chart5);
  }

  #uid(prefix: string) {
    return `${prefix}-${crypto.randomUUID().slice(0, 8)}`;
  }

  #time() {
    return new Date().toTimeString().slice(0, 8);
  }
}

const ACCENTS: Record<
  AccentKey,
  { primary: string; foreground: string; chart3: string; chart5: string }
> = {
  emerald: {
    primary: "oklch(0.62 0.13 155)",
    foreground: "oklch(0.17 0.03 155)",
    chart3: "oklch(0.46 0.07 150)",
    chart5: "oklch(0.44 0.05 150)",
  },
  gold: {
    primary: "oklch(0.78 0.11 80)",
    foreground: "oklch(0.2 0.04 80)",
    chart3: "oklch(0.58 0.07 75)",
    chart5: "oklch(0.5 0.06 65)",
  },
  slate: {
    primary: "oklch(0.72 0.02 250)",
    foreground: "oklch(0.16 0.02 250)",
    chart3: "oklch(0.54 0.03 250)",
    chart5: "oklch(0.48 0.025 250)",
  },
  teal: {
    primary: "oklch(0.66 0.1 190)",
    foreground: "oklch(0.15 0.04 190)",
    chart3: "oklch(0.5 0.06 190)",
    chart5: "oklch(0.46 0.04 190)",
  },
};

export const ACCENT_OPTIONS: readonly AccentKey[] = ["emerald", "gold", "slate", "teal"];

export const app = new AppState();
