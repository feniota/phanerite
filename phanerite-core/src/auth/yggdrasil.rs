use crate::download::Downloader;
use crate::download::authlib_injector::AuthlibInjector;
use crate::error::{Error, Result};
use crate::instance::arguments::LaunchArguments;
use crate::instance::variables::Variables;
use crate::storage::Storage;
use crate::utils::state::NotReady;
use crate::utils::uuid::UnhyphenatedUuid;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct YggdrasilError {
    pub error: String,
    pub error_message: String,
    pub cause: Option<String>,
}

impl Display for YggdrasilError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl std::error::Error for YggdrasilError {}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Success(T),
    Error(YggdrasilError),
}

impl<T> Response<T> {
    fn into_result(self) -> Result<T> {
        match self {
            Response::Success(v) => Ok(v),
            Response::Error(e) => Err(e.into()),
        }
    }
}

/// Yggdrasil 登录
#[derive(Serialize, Deserialize)]
pub struct Authentication {
    #[serde(with = "crate::utils::secret")]
    access_token: SecretString,
    #[serde(with = "crate::utils::secret")]
    client_token: SecretString,

    /// 可用的玩家档案
    pub available_profiles: Vec<GameProfile>,
    /// 当前选择的玩家档案
    pub selected_profile: Option<GameProfile>,
    /// 用户档案
    pub user: GameProfile,

    /// 邮箱
    pub username: String,

    /// 服务器 base URL
    pub server: Url,
    /// 材质域名白名单
    pub skin_domains: Vec<String>,
    /// 验证角色属性的数字签名公钥
    pub signature_publickey: String,
    /// 服务器元信息
    pub meta_info: MetaInfo,

    /// Storage 局部的路径，反序列化后需要 `set_injector()` 重新解析
    #[serde(skip)]
    authlib_injector: Option<PathBuf>,
}

pub struct Login<'a, S, U, P, D: Downloader> {
    downloader: &'a D,
    authlib_injector: Option<PathBuf>,

    // 登录服务器
    server: S,
    skin_domains: Vec<String>,
    signature_publickey: Option<String>,
    meta_info: MetaInfo,

    // 用户凭据
    username: U,
    password: P,
}

impl<'a> Authentication {
    /// 创建登录会话
    pub fn new_login<D: Downloader>(
        downloader: &'a D,
    ) -> Login<'a, NotReady, NotReady, NotReady, D> {
        Login {
            downloader,
            authlib_injector: None,
            server: NotReady,
            skin_domains: vec![],
            signature_publickey: None,
            meta_info: MetaInfo {
                server_name: None,
                implementation_name: None,
                implementation_version: None,
                links: None,
            },
            username: NotReady,
            password: NotReady,
        }
    }
    /// 刷新令牌
    pub async fn refresh(
        &mut self,
        update_user: bool,
        downloader: &impl Downloader,
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

        let mut url = self.server.clone();
        url.path_segments_mut()
            .map_err(|_| Error::other("cannot-be-a-base URL"))?
            .pop_if_empty()
            .extend(&["authserver", "refresh"]);
        let res = downloader.post_json(url, req).await?.into_body();
        let res = serde_json::from_slice::<Response<ResponseRefresh>>(&res)?.into_result()?;

        self.access_token = res.access_token;
        self.client_token = res.client_token;
        self.selected_profile = res.selected_profile;
        if let Some(user) = res.user {
            self.user = user
        }

        Ok(())
    }
    /// 检验令牌
    pub async fn validate(&self, downloader: &impl Downloader) -> Result<bool> {
        #[derive(Serialize)]
        struct RequestValidate<'a> {
            access_token: &'a str,
            client_token: &'a str,
        }

        let req = RequestValidate {
            access_token: self.access_token.expose_secret(),
            client_token: self.client_token.expose_secret(),
        };
        let req = serde_json::to_string(&req)?;

        let mut url = self.server.clone();
        url.path_segments_mut()
            .map_err(|_| Error::other("cannot-be-a-base URL"))?
            .pop_if_empty()
            .extend(&["authserver", "validate"]);
        let status = downloader.post_json(url, req).await?.status();

        if status == 204 { Ok(true) } else { Ok(false) }
    }
    /// 吊销令牌
    pub async fn invalidate(&self, downloader: &impl Downloader) -> Result<()> {
        #[derive(Serialize)]
        struct RequestInvalidate<'a> {
            access_token: &'a str,
            client_token: &'a str,
        }

        let req = RequestInvalidate {
            access_token: self.access_token.expose_secret(),
            client_token: self.client_token.expose_secret(),
        };
        let req = serde_json::to_string(&req)?;

        let mut url = self.server.clone();
        url.path_segments_mut()
            .map_err(|_| Error::other("cannot-be-a-base URL"))?
            .pop_if_empty()
            .extend(&["authserver", "invalidate"]);
        downloader.post_json(url, req).await?;

        Ok(())
    }
    /// 退出登录
    pub async fn signout(
        &self,
        password: &SecretString,
        downloader: &impl Downloader,
    ) -> Result<()> {
        signout(self.server.clone(), &self.username, password, downloader).await
    }
    /// 重新登录，用于 `ready()` 无法续期时
    ///
    /// 需要用户重新输入密码，登录后会保持原有的角色，
    /// 原角色已不可用时返回错误
    pub async fn relogin(
        &mut self,
        password: &SecretString,
        downloader: &impl Downloader,
    ) -> Result<()> {
        let res = authenticate(
            self.server.clone(),
            &self.username,
            password,
            &self.client_token,
            downloader,
        )
        .await?;

        let current = self.selected_profile.as_ref().map(GameProfile::uuid);

        self.access_token = res.access_token;
        self.client_token = res.client_token;
        self.available_profiles = res.available_profiles;
        self.user = res.user;

        // 原本就没有选择角色，沿用服务器的选择
        let Some(current) = current else {
            self.selected_profile = res.selected_profile;
            return Ok(());
        };

        // 服务器已经绑定角色，必须与原角色一致
        if let Some(selected) = res.selected_profile {
            if selected.uuid() != current {
                return Err(Error::other("The original profile is no longer available"));
            }
            self.selected_profile = Some(selected);
            return Ok(());
        }

        // 未绑定角色时在刷新中重新选择原角色
        if !self.available_profiles.iter().any(|p| p.uuid() == current) {
            return Err(Error::other("The original profile is no longer available"));
        }
        self.refresh(true, downloader, |p| p.uuid() == current)
            .await
    }

    /// 重新解析 Authlib-Injector 的路径，反序列化后需要
    pub async fn set_injector(
        &mut self,
        storage: &Storage,
        downloader: &impl Downloader,
    ) -> Result<()> {
        let path = AuthlibInjector::new(storage)
            .get_or_init(downloader)
            .await?
            .clone();
        self.authlib_injector = Some(path);
        Ok(())
    }

    /// 配置预获取
    pub fn meta_base64(&self) -> String {
        let meta = FullMeta {
            meta: self.meta_info.clone(),
            skin_domains: self.skin_domains.clone(),
            signature_publickey: self.signature_publickey.clone(),
        };
        let meta = serde_json::to_vec(&meta).expect("serializing Value to JSON should never fail");
        BASE64_STANDARD.encode(&meta)
    }
}

/// Yggdrasil 登录由服务器、用户名与角色确定
///
/// 同一账户下的不同角色是相互独立的
impl PartialEq for Authentication {
    fn eq(&self, other: &Self) -> bool {
        self.server == other.server
            && self.username == other.username
            && self.selected_profile.as_ref().map(GameProfile::uuid)
                == other.selected_profile.as_ref().map(GameProfile::uuid)
    }
}

impl Eq for Authentication {}

impl<'a, S, U, P, D: Downloader> Login<'a, S, U, P, D> {
    pub async fn inject(mut self, storage: &Storage) -> Result<Self> {
        let path = AuthlibInjector::new(storage)
            .get_or_init(self.downloader)
            .await?
            .clone();
        self.authlib_injector = Some(path);
        Ok(self)
    }
}

impl<'a, U, P, D: Downloader> Login<'a, NotReady, U, P, D> {
    /// 自定义的验证服务器地址
    pub async fn custom(mut self, url: impl Into<Url>) -> Result<Login<'a, Url, U, P, D>> {
        let url = self.get_ali(url.into()).await;
        self.update_meta(&url).await?;
        Ok(Login {
            downloader: self.downloader,
            authlib_injector: self.authlib_injector,
            server: url,
            skin_domains: self.skin_domains,
            signature_publickey: self.signature_publickey,
            meta_info: self.meta_info,
            username: self.username,
            password: self.password,
        })
    }
    /// API Location Indication
    async fn get_ali(&self, url: Url) -> Url {
        let response = match self.downloader.head(url.clone()).await {
            Ok(v) => v,
            Err(_) => return url,
        };
        response
            .headers()
            .get("X-Authlib-Injector-API-Location")
            .and_then(|t| t.to_str().ok())
            .and_then(|t| t.parse().ok())
            .unwrap_or(url)
    }
    /// 必须执行的操作，否则会 `unwrap()`
    async fn update_meta(&mut self, url: &Url) -> Result<()> {
        let res = self.downloader.fetch(url.clone(), None).await?;
        let res = serde_json::from_slice::<Response<FullMeta>>(&res)?.into_result()?;

        self.skin_domains = res.skin_domains;
        self.signature_publickey = Some(res.signature_publickey);
        self.meta_info = res.meta;

        Ok(())
    }
}

impl<'a, S, P, D: Downloader> Login<'a, S, NotReady, P, D> {
    pub fn username(self, username: impl Into<String>) -> Login<'a, S, String, P, D> {
        Login {
            downloader: self.downloader,
            authlib_injector: self.authlib_injector,
            server: self.server,
            skin_domains: self.skin_domains,
            signature_publickey: self.signature_publickey,
            meta_info: self.meta_info,
            username: username.into(),
            password: self.password,
        }
    }
}

impl<'a, S, U, D: Downloader> Login<'a, S, U, NotReady, D> {
    pub fn password(self, password: impl Into<String>) -> Login<'a, S, U, SecretString, D> {
        Login {
            downloader: self.downloader,
            authlib_injector: self.authlib_injector,
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullMeta {
    meta: MetaInfo,
    skin_domains: Vec<String>,
    signature_publickey: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaInfo {
    server_name: Option<String>,
    implementation_name: Option<String>,
    implementation_version: Option<String>,
    links: Option<LinksInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinksInfo {
    homepage: Option<String>,
    register: Option<String>,
}

// ———————————————————— 用户/玩家资料 ————————————————————

#[derive(Serialize, Deserialize)]
pub struct GameProfile {
    pub id: UnhyphenatedUuid,
    pub name: Option<String>,
    pub properties: Option<Vec<ProfileProperty>>,
}

impl GameProfile {
    /// 角色的 UUID
    pub fn uuid(&self) -> Uuid {
        self.id.clone().into()
    }
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

impl<'a, D: Downloader> Login<'a, Url, String, SecretString, D> {
    /// 完成登录
    pub async fn login(self) -> Result<Authentication> {
        let client_token = SecretString::from(Uuid::now_v7().simple().to_string());
        let res = authenticate(
            self.server.clone(),
            &self.username,
            &self.password,
            &client_token,
            self.downloader,
        )
        .await?;

        Ok(Authentication {
            access_token: res.access_token,
            client_token: res.client_token,
            available_profiles: res.available_profiles,
            selected_profile: res.selected_profile,
            user: res.user,
            username: self.username,
            server: self.server,
            skin_domains: self.skin_domains,
            signature_publickey: self
                .signature_publickey
                .expect("Unreachable code: Server meta information does not exist"),
            meta_info: self.meta_info,
            authlib_injector: self.authlib_injector,
        })
    }
    pub async fn signout(&self) -> Result<()> {
        signout(
            self.server.clone(),
            &self.username,
            &self.password,
            self.downloader,
        )
        .await
    }
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

async fn authenticate(
    mut server: Url,
    username: &str,
    password: &SecretString,
    client_token: &SecretString,
    downloader: &impl Downloader,
) -> Result<ResponseLogin> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RequestLogin<'a> {
        username: &'a str,
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

    let req = RequestLogin {
        username,
        password: password.expose_secret(),
        client_token: client_token.expose_secret(),
        request_user: true,
        agent: LoginAgent {
            name: "Minecraft",
            version: 1,
        },
    };
    let req = serde_json::to_string(&req)?;

    server
        .path_segments_mut()
        .map_err(|_| Error::other("cannot-be-a-base URL"))?
        .pop_if_empty()
        .extend(&["authserver", "authenticate"]);
    let res = downloader.post_json(server, &req).await?.into_body();

    serde_json::from_slice::<Response<ResponseLogin>>(&res)?.into_result()
}

async fn signout(
    mut server: Url,
    username: &str,
    password: &SecretString,
    downloader: &impl Downloader,
) -> Result<()> {
    #[derive(Serialize)]
    struct RequestSignout<'a> {
        username: &'a str,
        password: &'a str,
    }

    let req = RequestSignout {
        username,
        password: password.expose_secret(),
    };
    let req = serde_json::to_string(&req)?;

    server
        .path_segments_mut()
        .map_err(|_| Error::other("cannot-be-a-base URL"))?
        .pop_if_empty()
        .extend(&["authserver", "signout"]);
    let res = downloader.post_json(server, req).await?;

    if res.status() == 204 {
        Ok(())
    } else {
        let err = serde_json::from_slice::<YggdrasilError>(res.body())?;
        Err(err.into())
    }
}

impl super::Authentication for Authentication {
    async fn vars(&self) -> Result<Variables<NotReady>> {
        let Some(profile) = &self.selected_profile else {
            return Err(Error::other("No selected profile"));
        };
        let variables = Variables::new()
            .required(
                profile.name.clone().unwrap_or_default(),
                profile.id.to_string(),
                self.access_token.expose_secret(),
            )
            .legacy(self.access_token.expose_secret(), "mojang");
        Ok(variables)
    }
    fn inject(&self) -> impl AsyncFnOnce(&mut LaunchArguments) {
        async |args| {
            if let Some(i) = &self.authlib_injector {
                let agent = format!("-javaagent:{}={}", i.to_string_lossy(), self.server,);
                let meta = format!(
                    "-Dauthlibinjector.yggdrasil.prefetched={}",
                    self.meta_base64()
                );
                args.jvm.insert(agent, None);
                args.jvm.insert(meta, None);
            }
        }
    }
    /// Yggdrasil 的令牌没有明确的有效期，只能先校验再续期
    ///
    /// 返回错误时需要用户重新输入密码，并调用 [`Authentication::relogin()`]
    async fn ready(&mut self, downloader: &impl Downloader) -> Result<()> {
        if self.validate(downloader).await? {
            return Ok(());
        }
        // 令牌已经绑定角色，续期时不能再指定
        self.refresh(true, downloader, |_| false).await
    }
}
