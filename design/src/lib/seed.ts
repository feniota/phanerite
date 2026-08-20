import type { Account, MCInstance, NewsItem } from "./types";

/** Seed / mock data so every screen has something believable to show. */

function mod(
  id: string,
  name: string,
  _author: string,
  version: string,
  loader: MCInstance["loader"],
  _category: string,
  enabled: boolean,
  fileName = `${name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")}-${version}.jar`,
) {
  return { id, name, version, fileName, loader, enabled };
}

function unreadableMod(id: string, fileName: string, enabled: boolean) {
  return { id, name: null, version: null, fileName, loader: null, enabled };
}

function pack(
  id: string,
  name: string,
  author: string,
  version: string,
  description: string,
  size: string,
  enabled: boolean,
) {
  return { id, name, author, version, description, size, enabled };
}

export const seedInstances: MCInstance[] = [
  {
    id: "inst-fog",
    name: "The Fog",
    aphanite: true,
    favorite: true,
    description:
      "Vanilla-plus survival with Create, Sodium and JEI. The main world everyone actually plays on.",
    loader: "fabric",
    mcVersion: "1.21.4",
    loaderVersion: "0.115.1+1.21.4",
    java: "21",
    javaRuntimeId: "zulu-21",
    createdAt: "2026-02-14",
    lastPlayed: "2 hours ago",
    playCount: 312,
    lastCrashId: "crash-sodium-optifine",
    launchOverrides: { memory: 6 },
    mods: [
      mod("m-sodium", "Sodium", "CaffeineMC", "0.6.9", "fabric", "performance", true),
      mod("m-optifine", "OptiFine", "sp614x", "HD_U_I8", "fabric", "performance", true),
      mod("m-iris", "Iris Shaders", "coderbot", "1.8.9", "fabric", "performance", true),
      mod("m-create", "Create", "simibubi", "6.0.1", "fabric", "content", true),
      mod("m-jei", "Just Enough Items", "mezz", "19.21.0.7", "fabric", "utility", true),
      mod("m-lithium", "Lithium", "CaffeineMC", "0.16.1", "fabric", "performance", true),
      mod("m-kubejs", "KubeJS", "LatvianModder", "2101.6.1", "fabric", "utility", true),
      mod("m-yacl", "YetAnotherConfigLib", "isxander", "3.6.6", "fabric", "utility", true),
      mod("m-modmenu", "Mod Menu", "Prospector", "13.0.0", "fabric", "utility", true),
      mod("m-rei", "Roughly Enough Items", "shedaniel", "18.1.2", "fabric", "utility", false),
      mod("m-twigs", "Twigs", "ninnih", "4.0.2", "fabric", "content", false),
      mod(
        "m-sounds",
        "Sound Physics Remastered",
        "henkelmax",
        "1.21.4",
        "fabric",
        "content",
        false,
      ),
      mod(
        "m-backpacks",
        "Sophisticated Backpacks",
        "P3pp3rF1y",
        "3.22.1",
        "fabric",
        "content",
        false,
      ),
      mod("m-midnightlib", "MidnightLib", "TeamMidnightDust", "1.6.5", "fabric", "utility", true),
      mod("m-cloth", "Cloth Config API", "shedaniel", "18.1.1", "fabric", "utility", true),
      unreadableMod("m-unknown", "legacy-addon-1.21.4.jar", false),
    ],
    resourcePacks: [
      pack(
        "p-xray",
        "Xray Ultimate",
        "RayDyn",
        "1.21.4",
        "Friendly voxel outlines, no textures removed.",
        "6.4 MB",
        true,
      ),
      pack(
        "p-bare",
        "Bare Bones",
        "robotpant",
        "1.21.4",
        "A minimal, remastered take on the default look.",
        "4.2 MB",
        true,
      ),
      pack(
        "p-faithful",
        "Faithful 64x",
        "Vattic",
        "1.21.4",
        "The classic faithful higher-resolution pack.",
        "22.1 MB",
        false,
      ),
    ],
    shaderPacks: [
      {
        id: "s-comp",
        name: "Complementary Reimagined",
        author: "EminGT",
        version: "r5.4",
        gpu: "GTX 1060 6GB",
        enabled: true,
      },
      {
        id: "s-bsl",
        name: "BSL Shaders",
        author: "capttatsu",
        version: "v8.4",
        gpu: "GTX 1070 8GB",
        enabled: false,
      },
    ],
    worlds: [
      {
        id: "w-main",
        name: "Survival World",
        seed: "7421095342",
        version: "1.21.4",
        lastPlayed: "2 hours ago",
        players: 4,
      },
      {
        id: "w-creative",
        name: "Creative Testing",
        seed: "-92873144",
        version: "1.21.4",
        lastPlayed: "3 days ago",
        players: 1,
      },
      {
        id: "w-nethern",
        name: "Nether Farm",
        seed: "108892",
        version: "1.21.4",
        lastPlayed: "a week ago",
        players: 2,
      },
    ],
  },
  {
    id: "inst-vanilla",
    name: "Vanilla Survival",
    aphanite: false,
    description:
      "Pristine vanilla 1.21.4 with no modifications. For when the server demands purity.",
    loader: "vanilla",
    mcVersion: "1.21.4",
    loaderVersion: "—",
    java: "21",
    javaRuntimeId: "zulu-21",
    createdAt: "2026-03-02",
    lastPlayed: "yesterday",
    playCount: 48,
    lastCrashId: "crash-unknown",
    mods: [],
    resourcePacks: [
      pack(
        "p-faithful",
        "Faithful 64x",
        "Vattic",
        "1.21.4",
        "The classic faithful higher-resolution pack.",
        "22.1 MB",
        true,
      ),
    ],
    shaderPacks: [],
    worlds: [
      {
        id: "w-hardcore",
        name: "Hardcore Run",
        seed: "77",
        version: "1.21.4",
        lastPlayed: "yesterday",
        players: 1,
      },
    ],
  },
  {
    id: "inst-neo",
    name: "NeoForge Server Test",
    aphanite: true,
    description: "NeoForge test bed used to validate the server pack before it ships to the host.",
    loader: "neoforge",
    mcVersion: "1.21.1",
    loaderVersion: "21.1.181",
    java: "21",
    javaRuntimeId: "zulu-21",
    createdAt: "2026-05-11",
    lastPlayed: "5 days ago",
    playCount: 23,
    lastCrashId: "crash-possible",
    launchOverrides: { memory: 8 },
    mods: [
      mod("m-ftbq", "FTB Quests", "FTB Team", "2100.1.1", "neoforge", "content", true),
      mod("m-ftbl", "FTB Library", "FTB Team", "2101.1.5", "neoforge", "utility", true),
      mod("m-ftbc", "FTB Chunks", "FTB Team", "2100.1.1", "neoforge", "content", true),
      mod("m-arch", "Architectury API", "shedaniel", "13.0.8", "neoforge", "utility", true),
      mod("m-mek", "Mekanism", "aidancbrady", "10.7.10", "neoforge", "content", false),
      mod("m-cc", "CC: Tweaked", "SquidDev", "1.112.2", "neoforge", "content", false),
    ],
    resourcePacks: [
      pack(
        "p-bare",
        "Bare Bones",
        "robotpant",
        "1.21.1",
        "A minimal, remastered take on the default look.",
        "4.2 MB",
        true,
      ),
    ],
    shaderPacks: [],
    worlds: [
      {
        id: "w-test",
        name: "Server Test",
        seed: "11223344",
        version: "1.21.1",
        lastPlayed: "5 days ago",
        players: 0,
      },
    ],
  },
  {
    id: "inst-legacy",
    name: "Old Faithful",
    aphanite: false,
    favorite: true,
    description:
      "The 2018 modpack that will not die. Forge 1.12.2, still running a community server.",
    loader: "forge",
    mcVersion: "1.12.2",
    loaderVersion: "14.23.5.2860",
    java: "8",
    javaRuntimeId: "legacy-8",
    createdAt: "2026-01-08",
    lastPlayed: "a month ago",
    playCount: 509,
    lastCrashId: "crash-jvm",
    launchOverrides: { memory: 6 },
    mods: [
      mod("m-old-1", "Thaumcraft", "Azanor", "6.1.BETA26", "forge", "content", true),
      mod("m-old-2", "Buildcraft", "CovertJaguar", "7.99.24", "forge", "content", true),
      mod("m-old-3", "Tinkers\u2019 Construct", "boni", "2.13.0.183", "forge", "content", true),
    ],
    resourcePacks: [],
    shaderPacks: [],
    worlds: [],
  },
  {
    id: "inst-create",
    name: "Create: Engineering",
    aphanite: true,
    description:
      "Pure Create contraptions sandbox. Water wheels, trains, and very questionable elevators.",
    loader: "fabric",
    mcVersion: "1.20.1",
    loaderVersion: "0.15.11+1.20.1",
    java: "17",
    javaRuntimeId: "temurin-17",
    createdAt: "2026-06-21",
    lastPlayed: "never",
    playCount: 0,
    launchOverrides: { memory: 5 },
    mods: [
      mod("m-create-1", "Create", "simibubi", "0.5.1j", "fabric", "content", true),
      mod(
        "m-create-2",
        "Create: Steam \u2018n\u2019 Rails",
        "LayersOfRails",
        "1.6.4",
        "fabric",
        "content",
        true,
      ),
      mod("m-create-3", "Flywheel", "jozufozu", "0.6.10", "fabric", "performance", true),
    ],
    resourcePacks: [],
    shaderPacks: [],
    worlds: [],
  },
];

export const seedAccounts: Account[] = [
  {
    id: "acc-enita",
    username: "Enita_Nureya",
    type: "microsoft",
    lastUsed: "2 hours ago",
    activeProfileId: "profile-enita",
    profiles: [
      {
        id: "profile-enita",
        name: "enita",
        skinUrl: "https://mc-heads.net/skin/Enita_Nureya",
        isSlim: true,
      },
    ],
  },
  {
    id: "acc-steve",
    username: "Steve",
    type: "offline",
    lastUsed: "3 days ago",
    activeProfileId: "profile-steve",
    profiles: [{ id: "profile-steve", name: "Steve", skinUrl: "https://mc-heads.net/skin/Steve" }],
  },
  {
    id: "acc-alex",
    username: "alex_03",
    type: "offline",
    lastUsed: "a month ago",
    activeProfileId: "profile-alex",
    profiles: [
      {
        id: "profile-alex",
        name: "alex_03",
        skinUrl: "https://mc-heads.net/skin/Alex",
        isSlim: true,
      },
    ],
  },
];

export const seedNews: NewsItem[] = [
  { id: "n1", source: "Mojang", title: "Minecraft Java 1.21.5 Released", when: "yesterday" },
  { id: "n2", source: "Fabric", title: "Fabric API 0.115.1 for 1.21.4", when: "2 days ago" },
  { id: "n3", source: "NeoForged", title: "NeoForge 21.1.181 out now", when: "4 days ago" },
  { id: "n4", source: "CaffeineMC", title: "Sodium 0.6.9 — smaller, faster", when: "a week ago" },
];

/** Pool of placeholder mods used by the "import" dialogs. */
export const importableMods = [
  {
    name: "JourneyMap",
    version: "1.21.4-6.0.0",
    fileName: "journeymap-1.21.4-6.0.0-fabric.jar",
    loader: "fabric" as const,
  },
  {
    name: "Distant Horizons",
    version: "2.3.0",
    fileName: "DistantHorizons-2.3.0-fabric.jar",
    loader: "fabric" as const,
  },
  {
    name: "Supplementaries",
    version: "3.0.2",
    fileName: "supplementaries-3.0.2.jar",
    loader: "fabric" as const,
  },
  {
    name: "Better Combat",
    version: "1.9.5",
    fileName: "bettercombat-1.9.5.jar",
    loader: "fabric" as const,
  },
  {
    name: "Terralith",
    version: "2.5.8",
    fileName: "terralith-2.5.8.jar",
    loader: "fabric" as const,
  },
  {
    name: "AppleSkin",
    version: "3.0.4",
    fileName: "appleskin-fabric-mc1.21-3.0.4.jar",
    loader: "fabric" as const,
  },
];
