/** Shared types for the Phanerite launcher prototype. */

export type View =
  | "play"
  | "instances"
  | "aphanite"
  | "instance-detail"
  | "mods"
  | "packs"
  | "shaders"
  | "worlds"
  | "logs"
  | "crash"
  | "launch-settings"
  | "accounts"
  | "settings";

export type Loader = "vanilla" | "fabric" | "forge" | "neoforge";

export const LOADER_LABEL: Record<Loader, string> = {
  vanilla: "Vanilla",
  fabric: "Fabric",
  forge: "Forge",
  neoforge: "NeoForge",
};

export interface Mod {
  id: string;
  name: string | null;
  version: string | null;
  fileName: string;
  loader: Loader | null;
  enabled: boolean;
}

export interface ResourcePack {
  id: string;
  name: string;
  author: string;
  version: string;
  description: string;
  size: string;
  enabled: boolean;
}

export interface ShaderPack {
  id: string;
  name: string;
  author: string;
  version: string;
  gpu: string;
  enabled: boolean;
}

export interface World {
  id: string;
  name: string;
  seed: string;
  version: string;
  lastPlayed: string;
  players: number;
}

export interface MCInstance {
  id: string;
  name: string;
  aphanite: boolean;
  favorite?: boolean;
  description: string;
  loader: Loader;
  mcVersion: string;
  loaderVersion: string;
  java: string;
  javaRuntimeId: string;
  createdAt: string;
  lastPlayed: string | null;
  playCount: number;
  mods: Mod[];
  resourcePacks: ResourcePack[];
  shaderPacks: ShaderPack[];
  worlds: World[];
  launchOverrides?: Partial<LaunchSettings>;
  /** Most recent crash report, retained as a persistent detail-page entry point. */
  lastCrashId?: string | null;
}

export interface JavaRuntime {
  id: string;
  name: string;
  version: number;
  path: string;
  managed: boolean;
}

export type WindowMode = "windowed" | "maximized" | "fullscreen";
export type QuickPlayMode = "none" | "server" | "world" | "realms";

export interface LaunchSettings {
  memoryMode: "auto" | "manual";
  memory: number;
  windowMode: WindowMode;
  windowWidth: number;
  windowHeight: number;
  quickPlayMode: QuickPlayMode;
  quickPlayTarget: string;
  gameArgs: string;
  environmentVariables: string;
  javaArgs: string;
  useDefaultJvmArgs: boolean;
  useOptimizedJvmArgs: boolean;
  skipJvmValidation: boolean;
  allowAutoAgent: boolean;
  skipIntegrityCheck: boolean;
  preLaunchCommand: string;
  commandWrapper: string;
  postExitCommand: string;
  useCustomNatives: boolean;
  nativesPath: string;
  skipNativePatching: boolean;
  glfwPath: string;
  openalPath: string;
}

export interface LaunchJob {
  instanceId: string;
  progress: number;
  phase: number;
  status: "starting";
}

export type AccountType = "microsoft" | "offline" | "aphanite" | "yggdrasil";

export interface PlayerProfile {
  id: string;
  name: string;
  skinUrl: string;
  isSlim?: boolean;
}

export interface Account {
  id: string;
  username: string;
  type: AccountType;
  lastUsed: string;
  authServer?: string;
  profiles: PlayerProfile[];
  activeProfileId: string;
}

export interface NewsItem {
  id: string;
  source: string;
  title: string;
  when: string;
}

export type AccentKey = "emerald" | "gold" | "slate" | "teal";
