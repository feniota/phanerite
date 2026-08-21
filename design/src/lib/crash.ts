import type { Loader } from "./types";

/**
 * A local, signature-based match. This mirrors HMCL's CrashReportAnalyzer:
 * rules only report patterns that appear verbatim in the captured output.
 */
export interface CrashFinding {
  rule: string;
  title: string;
  explanation: string;
  evidenceLines: number[];
  implicatedModIds?: string[];
  suggestedMemory?: number;
}

export interface CrashEnvironment {
  mcVersion: string;
  loader: Loader;
  loaderVersion: string;
  javaName: string;
  javaVersion: number;
  javaPath: string;
  memory: number;
  os: string;
  gpu: string;
  enabledMods: { name: string; version: string }[];
  activeOverrides: string[];
  source: "aphanite" | "local";
  aphaniteServer?: string;
}

export interface CrashReport {
  id: string;
  instanceId: string;
  when: string;
  exitCode: number;
  lines: string[] | null;
  stderrTail: string[];
  hsErrPath: string | null;
  findings: CrashFinding[];
  environment: CrashEnvironment;
}

const fogEnvironment: CrashEnvironment = {
  mcVersion: "1.21.4",
  loader: "fabric",
  loaderVersion: "0.115.1+1.21.4",
  javaName: "Azul Zulu JDK 21",
  javaVersion: 21,
  javaPath: "/home/enita/.local/share/phanerite/java/zulu-21/bin/java",
  memory: 6,
  os: "Fedora Linux 44",
  gpu: "NVIDIA GeForce GTX 1060 6GB",
  enabledMods: [
    { name: "Sodium", version: "0.6.9" },
    { name: "OptiFine", version: "HD_U_I8" },
    { name: "Iris Shaders", version: "1.8.9" },
    { name: "Create", version: "6.0.1" },
    { name: "Just Enough Items", version: "19.21.0.7" },
  ],
  activeOverrides: ["memory"],
  source: "aphanite",
  aphaniteServer: "aphanite.enita.cn",
};

const crashHeader = [
  "---- Minecraft Crash Report ----",
  "// Oops. The game did not enjoy that.",
  "",
  "Time: 2026-08-17 14:22:18",
  "Description: Initializing game",
  "",
];

export const seedCrashReports: CrashReport[] = [
  {
    id: "crash-sodium-optifine",
    instanceId: "inst-fog",
    when: "2 minutes ago",
    exitCode: 1,
    lines: [
      ...crashHeader,
      "java.lang.RuntimeException: Mixin apply failed net.minecraft.class_310",
      "Mixin apply for mod optifine failed",
      "  at net.fabricmc.loader.impl.launch.knot.KnotClassDelegate.getPostMixinClassByteArray(KnotClassDelegate.java:427)",
      "  at net.optifine.reflect.Reflector.<clinit>(Reflector.java:155)",
      "  at me.jellysquid.mods.sodium.client.SodiumClientMod.onInitializeClient(SodiumClientMod.java:52)",
      "Caused by: org.spongepowered.asm.mixin.transformer.throwables.MixinTransformerError: An unexpected critical error was encountered",
      "  at org.spongepowered.asm.mixin.transformer.MixinProcessor.applyMixins(MixinProcessor.java:392)",
      "",
      "-- System Details --",
      "Details:",
      "\tMinecraft Version: 1.21.4",
      "\tOperating System: Linux (amd64) version 6.14.0",
      "\tJava Version: 21.0.8, Azul Systems, Inc.",
      "\tMemory: 1148928000 bytes (1095 MiB) / 6442450944 bytes (6144 MiB) up to 6442450944 bytes (6144 MiB)",
      "\tJVM Flags: 2 total; -Xmx6G --accessToken eyJhbGciOiJIUzI1NiJ9.fake.token",
      "\tLaunched Version: fabric-loader-0.16.9-1.21.4",
    ],
    stderrTail: [],
    hsErrPath: null,
    findings: [
      {
        rule: "MIXIN_APPLY_MOD_FAILED",
        title: "OptiFine failed while its mixin was being applied",
        explanation:
          "The mod id appears directly in the matched crash signature. Disable or update OptiFine before retrying.",
        evidenceLines: [7],
        implicatedModIds: ["m-optifine"],
      },
    ],
    environment: fogEnvironment,
  },
  {
    id: "crash-possible",
    instanceId: "inst-neo",
    when: "18 minutes ago",
    exitCode: 1,
    lines: [
      ...crashHeader,
      "java.lang.NoSuchMethodError: 'void net.neoforged.fml.ModList.registerEventHandler()'",
      "  at dev.ftb.mods.ftbchunks.FTBChunks.init(FTBChunks.java:84)",
      "  at net.neoforged.fml.ModLoader.gatherAndInitializeMods(ModLoader.java:203)",
      "Caused by: java.lang.IllegalStateException: Could not load mod ftbquests",
      "  at dev.ftb.mods.ftbquests.FTBQuestsCommon.init(FTBQuestsCommon.java:66)",
      "",
      "-- System Details --",
      "Details:",
      "\tMinecraft Version: 1.21.1",
      "\tMod Launcher: 11.0.4",
    ],
    stderrTail: [],
    hsErrPath: null,
    findings: [
      {
        rule: "NO_SUCH_METHOD_ERROR",
        title: "A required method was not available",
        explanation:
          "This signature can indicate a missing or incompatible mod or game component. It does not identify one mod by itself.",
        evidenceLines: [7],
      },
    ],
    environment: {
      mcVersion: "1.21.1",
      loader: "neoforge",
      loaderVersion: "21.1.181",
      javaName: "Azul Zulu JDK 21",
      javaVersion: 21,
      javaPath: "/home/enita/.local/share/phanerite/java/zulu-21/bin/java",
      memory: 8,
      os: "Fedora Linux 44",
      gpu: "NVIDIA GeForce GTX 1060 6GB",
      enabledMods: [
        { name: "FTB Quests", version: "2100.1.1" },
        { name: "FTB Library", version: "2101.1.5" },
        { name: "FTB Chunks", version: "2100.1.1" },
        { name: "Architectury API", version: "13.0.8" },
      ],
      activeOverrides: ["memory"],
      source: "aphanite",
      aphaniteServer: "aphanite.enita.cn",
    },
  },
  {
    id: "crash-unknown",
    instanceId: "inst-vanilla",
    when: "yesterday",
    exitCode: 1,
    lines: [
      ...crashHeader,
      "java.lang.IllegalStateException: Failed to load registries due to above errors",
      "  at net.minecraft.server.Bootstrap.bootStrap(Bootstrap.java:124)",
      "  at net.minecraft.client.main.Main.main(Main.java:201)",
      "Caused by: java.io.IOException: Error reading level.dat",
      "  at net.minecraft.world.level.storage.LevelStorageSource.readLevelData(LevelStorageSource.java:301)",
      "  at net.minecraft.client.Minecraft.createLevel(Minecraft.java:517)",
      "",
      "-- System Details --",
      "Details:",
      "\tMinecraft Version: 1.21.4",
      "\tOperating System: Linux (amd64) version 6.14.0",
      "\tJava Version: 21.0.8, Azul Systems, Inc.",
      "\tMemory: 402653184 bytes (384 MiB) / 4294967296 bytes (4096 MiB) up to 4294967296 bytes (4096 MiB)",
    ],
    stderrTail: [],
    hsErrPath: null,
    findings: [],
    environment: {
      ...fogEnvironment,
      memory: 4,
      enabledMods: [],
      activeOverrides: [],
      source: "local",
      aphaniteServer: undefined,
    },
  },
  {
    id: "crash-jvm",
    instanceId: "inst-legacy",
    when: "3 days ago",
    exitCode: -6,
    lines: null,
    stderrTail: [
      "# A fatal error has been detected by the Java Runtime Environment:",
      "#  SIGSEGV (0xb) at pc=0x00007f1dd405ddf2, pid=48721, tid=48742",
      "# JRE version: OpenJDK Runtime Environment (8.0_472-b08)",
      "# Java VM: OpenJDK 64-Bit Server VM (25.472-b08 mixed mode linux-amd64 compressed oops)",
      "# Problematic frame:",
      "# C  [libGLX_nvidia.so.0+0x2ddf2]",
      "Aborted (core dumped)",
    ],
    hsErrPath: "/home/enita/.local/share/phanerite/instances/old-faithful/hs_err_pid48721.log",
    findings: [],
    environment: {
      mcVersion: "1.12.2",
      loader: "forge",
      loaderVersion: "14.23.5.2860",
      javaName: "OpenJDK 8",
      javaVersion: 8,
      javaPath: "/usr/lib/jvm/java-8-openjdk/bin/java",
      memory: 6,
      os: "Fedora Linux 44",
      gpu: "NVIDIA GeForce GTX 1060 6GB",
      enabledMods: [
        { name: "Thaumcraft", version: "6.1.BETA26" },
        { name: "Buildcraft", version: "7.99.24" },
        { name: "Tinkers’ Construct", version: "2.13.0.183" },
      ],
      activeOverrides: ["memory"],
      source: "local",
    },
  },
];
