use crate::download::downloader::Downloader;
use crate::error::Result;
use crate::utils::uuid::UnhyphenatedUuid;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Authentication {
    access_token: SecretString,
    client_token: SecretString,

    /// 可用的玩家档案
    available_profiles: Vec<GameProfile>,
    /// 当前选择的玩家档案
    selected_profile: Option<GameProfile>,
    /// 用户档案
    user: GameProfile,

    /// 服务器 base URL
    server: String,
    /// 材质域名白名单
    skin_domains: Vec<String>,
    /// 验证角色属性的数字签名公钥
    signature_publickey: String,
    /// 服务器元信息
    meta_info: MetaInfo,
}

pub struct Login<'a, S, U, P> {
    downloader: &'a Downloader,

    // 登录服务器
    server: S,
    skin_domains: Vec<String>,
    signature_publickey: Option<String>,
    meta_info: MetaInfo,

    // 用户凭据
    username: U,
    password: P,
}

pub struct Missing;

impl Authentication {
    pub fn login(downloader: &Downloader) -> Login<'_, Missing, Missing, Missing> {
        Login {
            downloader,
            server: Missing,
            skin_domains: vec![],
            signature_publickey: None,
            meta_info: MetaInfo {
                server_name: None,
                implementation_name: None,
                implementation_version: None,
                links: None,
            },
            username: Missing,
            password: Missing,
        }
    }
    pub async fn refresh(
        &mut self,
        update_user: bool,
        downloader: &Downloader,
        select_profile: impl FnMut(&&GameProfile) -> bool,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RequestRefresh<'a> {
            access_token: &'a str,
            client_token: &'a str,
            request_user: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            selected_profile: Option<&'a GameProfile>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ResponseRefresh {
            access_token: SecretString,
            client_token: SecretString,
            selected_profile: Option<GameProfile>,
            user: Option<GameProfile>,
        }

        let selected = self.available_profiles.iter().find(select_profile);

        let req = RequestRefresh {
            access_token: self.access_token.expose_secret(),
            client_token: self.client_token.expose_secret(),
            request_user: update_user,
            selected_profile: selected,
        };
        let req = serde_json::to_string(&req)?;

        let res = downloader
            .post(format!("{}/authserver/refresh", self.server), req)
            .await?;
        let res = serde_json::from_slice::<ResponseRefresh>(&res)?;

        self.access_token = res.access_token;
        self.client_token = res.client_token;
        self.selected_profile = res.selected_profile;
        if let Some(user) = res.user {
            self.user = user
        }

        Ok(())
    }
}

impl<'a, U, P> Login<'a, Missing, U, P> {
    pub async fn official(self) -> Result<Login<'a, String, U, P>> {
        self.custom("https://authserver.mojang.com").await
    }
    pub async fn custom(mut self, url: impl AsRef<str>) -> Result<Login<'a, String, U, P>> {
        let url = url
            .as_ref()
            .strip_suffix('/')
            .unwrap_or(url.as_ref())
            .to_string();
        self.update_meta(&url).await?;
        Ok(Login {
            downloader: self.downloader,
            server: url,
            skin_domains: self.skin_domains,
            signature_publickey: self.signature_publickey,
            meta_info: self.meta_info,
            username: self.username,
            password: self.password,
        })
    }
    async fn update_meta(&mut self, url: impl AsRef<str>) -> Result<()> {
        let res = self.downloader.fetch(url, None).await?;
        let res = serde_json::from_slice::<ResponseMeta>(&res)?;

        self.skin_domains = res.skin_domains;
        self.signature_publickey = Some(res.signature_publickey);
        self.meta_info = res.meta;

        Ok(())
    }
}

impl<'a, S, P> Login<'a, S, Missing, P> {
    pub fn username(self, username: impl Into<String>) -> Login<'a, S, String, P> {
        Login {
            downloader: self.downloader,
            server: self.server,
            skin_domains: self.skin_domains,
            signature_publickey: self.signature_publickey,
            meta_info: self.meta_info,
            username: username.into(),
            password: self.password,
        }
    }
}

impl<'a, S, U> Login<'a, S, U, Missing> {
    pub fn password(self, password: impl Into<String>) -> Login<'a, S, U, SecretString> {
        Login {
            downloader: self.downloader,
            server: self.server,
            skin_domains: self.skin_domains,
            signature_publickey: self.signature_publickey,
            meta_info: self.meta_info,
            username: self.username,
            password: SecretString::from(password.into()),
        }
    }
}

// ———————————————————— 登录服务器元信息 ————————————————————

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    meta: MetaInfo,
    skin_domains: Vec<String>,
    signature_publickey: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetaInfo {
    server_name: Option<String>,
    implementation_name: Option<String>,
    implementation_version: Option<String>,
    links: Option<LinksInfo>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinksInfo {
    homepage: Option<String>,
    register: Option<String>,
}

// ———————————————————— 用户/玩家资料 ————————————————————

#[derive(Serialize, Deserialize)]
pub struct GameProfile {
    pub id: UnhyphenatedUuid,
    pub name: String,
    pub properties: Option<Vec<ProfileProperty>>,
}

#[derive(Serialize, Deserialize)]
pub struct ProfileProperty {
    /// The key of the property
    pub name: String,
    /// The value of the property
    pub value: String,
    /// The signature of the property
    pub signature: Option<String>,
}

impl<'a> Login<'a, String, String, SecretString> {
    pub async fn login(self) -> Result<Authentication> {
        let client_token = SecretString::from(Uuid::now_v7().simple().to_string());

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RequestLogin<'a> {
            username: String,
            password: &'a str,
            client_token: &'a str,
            request_user: bool,
            agent: LoginAgent,
        }
        #[derive(Serialize)]
        struct LoginAgent {
            name: &'static str,
            version: usize,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ResponseLogin {
            access_token: SecretString,
            client_token: SecretString,
            available_profiles: Vec<GameProfile>,
            selected_profile: Option<GameProfile>,
            user: GameProfile,
        }

        let req = RequestLogin {
            username: self.username,
            password: self.password.expose_secret(),
            client_token: client_token.expose_secret(),
            request_user: false,
            agent: LoginAgent {
                name: "Minecraft",
                version: 1,
            },
        };
        let req = serde_json::to_string(&req)?;

        let res = self
            .downloader
            .post(format!("{}/authserver/authenticate", self.server), &req)
            .await?;
        let res = serde_json::from_slice::<ResponseLogin>(&res)?;

        Ok(Authentication {
            access_token: res.access_token,
            client_token: res.client_token,
            available_profiles: res.available_profiles,
            selected_profile: res.selected_profile,
            user: res.user,
            server: self.server,
            skin_domains: self.skin_domains,
            signature_publickey: self
                .signature_publickey
                .expect("Unreachable code: Server meta information does not exist"),
            meta_info: self.meta_info,
        })
    }
}
