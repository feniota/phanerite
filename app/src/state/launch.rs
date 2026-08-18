#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchSettings {
    pub memory_mode: String,
    pub memory: u32,
    pub window_mode: String,
    pub window_width: u32,
    pub window_height: u32,
    pub quick_play_mode: String,
    pub quick_play_target: String,
    pub game_args: String,
    pub java_args: String,
    pub use_default_jvm_args: bool,
}
impl Default for LaunchSettings {
    fn default() -> Self {
        Self {
            memory_mode: "auto".into(),
            memory: 4,
            window_mode: "windowed".into(),
            window_width: 1280,
            window_height: 720,
            quick_play_mode: "none".into(),
            quick_play_target: String::new(),
            game_args: String::new(),
            java_args: String::new(),
            use_default_jvm_args: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchJob {
    pub instance_id: String,
    pub progress: u8,
    pub phase: String,
    pub error: Option<String>,
    pub cancellable: bool,
}
#[derive(Default)]
pub struct LaunchStore {
    pub job: Option<LaunchJob>,
    revision: u64,
}
impl LaunchStore {
    pub fn set_job(&mut self, j: Option<LaunchJob>) -> bool {
        if self.job == j {
            return false;
        }
        self.job = j;
        self.revision += 1;
        true
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
