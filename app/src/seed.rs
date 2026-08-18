use crate::{route::StorageId, state::*};
pub fn seed_instances(storage_id: StorageId) -> Vec<InstanceSummary> {
    let m = |id: &str, name: &str, version: &str, enabled| ModSummary {
        id: id.into(),
        name: Some(name.into()),
        version: Some(version.into()),
        file_name: format!("{name}-{version}.jar"),
        loader: Some("fabric".into()),
        enabled,
    };
    vec![InstanceSummary{storage_id,id:"inst-fog".into(),name:"The Fog".into(),aphanite:true,favorite:true,description:"Vanilla-plus survival with Create, Sodium and JEI. The main world everyone actually plays on.".into(),loader:"fabric".into(),mc_version:"1.21.4".into(),loader_version:"0.115.1+1.21.4".into(),java:"21".into(),java_runtime_id:"zulu-21".into(),created_at:"2026-02-14".into(),last_played:Some("2 hours ago".into()),play_count:312,last_crash_id:Some("crash-sodium-optifine".into()),launch_overrides:InstanceLaunchOverrides{memory:Some(6)},mods:vec![m("m-sodium","Sodium","0.6.9",true),m("m-optifine","OptiFine","HD_U_I8",true),m("m-iris","Iris Shaders","1.8.9",true),m("m-create","Create","6.0.1",true)],resource_packs:vec![],shader_packs:vec![],worlds:vec![]}]
}
pub fn seed_accounts() -> Vec<AccountSummary> {
    vec![AccountSummary {
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
    }]
}
