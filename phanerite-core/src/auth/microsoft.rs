use crate::download::Downloader;
use crate::error::{Error, Result};
use crate::instance::variables::Variables;
use crate::utils::state::NotReady;
use crate::utils::uuid::UnhyphenatedUuid;
use chrono::{DateTime, TimeDelta, Utc};
use http::{Request, Response};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use std::time::Duration;
use strum::IntoStaticStr;
use url::{Url, form_urlencoded};
use uuid::Uuid;

/// 微软身份平台
static AUTHORITY: LazyLock<Url> = LazyLock::new(|| endpoint("https://login.microsoftonline.com"));
/// Xbox Live 用户认证
static XBL_AUTHENTICATE: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://user.auth.xboxlive.com/user/authenticate"));
/// XSTS 授权
static XSTS_AUTHORIZE: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://xsts.auth.xboxlive.com/xsts/authorize"));
/// 使用 Xbox 身份登录 Minecraft
static MINECRAFT_LOGIN: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://api.minecraftservices.com/authentication/login_with_xbox"));
/// Minecraft 商店的所有权条目
static MINECRAFT_ENTITLEMENTS: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://api.minecraftservices.com/entitlements/mcstore"));
/// Minecraft 玩家档案
static MINECRAFT_PROFILE: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://api.minecraftservices.com/minecraft/profile"));

/// 设备码授权流程的 `grant_type`
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
/// 默认授权范围，`offline_access` 用于换取刷新令牌
const DEFAULT_SCOPE: &str = "XboxLive.signin offline_access";
/// Minecraft 对应的 XSTS 信赖方
const RELYING_PARTY: &str = "rp://api.minecraftservices.com/";
/// 服务端要求放慢轮询时增加的间隔
const SLOW_DOWN_STEP: Duration = Duration::from_secs(5);
/// 提前刷新访问令牌的余量，避免游戏启动过程中过期
const REFRESH_MARGIN: i64 = 5 * 60;
/// Minecraft: Java Edition 的所有权条目，Game Pass 只有后者
const JAVA_ENTITLEMENTS: [&str; 2] = ["product_minecraft", "game_minecraft"];

/// 解析写死的端点地址
fn endpoint(url: &str) -> Url {
    url.parse().expect("endpoint URL should always be valid")
}

/// 微软登录链路中的错误
#[derive(Debug, thiserror::Error)]
pub enum MicrosoftError {
    /// OAuth 2.0 端点返回的错误
    #[error("{0}")]
    OAuth(OAuthError),
    /// Xbox Live 或 XSTS 拒绝授权，通常是账户自身的问题
    #[error("{0}")]
    Xbox(XboxError),
    /// 用户在浏览器中拒绝了授权
    #[error("Authorization declined by the user")]
    Declined,
    /// 设备码在用户完成授权前过期
    #[error("Device code expired before authorization")]
    Expired,
    /// 账户没有 Minecraft: Java Edition 的所有权
    #[error("Account does not own Minecraft: Java Edition")]
    NotEntitled,
    /// 账户拥有游戏但尚未创建玩家档案
    #[error("Account has no Minecraft profile yet")]
    NoProfile,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OAuthError {
    pub error: String,
    pub error_description: Option<String>,
    pub correlation_id: Option<String>,
}

impl Display for OAuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.error_description {
            // 微软的描述里附带错误码和换行，只取第一行
            Some(d) => write!(f, "{}", d.lines().next().unwrap_or(&self.error)),
            None => write!(f, "{}", self.error),
        }
    }
}

impl std::error::Error for OAuthError {}

impl From<OAuthError> for Error {
    fn from(value: OAuthError) -> Self {
        Error::Microsoft(MicrosoftError::OAuth(value))
    }
}

/// Xbox Live 与 XSTS 返回的错误
#[derive(Deserialize, Debug)]
pub struct XboxError {
    /// Xbox 的错误码
    #[serde(rename = "XErr")]
    pub code: u64,
    #[serde(rename = "Message")]
    pub message: Option<String>,
    /// 用于解决问题的跳转地址
    #[serde(rename = "Redirect")]
    pub redirect: Option<String>,
}

impl XboxError {
    /// 已知错误码的说明，未知的错误码返回 `None`
    pub fn reason(&self) -> Option<&'static str> {
        Some(match self.code {
            2148916227 => "The account is banned from Xbox",
            2148916229 => "The account needs to be added to a family",
            2148916233 => "The account has no Xbox profile, create one first",
            2148916234 => "The account has not accepted the Xbox terms of service",
            2148916235 => "Xbox Live is not available in the account's country or region",
            2148916236 | 2148916237 => "The account needs adult verification",
            2148916238 => "The account is a child and must be added to a family",
            _ => return None,
        })
    }
}

impl Display for XboxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // 微软的 Message 往往为空字符串，此时只能依靠错误码
        let message = self.message.as_deref().filter(|m| !m.is_empty());
        match (self.reason(), message) {
            (Some(reason), _) => write!(f, "{reason}"),
            (None, Some(message)) => write!(f, "{message}"),
            (None, None) => write!(f, "Xbox declined the authorization: {}", self.code),
        }
    }
}

impl std::error::Error for XboxError {}

impl From<XboxError> for Error {
    fn from(value: XboxError) -> Self {
        Error::Microsoft(MicrosoftError::Xbox(value))
    }
}

/// OAuth 2.0 端点的响应，错误与成功共用同一个响应体
#[derive(Deserialize)]
#[serde(untagged)]
enum OAuthResponse<T> {
    Success(T),
    Error(OAuthError),
}

impl<T> OAuthResponse<T> {
    fn into_result(self) -> Result<T> {
        match self {
            OAuthResponse::Success(v) => Ok(v),
            OAuthResponse::Error(e) => Err(e.into()),
        }
    }
}

/// Xbox 端点的响应，错误与成功共用同一个响应体
#[derive(Deserialize)]
#[serde(untagged)]
enum XboxResponse<T> {
    Success(T),
    Error(XboxError),
}

impl<T> XboxResponse<T> {
    fn into_result(self) -> Result<T> {
        match self {
            XboxResponse::Success(v) => Ok(v),
            XboxResponse::Error(e) => Err(e.into()),
        }
    }
}

/// Minecraft 服务端点的错误响应
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceError {
    error: String,
    error_message: Option<String>,
}

/// 将 Minecraft 服务的失败响应转换为错误
fn service_error(res: &Response<Vec<u8>>) -> Error {
    match serde_json::from_slice::<ServiceError>(res.body()) {
        Ok(e) => Error::other(e.error_message.unwrap_or(e.error)),
        // 失败响应不一定带有响应体，此时只能依靠状态码
        Err(_) => Error::other(format!(
            "Minecraft services declined the request: {}",
            res.status()
        )),
    }
}

/// 构造表单编码的 POST 请求
fn post_form(url: &Url, pairs: &[(&str, &str)]) -> Request<Vec<u8>> {
    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    Request::post(url.as_str())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .body(body.into_bytes())
        .expect("building a request from a valid URL should never fail")
}

/// 构造携带 Bearer 令牌的 GET 请求
fn get_bearer(url: &Url, token: &str) -> Request<Vec<u8>> {
    Request::get(url.as_str())
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .body(Vec::new())
        .expect("building a request from a valid URL should never fail")
}

/// 将秒数转换为 `TimeDelta`，超出范围时归零
fn seconds(secs: i64) -> TimeDelta {
    TimeDelta::try_seconds(secs).unwrap_or_default()
}

/// 微软身份平台的租户，决定哪些账户可以登录
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, IntoStaticStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Tenant {
    /// 仅个人微软账户，Minecraft 登录的默认选择
    #[default]
    Consumers,
    /// 个人账户与工作/学校账户
    Common,
    /// 仅工作/学校账户
    Organizations,
}

impl Tenant {
    /// 构造该租户下的 OAuth 2.0 端点
    fn oauth(self, path: &str) -> Url {
        let mut url = AUTHORITY.clone();
        url.path_segments_mut()
            .expect("AUTHORITY should always be a base URL")
            .extend(&[<&str>::from(self), "oauth2", "v2.0", path]);
        url
    }
}

/// 微软账户的设备码登录
pub struct Login<'a, C, D: Downloader> {
    downloader: &'a D,

    /// Azure 应用注册的客户端 ID
    client_id: C,
    /// 可登录的账户类型
    tenant: Tenant,
    /// 授权范围
    scope: String,
}

impl<'a> Authentication {
    /// 创建登录会话
    pub fn new_login<D: Downloader>(downloader: &'a D) -> Login<'a, NotReady, D> {
        Login {
            downloader,
            client_id: NotReady,
            tenant: Tenant::default(),
            scope: DEFAULT_SCOPE.to_owned(),
        }
    }
}

impl<C, D: Downloader> Login<'_, C, D> {
    /// 可登录的账户类型
    pub fn tenant(mut self, tenant: Tenant) -> Self {
        self.tenant = tenant;
        self
    }
    /// 自定义授权范围，需要包含 `offline_access` 才能获得刷新令牌
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = scope.into();
        self
    }
}

impl<'a, D: Downloader> Login<'a, NotReady, D> {
    /// Azure 应用注册的客户端 ID，需要允许公共客户端流
    pub fn client_id(self, client_id: impl Into<String>) -> Login<'a, String, D> {
        Login {
            downloader: self.downloader,
            client_id: client_id.into(),
            tenant: self.tenant,
            scope: self.scope,
        }
    }
}

impl<'a, D: Downloader> Login<'a, String, D> {
    /// 申请设备码，之后需要用户在浏览器中完成授权
    pub async fn device_code(self) -> Result<Pending<'a, D>> {
        #[derive(Deserialize)]
        struct ResponseDeviceCode {
            device_code: SecretString,
            user_code: String,
            verification_uri: Url,
            expires_in: i64,
            interval: u64,
            message: String,
        }

        let req = post_form(
            &self.tenant.oauth("devicecode"),
            &[("client_id", &self.client_id), ("scope", &self.scope)],
        );
        let res = self.downloader.send(req).await?.into_body();
        let res =
            serde_json::from_slice::<OAuthResponse<ResponseDeviceCode>>(&res)?.into_result()?;

        Ok(Pending {
            downloader: self.downloader,
            client_id: self.client_id,
            tenant: self.tenant,
            scope: self.scope,
            device_code: res.device_code,
            user_code: res.user_code,
            verification_uri: res.verification_uri,
            message: res.message,
            interval: Duration::from_secs(res.interval),
            expires_at: Utc::now() + seconds(res.expires_in),
        })
    }
    /// 使用持久化的刷新令牌免交互登录
    pub async fn refresh(self, refresh_token: impl Into<String>) -> Result<Authentication> {
        let refresh_token = SecretString::from(refresh_token.into());
        let token = refresh_grant(
            self.downloader,
            self.tenant,
            &self.client_id,
            &self.scope,
            &refresh_token,
        )
        .await?;
        let session = authorize(self.downloader, &token.access_token).await?;

        Ok(Authentication {
            access_token: session.access_token,
            expires_at: session.expires_at,
            // 微软可能签发新的刷新令牌，旧的仍然可用
            refresh_token: Some(token.refresh_token.unwrap_or(refresh_token)),
            client_id: self.client_id,
            tenant: self.tenant,
            scope: self.scope,
            xuid: session.xuid,
            profile: session.profile,
        })
    }
}

/// 令牌端点签发的微软令牌
#[derive(Deserialize)]
struct Token {
    /// 微软访问令牌，只用于换取 Xbox 身份
    access_token: SecretString,
    /// 刷新令牌，仅在授权范围包含 `offline_access` 时签发
    refresh_token: Option<SecretString>,
}

/// 用刷新令牌换取新的微软令牌
async fn refresh_grant(
    downloader: &impl Downloader,
    tenant: Tenant,
    client_id: &str,
    scope: &str,
    refresh_token: &SecretString,
) -> Result<Token> {
    let req = post_form(
        &tenant.oauth("token"),
        &[
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.expose_secret()),
            ("scope", scope),
        ],
    );
    let res = downloader.send(req).await?.into_body();
    serde_json::from_slice::<OAuthResponse<Token>>(&res)?.into_result()
}

/// 等待用户完成授权的设备码会话
pub struct Pending<'a, D: Downloader> {
    downloader: &'a D,

    client_id: String,
    tenant: Tenant,
    scope: String,

    /// 轮询用的设备码
    device_code: SecretString,

    /// 需要展示给用户的用户码
    pub user_code: String,
    /// 需要用户打开的地址
    pub verification_uri: Url,
    /// 微软给出的完整提示语，已经包含用户码与地址
    pub message: String,

    /// 服务端建议的轮询间隔
    interval: Duration,
    /// 设备码的过期时刻
    expires_at: DateTime<Utc>,
}

impl<D: Downloader> Pending<'_, D> {
    /// 服务端建议的轮询间隔
    pub fn interval(&self) -> Duration {
        self.interval
    }
    /// 设备码的过期时刻
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
    /// 设备码是否已经过期
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
    /// 轮询一次授权状态，用户尚未完成授权时返回 `None`
    ///
    /// 轮询间隔不应该小于 `interval()`，否则服务端会要求放慢
    pub async fn poll(&mut self) -> Result<Option<Authentication>> {
        let req = post_form(
            &self.tenant.oauth("token"),
            &[
                ("client_id", &self.client_id),
                ("grant_type", DEVICE_CODE_GRANT),
                ("device_code", self.device_code.expose_secret()),
            ],
        );
        let res = self.downloader.send(req).await?.into_body();
        let token = match serde_json::from_slice::<OAuthResponse<Token>>(&res)? {
            OAuthResponse::Success(v) => v,
            // 用户尚未完成授权不是错误
            OAuthResponse::Error(e) => return self.keep_waiting(e).map(|_| None),
        };

        let session = authorize(self.downloader, &token.access_token).await?;

        Ok(Some(Authentication {
            access_token: session.access_token,
            expires_at: session.expires_at,
            refresh_token: token.refresh_token,
            client_id: self.client_id.clone(),
            tenant: self.tenant,
            scope: self.scope.clone(),
            xuid: session.xuid,
            profile: session.profile,
        }))
    }
    /// 按建议的间隔轮询直到授权完成，计时器由调用方提供
    ///
    /// 例如 `wait(async |d| { smol::Timer::after(d).await; })`
    pub async fn wait(&mut self, mut timer: impl AsyncFnMut(Duration)) -> Result<Authentication> {
        loop {
            timer(self.interval).await;
            if let Some(auth) = self.poll().await? {
                return Ok(auth);
            }
            // 服务端不一定会在过期后返回 `expired_token`
            if self.is_expired() {
                return Err(MicrosoftError::Expired.into());
            }
        }
    }
    /// 处理轮询中的错误，可以继续等待时返回 `Ok(())`
    fn keep_waiting(&mut self, error: OAuthError) -> Result<()> {
        match error.error.as_str() {
            "authorization_pending" => Ok(()),
            // 轮询过快，官方要求每次增加 5 秒
            "slow_down" => {
                self.interval += SLOW_DOWN_STEP;
                Ok(())
            }
            "authorization_declined" => Err(MicrosoftError::Declined.into()),
            "expired_token" => Err(MicrosoftError::Expired.into()),
            _ => Err(error.into()),
        }
    }
}

/// Xbox Live 与 XSTS 的成功响应
#[derive(Deserialize)]
struct ResponseXbox {
    #[serde(rename = "Token")]
    token: SecretString,
    #[serde(rename = "DisplayClaims")]
    claims: DisplayClaims,
}

#[derive(Deserialize)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

/// Xbox 的用户信息，`xid` 只在 XSTS 的响应中出现
#[derive(Deserialize)]
struct Xui {
    /// 用户哈希
    uhs: String,
    /// Xbox 用户 ID
    xid: Option<String>,
}

/// 解析 Xbox 端点的响应
fn xbox_result(res: Response<Vec<u8>>) -> Result<ResponseXbox> {
    let status = res.status();
    match serde_json::from_slice::<XboxResponse<ResponseXbox>>(res.body()) {
        Ok(v) => v.into_result(),
        // 拒绝授权时不一定带有响应体，此时只能依靠状态码
        Err(e) if status.is_success() => Err(e.into()),
        Err(_) => Err(Error::other(format!(
            "Xbox declined the authorization: {status}"
        ))),
    }
}

/// Xbox 授权链换取的 Minecraft 会话
struct Session {
    access_token: SecretString,
    expires_at: DateTime<Utc>,
    xuid: String,
    profile: Profile,
}

/// 用微软令牌走完 Xbox 与 Minecraft 的授权链
async fn authorize(downloader: &impl Downloader, microsoft: &SecretString) -> Result<Session> {
    let xbl = xbl_authenticate(downloader, microsoft).await?;
    let xsts = xsts_authorize(downloader, &xbl).await?;
    // 没有用户哈希就无法构造 Minecraft 的身份令牌
    let Some(xui) = xsts.claims.xui.into_iter().next() else {
        return Err(Error::other("XSTS returned no display claims"));
    };

    let (access_token, expires_at) = minecraft_login(downloader, &xui.uhs, &xsts.token).await?;
    check_entitlements(downloader, &access_token).await?;
    let profile = fetch_profile(downloader, &access_token).await?;

    Ok(Session {
        access_token,
        expires_at,
        xuid: xui.xid.unwrap_or_default(),
        profile,
    })
}

/// Xbox Live 用户认证，用微软令牌换取用户令牌
async fn xbl_authenticate(
    downloader: &impl Downloader,
    microsoft: &SecretString,
) -> Result<SecretString> {
    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct RequestAuthenticate<'a> {
        properties: Properties<'a>,
        relying_party: &'static str,
        token_type: &'static str,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct Properties<'a> {
        auth_method: &'static str,
        site_name: &'static str,
        rps_ticket: &'a str,
    }

    // 个人账户的票据需要 `d=` 前缀
    let ticket = format!("d={}", microsoft.expose_secret());
    let req = RequestAuthenticate {
        properties: Properties {
            auth_method: "RPS",
            site_name: "user.auth.xboxlive.com",
            rps_ticket: &ticket,
        },
        relying_party: "http://auth.xboxlive.com",
        token_type: "JWT",
    };
    let req = serde_json::to_string(&req)?;

    let res = downloader.post_json(XBL_AUTHENTICATE.clone(), req).await?;

    Ok(xbox_result(res)?.token)
}

/// XSTS 授权，用用户令牌换取 Minecraft 信赖方的令牌
async fn xsts_authorize(downloader: &impl Downloader, xbl: &SecretString) -> Result<ResponseXbox> {
    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct RequestAuthorize<'a> {
        properties: Properties<'a>,
        relying_party: &'static str,
        token_type: &'static str,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct Properties<'a> {
        sandbox_id: &'static str,
        user_tokens: [&'a str; 1],
    }

    let req = RequestAuthorize {
        properties: Properties {
            sandbox_id: "RETAIL",
            user_tokens: [xbl.expose_secret()],
        },
        relying_party: RELYING_PARTY,
        token_type: "JWT",
    };
    let req = serde_json::to_string(&req)?;

    let res = downloader.post_json(XSTS_AUTHORIZE.clone(), req).await?;

    xbox_result(res)
}

/// 用 Xbox 身份换取 Minecraft 令牌
async fn minecraft_login(
    downloader: &impl Downloader,
    uhs: &str,
    xsts: &SecretString,
) -> Result<(SecretString, DateTime<Utc>)> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RequestLogin {
        identity_token: String,
    }
    #[derive(Deserialize)]
    struct ResponseLogin {
        access_token: SecretString,
        expires_in: i64,
    }

    let req = RequestLogin {
        identity_token: format!("XBL3.0 x={uhs};{}", xsts.expose_secret()),
    };
    let req = serde_json::to_string(&req)?;

    let res = downloader.post_json(MINECRAFT_LOGIN.clone(), req).await?;
    if !res.status().is_success() {
        return Err(service_error(&res));
    }
    let res = serde_json::from_slice::<ResponseLogin>(res.body())?;

    Ok((res.access_token, Utc::now() + seconds(res.expires_in)))
}

/// 检查账户是否拥有 Minecraft: Java Edition
async fn check_entitlements(downloader: &impl Downloader, token: &SecretString) -> Result<()> {
    #[derive(Deserialize)]
    struct ResponseEntitlements {
        items: Vec<Entitlement>,
    }
    #[derive(Deserialize)]
    struct Entitlement {
        name: String,
    }

    let req = get_bearer(&MINECRAFT_ENTITLEMENTS, token.expose_secret());
    let res = downloader.send(req).await?;
    if !res.status().is_success() {
        return Err(service_error(&res));
    }
    let res = serde_json::from_slice::<ResponseEntitlements>(res.body())?;

    if !res
        .items
        .iter()
        .any(|i| JAVA_ENTITLEMENTS.contains(&i.name.as_str()))
    {
        return Err(MicrosoftError::NotEntitled.into());
    }
    Ok(())
}

/// 获取玩家档案
async fn fetch_profile(downloader: &impl Downloader, token: &SecretString) -> Result<Profile> {
    let req = get_bearer(&MINECRAFT_PROFILE, token.expose_secret());
    let res = downloader.send(req).await?;
    // 拥有游戏但还没有创建角色时返回 404
    if res.status() == 404 {
        return Err(MicrosoftError::NoProfile.into());
    }
    if !res.status().is_success() {
        return Err(service_error(&res));
    }

    Ok(serde_json::from_slice(res.body())?)
}

/// Minecraft 玩家档案
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    /// 玩家 UUID
    pub id: UnhyphenatedUuid,
    /// 玩家名
    pub name: String,
    /// 账户拥有的皮肤
    #[serde(default)]
    pub skins: Vec<Skin>,
    /// 账户拥有的披风
    #[serde(default)]
    pub capes: Vec<Cape>,
}

impl Profile {
    /// 当前使用的皮肤
    pub fn skin(&self) -> Option<&Skin> {
        self.skins.iter().find(|s| s.state == TextureState::Active)
    }
    /// 当前使用的披风
    pub fn cape(&self) -> Option<&Cape> {
        self.capes.iter().find(|c| c.state == TextureState::Active)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub id: Uuid,
    pub state: TextureState,
    /// 材质地址
    pub url: Url,
    /// 手臂模型
    pub variant: SkinVariant,
    /// 材质名，例如 `X-Steve`
    pub alias: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cape {
    pub id: Uuid,
    pub state: TextureState,
    /// 材质地址
    pub url: Url,
    /// 披风名，例如 `Migrator`
    pub alias: Option<String>,
}

/// 材质的启用状态
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextureState {
    Active,
    Inactive,
}

/// 皮肤的手臂模型
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkinVariant {
    Classic,
    Slim,
}

/// 微软账户的登录凭据
#[derive(Serialize, Deserialize)]
pub struct Authentication {
    /// Minecraft 访问令牌
    #[serde(with = "crate::utils::secret")]
    access_token: SecretString,
    /// 访问令牌的过期时刻
    expires_at: DateTime<Utc>,
    /// 微软刷新令牌，持久化后可用于免交互登录
    #[serde(with = "crate::utils::secret::option")]
    refresh_token: Option<SecretString>,

    /// Azure 应用注册的客户端 ID
    client_id: String,
    /// 可登录的账户类型
    tenant: Tenant,
    /// 授权范围
    scope: String,

    /// Xbox 用户 ID
    pub xuid: String,
    /// 玩家档案
    pub profile: Profile,
}

impl Authentication {
    /// Minecraft 访问令牌，有效期约一天
    pub fn access_token(&self) -> &SecretString {
        &self.access_token
    }
    /// 微软刷新令牌，持久化后可用于免交互登录
    ///
    /// 授权范围不包含 `offline_access` 时不存在
    pub fn refresh_token(&self) -> Option<&SecretString> {
        self.refresh_token.as_ref()
    }
    /// 访问令牌的过期时刻
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
    /// 访问令牌是否已经过期
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
    /// 访问令牌是否即将过期，`ready()` 会提前刷新
    pub fn is_stale(&self) -> bool {
        Utc::now() + seconds(REFRESH_MARGIN) >= self.expires_at
    }
    /// 刷新令牌，同时更新玩家档案
    pub async fn refresh(&mut self, downloader: &impl Downloader) -> Result<()> {
        let Some(refresh_token) = &self.refresh_token else {
            return Err(Error::other(
                "Refreshing requires the `offline_access` scope",
            ));
        };
        let token = refresh_grant(
            downloader,
            self.tenant,
            &self.client_id,
            &self.scope,
            refresh_token,
        )
        .await?;
        let session = authorize(downloader, &token.access_token).await?;

        self.access_token = session.access_token;
        self.expires_at = session.expires_at;
        // 微软可能签发新的刷新令牌，旧的仍然可用
        if let Some(refresh_token) = token.refresh_token {
            self.refresh_token = Some(refresh_token)
        }
        self.xuid = session.xuid;
        self.profile = session.profile;

        Ok(())
    }
}

/// 微软账户由 Xbox 用户 ID 确定
impl PartialEq for Authentication {
    fn eq(&self, other: &Self) -> bool {
        self.xuid == other.xuid
    }
}

impl Eq for Authentication {}

impl super::Authentication for Authentication {
    async fn vars(&self) -> Result<Variables<NotReady>> {
        let variables = Variables::new()
            .required(
                &self.profile.name,
                self.profile.id.to_string(),
                self.access_token.expose_secret(),
            )
            .legacy(self.access_token.expose_secret(), "msa")
            .modern(&self.client_id, &self.xuid);
        Ok(variables)
    }
    async fn ready(&mut self, downloader: &impl Downloader) -> Result<()> {
        // 微软的访问令牌只有一天，启动前刷新即可
        if self.is_stale() {
            self.refresh(downloader).await?
        }
        Ok(())
    }
}
