use crate::error::Result;
use crate::instance::instance_info::VersionType;
use crate::instance::Instance;
use crate::storage::Storage;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub struct Variables {
    vars: HashMap<&'static str, String>,

    pub(super) feat: HashSet<&'static str>,
}

impl Variables {
    pub fn builder() -> VariablesBuilder<Missing, Missing, Missing> {
        VariablesBuilder {
            auth_player_name: Missing,
            auth_uuid: Missing,
            auth_access_token: Missing,
            auth_session: Missing,
            user_type: Missing,
            clientid: Missing,
            auth_xuid: Missing,
            resolution_width: None,
            resolution_height: None,
            quick_play_path: None,
            quick_play_singleplayer: None,
            quick_play_multiplayer: None,
            quick_play_realms: None,
            features: HashSet::new(),
        }
    }
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

pub struct Missing;

pub struct VariablesBuilder<Required, Legacy, Modern> {
    // 新版 + 旧版
    auth_player_name: Required,
    auth_uuid: Required,
    auth_access_token: Required,
    // 仅旧版
    auth_session: Legacy,
    user_type: Legacy,
    // 仅新版
    clientid: Modern,
    auth_xuid: Modern,

    // 新版可选设置
    resolution_width: Option<String>,
    resolution_height: Option<String>,
    quick_play_path: Option<String>,
    quick_play_singleplayer: Option<String>,
    quick_play_multiplayer: Option<String>,
    quick_play_realms: Option<String>,

    // Features
    features: HashSet<&'static str>,
}

// ————————————————————————————————————————
// 新旧版必选配置
// ————————————————————————————————————————

impl<Required, Legacy, Modern> VariablesBuilder<Required, Legacy, Modern> {
    pub fn required(
        self,
        auth_player_name: impl Into<String>,
        auth_uuid: impl Into<String>,
        auth_access_token: impl Into<String>,
    ) -> VariablesBuilder<String, Legacy, Modern> {
        VariablesBuilder {
            auth_player_name: auth_player_name.into(),
            auth_uuid: auth_uuid.into(),
            auth_access_token: auth_access_token.into(),
            auth_session: self.auth_session,
            user_type: self.user_type,
            clientid: self.clientid,
            auth_xuid: self.auth_xuid,
            resolution_width: self.resolution_width,
            resolution_height: self.resolution_height,
            quick_play_path: self.quick_play_path,
            quick_play_singleplayer: self.quick_play_singleplayer,
            quick_play_multiplayer: self.quick_play_multiplayer,
            quick_play_realms: self.quick_play_realms,
            features: self.features,
        }
    }
}

// ————————————————————————————————————————
// 旧版必选配置
// ————————————————————————————————————————

impl<Legacy> VariablesBuilder<String, Legacy, Missing> {
    pub fn legacy(
        self,
        auth_session: impl Into<String>,
        user_type: impl Into<String>,
    ) -> VariablesBuilder<String, String, Missing> {
        VariablesBuilder {
            auth_player_name: self.auth_player_name,
            auth_uuid: self.auth_uuid,
            auth_access_token: self.auth_access_token,
            auth_session: auth_session.into(),
            user_type: user_type.into(),
            clientid: self.clientid,
            auth_xuid: self.auth_xuid,
            resolution_width: self.resolution_width,
            resolution_height: self.resolution_height,
            quick_play_path: self.quick_play_path,
            quick_play_singleplayer: self.quick_play_singleplayer,
            quick_play_multiplayer: self.quick_play_multiplayer,
            quick_play_realms: self.quick_play_realms,
            features: self.features,
        }
    }
}

// ————————————————————————————————————————
// 新版必选配置
// ————————————————————————————————————————

impl<Modern> VariablesBuilder<String, Missing, Modern> {
    pub fn modern(
        self,
        clientid: impl Into<String>,
        auth_xuid: impl Into<String>,
    ) -> VariablesBuilder<String, Missing, String> {
        VariablesBuilder {
            auth_player_name: self.auth_player_name,
            auth_uuid: self.auth_uuid,
            auth_access_token: self.auth_access_token,
            auth_session: self.auth_session,
            user_type: self.user_type,
            clientid: clientid.into(),
            auth_xuid: auth_xuid.into(),
            resolution_width: self.resolution_width,
            resolution_height: self.resolution_height,
            quick_play_path: self.quick_play_path,
            quick_play_singleplayer: self.quick_play_singleplayer,
            quick_play_multiplayer: self.quick_play_multiplayer,
            quick_play_realms: self.quick_play_realms,
            features: self.features,
        }
    }
}

// ————————————————————————————————————————
// 新版可选设置
// ————————————————————————————————————————
impl VariablesBuilder<String, Missing, String> {
    pub fn resolution_width(mut self, width: u32) -> Self {
        self.resolution_width = Some(width.to_string());
        self
    }
    pub fn resolution_height(mut self, height: u32) -> Self {
        self.resolution_height = Some(height.to_string());
        self
    }
    pub fn quick_play_path(mut self, value: impl Into<String>) -> Self {
        self.quick_play_path = Some(value.into());
        self
    }
    pub fn quick_play_singleplayer(mut self, value: impl Into<String>) -> Self {
        self.quick_play_singleplayer = Some(value.into());
        self
    }
    pub fn quick_play_multiplayer(mut self, value: impl Into<String>) -> Self {
        self.quick_play_multiplayer = Some(value.into());
        self
    }
    pub fn quick_play_realms(mut self, value: impl Into<String>) -> Self {
        self.quick_play_realms = Some(value.into());
        self
    }
}

// ————————————————————————————————————————
// 启用特性
// ————————————————————————————————————————

impl<R, L, M> VariablesBuilder<R, L, M> {
    pub fn feature(mut self, feature: &'static str) -> Self {
        self.features.insert(feature);
        self
    }
}

// ————————————————————————————————————————
// 自动生成配置
// ————————————————————————————————————————

struct Generated {
    /// 实例名称
    version_name: String,
    /// 版本类型
    version_type: VersionType,
    /// 实例根目录
    game_directory: PathBuf,
    /// 资源根目录
    assets_root: PathBuf,
    /// 资源索引名
    assets_index_name: String,
    /// 游戏位置
    classpath: String,
    /// 二进制库位置
    natives_directory: PathBuf,
    /// 启动器名称
    launcher_name: &'static str,
    /// 启动器版本
    launcher_version: &'static str,
    /// 日志前缀
    path: PathBuf,
}

impl Generated {
    fn from_instance(instance: &Instance, storage: &Storage) -> Result<Self> {
        let instance_dir = &instance.instance_dir;

        let cp = instance
            .manifest
            .libraries
            .iter()
            .map(|lib| storage.libraries_dir().join(&lib.downloads.artifact.path))
            .chain(std::iter::once(instance.client_file()))
            .filter_map(|p| std::fs::canonicalize(p).ok())
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(if cfg!(windows) { ";" } else { ":" });

        Ok(Self {
            version_name: instance.manifest.id.clone(),
            version_type: instance.manifest.version_type,
            game_directory: std::path::absolute(instance_dir)?,
            assets_root: std::path::absolute(storage.assets_dir())?,
            assets_index_name: instance.manifest.assets.clone(),
            classpath: cp,
            natives_directory: instance_dir.join("native"),
            launcher_name: "Phanerite",
            launcher_version: env!("CARGO_PKG_VERSION"),
            path: std::path::absolute(instance_dir.join("logs"))?,
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
    }
}

// ————————————————————————————————————————
// 构建完整变量
// ————————————————————————————————————————

macro_rules! insert_fields {
    ($map:expr, $self:expr, [$($field:ident),* $(,)?]) => {
        $(
            $map.insert(stringify!($field), $self.$field.into());
        )*
    };

    ($map:expr, $self:expr, optional [$($field:ident),* $(,)?]) => {
        $(
            if let Some(value) = $self.$field {
                $map.insert(stringify!($field), value.into());
            }
        )*
    };
}

/// 旧版配置
impl VariablesBuilder<String, String, Missing> {
    pub fn build(self, instance: &Instance, storage: &Storage) -> Result<Variables> {
        let mut vars = HashMap::new();

        insert_fields!(
            vars,
            self,
            [
                auth_player_name,
                auth_uuid,
                auth_access_token,
                auth_session,
                user_type,
            ]
        );

        let generated = Generated::from_instance(instance, storage)?;
        generated.insert_into(&mut vars);

        Ok(Variables {
            vars,
            feat: self.features,
        })
    }
}

/// 新版配置
impl VariablesBuilder<String, Missing, String> {
    pub fn build(self, instance: &Instance, storage: &Storage) -> Result<Variables> {
        let mut vars = HashMap::new();

        insert_fields!(
            vars,
            self,
            [
                auth_player_name,
                auth_uuid,
                auth_access_token,
                clientid,
                auth_xuid
            ]
        );

        insert_fields!(
            vars,
            self,
            optional [
                resolution_width,
                resolution_height,
                quick_play_path,
                quick_play_singleplayer,
                quick_play_multiplayer,
                quick_play_realms,
            ]
        );

        let generated = Generated::from_instance(instance, storage)?;
        generated.insert_into(&mut vars);

        Ok(Variables {
            vars,
            feat: self.features,
        })
    }
}
