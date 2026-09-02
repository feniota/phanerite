use crate::error::Result;
use crate::instance::Instance;
use crate::instance::manifest::VersionType;
use crate::utils::state::{NotReady, Ready};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Default)]
pub struct Variables<S: Default> {
    _state: S,
    vars: HashMap<&'static str, String>,

    pub(super) feat: HashSet<&'static str>,
}

impl<S: Default> Variables<S> {
    // 新旧版必选配置
    /// Settings required by both old and new versions
    pub fn required(
        mut self,
        auth_player_name: impl Into<String>,
        auth_uuid: impl Into<String>,
        auth_access_token: impl Into<String>,
    ) -> Self {
        self.vars
            .insert("auth_player_name", auth_player_name.into());
        self.vars.insert("auth_uuid", auth_uuid.into());
        self.vars
            .insert("auth_access_token", auth_access_token.into());
        self
    }
    // 旧版必选配置
    /// Settings required by older versions
    pub fn legacy(mut self, auth_session: impl Into<String>, user_type: impl Into<String>) -> Self {
        self.vars.insert("auth_session", auth_session.into());
        self.vars.insert("user_type", user_type.into());
        self
    }
    // 新版必选配置
    /// Settings required by newer versions
    pub fn modern(mut self, clientid: impl Into<String>, auth_xuid: impl Into<String>) -> Self {
        self.vars.insert("clientid", clientid.into());
        self.vars.insert("auth_xuid", auth_xuid.into());
        self
    }
    pub fn resolution_width(mut self, width: u32) -> Self {
        self.vars.insert("resolution_width", width.to_string());
        self
    }
    pub fn resolution_height(mut self, height: u32) -> Self {
        self.vars.insert("resolution_height", height.to_string());
        self
    }
    pub fn quick_play_path(mut self, quick_play_path: impl Into<String>) -> Self {
        self.vars.insert("quick_play_path", quick_play_path.into());
        self
    }
    pub fn quick_play_singleplayer(mut self, quick_play_singleplayer: impl Into<String>) -> Self {
        self.vars
            .insert("quick_play_singleplayer", quick_play_singleplayer.into());
        self
    }
    pub fn quick_play_multiplayer(mut self, quick_play_multiplayer: impl Into<String>) -> Self {
        self.vars
            .insert("quick_play_multiplayer", quick_play_multiplayer.into());
        self
    }
    pub fn quick_play_realms(mut self, quick_play_realms: impl Into<String>) -> Self {
        self.vars
            .insert("quick_play_realms", quick_play_realms.into());
        self
    }
    // 启用特性
    /// Enables a feature
    pub fn feature(mut self, feature: &'static str) -> Self {
        self.feat.insert(feature);
        self
    }
}

impl Variables<NotReady> {
    pub fn new() -> Self {
        Default::default()
    }
    // 从实例生成必要项
    /// Generates the required entries from an instance
    pub fn generated<R, C>(mut self, instance: &Instance<R, C>) -> Result<Variables<Ready>> {
        let generated = ReadyVariables::from_instance(instance)?;
        generated.insert_into(&mut self.vars);
        Ok(Variables {
            _state: Ready,
            vars: self.vars,
            feat: self.feat,
        })
    }
}

impl Variables<Ready> {
    pub(super) fn resolve(&self, input: &str) -> Option<String> {
        if let Some(key) = input.strip_prefix("${").and_then(|x| x.strip_suffix('}')) {
            return self.vars.get(key).cloned();
        }
        self.resolve_template(input)
    }
    fn resolve_template(&self, input: &str) -> Option<String> {
        let mut output = String::with_capacity(input.len());
        let mut rest = input;

        while let Some(start) = rest.find("${") {
            output.push_str(&rest[..start]);
            let rest2 = &rest[start + 2..];
            let end = rest2.find('}')?;
            let key = &rest2[..end];
            let value = self.vars.get(key)?;
            output.push_str(value);
            rest = &rest2[end + 1..];
        }

        output.push_str(rest);
        Some(output)
    }
}

struct ReadyVariables {
    // 实例名称
    /// Instance name
    version_name: String,
    // 版本类型
    /// Version type
    version_type: VersionType,
    // 实例根目录
    /// Instance root directory
    game_directory: PathBuf,
    // 资源根目录
    /// Asset root directory
    assets_root: PathBuf,
    // 资源索引名
    /// Asset index name
    assets_index_name: String,
    // 游戏位置
    /// Game location
    classpath: String,
    // 二进制库位置
    /// Native library location
    natives_directory: PathBuf,
    // 启动器名称
    /// Launcher name
    launcher_name: &'static str,
    // 启动器版本
    /// Launcher version
    launcher_version: &'static str,
    // 日志前缀
    /// Log prefix
    path: PathBuf,

    // NeoForge 扩展
    // Library 根目录
    /// Library root directory
    library_directory: PathBuf,
    // 路径分隔符
    /// Path separator
    classpath_separator: &'static str,
}

impl ReadyVariables {
    fn from_instance<R, C>(instance: &Instance<R, C>) -> Result<Self> {
        Ok(Self {
            version_name: instance.manifest.id.clone(),
            version_type: instance.manifest.version_type,
            game_directory: std::path::absolute(&instance.instance_dir)?,
            assets_root: std::path::absolute(instance.storage.assets_dir())?,
            assets_index_name: instance.manifest.assets.clone(),
            classpath: instance
                .manifest
                .libraries
                .iter()
                .map(|lib| instance.storage.libraries_dir().join(lib.name.path()))
                .chain(std::iter::once(instance.client_file()))
                .map(std::path::absolute)
                .map(|p| p.map(|x| x.to_string_lossy().into_owned()))
                .collect::<std::result::Result<HashSet<_>, _>>()?
                .into_iter()
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" }),
            natives_directory: instance.instance_dir.join("native"),
            launcher_name: "Phanerite",
            launcher_version: env!("CARGO_PKG_VERSION"),
            path: std::path::absolute(instance.instance_dir.join("logs"))?,
            library_directory: instance.storage.libraries_dir().to_owned(),
            classpath_separator: if cfg!(windows) { ";" } else { ":" },
        })
    }

    fn insert_into(&self, vars: &mut HashMap<&'static str, String>) {
        vars.insert("version_name", self.version_name.clone());
        vars.insert(
            "game_directory",
            self.game_directory.to_string_lossy().into(),
        );
        vars.insert("version_type", self.version_type.to_string());
        vars.insert("assets_root", self.assets_root.to_string_lossy().into());
        vars.insert("assets_index_name", self.assets_index_name.clone());
        vars.insert("classpath", self.classpath.clone());
        vars.insert(
            "natives_directory",
            self.natives_directory.to_string_lossy().into(),
        );
        vars.insert("launcher_name", self.launcher_name.to_string());
        vars.insert("launcher_version", self.launcher_version.to_string());
        vars.insert("path", self.path.to_string_lossy().into());
        vars.insert(
            "library_directory",
            self.library_directory.to_string_lossy().into(),
        );
        vars.insert("classpath_separator", self.classpath_separator.to_string());
    }
}
