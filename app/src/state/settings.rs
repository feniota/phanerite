//! Global launcher preferences and settings store operations.

use std::path::Path;

use crate::state::{JavaRuntimeSummary, LaunchSettings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessPriority {
    Low,
    Normal,
    High,
}

impl ProcessPriority {
    pub const ALL: [ProcessPriority; 3] = [
        ProcessPriority::Low,
        ProcessPriority::Normal,
        ProcessPriority::High,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ProcessPriority::Low => "Low",
            ProcessPriority::Normal => "Normal",
            ProcessPriority::High => "High",
        }
    }
}

/// Languages offered by the appearance section.
pub const LANGUAGES: [(&str, &str); 4] = [
    ("en", "English"),
    ("zh", "简体中文"),
    ("de", "Deutsch"),
    ("ja", "日本語"),
];

/// UI font size choices offered by the appearance section.
pub const FONT_SIZES: [(&str, &str); 3] = [("sm", "Small"), ("md", "Medium"), ("lg", "Large")];

/// Global launcher preferences that are not launch settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub close_after_launch: bool,
    pub hide_on_launch: bool,
    pub multi_instance: bool,
    pub check_updates: bool,
    pub download_threads: u32,
    pub connection_timeout: u32,
    pub language: String,
    pub font_size: String,
    pub show_news: bool,
    pub aphanite_server: String,
    pub show_game_logs: bool,
    pub debug_logs: bool,
    pub generate_game_options: bool,
    pub process_priority: ProcessPriority,
    pub opengl_renderer: String,
    pub vulkan_renderer: String,
    pub prefer_high_performance_gpu: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            close_after_launch: false,
            hide_on_launch: true,
            multi_instance: true,
            check_updates: true,
            download_threads: 4,
            connection_timeout: 30,
            language: "en".into(),
            font_size: "md".into(),
            show_news: true,
            aphanite_server: "https://aphanite.enita.cn/".into(),
            show_game_logs: true,
            debug_logs: false,
            generate_game_options: true,
            process_priority: ProcessPriority::Normal,
            opengl_renderer: "Default".into(),
            vulkan_renderer: "Default".into(),
            prefer_high_performance_gpu: true,
        }
    }
}

/// Boolean preference keys the settings page toggles generically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceFlag {
    CloseAfterLaunch,
    HideOnLaunch,
    MultiInstance,
    CheckUpdates,
    ShowNews,
    ShowGameLogs,
    DebugLogs,
    GenerateGameOptions,
    PreferHighPerformanceGpu,
}

impl PreferenceFlag {
    pub fn key(self) -> &'static str {
        match self {
            PreferenceFlag::CloseAfterLaunch => "closeAfterLaunch",
            PreferenceFlag::HideOnLaunch => "hideOnLaunch",
            PreferenceFlag::MultiInstance => "multiInstance",
            PreferenceFlag::CheckUpdates => "checkUpdates",
            PreferenceFlag::ShowNews => "showNews",
            PreferenceFlag::ShowGameLogs => "showGameLogs",
            PreferenceFlag::DebugLogs => "debugLogs",
            PreferenceFlag::GenerateGameOptions => "generateGameOptions",
            PreferenceFlag::PreferHighPerformanceGpu => "preferHighPerformanceGpu",
        }
    }
}

impl Preferences {
    pub fn flag(&self, flag: PreferenceFlag) -> bool {
        match flag {
            PreferenceFlag::CloseAfterLaunch => self.close_after_launch,
            PreferenceFlag::HideOnLaunch => self.hide_on_launch,
            PreferenceFlag::MultiInstance => self.multi_instance,
            PreferenceFlag::CheckUpdates => self.check_updates,
            PreferenceFlag::ShowNews => self.show_news,
            PreferenceFlag::ShowGameLogs => self.show_game_logs,
            PreferenceFlag::DebugLogs => self.debug_logs,
            PreferenceFlag::GenerateGameOptions => self.generate_game_options,
            PreferenceFlag::PreferHighPerformanceGpu => self.prefer_high_performance_gpu,
        }
    }

    pub fn set_flag(&mut self, flag: PreferenceFlag, value: bool) {
        match flag {
            PreferenceFlag::CloseAfterLaunch => self.close_after_launch = value,
            PreferenceFlag::HideOnLaunch => self.hide_on_launch = value,
            PreferenceFlag::MultiInstance => self.multi_instance = value,
            PreferenceFlag::CheckUpdates => self.check_updates = value,
            PreferenceFlag::ShowNews => self.show_news = value,
            PreferenceFlag::ShowGameLogs => self.show_game_logs = value,
            PreferenceFlag::DebugLogs => self.debug_logs = value,
            PreferenceFlag::GenerateGameOptions => self.generate_game_options = value,
            PreferenceFlag::PreferHighPerformanceGpu => self.prefer_high_performance_gpu = value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub launch: LaunchSettings,
    pub preferences: Preferences,
    pub accent: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            launch: LaunchSettings::default(),
            preferences: Preferences::default(),
            accent: "emerald".into(),
        }
    }
}

#[derive(Default)]
pub struct SettingsStore {
    settings: Settings,
    runtimes: Vec<JavaRuntimeSummary>,
    next_id: u64,
    revision: u64,
}

impl SettingsStore {
    pub fn new(settings: Settings, runtimes: Vec<JavaRuntimeSummary>) -> Self {
        Self {
            settings,
            runtimes,
            ..Default::default()
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn launch(&self) -> &LaunchSettings {
        &self.settings.launch
    }

    pub fn preferences(&self) -> &Preferences {
        &self.settings.preferences
    }

    pub fn accent(&self) -> &str {
        &self.settings.accent
    }

    pub fn runtimes(&self) -> &[JavaRuntimeSummary] {
        &self.runtimes
    }

    pub fn runtime(&self, id: &str) -> Option<&JavaRuntimeSummary> {
        self.runtimes.iter().find(|runtime| runtime.id == id)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn set_accent(&mut self, value: impl Into<String>) -> bool {
        let value = value.into();
        if self.settings.accent == value {
            return false;
        }
        self.settings.accent = value;
        self.revision += 1;
        true
    }

    pub fn set_launch(&mut self, value: LaunchSettings) -> bool {
        if self.settings.launch == value {
            return false;
        }
        self.settings.launch = value;
        self.revision += 1;
        true
    }

    pub fn update_launch(&mut self, edit: impl FnOnce(&mut LaunchSettings)) -> bool {
        let mut next = self.settings.launch.clone();
        edit(&mut next);
        self.set_launch(next)
    }

    pub fn set_preferences(&mut self, value: Preferences) -> bool {
        if self.settings.preferences == value {
            return false;
        }
        self.settings.preferences = value;
        self.revision += 1;
        true
    }

    pub fn update_preferences(&mut self, edit: impl FnOnce(&mut Preferences)) -> bool {
        let mut next = self.settings.preferences.clone();
        edit(&mut next);
        self.set_preferences(next)
    }

    pub fn set_flag(&mut self, flag: PreferenceFlag, value: bool) -> bool {
        self.update_preferences(|preferences| preferences.set_flag(flag, value))
    }

    pub fn add_runtime(
        &mut self,
        name: impl Into<String>,
        version: u32,
        path: impl AsRef<Path>,
    ) -> bool {
        let name = name.into();
        let path = path.as_ref();
        if name.trim().is_empty() || path.as_os_str().is_empty() {
            return false;
        }
        self.next_id += 1;
        self.runtimes.push(JavaRuntimeSummary {
            id: format!("java-{:08x}", self.next_id),
            name,
            version,
            version_string: version.to_string(),
            path: path.to_path_buf(),
            managed: false,
        });
        self.revision += 1;
        true
    }
}
