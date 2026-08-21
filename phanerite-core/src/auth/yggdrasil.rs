use crate::download::Downloader;
use crate::download::authlib_injector::AuthlibInjector;
use crate::error::{Error, Result};
use crate::instance::arguments::LaunchArguments;
use crate::instance::variables::Variables;
use crate::storage::Storage;
use crate::utils::secret::copy_secret;
use crate::utils::state::NotReady;
use crate::utils::uuid::UnhyphenatedUuid;
use async_lock::{OnceCell, RwLock, RwLockUpgradableReadGuard};
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
#[derive(Deserialize)]
pub struct Authentication {
    /// 服务器 base URL
    pub server: Url,
    /// 邮箱
    pub username: String,
    /// 当前角色的 UUID，账户的身份，登录后不再变化
    ///
    /// 与 `State::selected_profile` 的 `id` 一致，
    /// 后者的名称与属性会随续期更新
    selected: Option<Uuid>,

    /// 材质域名白名单
    pub skin_domains: Vec<String>,
    /// 验证角色属性的数字签名公钥
    pub signature_publickey: String,
    /// 服务器元信息
    pub meta_info: MetaInfo,

    /// 会随续期变化的部分
    #[serde(with = "crate::utils::lock")]
    state: RwLock<State>,

    /// Storage 局部的路径，反序列化后需要 `set_injector()` 重新解析
    #[serde(skip)]
    authlib_injector: OnceCell<PathBuf>,
}

/// [`Authentication`] 的快照，与 [`Authentication`] 的序列化格式一致
///
/// 不变的部分借用自 [`Authentication`]，只复制锁内的状态
#[derive(Serialize)]
pub struct Data<'a> {
    server: &'a Url,
    username: &'a str,
    selected: Option<Uuid>,
    skin_domains: &'a [String],
    signature_publickey: &'a str,
    meta_info: &'a MetaInfo,
    state: State,
}

/// 会随续期变化的会话状态
#[derive(Serialize, Deserialize)]
struct State {
    #[serde(with = "crate::utils::secret")]
    access_token: SecretString,
    #[serde(with = "crate::utils::secret")]
    client_token: SecretString,

    /// 可用的玩家档案
    available_profiles: Vec<GameProfile>,
    /// 当前选择的玩家档案
    selected_profile: Option<GameProfile>,
    /// 用户档案
    user: GameProfile,
}

impl State {
    /// 复制一份用于序列化
    ///
    /// `SecretString` 没有 `Clone`，这里明确地复制明文；
    /// 序列化本身就要写出明文，多这一份短暂的副本不会额外暴露什么
    fn snapshot(&self) -> Self {
        Self {
            access_token: copy_secret(&self.access_token),
            client_token: copy_secret(&self.client_token),
            available_profiles: self.available_profiles.clone(),
            selected_profile: self.selected_profile.clone(),
            user: self.user.clone(),
        }
    }
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

    /// 取一份可以持久化的快照
    pub async fn snapshot(&self) -> Data<'_> {
        Data {
            server: &self.server,
            username: &self.username,
            selected: self.selected,
            skin_domains: &self.skin_domains,
            signature_publickey: &self.signature_publickey,
            meta_info: &self.meta_info,
            state: self.state.read().await.snapshot(),
        }
    }
    /// 当前角色的 UUID，账户的身份
    pub fn selected(&self) -> Option<Uuid> {
        self.selected
    }
    /// 当前选择的玩家档案的副本
    pub async fn selected_profile(&self) -> Option<GameProfile> {
        self.state.read().await.selected_profile.clone()
    }
    /// 可用玩家档案的副本
    pub async fn available_profiles(&self) -> Vec<GameProfile> {
        self.state.read().await.available_profiles.clone()
    }
    /// 用户档案的副本
    pub async fn user(&self) -> GameProfile {
        self.state.read().await.user.clone()
    }
    /// 在回调中读取访问令牌
    pub async fn with_access_token<R>(&self, f: impl FnOnce(&str) -> R) -> R {
        f(self.state.read().await.access_token.expose_secret())
    }

    /// 刷新令牌
    ///
    /// 令牌已经绑定角色，续期时不能再指定，
    /// 换角色参见 [`Authentication::select()`]
    pub async fn refresh(&self, update_user: bool, downloader: &impl Downloader) -> Result<()> {
        self.renew(self.state.upgradable_read().await, update_user, downloader)
            .await
    }
    /// 在续期临界区内完成刷新并提交
    ///
    /// 网络交互期间只持有可升级读锁，不阻塞读者；
    /// 中途返回错误时本地状态原封不动
    async fn renew(
        &self,
        guard: RwLockUpgradableReadGuard<'_, State>,
        update_user: bool,
        downloader: &impl Downloader,
    ) -> Result<()> {
        let res = refresh_request(
            &self.server,
            &guard.access_token,
            &guard.client_token,
            update_user,
            None,
            downloader,
        )
        .await?;
        // 续期不应该改变账户绑定的角色
        if res.selected_profile.as_ref().map(GameProfile::uuid) != self.selected {
            return Err(Error::other("Refreshed into a different profile"));
        }

        // 网络交互已经结束，提交期间不再 await
        let mut state = RwLockUpgradableReadGuard::upgrade(guard).await;
        state.access_token = res.access_token;
        state.client_token = res.client_token;
        state.selected_profile = res.selected_profile;
        if let Some(user) = res.user {
            state.user = user
        }

        Ok(())
    }
    /// 为尚未绑定角色的令牌选择角色
    ///
    /// Yggdrasil 的令牌一旦绑定角色就不能改选，因此这里返回一个绑定了
    /// 该角色的新账户；原账户的令牌已经被服务端作废，调用方应当把它
    /// 从容器中移除，再放入返回的账户。
    ///
    /// 已经绑定角色时返回错误，换角色需要重新登录
    pub async fn select(&self, profile: Uuid, downloader: &impl Downloader) -> Result<Self> {
        if self.selected.is_some() {
            return Err(Error::other("The token is already bound to a profile"));
        }
        let guard = self.state.upgradable_read().await;
        let Some(selected) = guard
            .available_profiles
            .iter()
            .find(|p| p.uuid() == profile)
        else {
            return Err(Error::other("The requested profile is not available"));
        };
        let res = refresh_request(
            &self.server,
            &guard.access_token,
            &guard.client_token,
            true,
            Some(selected),
            downloader,
        )
        .await?;

        Ok(Authentication {
            server: self.server.clone(),
            username: self.username.clone(),
            selected: res.selected_profile.as_ref().map(GameProfile::uuid),
            skin_domains: self.skin_domains.clone(),
            signature_publickey: self.signature_publickey.clone(),
            meta_info: self.meta_info.clone(),
            state: RwLock::new(State {
                access_token: res.access_token,
                client_token: res.client_token,
                available_profiles: guard.available_profiles.clone(),
                selected_profile: res.selected_profile,
                user: res.user.unwrap_or_else(|| guard.user.clone()),
            }),
            authlib_injector: self
                .authlib_injector
                .get()
                .cloned()
                .map(OnceCell::from)
                .unwrap_or_default(),
        })
    }
    /// 检验令牌
    pub async fn validate(&self, downloader: &impl Downloader) -> Result<bool> {
        // 查询的是服务端的令牌状态，不与续期并发
        let guard = self.state.upgradable_read().await;
        validate_request(
            &self.server,
            &guard.access_token,
            &guard.client_token,
            downloader,
        )
        .await
    }
    /// 吊销令牌
    ///
    /// 调用后本账户的令牌即失效，应当立即把它从容器中移除
    pub async fn invalidate(&self, downloader: &impl Downloader) -> Result<()> {
        // 会作废令牌，不能与续期并发
        let guard = self.state.upgradable_read().await;
        invalidate_request(
            &self.server,
            &guard.access_token,
            &guard.client_token,
            downloader,
        )
        .await
    }
    /// 退出登录
    ///
    /// 会作废该账户在服务器上的所有令牌，
    /// 应当立即把相关账户从容器中移除
    pub async fn signout(
        &self,
        password: &SecretString,
        downloader: &impl Downloader,
    ) -> Result<()> {
        let _guard = self.state.upgradable_read().await;
        signout(self.server.clone(), &self.username, password, downloader).await
    }
    /// 重新登录，用于 `ready()` 无法续期时
    ///
    /// 需要用户重新输入密码，登录后会保持原有的角色，
    /// 原角色已不可用时返回错误
    pub async fn relogin(
        &self,
        password: &SecretString,
        downloader: &impl Downloader,
    ) -> Result<()> {
        let guard = self.state.upgradable_read().await;
        let mut res = authenticate(
            self.server.clone(),
            &self.username,
            password,
            &guard.client_token,
            downloader,
        )
        .await?;
        // 服务端已经签发新令牌并作废旧的，
        // 无论角色能否恢复，本地都必须反映这个事实
        let outcome = self.rebind(&mut res, downloader).await;

        // 网络交互已经结束，提交期间不再 await
        let mut state = RwLockUpgradableReadGuard::upgrade(guard).await;
        state.access_token = res.access_token;
        state.client_token = res.client_token;
        state.available_profiles = res.available_profiles;
        state.selected_profile = res.selected_profile;
        state.user = res.user;

        outcome
    }
    /// 重新登录后恢复原有的角色绑定，就地更新 `res`
    async fn rebind(&self, res: &mut ResponseLogin, downloader: &impl Downloader) -> Result<()> {
        let selected = res.selected_profile.as_ref().map(GameProfile::uuid);

        // 原本就没有绑定角色，服务器也不应该替我们绑定
        let Some(current) = self.selected else {
            return match selected {
                None => Ok(()),
                Some(_) => Err(Error::other(
                    "The server bound a profile on its own, add the account again",
                )),
            };
        };

        // 服务器已经绑定角色，必须与原角色一致
        if let Some(selected) = selected {
            return if selected == current {
                Ok(())
            } else {
                Err(Error::other("The original profile is no longer available"))
            };
        }

        // 未绑定角色时在刷新中重新选择原角色
        let Some(profile) = res.available_profiles.iter().find(|p| p.uuid() == current) else {
            return Err(Error::other("The original profile is no longer available"));
        };
        let refreshed = refresh_request(
            &self.server,
            &res.access_token,
            &res.client_token,
            true,
            Some(profile),
            downloader,
        )
        .await?;

        res.access_token = refreshed.access_token;
        res.client_token = refreshed.client_token;
        res.selected_profile = refreshed.selected_profile;
        if let Some(user) = refreshed.user {
            res.user = user
        }
        Ok(())
    }

    /// 重新解析 Authlib-Injector 的路径，反序列化后需要
    pub async fn set_injector(
        &self,
        storage: &Storage,
        downloader: &impl Downloader,
    ) -> Result<()> {
        self.authlib_injector
            .get_or_try_init(|| async {
                AuthlibInjector::new(storage)
                    .get_or_init(downloader)
                    .await
                    .cloned()
            })
            .await?;
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

/// 续期请求，`select` 只在令牌尚未绑定角色时可以指定
async fn refresh_request(
    server: &Url,
    access_token: &SecretString,
    client_token: &SecretString,
    request_user: bool,
    select: Option<&GameProfile>,
    downloader: &impl Downloader,
) -> Result<ResponseRefresh> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RequestRefresh<'a> {
        access_token: &'a str,
        client_token: &'a str,
        request_user: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        selected_profile: Option<&'a GameProfile>,
    }

    let req = RequestRefresh {
        access_token: access_token.expose_secret(),
        client_token: client_token.expose_secret(),
        request_user,
        selected_profile: select,
    };
    let req = serde_json::to_string(&req)?;

    let res = downloader
        .post_json(endpoint(server, "refresh")?, req)
        .await?
        .into_body();

    serde_json::from_slice::<Response<ResponseRefresh>>(&res)?.into_result()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseRefresh {
    access_token: SecretString,
    client_token: SecretString,
    selected_profile: Option<GameProfile>,
    user: Option<GameProfile>,
}

/// 检验令牌，仍然有效时返回 `true`
async fn validate_request(
    server: &Url,
    access_token: &SecretString,
    client_token: &SecretString,
    downloader: &impl Downloader,
) -> Result<bool> {
    let req = serde_json::to_string(&RequestToken {
        access_token: access_token.expose_secret(),
        client_token: client_token.expose_secret(),
    })?;

    let status = downloader
        .post_json(endpoint(server, "validate")?, req)
        .await?
        .status();

    Ok(status == 204)
}

/// 吊销令牌
async fn invalidate_request(
    server: &Url,
    access_token: &SecretString,
    client_token: &SecretString,
    downloader: &impl Downloader,
) -> Result<()> {
    let req = serde_json::to_string(&RequestToken {
        access_token: access_token.expose_secret(),
        client_token: client_token.expose_secret(),
    })?;
    downloader
        .post_json(endpoint(server, "invalidate")?, req)
        .await?;

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestToken<'a> {
    access_token: &'a str,
    client_token: &'a str,
}

/// 拼接 `authserver` 下的端点地址
fn endpoint(server: &Url, path: &str) -> Result<Url> {
    let mut url = server.clone();
    url.path_segments_mut()
        .map_err(|_| Error::other("cannot-be-a-base URL"))?
        .pop_if_empty()
        .extend(&["authserver", path]);
    Ok(url)
}

/// Yggdrasil 登录由服务器、用户名与角色确定
///
/// 同一账户下的不同角色是相互独立的
impl PartialEq for Authentication {
    fn eq(&self, other: &Self) -> bool {
        self.server == other.server
            && self.username == other.username
            && self.selected == other.selected
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

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
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
            server: self.server,
            username: self.username,
            selected: res.selected_profile.as_ref().map(GameProfile::uuid),
            skin_domains: self.skin_domains,
            signature_publickey: self
                .signature_publickey
                .expect("Unreachable code: Server meta information does not exist"),
            meta_info: self.meta_info,
            state: RwLock::new(State {
                access_token: res.access_token,
                client_token: res.client_token,
                available_profiles: res.available_profiles,
                selected_profile: res.selected_profile,
                user: res.user,
            }),
            authlib_injector: self
                .authlib_injector
                .map(OnceCell::from)
                .unwrap_or_default(),
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
        let state = self.state.read().await;
        let Some(profile) = &state.selected_profile else {
            return Err(Error::other("No selected profile"));
        };
        let variables = Variables::new()
            .required(
                profile.name.clone().unwrap_or_default(),
                profile.id.to_string(),
                state.access_token.expose_secret(),
            )
            .legacy(state.access_token.expose_secret(), "mojang");
        Ok(variables)
    }
    async fn serialize(&self) -> impl Serialize {
        self.snapshot().await
    }
    fn inject(&self) -> impl AsyncFnOnce(&mut LaunchArguments) {
        async |args| {
            if let Some(i) = self.authlib_injector.get() {
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
    async fn ready(&self, downloader: &impl Downloader) -> Result<()> {
        // 校验和续期是一个整体，中途被其他续期插入会让校验结果失效，
        // 因此在同一个临界区里完成，期间不再调用其他 `&self` 方法
        let guard = self.state.upgradable_read().await;
        let valid = validate_request(
            &self.server,
            &guard.access_token,
            &guard.client_token,
            downloader,
        )
        .await?;
        if valid {
            return Ok(());
        }
        self.renew(guard, true, downloader).await
    }
}
