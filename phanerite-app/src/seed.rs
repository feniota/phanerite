//! Owned, deterministic gallery data translated from `design/src/lib/seed.ts`,
//! `crash.ts` and the prototype's initial `AppState`.

use crate::{route::StorageIdent, state::*};
use std::path::PathBuf;

/// Creates the deterministic storage identity used by the gallery seed.
pub fn storage_ident(value: u64) -> StorageIdent {
    StorageIdent {
        root_dir: PathBuf::from(format!("/phanerite-test-storage/{value}")),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewsItem {
    pub id: String,
    pub source: String,
    pub title: String,
    pub when: String,
}

/// A mod the import dialog can add, used by the gallery's sample import.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportableMod {
    pub name: String,
    pub version: String,
    pub file_name: String,
    pub loader: Loader,
}

fn file_name_for(name: &str, version: &str) -> String {
    match name {
        "OptiFine" => "optifine-hd_u_i8.jar".into(),
        "Iris Shaders" => "iris-shaders-1.8.9.jar".into(),
        "Just Enough Items" => "just-enough-items-19.21.0.7.jar".into(),
        "YetAnotherConfigLib" => "yetanotherconfiglib-3.6.6.jar".into(),
        "Sound Physics Remastered" => "sound-physics-remastered-1.21.4.jar".into(),
        "Sophisticated Backpacks" => "sophisticated-backpacks-3.22.1.jar".into(),
        "Tinkers’ Construct" => "tinkers-construct-2.13.0.183.jar".into(),
        "Create: Steam ‘n’ Rails" => "create-steam-n-rails-1.6.4.jar".into(),
        "CC: Tweaked" => "cc-tweaked-1.112.2.jar".into(),
        _ => format!("{name}-{version}.jar")
            .to_lowercase()
            .replace(' ', "-"),
    }
}

fn item(id: &str, name: &str, version: &str, loader: Loader, enabled: bool) -> ModSummary {
    ModSummary {
        id: id.into(),
        name: Some(name.into()),
        version: Some(version.into()),
        file_name: file_name_for(name, version),
        loader: Some(loader),
        enabled,
    }
}

fn unreadable(id: &str, file_name: &str, enabled: bool) -> ModSummary {
    ModSummary {
        id: id.into(),
        name: None,
        version: None,
        file_name: file_name.into(),
        loader: None,
        enabled,
    }
}

fn pack(
    id: &str,
    name: &str,
    author: &str,
    version: &str,
    description: &str,
    size: &str,
    enabled: bool,
) -> ResourcePackSummary {
    ResourcePackSummary {
        id: id.into(),
        name: name.into(),
        author: author.into(),
        version: version.into(),
        description: description.into(),
        size: size.into(),
        enabled,
    }
}

fn world(
    id: &str,
    name: &str,
    seed: &str,
    version: &str,
    last_played: &str,
    players: u32,
) -> WorldSummary {
    WorldSummary {
        id: id.into(),
        name: name.into(),
        seed: seed.into(),
        version: version.into(),
        last_played: last_played.into(),
        players,
    }
}

fn memory_override(gigabytes: u32) -> LaunchOverrides {
    let mut overrides = LaunchOverrides::default();
    overrides.set(LaunchField::Memory, LaunchValue::Number(gigabytes));
    overrides
}

#[allow(clippy::too_many_arguments)]
struct InstanceSeed {
    id: &'static str,
    name: &'static str,
    aphanite: bool,
    favorite: bool,
    description: &'static str,
    loader: Loader,
    mc_version: &'static str,
    loader_version: &'static str,
    java: &'static str,
    java_runtime_id: &'static str,
    created_at: &'static str,
    last_played: Option<&'static str>,
    play_count: u32,
    last_crash_id: Option<&'static str>,
    memory: Option<u32>,
}

impl InstanceSeed {
    fn build(
        self,
        storage: StorageIdent,
        mods: Vec<ModSummary>,
        resource_packs: Vec<ResourcePackSummary>,
        shader_packs: Vec<ShaderPackSummary>,
        worlds: Vec<WorldSummary>,
    ) -> InstanceSummary {
        InstanceSummary {
            storage: storage.clone(),
            id: self.id.into(),
            icon_seed: icon_seed(self.name, self.mc_version, self.loader),
            name: self.name.into(),
            aphanite: self.aphanite,
            favorite: self.favorite,
            description: self.description.into(),
            loader: self.loader,
            mc_version: self.mc_version.into(),
            loader_version: self.loader_version.into(),
            java: self.java.into(),
            java_runtime_id: self.java_runtime_id.into(),
            created_at: self.created_at.into(),
            last_played: self.last_played.map(Into::into),
            play_count: self.play_count,
            last_crash_id: self.last_crash_id.map(Into::into),
            launch_overrides: self.memory.map(memory_override).unwrap_or_default(),
            mods,
            resource_packs,
            shader_packs,
            worlds,
        }
    }
}

pub fn seed_instances(storage: StorageIdent) -> Vec<InstanceSummary> {
    vec![
        InstanceSeed {
            id: "inst-fog",
            name: "The Fog",
            aphanite: true,
            favorite: true,
            description: "Vanilla-plus survival with Create, Sodium and JEI. The main world everyone actually plays on.",
            loader: Loader::Fabric,
            mc_version: "1.21.4",
            loader_version: "0.115.1+1.21.4",
            java: "21",
            java_runtime_id: "zulu-21",
            created_at: "2026-02-14",
            last_played: Some("2 hours ago"),
            play_count: 312,
            last_crash_id: Some("crash-sodium-optifine"),
            memory: Some(6),
        }
        .build(
            storage.clone(),
            vec![
                item("m-sodium", "Sodium", "0.6.9", Loader::Fabric, true),
                item("m-optifine", "OptiFine", "HD_U_I8", Loader::Fabric, true),
                item("m-iris", "Iris Shaders", "1.8.9", Loader::Fabric, true),
                item("m-create", "Create", "6.0.1", Loader::Fabric, true),
                item("m-jei", "Just Enough Items", "19.21.0.7", Loader::Fabric, true),
                item("m-lithium", "Lithium", "0.16.1", Loader::Fabric, true),
                item("m-kubejs", "KubeJS", "2101.6.1", Loader::Fabric, true),
                item("m-yacl", "YetAnotherConfigLib", "3.6.6", Loader::Fabric, true),
                item("m-modmenu", "Mod Menu", "13.0.0", Loader::Fabric, true),
                item("m-rei", "Roughly Enough Items", "18.1.2", Loader::Fabric, false),
                item("m-twigs", "Twigs", "4.0.2", Loader::Fabric, false),
                item("m-sounds", "Sound Physics Remastered", "1.21.4", Loader::Fabric, false),
                item("m-backpacks", "Sophisticated Backpacks", "3.22.1", Loader::Fabric, false),
                item("m-midnightlib", "MidnightLib", "1.6.5", Loader::Fabric, true),
                item("m-cloth", "Cloth Config API", "18.1.1", Loader::Fabric, true),
                unreadable("m-unknown", "legacy-addon-1.21.4.jar", false),
            ],
            vec![
                pack("p-xray", "Xray Ultimate", "RayDyn", "1.21.4", "Friendly voxel outlines, no textures removed.", "6.4 MB", true),
                pack("p-bare", "Bare Bones", "robotpant", "1.21.4", "A minimal, remastered take on the default look.", "4.2 MB", true),
                pack("p-faithful", "Faithful 64x", "Vattic", "1.21.4", "The classic faithful higher-resolution pack.", "22.1 MB", false),
            ],
            vec![
                ShaderPackSummary {
                    id: "s-comp".into(),
                    name: "Complementary Reimagined".into(),
                    author: "EminGT".into(),
                    version: "r5.4".into(),
                    gpu: "GTX 1060 6GB".into(),
                    enabled: true,
                },
                ShaderPackSummary {
                    id: "s-bsl".into(),
                    name: "BSL Shaders".into(),
                    author: "capttatsu".into(),
                    version: "v8.4".into(),
                    gpu: "GTX 1070 8GB".into(),
                    enabled: false,
                },
            ],
            vec![
                world("w-main", "Survival World", "7421095342", "1.21.4", "2 hours ago", 4),
                world("w-creative", "Creative Testing", "-92873144", "1.21.4", "3 days ago", 1),
                world("w-nethern", "Nether Farm", "108892", "1.21.4", "a week ago", 2),
            ],
        ),
        InstanceSeed {
            id: "inst-vanilla",
            name: "Vanilla Survival",
            aphanite: false,
            favorite: false,
            description: "Pristine vanilla 1.21.4 with no modifications. For when the server demands purity.",
            loader: Loader::Vanilla,
            mc_version: "1.21.4",
            loader_version: "—",
            java: "21",
            java_runtime_id: "zulu-21",
            created_at: "2026-03-02",
            last_played: Some("yesterday"),
            play_count: 48,
            last_crash_id: Some("crash-unknown"),
            memory: None,
        }
        .build(
            storage.clone(),
            Vec::new(),
            vec![pack("p-faithful", "Faithful 64x", "Vattic", "1.21.4", "The classic faithful higher-resolution pack.", "22.1 MB", true)],
            Vec::new(),
            vec![world("w-hardcore", "Hardcore Run", "77", "1.21.4", "yesterday", 1)],
        ),
        InstanceSeed {
            id: "inst-neo",
            name: "NeoForge Server Test",
            aphanite: true,
            favorite: false,
            description: "NeoForge test bed used to validate the server pack before it ships to the host.",
            loader: Loader::NeoForge,
            mc_version: "1.21.1",
            loader_version: "21.1.181",
            java: "21",
            java_runtime_id: "zulu-21",
            created_at: "2026-05-11",
            last_played: Some("5 days ago"),
            play_count: 23,
            last_crash_id: Some("crash-possible"),
            memory: Some(8),
        }
        .build(
            storage.clone(),
            vec![
                item("m-ftbq", "FTB Quests", "2100.1.1", Loader::NeoForge, true),
                item("m-ftbl", "FTB Library", "2101.1.5", Loader::NeoForge, true),
                item("m-ftbc", "FTB Chunks", "2100.1.1", Loader::NeoForge, true),
                item("m-arch", "Architectury API", "13.0.8", Loader::NeoForge, true),
                item("m-mek", "Mekanism", "10.7.10", Loader::NeoForge, false),
                item("m-cc", "CC: Tweaked", "1.112.2", Loader::NeoForge, false),
            ],
            vec![pack("p-bare", "Bare Bones", "robotpant", "1.21.1", "A minimal, remastered take on the default look.", "4.2 MB", true)],
            Vec::new(),
            vec![world("w-test", "Server Test", "11223344", "1.21.1", "5 days ago", 0)],
        ),
        InstanceSeed {
            id: "inst-legacy",
            name: "Old Faithful",
            aphanite: false,
            favorite: true,
            description: "The 2018 modpack that will not die. Forge 1.12.2, still running a community server.",
            loader: Loader::Forge,
            mc_version: "1.12.2",
            loader_version: "14.23.5.2860",
            java: "8",
            java_runtime_id: "legacy-8",
            created_at: "2026-01-08",
            last_played: Some("a month ago"),
            play_count: 509,
            last_crash_id: Some("crash-jvm"),
            memory: Some(6),
        }
        .build(
            storage.clone(),
            vec![
                item("m-old-1", "Thaumcraft", "6.1.BETA26", Loader::Forge, true),
                item("m-old-2", "Buildcraft", "7.99.24", Loader::Forge, true),
                item("m-old-3", "Tinkers’ Construct", "2.13.0.183", Loader::Forge, true),
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        InstanceSeed {
            id: "inst-create",
            name: "Create: Engineering",
            aphanite: true,
            favorite: false,
            description: "Pure Create contraptions sandbox. Water wheels, trains, and very questionable elevators.",
            loader: Loader::Fabric,
            mc_version: "1.20.1",
            loader_version: "0.15.11+1.20.1",
            java: "17",
            java_runtime_id: "temurin-17",
            created_at: "2026-06-21",
            last_played: None,
            play_count: 0,
            last_crash_id: None,
            memory: Some(5),
        }
        .build(
            storage.clone(),
            vec![
                item("m-create-1", "Create", "0.5.1j", Loader::Fabric, true),
                item("m-create-2", "Create: Steam ‘n’ Rails", "1.6.4", Loader::Fabric, true),
                item("m-create-3", "Flywheel", "0.6.10", Loader::Fabric, true),
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
    ]
}

pub fn seed_accounts() -> Vec<phanerite_core::auth::Account> {
    ["Enita_Nureya", "Steve", "alex_03"]
        .into_iter()
        .map(|name| {
            phanerite_core::auth::Account::Offline(
                phanerite_core::auth::offline::Authentication::new(name),
            )
        })
        .collect()
}

pub fn seed_runtimes() -> Vec<JavaRuntimeSummary> {
    vec![
        JavaRuntimeSummary {
            id: "zulu-21".into(),
            name: "Azul Zulu JDK 21".into(),
            version: 21,
            version_string: "21.0.6".into(),
            path: "Managed by Phanerite".into(),
            managed: true,
        },
        JavaRuntimeSummary {
            id: "temurin-17".into(),
            name: "Eclipse Temurin JDK 17".into(),
            version: 17,
            version_string: "17.0.13".into(),
            path: "/usr/lib/jvm/java-17-openjdk".into(),
            managed: false,
        },
        JavaRuntimeSummary {
            id: "legacy-8".into(),
            name: "OpenJDK 8".into(),
            version: 8,
            version_string: "1.8.0_392".into(),
            path: "/usr/lib/jvm/java-8-openjdk".into(),
            managed: false,
        },
    ]
}

pub fn seed_news() -> Vec<NewsItem> {
    [
        (
            "n1",
            "Mojang",
            "Minecraft Java 1.21.5 Released",
            "yesterday",
        ),
        (
            "n2",
            "Fabric",
            "Fabric API 0.115.1 for 1.21.4",
            "2 days ago",
        ),
        ("n3", "NeoForged", "NeoForge 21.1.181 out now", "4 days ago"),
        (
            "n4",
            "CaffeineMC",
            "Sodium 0.6.9 — smaller, faster",
            "a week ago",
        ),
    ]
    .into_iter()
    .map(|(id, source, title, when)| NewsItem {
        id: id.into(),
        source: source.into(),
        title: title.into(),
        when: when.into(),
    })
    .collect()
}

pub fn importable_mods() -> Vec<ImportableMod> {
    [
        (
            "JourneyMap",
            "1.21.4-6.0.0",
            "journeymap-1.21.4-6.0.0-fabric.jar",
        ),
        (
            "Distant Horizons",
            "2.3.0",
            "DistantHorizons-2.3.0-fabric.jar",
        ),
        ("Supplementaries", "3.0.2", "supplementaries-3.0.2.jar"),
        ("Better Combat", "1.9.5", "bettercombat-1.9.5.jar"),
        ("Terralith", "2.5.8", "terralith-2.5.8.jar"),
        ("AppleSkin", "3.0.4", "appleskin-fabric-mc1.21-3.0.4.jar"),
    ]
    .into_iter()
    .map(|(name, version, file_name)| ImportableMod {
        name: name.into(),
        version: version.into(),
        file_name: file_name.into(),
        loader: Loader::Fabric,
    })
    .collect()
}

/// Resource packs the import dialog offers, from `AddResourcesDialog.svelte`.
pub fn importable_resource_packs() -> Vec<ResourcePackSummary> {
    vec![
        pack(
            "",
            "Vanilla Tweaks",
            "xisumavoid",
            "1.21.4",
            "Subtle quality-of-life texture tweaks.",
            "2.3 MB",
            true,
        ),
        pack(
            "",
            "Programmer Art",
            "Mojang",
            "1.21.4",
            "The classic pixel look, restored.",
            "1.8 MB",
            true,
        ),
    ]
}

/// Shader packs the import dialog offers.
pub fn importable_shader_packs() -> Vec<ShaderPackSummary> {
    [
        ("Complementary Unbound", "EminGT", "r5.3", "GTX 1060 6GB"),
        ("MakeUp Ultra Fast", "Capt Tatsu", "v9.1c", "Intel UHD 630"),
    ]
    .into_iter()
    .map(|(name, author, version, gpu)| ShaderPackSummary {
        id: String::new(),
        name: name.into(),
        author: author.into(),
        version: version.into(),
        gpu: gpu.into(),
        enabled: true,
    })
    .collect()
}

/// The captured output every instance starts with in the prototype.
pub fn seed_instance_log(instance_name: &str) -> Vec<LiveLogLine> {
    vec![
        LiveLogLine::stdout("[12:14:03] [Render thread/INFO]: Reloading ResourceManager: vanilla"),
        LiveLogLine::stdout(format!(
            "[12:14:06] [Server thread/INFO]: {instance_name} local session started"
        )),
        LiveLogLine::stdout(
            "[12:14:08] [Render thread/INFO]: OpenAL initialized on device: Default",
        ),
        LiveLogLine::stdout(
            "[12:15:17] [Server thread/INFO]: Saving chunks for level 'ServerLevel[world]'",
        ),
    ]
}

/// `latest.log`, as shown by the prototype's log source selector.
pub fn seed_latest_log() -> Vec<String> {
    [
        "[12:14:03] [Render thread/INFO]: Reloading ResourceManager: vanilla",
        "[12:14:06] [Server thread/INFO]: Local session started",
        "[12:14:08] [Render thread/WARN]: Missing optional resource pack metadata",
        "[12:15:17] [Server thread/INFO]: Saving chunks for level 'ServerLevel[world]'",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// `debug.log`, as shown by the prototype's log source selector.
pub fn seed_debug_log() -> Vec<String> {
    [
        "[12:14:02] [main/DEBUG]: Launch arguments resolved",
        "[12:14:03] [main/INFO]: Using Java runtime version 21",
        "[12:14:04] [main/DEBUG]: Fabric loader initialized",
        "[12:14:07] [Render thread/WARN]: Skipping unavailable GPU extension",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// The repeating lines a running game emits in the prototype.
pub fn live_output_samples() -> [&'static str; 4] {
    [
        "[Render thread/INFO]: Chunk render batches rebuilt",
        "[Server thread/INFO]: Saving players",
        "[Render thread/INFO]: Sound engine tick",
        "[Server thread/INFO]: Autosave completed",
    ]
}

fn crash_header() -> Vec<String> {
    [
        "---- Minecraft Crash Report ----",
        "// Oops. The game did not enjoy that.",
        "",
        "Time: 2026-08-17 14:22:18",
        "Description: Initializing game",
        "",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn body(extra: &[&str]) -> Option<Vec<String>> {
    let mut lines = crash_header();
    lines.extend(extra.iter().map(|line| line.to_string()));
    Some(lines)
}

fn environment(
    mc_version: &str,
    loader: Loader,
    loader_version: &str,
    java_name: &str,
    java_version: u32,
    java_path: &str,
    memory: u32,
    enabled_mods: &[(&str, &str)],
    active_overrides: &[&str],
    source: CrashSource,
) -> CrashEnvironment {
    CrashEnvironment {
        mc_version: mc_version.into(),
        loader,
        loader_version: loader_version.into(),
        java_name: java_name.into(),
        java_version,
        java_path: java_path.into(),
        memory,
        os: "Fedora Linux 44".into(),
        gpu: "NVIDIA GeForce GTX 1060 6GB".into(),
        enabled_mods: enabled_mods
            .iter()
            .map(|(name, version)| ((*name).to_string(), (*version).to_string()))
            .collect(),
        active_overrides: active_overrides
            .iter()
            .map(|key| (*key).to_string())
            .collect(),
        source,
        aphanite_server: match source {
            CrashSource::Aphanite => Some("aphanite.enita.cn".into()),
            CrashSource::Local => None,
        },
    }
}

const ZULU_PATH: &str = "/home/enita/.local/share/phanerite/java/zulu-21/bin/java";

pub fn seed_crash_reports(storage: StorageIdent) -> Vec<CrashReport> {
    vec![
        CrashReport {
            storage: storage.clone(),
            id: "crash-sodium-optifine".into(),
            instance_id: "inst-fog".into(),
            when: "2 minutes ago".into(),
            exit_code: 1,
            lines: body(&[
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
            ]),
            stderr_tail: Vec::new(),
            hs_err_path: None,
            findings: vec![CrashFinding {
                rule: "MIXIN_APPLY_MOD_FAILED".into(),
                title: "OptiFine failed while its mixin was being applied".into(),
                explanation: "The mod id appears directly in the matched crash signature. Disable or update OptiFine before retrying.".into(),
                evidence_lines: vec![7],
                implicated_mod_ids: vec!["m-optifine".into()],
                suggested_memory: None,
            }],
            environment: environment(
                "1.21.4",
                Loader::Fabric,
                "0.115.1+1.21.4",
                "Azul Zulu JDK 21",
                21,
                ZULU_PATH,
                6,
                &[
                    ("Sodium", "0.6.9"),
                    ("OptiFine", "HD_U_I8"),
                    ("Iris Shaders", "1.8.9"),
                    ("Create", "6.0.1"),
                    ("Just Enough Items", "19.21.0.7"),
                ],
                &["memory"],
                CrashSource::Aphanite,
            ),
        },
        CrashReport {
            storage: storage.clone(),
            id: "crash-possible".into(),
            instance_id: "inst-neo".into(),
            when: "18 minutes ago".into(),
            exit_code: 1,
            lines: body(&[
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
            ]),
            stderr_tail: Vec::new(),
            hs_err_path: None,
            findings: vec![CrashFinding {
                rule: "NO_SUCH_METHOD_ERROR".into(),
                title: "A required method was not available".into(),
                explanation: "This signature can indicate a missing or incompatible mod or game component. It does not identify one mod by itself.".into(),
                evidence_lines: vec![7],
                implicated_mod_ids: Vec::new(),
                suggested_memory: None,
            }],
            environment: environment(
                "1.21.1",
                Loader::NeoForge,
                "21.1.181",
                "Azul Zulu JDK 21",
                21,
                ZULU_PATH,
                8,
                &[
                    ("FTB Quests", "2100.1.1"),
                    ("FTB Library", "2101.1.5"),
                    ("FTB Chunks", "2100.1.1"),
                    ("Architectury API", "13.0.8"),
                ],
                &["memory"],
                CrashSource::Aphanite,
            ),
        },
        CrashReport {
            storage: storage.clone(),
            id: "crash-unknown".into(),
            instance_id: "inst-vanilla".into(),
            when: "yesterday".into(),
            exit_code: 1,
            lines: body(&[
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
            ]),
            stderr_tail: Vec::new(),
            hs_err_path: None,
            findings: Vec::new(),
            environment: environment(
                "1.21.4",
                Loader::Fabric,
                "0.115.1+1.21.4",
                "Azul Zulu JDK 21",
                21,
                ZULU_PATH,
                4,
                &[],
                &[],
                CrashSource::Local,
            ),
        },
        CrashReport {
            storage: storage.clone(),
            id: "crash-jvm".into(),
            instance_id: "inst-legacy".into(),
            when: "3 days ago".into(),
            exit_code: -6,
            lines: None,
            stderr_tail: [
                "# A fatal error has been detected by the Java Runtime Environment:",
                "#  SIGSEGV (0xb) at pc=0x00007f1dd405ddf2, pid=48721, tid=48742",
                "# JRE version: OpenJDK Runtime Environment (8.0_472-b08)",
                "# Java VM: OpenJDK 64-Bit Server VM (25.472-b08 mixed mode linux-amd64 compressed oops)",
                "# Problematic frame:",
                "# C  [libGLX_nvidia.so.0+0x2ddf2]",
                "Aborted (core dumped)",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            hs_err_path: Some(
                "/home/enita/.local/share/phanerite/instances/old-faithful/hs_err_pid48721.log"
                    .into(),
            ),
            findings: Vec::new(),
            environment: environment(
                "1.12.2",
                Loader::Forge,
                "14.23.5.2860",
                "OpenJDK 8",
                8,
                "/usr/lib/jvm/java-8-openjdk/bin/java",
                6,
                &[
                    ("Thaumcraft", "6.1.BETA26"),
                    ("Buildcraft", "7.99.24"),
                    ("Tinkers’ Construct", "2.13.0.183"),
                ],
                &["memory"],
                CrashSource::Local,
            ),
        },
    ]
}

/// The Aphanite server the prototype is connected to.
pub fn seed_aphanite(storage: StorageIdent) -> AphaniteSummary {
    AphaniteSummary {
        storage,
        server_name: "Enita's Aphanite Server".into(),
        server_url: "https://aphanite.enita.cn/".into(),
    }
}
