use crate::download::Downloader;
use crate::error::{Error, Result};
use crate::instance::variables::Variables;
use crate::utils::secret::copy_secret;
use crate::utils::state::NotReady;
use crate::utils::uuid::UnhyphenatedUuid;
use async_lock::{RwLock, RwLockUpgradableReadGuard};
use bytes::Bytes;
use chrono::{DateTime, TimeDelta, Utc};
use http::{Request, Response};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use std::time::Duration;
use strum::IntoStaticStr;
use tracing::debug;
use url::{Url, form_urlencoded};
use uuid::Uuid;

// 微软身份平台
/// Microsoft identity platform
static AUTHORITY: LazyLock<Url> = LazyLock::new(|| endpoint("https://login.microsoftonline.com"));
// Xbox Live 用户认证
/// Xbox Live user authentication
static XBL_AUTHENTICATE: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://user.auth.xboxlive.com/user/authenticate"));
// XSTS 授权
/// XSTS authorization
static XSTS_AUTHORIZE: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://xsts.auth.xboxlive.com/xsts/authorize"));
// 使用 Xbox 身份登录 Minecraft
/// Log in to Minecraft with an Xbox identity
static MINECRAFT_LOGIN: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://api.minecraftservices.com/authentication/login_with_xbox"));
// Minecraft 商店的所有权条目
/// Ownership entries in the Minecraft store
static MINECRAFT_ENTITLEMENTS: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://api.minecraftservices.com/entitlements/mcstore"));
// Minecraft 玩家档案
/// Minecraft player profile
static MINECRAFT_PROFILE: LazyLock<Url> =
    LazyLock::new(|| endpoint("https://api.minecraftservices.com/minecraft/profile"));

// 设备码授权流程的 `grant_type`
/// `grant_type` for the device code authorization flow
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
// 默认授权范围，`offline_access` 用于换取刷新令牌
/// Default scope; `offline_access` is what buys the refresh token
const DEFAULT_SCOPE: &str = "XboxLive.signin offline_access";
// Minecraft 对应的 XSTS 信赖方
/// The XSTS relying party for Minecraft
const RELYING_PARTY: &str = "rp://api.minecraftservices.com/";
// 服务端要求放慢轮询时增加的间隔
/// Interval added when the server asks us to slow down polling
const SLOW_DOWN_STEP: Duration = Duration::from_secs(5);
// 提前刷新访问令牌的余量，避免游戏启动过程中过期
/// Margin for refreshing the access token early, so it does not expire while
/// the game is starting
const REFRESH_MARGIN: i64 = 5 * 60;
// Minecraft: Java Edition 的所有权条目，Game Pass 只有后者
/// Ownership entries for Minecraft: Java Edition; Game Pass only has the
/// latter
const JAVA_ENTITLEMENTS: [&str; 2] = ["product_minecraft", "game_minecraft"];

// 解析写死的端点地址
/// Parses a hard-coded endpoint URL
fn endpoint(url: &str) -> Url {
    url.parse().expect("endpoint URL should always be valid")
}

// 微软登录链路中的错误
/// Errors in the Microsoft login chain
#[derive(Debug, thiserror::Error)]
pub enum MicrosoftError {
    // OAuth 2.0 端点返回的错误
    /// Error returned by an OAuth 2.0 endpoint
    #[error("{0}")]
    OAuth(OAuthError),
    // Xbox Live 或 XSTS 拒绝授权，通常是账户自身的问题
    /// Xbox Live or XSTS declined the authorization, usually a problem with
    /// the account itself
    #[error("{0}")]
    Xbox(XboxError),
    // 用户在浏览器中拒绝了授权
    /// The user declined the authorization in the browser
    #[error("Authorization declined by the user")]
    Declined,
    // 设备码在用户完成授权前过期
    /// The device code expired before the user completed the authorization
    #[error("Device code expired before authorization")]
    Expired,
    // 账户没有 Minecraft: Java Edition 的所有权
    /// The account does not own Minecraft: Java Edition
    #[error("Account does not own Minecraft: Java Edition")]
    NotEntitled,
    // 账户拥有游戏但尚未创建玩家档案
    /// The account owns the game but has not created a player profile yet
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

// Xbox Live 与 XSTS 返回的错误
/// Errors returned by Xbox Live and XSTS
#[derive(Deserialize, Debug)]
pub struct XboxError {
    // Xbox 的错误码
    /// Xbox error code
    #[serde(rename = "XErr")]
    pub code: u64,
    #[serde(rename = "Message")]
    pub message: Option<String>,
    // 用于解决问题的跳转地址
    /// URL to visit in order to resolve the problem
    #[serde(rename = "Redirect")]
    pub redirect: Option<String>,
}

impl XboxError {
    // 已知错误码的说明，未知的错误码返回 `None`
    /// Explanation for a known error code; returns `None` for unknown codes
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

// OAuth 2.0 端点的响应，错误与成功共用同一个响应体
/// Response from an OAuth 2.0 endpoint; success and failure share one body
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

// Xbox 端点的响应，错误与成功共用同一个响应体
/// Response from an Xbox endpoint; success and failure share one body
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

// Minecraft 服务端点的错误响应
/// Error response from a Minecraft service endpoint
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceError {
    // 部分端点只返回 `errorMessage`，两者都不能作为必需字段
    /// Some endpoints only return `errorMessage`, so neither field can be
    /// required
    error: Option<String>,
    error_message: Option<String>,
}

// 将 Minecraft 服务的失败响应转换为错误
//
// 授权链有五个端点，失败时必须指出是哪一个，
// 否则只剩一个状态码无从排查
/// Turns a failed Minecraft service response into an error
///
/// The authorization chain has five endpoints, so a failure has to say which
/// one it was; otherwise all that is left is a status code and nothing to
/// diagnose from
fn service_error(endpoint: &str, res: &Response<Bytes>) -> Error {
    // 这些端点的失败响应不含凭据，可以整体记录
    debug!(
        "{endpoint} responded {}: {}",
        res.status(),
        String::from_utf8_lossy(res.body())
    );
    match serde_json::from_slice::<ServiceError>(res.body())
        .ok()
        .and_then(|e| e.error_message.or(e.error))
    {
        Some(message) => Error::other(format!("{endpoint}: {message}")),
        // 失败响应不一定带有可解析的响应体，此时只能依靠状态码
        None => Error::other(format!("{endpoint} declined the request: {}", res.status())),
    }
}

// 构造表单编码的 POST 请求
/// Builds a form-encoded POST request
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

// 构造携带 Bearer 令牌的 GET 请求
/// Builds a GET request carrying a bearer token
fn get_bearer(url: &Url, token: &str) -> Request<Vec<u8>> {
    Request::get(url.as_str())
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .body(Vec::new())
        .expect("building a request from a valid URL should never fail")
}

// 将秒数转换为 `TimeDelta`，超出范围时归零
/// Converts seconds into a `TimeDelta`, falling back to zero when out of range
fn seconds(secs: i64) -> TimeDelta {
    TimeDelta::try_seconds(secs).unwrap_or_default()
}

// 微软身份平台的租户，决定哪些账户可以登录
/// Microsoft identity platform tenant, which decides what accounts can log in
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, IntoStaticStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Tenant {
    // 仅个人微软账户，Minecraft 登录的默认选择
    /// Personal Microsoft accounts only, the default for Minecraft login
    #[default]
    Consumers,
    // 个人账户与工作/学校账户
    /// Personal accounts plus work/school accounts
    Common,
    // 仅工作/学校账户
    /// Work/school accounts only
    Organizations,
}

impl Tenant {
    // 构造该租户下的 OAuth 2.0 端点
    /// Builds the OAuth 2.0 endpoint for this tenant
    fn oauth(self, path: &str) -> Url {
        let mut url = AUTHORITY.clone();
        url.path_segments_mut()
            .expect("AUTHORITY should always be a base URL")
            .extend([<&str>::from(self), "oauth2", "v2.0", path]);
        url
    }
}

// 微软账户的设备码登录
/// Device code login for a Microsoft account
pub struct Login<'a, C, D: Downloader> {
    downloader: &'a D,

    // Azure 应用注册的客户端 ID
    /// Client ID of the Azure app registration
    client_id: C,
    // 可登录的账户类型
    /// What kinds of account can log in
    tenant: Tenant,
    // 授权范围
    /// Scope
    scope: String,
}

impl<'a> Authentication {
    // 创建登录会话
    /// Creates a login session
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
    // 可登录的账户类型
    /// What kinds of account can log in
    pub fn tenant(mut self, tenant: Tenant) -> Self {
        self.tenant = tenant;
        self
    }
    // 自定义授权范围，需要包含 `offline_access` 才能获得刷新令牌
    /// Custom scope; it must include `offline_access` to get a refresh token
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = scope.into();
        self
    }
}

impl<'a, D: Downloader> Login<'a, NotReady, D> {
    // Azure 应用注册的客户端 ID，需要允许公共客户端流
    /// Client ID of the Azure app registration; it must allow public client
    /// flows
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
    // 申请设备码，之后需要用户在浏览器中完成授权
    /// Requests a device code; the user then has to complete the authorization
    /// in a browser
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
    // 使用持久化的刷新令牌免交互登录
    /// Logs in without interaction, using a persisted refresh token
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
        let authorized = authorize(self.downloader, &token.access_token).await?;

        Ok(Authentication {
            client_id: self.client_id,
            tenant: self.tenant,
            scope: self.scope,
            xuid: authorized.xuid,
            state: RwLock::new(State {
                access_token: authorized.access_token,
                expires_at: authorized.expires_at,
                // 微软可能签发新的刷新令牌，旧的仍然可用
                refresh_token: Some(token.refresh_token.unwrap_or(refresh_token)),
                profile: authorized.profile,
            }),
        })
    }
}

// 令牌端点签发的微软令牌
/// Microsoft token issued by the token endpoint
#[derive(Deserialize)]
struct Token {
    // 微软访问令牌，只用于换取 Xbox 身份
    /// Microsoft access token, only used to exchange for an Xbox identity
    access_token: SecretString,
    // 刷新令牌，仅在授权范围包含 `offline_access` 时签发
    /// Refresh token, only issued when the scope includes `offline_access`
    refresh_token: Option<SecretString>,
}

// 用刷新令牌换取新的微软令牌
/// Exchanges a refresh token for a new Microsoft token
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

// 等待用户完成授权的设备码会话
/// Device code session waiting for the user to complete the authorization
pub struct Pending<'a, D: Downloader> {
    downloader: &'a D,

    client_id: String,
    tenant: Tenant,
    scope: String,

    // 轮询用的设备码
    /// Device code used for polling
    device_code: SecretString,

    // 需要展示给用户的用户码
    /// User code that has to be shown to the user
    pub user_code: String,
    // 需要用户打开的地址
    /// Address the user has to open
    pub verification_uri: Url,
    // 微软给出的完整提示语，已经包含用户码与地址
    /// Full message given by Microsoft, already containing the user code and
    /// the address
    pub message: String,

    // 服务端建议的轮询间隔
    /// Polling interval suggested by the server
    interval: Duration,
    // 设备码的过期时刻
    /// When the device code expires
    expires_at: DateTime<Utc>,
}

impl<D: Downloader> Pending<'_, D> {
    // 预填了用户码的地址，适合做成可点击的链接或二维码
    //
    // 微软不返回 RFC 8628 的 `verification_uri_complete`，这里用登录页支持的
    // `otc` 参数拼出来，属于未文档化的行为，仍然需要展示 `message` 兜底
    /// Address with the user code pre-filled, suitable for a clickable link or
    /// a QR code
    ///
    /// Microsoft does not return RFC 8628's `verification_uri_complete`, so it
    /// is assembled here from the `otc` parameter that the login page supports.
    /// That is undocumented behavior, so `message` still has to be displayed as
    /// a fallback
    pub fn verification_uri_complete(&self) -> Url {
        let mut url = self.verification_uri.clone();
        url.query_pairs_mut().append_pair("otc", &self.user_code);
        url
    }
    // 服务端建议的轮询间隔
    /// Polling interval suggested by the server
    pub fn interval(&self) -> Duration {
        self.interval
    }
    // 设备码的过期时刻
    /// When the device code expires
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
    // 设备码是否已经过期
    /// Whether the device code has already expired
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
    // 轮询一次授权状态，用户尚未完成授权时返回 `None`
    //
    // 轮询间隔不应该小于 `interval()`，否则服务端会要求放慢
    /// Polls the authorization state once; returns `None` while the user has
    /// not completed the authorization yet
    ///
    /// The polling interval should not be shorter than `interval()`, otherwise
    /// the server will ask to slow down
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

        let authorized = authorize(self.downloader, &token.access_token).await?;

        Ok(Some(Authentication {
            client_id: self.client_id.clone(),
            tenant: self.tenant,
            scope: self.scope.clone(),
            xuid: authorized.xuid,
            state: RwLock::new(State {
                access_token: authorized.access_token,
                expires_at: authorized.expires_at,
                refresh_token: token.refresh_token,
                profile: authorized.profile,
            }),
        }))
    }
    // 按建议的间隔轮询直到授权完成，计时器由调用方提供
    //
    // 例如 `wait(async |d| { smol::Timer::after(d).await; })`
    /// Polls at the suggested interval until the authorization completes; the
    /// timer is supplied by the caller
    ///
    /// For example `wait(async |d| { smol::Timer::after(d).await; })`
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
    // 处理轮询中的错误，可以继续等待时返回 `Ok(())`
    /// Handles an error from polling; returns `Ok(())` when waiting can
    /// continue
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

// Xbox Live 与 XSTS 的成功响应
/// Successful response from Xbox Live and XSTS
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

// Xbox 的用户信息，`xid` 只在 XSTS 的响应中出现
/// Xbox user information; `xid` only appears in the XSTS response
#[derive(Deserialize)]
struct Xui {
    // 用户哈希
    /// User hash
    uhs: String,
    // Xbox 用户 ID
    /// Xbox user ID
    xid: Option<String>,
}

// 解析 Xbox 端点的响应
/// Parses the response of an Xbox endpoint
fn xbox_result(endpoint: &str, res: Response<Bytes>) -> Result<ResponseXbox> {
    let status = res.status();
    // Xbox 的失败响应只有错误码与跳转地址，可以整体记录
    if !status.is_success() {
        debug!(
            "{endpoint} responded {status}: {}",
            String::from_utf8_lossy(res.body())
        );
    }
    match serde_json::from_slice::<XboxResponse<ResponseXbox>>(res.body()) {
        Ok(v) => v.into_result(),
        // 拒绝授权时不一定带有响应体，此时只能依靠状态码
        Err(e) if status.is_success() => Err(e.into()),
        Err(_) => Err(Error::other(format!(
            "{endpoint} declined the authorization: {status}"
        ))),
    }
}

// Xbox 授权链换取的 Minecraft 会话
/// Minecraft session obtained through the Xbox authorization chain
struct Authorized {
    access_token: SecretString,
    expires_at: DateTime<Utc>,
    xuid: String,
    profile: Profile,
}

// 用微软令牌走完 Xbox 与 Minecraft 的授权链
/// Walks the whole Xbox and Minecraft authorization chain with a Microsoft
/// token
async fn authorize(downloader: &impl Downloader, microsoft: &SecretString) -> Result<Authorized> {
    let xbl = xbl_authenticate(downloader, microsoft).await?;
    let xsts = xsts_authorize(downloader, &xbl).await?;
    // 没有用户哈希就无法构造 Minecraft 的身份令牌
    let Some(xui) = xsts.claims.xui.into_iter().next() else {
        return Err(Error::other("XSTS returned no display claims"));
    };

    let (access_token, expires_at) = minecraft_login(downloader, &xui.uhs, &xsts.token).await?;
    check_entitlements(downloader, &access_token).await?;
    let profile = fetch_profile(downloader, &access_token).await?;

    Ok(Authorized {
        access_token,
        expires_at,
        xuid: xui.xid.unwrap_or_default(),
        profile,
    })
}

// Xbox Live 用户认证，用微软令牌换取用户令牌
/// Xbox Live user authentication; exchanges a Microsoft token for a user token
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

    Ok(xbox_result("Xbox Live authenticate", res)?.token)
}

// XSTS 授权，用用户令牌换取 Minecraft 信赖方的令牌
/// XSTS authorization; exchanges a user token for a token for the Minecraft
/// relying party
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

    xbox_result("XSTS authorize", res)
}

// 用 Xbox 身份换取 Minecraft 令牌
/// Exchanges an Xbox identity for a Minecraft token
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
        return Err(service_error("Minecraft login_with_xbox", &res));
    }
    let res = serde_json::from_slice::<ResponseLogin>(res.body())?;

    Ok((res.access_token, Utc::now() + seconds(res.expires_in)))
}

// 检查账户是否拥有 Minecraft: Java Edition
/// Checks whether the account owns Minecraft: Java Edition
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
        return Err(service_error("Minecraft entitlements", &res));
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

// 获取玩家档案
/// Fetches the player profile
async fn fetch_profile(downloader: &impl Downloader, token: &SecretString) -> Result<Profile> {
    let req = get_bearer(&MINECRAFT_PROFILE, token.expose_secret());
    let res = downloader.send(req).await?;
    // 拥有游戏但还没有创建角色时返回 404
    if res.status() == 404 {
        return Err(MicrosoftError::NoProfile.into());
    }
    if !res.status().is_success() {
        return Err(service_error("Minecraft profile", &res));
    }

    Ok(serde_json::from_slice(res.body())?)
}

// Minecraft 玩家档案
/// Minecraft player profile
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    // 玩家 UUID
    /// Player UUID
    pub id: UnhyphenatedUuid,
    // 玩家名
    /// Player name
    pub name: String,
    // 账户拥有的皮肤
    /// Skins the account owns
    #[serde(default)]
    pub skins: Vec<Skin>,
    // 账户拥有的披风
    /// Capes the account owns
    #[serde(default)]
    pub capes: Vec<Cape>,
}

impl Profile {
    // 当前使用的皮肤
    /// The skin currently in use
    pub fn skin(&self) -> Option<&Skin> {
        self.skins.iter().find(|s| s.state == TextureState::Active)
    }
    // 当前使用的披风
    /// The cape currently in use
    pub fn cape(&self) -> Option<&Cape> {
        self.capes.iter().find(|c| c.state == TextureState::Active)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub id: Uuid,
    pub state: TextureState,
    // 材质地址
    /// Texture address
    pub url: Url,
    // 手臂模型
    /// Arm model
    pub variant: SkinVariant,
    // 材质名，例如 `X-Steve`
    /// Texture name, for example `X-Steve`
    pub alias: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cape {
    pub id: Uuid,
    pub state: TextureState,
    // 材质地址
    /// Texture address
    pub url: Url,
    // 披风名，例如 `Migrator`
    /// Cape name, for example `Migrator`
    pub alias: Option<String>,
}

// 材质的启用状态
/// Whether a texture is enabled
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextureState {
    Active,
    Inactive,
}

// 皮肤的手臂模型
/// Arm model of a skin
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkinVariant {
    Classic,
    Slim,
}

// 微软账户的登录凭据
/// Login credentials of a Microsoft account
#[derive(Deserialize)]
pub struct Authentication {
    // Azure 应用注册的客户端 ID
    /// Client ID of the Azure app registration
    client_id: String,
    // 可登录的账户类型
    /// What kinds of account can log in
    tenant: Tenant,
    // 授权范围
    /// Scope
    scope: String,

    // Xbox 用户 ID，账户的身份，登录后不再变化
    /// Xbox user ID, the identity of the account, which does not change after
    /// login
    pub xuid: String,

    // 会随续期变化的部分
    /// The part that changes on renewal
    #[serde(with = "crate::utils::lock")]
    state: RwLock<State>,
}

// [`Authentication`] 的快照，与 [`Authentication`] 的序列化格式一致
//
// 不变的部分借用自 [`Authentication`]，只复制锁内的状态
/// Snapshot of an [`Authentication`], matching [`Authentication`]'s
/// serialization format
///
/// The immutable parts are borrowed from the [`Authentication`]; only the state
/// inside the lock is copied
#[derive(Serialize)]
pub struct Data<'a> {
    client_id: &'a str,
    tenant: Tenant,
    scope: &'a str,
    xuid: &'a str,
    state: State,
}

// 会随续期变化的会话状态
/// Session state that changes on renewal
#[derive(Serialize, Deserialize)]
struct State {
    // Minecraft 访问令牌
    /// Minecraft access token
    #[serde(with = "crate::utils::secret")]
    access_token: SecretString,
    // 访问令牌的过期时刻
    /// When the access token expires
    expires_at: DateTime<Utc>,
    // 微软刷新令牌，持久化后可用于免交互登录
    /// Microsoft refresh token; once persisted it can be used to log in without
    /// interaction
    #[serde(with = "crate::utils::secret::option")]
    refresh_token: Option<SecretString>,
    // 玩家档案
    /// Player profile
    profile: Profile,
}

impl State {
    // 访问令牌是否即将过期
    /// Whether the access token is about to expire
    fn is_stale(&self) -> bool {
        Utc::now() + seconds(REFRESH_MARGIN) >= self.expires_at
    }
    // 复制一份用于序列化
    //
    // `SecretString` 没有 `Clone`，这里明确地复制明文；
    // 序列化本身就要写出明文，多这一份短暂的副本不会额外暴露什么
    /// Makes a copy for serialization
    ///
    /// `SecretString` is not `Clone`, so the plaintext is copied explicitly
    /// here; serialization has to write out the plaintext anyway, so this one
    /// short-lived copy exposes nothing extra
    fn snapshot(&self) -> Self {
        Self {
            access_token: copy_secret(&self.access_token),
            expires_at: self.expires_at,
            refresh_token: self.refresh_token.as_ref().map(copy_secret),
            profile: self.profile.clone(),
        }
    }
}

impl Authentication {
    // 取一份可以持久化的快照
    /// Takes a snapshot that can be persisted
    pub async fn snapshot(&self) -> Data<'_> {
        Data {
            client_id: &self.client_id,
            tenant: self.tenant,
            scope: &self.scope,
            xuid: &self.xuid,
            state: self.state.read().await.snapshot(),
        }
    }
    // 在回调中读取 Minecraft 访问令牌，有效期约一天
    /// Reads the Minecraft access token inside a callback; it is valid for
    /// about a day
    pub async fn with_access_token<R>(&self, f: impl FnOnce(&str) -> R) -> R {
        f(self.state.read().await.access_token.expose_secret())
    }
    // 在回调中读取微软刷新令牌，持久化后可用于免交互登录
    //
    // 授权范围不包含 `offline_access` 时为 `None`
    /// Reads the Microsoft refresh token inside a callback; once persisted it
    /// can be used to log in without interaction
    ///
    /// It is `None` when the scope does not include `offline_access`
    pub async fn with_refresh_token<R>(&self, f: impl FnOnce(Option<&str>) -> R) -> R {
        f(self
            .state
            .read()
            .await
            .refresh_token
            .as_ref()
            .map(ExposeSecret::expose_secret))
    }
    // 玩家档案的副本
    /// A copy of the player profile
    pub async fn profile(&self) -> Profile {
        self.state.read().await.profile.clone()
    }
    // 在回调中读取玩家档案，避免复制
    /// Reads the player profile inside a callback, avoiding a copy
    pub async fn with_profile<R>(&self, f: impl FnOnce(&Profile) -> R) -> R {
        f(&self.state.read().await.profile)
    }
    // 访问令牌的过期时刻
    /// When the access token expires
    pub async fn expires_at(&self) -> DateTime<Utc> {
        self.state.read().await.expires_at
    }
    // 访问令牌是否已经过期
    /// Whether the access token has already expired
    pub async fn is_expired(&self) -> bool {
        Utc::now() >= self.state.read().await.expires_at
    }
    // 访问令牌是否即将过期，`ready()` 会提前刷新
    /// Whether the access token is about to expire; `ready()` refreshes it
    /// ahead of time
    pub async fn is_stale(&self) -> bool {
        self.state.read().await.is_stale()
    }
    // 立即刷新令牌，同时更新玩家档案
    //
    // 通常不需要手动调用，`ready()` 会在令牌即将过期时自行续期
    /// Refreshes the token right away, updating the player profile as well
    ///
    /// It usually does not need to be called by hand; `ready()` renews the
    /// token itself when it is about to expire
    pub async fn refresh(&self, downloader: &impl Downloader) -> Result<()> {
        self.renew(self.state.upgradable_read().await, downloader)
            .await
    }
    // 在续期临界区内完成刷新并提交
    //
    // 网络交互期间只持有可升级读锁，不阻塞读者；
    // 中途返回错误时本地状态原封不动
    /// Performs the refresh and commits it inside the renewal critical section
    ///
    /// Only an upgradable read lock is held during the network exchange, so
    /// readers are not blocked; if an error is returned partway through, the
    /// local state is left untouched
    async fn renew(
        &self,
        guard: RwLockUpgradableReadGuard<'_, State>,
        downloader: &impl Downloader,
    ) -> Result<()> {
        let Some(refresh_token) = &guard.refresh_token else {
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
        let authorized = authorize(downloader, &token.access_token).await?;
        // `xuid` 是账户的身份，续期不应该把它换成另一个账户
        if authorized.xuid != self.xuid {
            return Err(Error::other("Refreshed into a different account"));
        }

        // 网络交互已经结束，提交期间不再 await
        let mut state = RwLockUpgradableReadGuard::upgrade(guard).await;
        state.access_token = authorized.access_token;
        state.expires_at = authorized.expires_at;
        // 微软可能签发新的刷新令牌，旧的仍然可用
        if let Some(refresh_token) = token.refresh_token {
            state.refresh_token = Some(refresh_token)
        }
        state.profile = authorized.profile;

        Ok(())
    }
}

// 微软账户由 Xbox 用户 ID 确定
/// A Microsoft account is identified by its Xbox user ID
impl PartialEq for Authentication {
    fn eq(&self, other: &Self) -> bool {
        self.xuid == other.xuid
    }
}

impl Eq for Authentication {}

impl super::Authentication for Authentication {
    async fn vars(&self) -> Result<Variables<NotReady>> {
        let state = self.state.read().await;
        let variables = Variables::new()
            .required(
                &state.profile.name,
                state.profile.id.to_string(),
                state.access_token.expose_secret(),
            )
            .legacy(state.access_token.expose_secret(), "msa")
            .modern(&self.client_id, &self.xuid);
        Ok(variables)
    }
    async fn serialize(&self) -> impl Serialize {
        self.snapshot().await
    }
    async fn ready(&self, downloader: &impl Downloader) -> Result<()> {
        // 微软的访问令牌只有一天，启动前刷新即可
        //
        // 快路径：令牌还新鲜时不必进入续期临界区
        if !self.is_stale().await {
            return Ok(());
        }
        let guard = self.state.upgradable_read().await;
        // 双检：排队等待期间可能已经有人续期过
        if !guard.is_stale() {
            return Ok(());
        }
        self.renew(guard, downloader).await
    }
}
