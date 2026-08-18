//! Owned, deterministic gallery data translated from `design/src/lib/seed.ts`.

use crate::{route::StorageId, state::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewsItem {
    pub id: String,
    pub source: String,
    pub title: String,
    pub when: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportableModSummary {
    pub name: String,
    pub version: String,
    pub file_name: String,
    pub loader: String,
}

fn m(id: &str, name: &str, version: &str, loader: &str, enabled: bool) -> ModSummary {
    let file_name = match name {
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
    };
    ModSummary {
        id: id.into(),
        name: Some(name.into()),
        version: Some(version.into()),
        file_name,
        loader: Some(loader.into()),
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

fn instance(
    storage_id: StorageId,
    icon_seed: u64,
    id: &str,
    name: &str,
    aphanite: bool,
    favorite: bool,
    description: &str,
    loader: &str,
    mc: &str,
    loader_version: &str,
    java: &str,
    runtime: &str,
    created: &str,
    last_played: Option<&str>,
    play_count: u32,
    crash: Option<&str>,
    memory: Option<u32>,
    mods: Vec<ModSummary>,
    packs: Vec<ResourcePackSummary>,
    worlds: Vec<WorldSummary>,
) -> InstanceSummary {
    InstanceSummary {
        storage_id,
        id: id.into(),
        icon_seed,
        name: name.into(),
        aphanite,
        favorite,
        description: description.into(),
        loader: loader.into(),
        mc_version: mc.into(),
        loader_version: loader_version.into(),
        java: java.into(),
        java_runtime_id: runtime.into(),
        created_at: created.into(),
        last_played: last_played.map(Into::into),
        play_count,
        last_crash_id: crash.map(Into::into),
        launch_overrides: InstanceLaunchOverrides { memory },
        mods,
        resource_packs: packs,
        shader_packs: Vec::new(),
        worlds,
    }
}

pub fn seed_instances(storage_id: StorageId) -> Vec<InstanceSummary> {
    let mut instances = vec![
        instance(storage_id, 0x_f06, "inst-fog", "The Fog", true, true, "Vanilla-plus survival with Create, Sodium and JEI. The main world everyone actually plays on.", "fabric", "1.21.4", "0.115.1+1.21.4", "21", "zulu-21", "2026-02-14", Some("2 hours ago"), 312, Some("crash-sodium-optifine"), Some(6),
            vec![m("m-sodium","Sodium","0.6.9","fabric",true),m("m-optifine","OptiFine","HD_U_I8","fabric",true),m("m-iris","Iris Shaders","1.8.9","fabric",true),m("m-create","Create","6.0.1","fabric",true),m("m-jei","Just Enough Items","19.21.0.7","fabric",true),m("m-lithium","Lithium","0.16.1","fabric",true),m("m-kubejs","KubeJS","2101.6.1","fabric",true),m("m-yacl","YetAnotherConfigLib","3.6.6","fabric",true),m("m-modmenu","Mod Menu","13.0.0","fabric",true),m("m-rei","Roughly Enough Items","18.1.2","fabric",false),m("m-twigs","Twigs","4.0.2","fabric",false),m("m-sounds","Sound Physics Remastered","1.21.4","fabric",false),m("m-backpacks","Sophisticated Backpacks","3.22.1","fabric",false),m("m-midnightlib","MidnightLib","1.6.5","fabric",true),m("m-cloth","Cloth Config API","18.1.1","fabric",true),unreadable("m-unknown","legacy-addon-1.21.4.jar",false)],
            vec![pack("p-xray","Xray Ultimate","RayDyn","1.21.4","Friendly voxel outlines, no textures removed.","6.4 MB",true),pack("p-bare","Bare Bones","robotpant","1.21.4","A minimal, remastered take on the default look.","4.2 MB",true),pack("p-faithful","Faithful 64x","Vattic","1.21.4","The classic faithful higher-resolution pack.","22.1 MB",false)],
            vec![world("w-main","Survival World","7421095342","1.21.4","2 hours ago",4),world("w-creative","Creative Testing","-92873144","1.21.4","3 days ago",1),world("w-nethern","Nether Farm","108892","1.21.4","a week ago",2)]),
        instance(storage_id, 0x2a1, "inst-vanilla", "Vanilla Survival", false, false, "Pristine vanilla 1.21.4 with no modifications. For when the server demands purity.", "vanilla", "1.21.4", "—", "21", "zulu-21", "2026-03-02", Some("yesterday"), 48, Some("crash-unknown"), None, vec![], vec![pack("p-faithful","Faithful 64x","Vattic","1.21.4","The classic faithful higher-resolution pack.","22.1 MB",true)], vec![world("w-hardcore","Hardcore Run","77","1.21.4","yesterday",1)]),
        instance(storage_id, 0x2a2, "inst-neo", "NeoForge Server Test", true, false, "NeoForge test bed used to validate the server pack before it ships to the host.", "neoforge", "1.21.1", "21.1.181", "21", "zulu-21", "2026-05-11", Some("5 days ago"), 23, Some("crash-possible"), Some(8), vec![m("m-ftbq","FTB Quests","2100.1.1","neoforge",true),m("m-ftbl","FTB Library","2101.1.5","neoforge",true),m("m-ftbc","FTB Chunks","2100.1.1","neoforge",true),m("m-arch","Architectury API","13.0.8","neoforge",true),m("m-mek","Mekanism","10.7.10","neoforge",false),m("m-cc","CC: Tweaked","1.112.2","neoforge",false)], vec![pack("p-bare","Bare Bones","robotpant","1.21.1","A minimal, remastered take on the default look.","4.2 MB",true)], vec![world("w-test","Server Test","11223344","1.21.1","5 days ago",0)]),
        instance(storage_id, 0x2a3, "inst-legacy", "Old Faithful", false, true, "The 2018 modpack that will not die. Forge 1.12.2, still running a community server.", "forge", "1.12.2", "14.23.5.2860", "8", "legacy-8", "2026-01-08", Some("a month ago"), 509, Some("crash-jvm"), Some(6), vec![m("m-old-1","Thaumcraft","6.1.BETA26","forge",true),m("m-old-2","Buildcraft","7.99.24","forge",true),m("m-old-3","Tinkers’ Construct","2.13.0.183","forge",true)], vec![], vec![]),
        instance(storage_id, 0x2a4, "inst-create", "Create: Engineering", true, false, "Pure Create contraptions sandbox. Water wheels, trains, and very questionable elevators.", "fabric", "1.20.1", "0.15.11+1.20.1", "17", "temurin-17", "2026-06-21", Some("never"), 0, None, Some(5), vec![m("m-create-1","Create","0.5.1j","fabric",true),m("m-create-2","Create: Steam ‘n’ Rails","1.6.4","fabric",true),m("m-create-3","Flywheel","0.6.10","fabric",true)], vec![], vec![]),
    ];
    instances[0].shader_packs = vec![
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
    ];
    instances
}

pub fn seed_accounts() -> Vec<AccountSummary> {
    vec![
        AccountSummary {
            id: "acc-enita".into(),
            username: "Enita_Nureya".into(),
            account_type: "microsoft".into(),
            last_used: "2 hours ago".into(),
            active_profile_id: "profile-enita".into(),
            profiles: vec![PlayerProfileSummary {
                id: "profile-enita".into(),
                name: "enita".into(),
                skin_url: "https://mc-heads.net/skin/Enita_Nureya".into(),
                is_slim: true,
            }],
        },
        AccountSummary {
            id: "acc-steve".into(),
            username: "Steve".into(),
            account_type: "offline".into(),
            last_used: "3 days ago".into(),
            active_profile_id: "profile-steve".into(),
            profiles: vec![PlayerProfileSummary {
                id: "profile-steve".into(),
                name: "Steve".into(),
                skin_url: "https://mc-heads.net/skin/Steve".into(),
                is_slim: false,
            }],
        },
        AccountSummary {
            id: "acc-alex".into(),
            username: "alex_03".into(),
            account_type: "offline".into(),
            last_used: "a month ago".into(),
            active_profile_id: "profile-alex".into(),
            profiles: vec![PlayerProfileSummary {
                id: "profile-alex".into(),
                name: "alex_03".into(),
                skin_url: "https://mc-heads.net/skin/Alex".into(),
                is_slim: true,
            }],
        },
    ]
}

pub fn seed_news() -> Vec<NewsItem> {
    vec![
        NewsItem {
            id: "n1".into(),
            source: "Mojang".into(),
            title: "Minecraft Java 1.21.5 Released".into(),
            when: "yesterday".into(),
        },
        NewsItem {
            id: "n2".into(),
            source: "Fabric".into(),
            title: "Fabric API 0.115.1 for 1.21.4".into(),
            when: "2 days ago".into(),
        },
        NewsItem {
            id: "n3".into(),
            source: "NeoForged".into(),
            title: "NeoForge 21.1.181 out now".into(),
            when: "4 days ago".into(),
        },
        NewsItem {
            id: "n4".into(),
            source: "CaffeineMC".into(),
            title: "Sodium 0.6.9 — smaller, faster".into(),
            when: "a week ago".into(),
        },
    ]
}

pub fn importable_mods() -> Vec<ImportableModSummary> {
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
    .map(|(name, version, file_name)| ImportableModSummary {
        name: name.into(),
        version: version.into(),
        file_name: file_name.into(),
        loader: "fabric".into(),
    })
    .collect()
}

pub fn seed_crash_reports(storage_id: StorageId) -> Vec<CrashReport> {
    let header = || {
        vec![
            "---- Minecraft Crash Report ----".into(),
            "// Oops. The game did not enjoy that.".into(),
            "".into(),
            "Time: 2026-08-17 14:22:18".into(),
            "Description: Initializing game".into(),
            "".into(),
        ]
    };
    let env = |mc: &str,
               loader: &str,
               loader_version: &str,
               java: &str,
               java_version: u32,
               memory: u32,
               source: &str| CrashEnvironment {
        mc_version: mc.into(),
        loader: loader.into(),
        loader_version: loader_version.into(),
        java_name: java.into(),
        java_version,
        java_path: "/home/enita/.local/share/phanerite/java/zulu-21/bin/java".into(),
        memory,
        os: "Fedora Linux 44".into(),
        gpu: "NVIDIA GeForce GTX 1060 6GB".into(),
        enabled_mods: Vec::new(),
        active_overrides: if memory > 0 {
            vec!["memory".into()]
        } else {
            vec![]
        },
        source: source.into(),
        aphanite_server: if source == "aphanite" {
            Some("aphanite.enita.cn".into())
        } else {
            None
        },
    };
    let finding = |rule: &str, title: &str, explanation: &str, lines: Vec<u32>, mods: Vec<&str>| {
        CrashFinding {
            rule: rule.into(),
            title: title.into(),
            explanation: explanation.into(),
            evidence_lines: lines,
            implicated_mod_ids: mods.into_iter().map(Into::into).collect(),
            suggested_memory: None,
        }
    };
    let mut fog = env(
        "1.21.4",
        "fabric",
        "0.115.1+1.21.4",
        "Azul Zulu JDK 21",
        21,
        6,
        "aphanite",
    );
    fog.enabled_mods = vec![
        ("Sodium".into(), "0.6.9".into()),
        ("OptiFine".into(), "HD_U_I8".into()),
        ("Iris Shaders".into(), "1.8.9".into()),
        ("Create".into(), "6.0.1".into()),
        ("Just Enough Items".into(), "19.21.0.7".into()),
    ];
    let mut neo = env(
        "1.21.1",
        "neoforge",
        "21.1.181",
        "Azul Zulu JDK 21",
        21,
        8,
        "aphanite",
    );
    neo.enabled_mods = vec![
        ("FTB Quests".into(), "2100.1.1".into()),
        ("FTB Library".into(), "2101.1.5".into()),
        ("FTB Chunks".into(), "2100.1.1".into()),
        ("Architectury API".into(), "13.0.8".into()),
    ];
    let mut vanilla = env("1.21.4", "vanilla", "—", "Azul Zulu JDK 21", 21, 4, "local");
    vanilla.enabled_mods.clear();
    vanilla.active_overrides.clear();
    let mut legacy = env(
        "1.12.2",
        "forge",
        "14.23.5.2860",
        "OpenJDK 8",
        8,
        6,
        "local",
    );
    legacy.java_path = "/usr/lib/jvm/java-8-openjdk/bin/java".into();
    legacy.enabled_mods = vec![
        ("Thaumcraft".into(), "6.1.BETA26".into()),
        ("Buildcraft".into(), "7.99.24".into()),
        ("Tinkers’ Construct".into(), "2.13.0.183".into()),
    ];
    vec![
        CrashReport{storage_id,id:"crash-sodium-optifine".into(),instance_id:"inst-fog".into(),when:"2 minutes ago".into(),exit_code:1,lines:Some({let mut x=header();x.extend(["java.lang.RuntimeException: Mixin apply failed net.minecraft.class_310","Mixin apply for mod optifine failed","  at net.fabricmc.loader.impl.launch.knot.KnotClassDelegate.getPostMixinClassByteArray(KnotClassDelegate.java:427)","  at net.optifine.reflect.Reflector.<clinit>(Reflector.java:155)","  at me.jellysquid.mods.sodium.client.SodiumClientMod.onInitializeClient(SodiumClientMod.java:52)","Caused by: org.spongepowered.asm.mixin.transformer.throwables.MixinTransformerError: An unexpected critical error was encountered","  at org.spongepowered.asm.mixin.transformer.MixinProcessor.applyMixins(MixinProcessor.java:392)","", "-- System Details --","Details:","\tMinecraft Version: 1.21.4","\tOperating System: Linux (amd64) version 6.14.0","\tJava Version: 21.0.8, Azul Systems, Inc.","\tMemory: 1148928000 bytes (1095 MiB) / 6442450944 bytes (6144 MiB) up to 6442450944 bytes (6144 MiB)","\tJVM Flags: 2 total; -Xmx6G --accessToken *******************************","\tLaunched Version: fabric-loader-0.16.9-1.21.4"].into_iter().map(String::from));x}),stderr_tail:vec![],hs_err_path:None,findings:vec![finding("MIXIN_APPLY_MOD_FAILED","OptiFine failed while its mixin was being applied","The mod id appears directly in the matched crash signature. Disable or update OptiFine before retrying.",vec![7],vec!["m-optifine"])],environment:fog},
        CrashReport{storage_id,id:"crash-possible".into(),instance_id:"inst-neo".into(),when:"18 minutes ago".into(),exit_code:1,lines:Some({let mut x=header();x.extend(["java.lang.NoSuchMethodError: 'void net.neoforged.fml.ModList.registerEventHandler()'","  at dev.ftb.mods.ftbchunks.FTBChunks.init(FTBChunks.java:84)","  at net.neoforged.fml.ModLoader.gatherAndInitializeMods(ModLoader.java:203)","Caused by: java.lang.IllegalStateException: Could not load mod ftbquests","  at dev.ftb.mods.ftbquests.FTBQuestsCommon.init(FTBQuestsCommon.java:66)","", "-- System Details --","Details:","\tMinecraft Version: 1.21.1","\tMod Launcher: 11.0.4"].into_iter().map(String::from));x}),stderr_tail:vec![],hs_err_path:None,findings:vec![finding("NO_SUCH_METHOD_ERROR","A required method was not available","This signature can indicate a missing or incompatible mod or game component. It does not identify one mod by itself.",vec![7],vec![])],environment:neo},
        CrashReport{storage_id,id:"crash-unknown".into(),instance_id:"inst-vanilla".into(),when:"yesterday".into(),exit_code:1,lines:Some({let mut x=header();x.extend(["java.lang.IllegalStateException: Failed to load registries due to above errors","  at net.minecraft.server.Bootstrap.bootStrap(Bootstrap.java:124)","  at net.minecraft.client.main.Main.main(Main.java:201)","Caused by: java.io.IOException: Error reading level.dat","  at net.minecraft.world.level.storage.LevelStorageSource.readLevelData(LevelStorageSource.java:301)","  at net.minecraft.client.Minecraft.createLevel(Minecraft.java:517)","", "-- System Details --","Details:","\tMinecraft Version: 1.21.4","\tOperating System: Linux (amd64) version 6.14.0","\tJava Version: 21.0.8, Azul Systems, Inc.","\tMemory: 402653184 bytes (384 MiB) / 4294967296 bytes (4096 MiB) up to 4294967296 bytes (4096 MiB)"].into_iter().map(String::from));x}),stderr_tail:vec![],hs_err_path:None,findings:vec![],environment:vanilla},
        CrashReport{storage_id,id:"crash-jvm".into(),instance_id:"inst-legacy".into(),when:"3 days ago".into(),exit_code:-6,lines:None,stderr_tail:vec!["# A fatal error has been detected by the Java Runtime Environment:".into(),"#  SIGSEGV (0xb) at pc=0x00007f1dd405ddf2, pid=48721, tid=48742".into(),"# JRE version: OpenJDK Runtime Environment (8.0_472-b08)".into(),"# Java VM: OpenJDK 64-Bit Server VM (25.472-b08 mixed mode linux-amd64 compressed oops)".into(),"# Problematic frame:".into(),"# C  [libGLX_nvidia.so.0+0x2ddf2]".into(),"Aborted (core dumped)".into()],hs_err_path:Some("/home/enita/.local/share/phanerite/instances/old-faithful/hs_err_pid48721.log".into()),findings:vec![],environment:legacy},
    ]
}
