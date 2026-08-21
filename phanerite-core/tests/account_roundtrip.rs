use phanerite_core::auth::{Account, Authentication};

const MSA: &str = r#"{
  "type": "microsoft",
  "client_id": "cid",
  "tenant": "consumers",
  "scope": "XboxLive.signin offline_access",
  "xuid": "2535000000000000",
  "state": {
    "access_token": "at",
    "expires_at": "2026-08-21T00:00:00Z",
    "refresh_token": "rt",
    "profile": { "id": "00000000000040008000000000000000", "name": "Steve", "skins": [], "capes": [] }
  }
}"#;

const YGG: &str = r#"{
  "type": "yggdrasil",
  "server": "https://example.com/api/yggdrasil",
  "username": "a@b.c",
  "selected": "00000000-0000-4000-8000-000000000000",
  "skin_domains": ["example.com"],
  "signature_publickey": "pk",
  "meta_info": { "serverName": null, "implementationName": null, "implementationVersion": null, "links": null },
  "state": {
    "access_token": "at",
    "client_token": "ct",
    "available_profiles": [{ "id": "00000000000040008000000000000000", "name": "Steve", "properties": null }],
    "selected_profile": { "id": "00000000000040008000000000000000", "name": "Steve", "properties": null },
    "user": { "id": "00000000000040008000000000000000", "name": null, "properties": null }
  }
}"#;

const OFFLINE: &str =
    r#"{ "type": "offline", "nickname": "Steve", "uuid": "00000000-0000-4000-8000-000000000000" }"#;

fn roundtrip(src: &str) {
    smol::block_on(async {
        let account = serde_json::from_str::<Account>(src).expect("deserialize");
        let out = serde_json::to_value(account.serialize().await).expect("serialize");
        let expected = serde_json::from_str::<serde_json::Value>(src).expect("parse");
        assert_eq!(out, expected);
    })
}

#[test]
fn microsoft_roundtrip() {
    roundtrip(MSA)
}

#[test]
fn yggdrasil_roundtrip() {
    roundtrip(YGG)
}

#[test]
fn offline_roundtrip() {
    roundtrip(OFFLINE)
}
