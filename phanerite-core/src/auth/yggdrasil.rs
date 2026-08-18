use crate::download::Downloader;
use crate::download::authlib_injector::AuthlibInjector;
use crate::error::{Error, Result};
use crate::instance::Instance;
use crate::instance::arguments::LaunchArguments;
use crate::instance::variables::Variables;
use crate::utils::uuid::UnhyphenatedUuid;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;
use uuid::Uuid;

pub struct Missing;

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
pub struct Authentication<'a> {
    access_token: SecretString,
    client_token: SecretString,

    /// 可用的玩家档案
    pub available_profiles: Vec<GameProfile>,
    /// 当前选择的玩家档案
    pub selected_profile: Option<GameProfile>,
    /// 用户档案
    pub user: GameProfile,

    /// 邮箱
    pub username: String,
    /// 密码，不应该被持久化
    password: SecretString,

    /// 服务器 base URL
    pub server: Url,
    /// 材质域名白名单
    pub skin_domains: Vec<String>,
    /// 验证角色属性的数字签名公钥
    pub signature_publickey: String,
    /// 服务器元信息
    pub meta_info: MetaInfo,

    authlib_injector: Option<&'a AuthlibInjector<'a>>,
}

pub struct Login<'a, S, U, P, D: Downloader> {
    downloader: &'a D,
    authlib_injector: Option<&'a AuthlibInjector<'a>>,

    // 登录服务器
    server: S,
    skin_domains: Vec<String>,
    signature_publickey: Option<String>,
    meta_info: MetaInfo,

    // 用户凭据
    username: U,
    password: P,
}

impl<'a> Authentication<'a> {
    /// 创建登录会话
    pub fn new_login<D: Downloader>(downloader: &'a D) -> Login<'a, Missing, Missing, Missing, D> {
        Login {
            downloader,
            authlib_injector: None,
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
        let (_, res) = downloader.post_json(url, req).await?;
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
        let (status, _) = downloader.post_json(url, req).await?;

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
        let (_, _) = downloader.post_json(url, req).await?;

        Ok(())
    }
    /// 退出登录
    pub async fn signout(&self, downloader: &impl Downloader) -> Result<()> {
        #[derive(Serialize)]
        struct RequestSignout<'a> {
            username: &'a str,
            password: &'a str,
        }

        let req = RequestSignout {
            username: &self.username,
            password: self.password.expose_secret(),
        };
        let req = serde_json::to_string(&req)?;

        let mut url = self.server.clone();
        url.path_segments_mut()
            .map_err(|_| Error::other("cannot-be-a-base URL"))?
            .pop_if_empty()
            .extend(&["authserver", "signout"]);
        let (status, err) = downloader.post_json(url, req).await?;

        if status == 204 {
            Ok(())
        } else {
            let err = serde_json::from_slice::<YggdrasilError>(&err)?;
            Err(err.into())
        }
    }

    /// 配置预获取
    pub fn meta_base64(&self) -> Result<String> {
        let meta = FullMeta {
            meta: self.meta_info.clone(),
            skin_domains: self.skin_domains.clone(),
            signature_publickey: self.signature_publickey.clone(),
        };
        let meta = serde_json::to_vec(&meta)?;
        let encoded = BASE64_STANDARD.encode(&meta);
        Ok(encoded)
    }
    /// 生成启动参数
    fn args<R: Clone, C: Clone>(&self, instance: &Instance<R, C>) -> Result<LaunchArguments> {
        let Some(profile) = &self.selected_profile else {
            return Err(Error::other("No selected profile"));
        };

        let variables = Variables::new()
            .required(
                profile.name.clone().unwrap_or_default(),
                profile.id.to_string(),
                self.access_token.expose_secret(),
            )
            .legacy(self.access_token.expose_secret(), "mojang")
            .generated(instance)?;

        let arguments = variables.to_arguments(instance);
        Ok(arguments)
    }
    /// 生成启动参数（注入 `authlib-injector`）
    async fn injected_args<R: Clone, C: Clone>(
        &self,
        instance: &Instance<'_, R, C>,
        authlib_injector: &AuthlibInjector<'_>,
    ) -> Result<LaunchArguments> {
        let mut args = self.args(instance)?;
        let agent = format!(
            "-javaagent:{}={}",
            authlib_injector.get().await?.to_string_lossy(),
            self.server,
        );
        let meta = format!(
            "-Dauthlibinjector.yggdrasil.prefetched={}",
            self.meta_base64()?
        );
        args.jvm.insert(agent, None);
        args.jvm.insert(meta, None);
        Ok(args)
    }
}

impl super::Authentication for Authentication<'_> {
    async fn args<R: Clone, C: Clone>(
        &self,
        instance: &Instance<'_, R, C>,
    ) -> Result<LaunchArguments> {
        Ok(match self.authlib_injector {
            None => self.args(instance)?,
            Some(i) => self.injected_args(instance, i).await?,
        })
    }
}

impl<'a, S, U, P, D: Downloader> Login<'a, S, U, P, D> {
    pub fn inject(mut self, authlib_injector: &'a AuthlibInjector) -> Self {
        self.authlib_injector = Some(authlib_injector);
        self
    }
}

impl<'a, U, P, D: Downloader> Login<'a, Missing, U, P, D> {
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

impl<'a, S, P, D: Downloader> Login<'a, S, Missing, P, D> {
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

impl<'a, S, U, D: Downloader> Login<'a, S, U, Missing, D> {
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
    pub async fn login(self) -> Result<Authentication<'a>> {
        let client_token = SecretString::from(Uuid::now_v7().simple().to_string());

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
            username: &self.username,
            password: self.password.expose_secret(),
            client_token: client_token.expose_secret(),
            request_user: true,
            agent: LoginAgent {
                name: "Minecraft",
                version: 1,
            },
        };
        let req = serde_json::to_string(&req)?;

        let mut url = self.server.clone();
        url.path_segments_mut()
            .map_err(|_| Error::other("cannot-be-a-base URL"))?
            .pop_if_empty()
            .extend(&["authserver", "authenticate"]);
        let (_, res) = self.downloader.post_json(url, &req).await?;
        let res = serde_json::from_slice::<Response<ResponseLogin>>(&res)?.into_result()?;

        Ok(Authentication {
            access_token: res.access_token,
            client_token: res.client_token,
            available_profiles: res.available_profiles,
            selected_profile: res.selected_profile,
            user: res.user,
            username: self.username,
            password: self.password,
            server: self.server,
            skin_domains: self.skin_domains,
            signature_publickey: self
                .signature_publickey
                .expect("Unreachable code: Server meta information does not exist"),
            meta_info: self.meta_info,
            authlib_injector: self.authlib_injector,
        })
    }
}
