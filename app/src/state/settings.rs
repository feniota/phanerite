use super::launch::LaunchSettings;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub launch: LaunchSettings,
    pub accent: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            launch: LaunchSettings::default(),
            accent: "emerald".into(),
        }
    }
}
#[derive(Default)]
pub struct SettingsStore {
    pub settings: Settings,
    revision: u64,
}
impl SettingsStore {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            ..Default::default()
        }
    }
    pub fn set_accent(&mut self, v: impl Into<String>) -> bool {
        let v = v.into();
        if self.settings.accent == v {
            return false;
        }
        self.settings.accent = v;
        self.revision += 1;
        true
    }
    pub fn set_launch(&mut self, v: LaunchSettings) -> bool {
        if self.settings.launch == v {
            return false;
        }
        self.settings.launch = v;
        self.revision += 1;
        true
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
