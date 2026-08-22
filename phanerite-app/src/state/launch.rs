//! Launch configuration, overrides, progress, and launch-state storage.

use std::collections::BTreeMap;

use crate::route::InstanceRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryMode {
    Auto,
    Manual,
}

impl MemoryMode {
    pub const ALL: [MemoryMode; 2] = [MemoryMode::Auto, MemoryMode::Manual];

    pub fn label(self) -> &'static str {
        match self {
            MemoryMode::Auto => "Automatic",
            MemoryMode::Manual => "Manual",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowMode {
    Windowed,
    Maximized,
    Fullscreen,
}

impl WindowMode {
    pub const ALL: [WindowMode; 3] = [
        WindowMode::Windowed,
        WindowMode::Maximized,
        WindowMode::Fullscreen,
    ];

    pub fn label(self) -> &'static str {
        match self {
            WindowMode::Windowed => "Windowed",
            WindowMode::Maximized => "Maximized",
            WindowMode::Fullscreen => "Fullscreen",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuickPlayMode {
    None,
    Server,
    World,
    Realms,
}

impl QuickPlayMode {
    pub const ALL: [QuickPlayMode; 4] = [
        QuickPlayMode::None,
        QuickPlayMode::Server,
        QuickPlayMode::World,
        QuickPlayMode::Realms,
    ];

    pub fn label(self) -> &'static str {
        match self {
            QuickPlayMode::None => "No quick play",
            QuickPlayMode::Server => "Multiplayer server",
            QuickPlayMode::World => "Single-player world",
            QuickPlayMode::Realms => "Realms ID",
        }
    }

    /// Placeholder for the target input, mirroring the prototype's copy.
    pub fn target_placeholder(self) -> &'static str {
        match self {
            QuickPlayMode::None => "",
            QuickPlayMode::Server => "Server address",
            QuickPlayMode::World => "World name",
            QuickPlayMode::Realms => "Realms ID",
        }
    }
}

/// Every launch setting an instance may override individually. Ordering is the
/// order the instance launch settings page renders them in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LaunchField {
    Memory,
    MemoryMode,
    WindowMode,
    WindowWidth,
    WindowHeight,
    QuickPlayMode,
    QuickPlayTarget,
    GameArgs,
    EnvironmentVariables,
    JavaArgs,
    PreLaunchCommand,
    CommandWrapper,
    PostExitCommand,
    NativesPath,
    GlfwPath,
    OpenalPath,
    UseDefaultJvmArgs,
    UseOptimizedJvmArgs,
    SkipJvmValidation,
    AllowAutoAgent,
    SkipIntegrityCheck,
    UseCustomNatives,
    SkipNativePatching,
}

impl LaunchField {
    /// Stable identifier used for element IDs and the crash environment's
    /// "overridden launch settings" summary.
    pub fn key(self) -> &'static str {
        match self {
            LaunchField::Memory => "memory",
            LaunchField::MemoryMode => "memoryMode",
            LaunchField::WindowMode => "windowMode",
            LaunchField::WindowWidth => "windowWidth",
            LaunchField::WindowHeight => "windowHeight",
            LaunchField::QuickPlayMode => "quickPlayMode",
            LaunchField::QuickPlayTarget => "quickPlayTarget",
            LaunchField::GameArgs => "gameArgs",
            LaunchField::EnvironmentVariables => "environmentVariables",
            LaunchField::JavaArgs => "javaArgs",
            LaunchField::PreLaunchCommand => "preLaunchCommand",
            LaunchField::CommandWrapper => "commandWrapper",
            LaunchField::PostExitCommand => "postExitCommand",
            LaunchField::NativesPath => "nativesPath",
            LaunchField::GlfwPath => "glfwPath",
            LaunchField::OpenalPath => "openalPath",
            LaunchField::UseDefaultJvmArgs => "useDefaultJvmArgs",
            LaunchField::UseOptimizedJvmArgs => "useOptimizedJvmArgs",
            LaunchField::SkipJvmValidation => "skipJvmValidation",
            LaunchField::AllowAutoAgent => "allowAutoAgent",
            LaunchField::SkipIntegrityCheck => "skipIntegrityCheck",
            LaunchField::UseCustomNatives => "useCustomNatives",
            LaunchField::SkipNativePatching => "skipNativePatching",
        }
    }
}

/// Text fields shown by the instance "Advanced" section, with their captions.
pub const ADVANCED_TEXT_FIELDS: [(&str, LaunchField, &str); 9] = [
    (
        "Game arguments",
        LaunchField::GameArgs,
        "Additional arguments passed to Minecraft.",
    ),
    (
        "Environment variables",
        LaunchField::EnvironmentVariables,
        "KEY=value pairs, separated by spaces.",
    ),
    (
        "JVM arguments",
        LaunchField::JavaArgs,
        "Additional arguments passed to Java.",
    ),
    (
        "Pre-launch command",
        LaunchField::PreLaunchCommand,
        "Runs before the game starts.",
    ),
    (
        "Command wrapper",
        LaunchField::CommandWrapper,
        "Wraps the game launch command.",
    ),
    (
        "Post-exit command",
        LaunchField::PostExitCommand,
        "Runs after the game exits.",
    ),
    (
        "Custom natives path",
        LaunchField::NativesPath,
        "Overrides the generated natives directory.",
    ),
    (
        "Custom GLFW path",
        LaunchField::GlfwPath,
        "Optional GLFW native library.",
    ),
    (
        "Custom OpenAL path",
        LaunchField::OpenalPath,
        "Optional OpenAL native library.",
    ),
];

/// Boolean fields shown by the instance "Advanced" section.
pub const ADVANCED_FLAG_FIELDS: [(&str, LaunchField); 7] = [
    ("Use default JVM arguments", LaunchField::UseDefaultJvmArgs),
    (
        "Use optimized JVM arguments",
        LaunchField::UseOptimizedJvmArgs,
    ),
    ("Skip JVM validation", LaunchField::SkipJvmValidation),
    ("Allow automatic Java Agent", LaunchField::AllowAutoAgent),
    ("Skip integrity check", LaunchField::SkipIntegrityCheck),
    ("Use custom natives", LaunchField::UseCustomNatives),
    ("Skip native patching", LaunchField::SkipNativePatching),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchValue {
    Text(String),
    Number(u32),
    Flag(bool),
    Memory(MemoryMode),
    Window(WindowMode),
    QuickPlay(QuickPlayMode),
}

impl LaunchValue {
    pub fn as_text(&self) -> &str {
        match self {
            LaunchValue::Text(value) => value,
            _ => "",
        }
    }

    pub fn as_number(&self) -> u32 {
        match self {
            LaunchValue::Number(value) => *value,
            _ => 0,
        }
    }

    pub fn as_flag(&self) -> bool {
        matches!(self, LaunchValue::Flag(true))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchSettings {
    pub memory_mode: MemoryMode,
    pub memory: u32,
    pub window_mode: WindowMode,
    pub window_width: u32,
    pub window_height: u32,
    pub quick_play_mode: QuickPlayMode,
    pub quick_play_target: String,
    pub game_args: String,
    pub environment_variables: String,
    pub java_args: String,
    pub use_default_jvm_args: bool,
    pub use_optimized_jvm_args: bool,
    pub skip_jvm_validation: bool,
    pub allow_auto_agent: bool,
    pub skip_integrity_check: bool,
    pub pre_launch_command: String,
    pub command_wrapper: String,
    pub post_exit_command: String,
    pub use_custom_natives: bool,
    pub natives_path: String,
    pub skip_native_patching: bool,
    pub glfw_path: String,
    pub openal_path: String,
}

impl Default for LaunchSettings {
    fn default() -> Self {
        Self {
            memory_mode: MemoryMode::Manual,
            memory: 4,
            window_mode: WindowMode::Windowed,
            window_width: 1280,
            window_height: 720,
            quick_play_mode: QuickPlayMode::None,
            quick_play_target: String::new(),
            game_args: String::new(),
            environment_variables: String::new(),
            java_args: "-XX:+UseG1GC -XX:MaxGCPauseMillis=150".into(),
            use_default_jvm_args: true,
            use_optimized_jvm_args: true,
            skip_jvm_validation: false,
            allow_auto_agent: false,
            skip_integrity_check: false,
            pre_launch_command: String::new(),
            command_wrapper: String::new(),
            post_exit_command: String::new(),
            use_custom_natives: false,
            natives_path: String::new(),
            skip_native_patching: false,
            glfw_path: String::new(),
            openal_path: String::new(),
        }
    }
}

impl LaunchSettings {
    pub fn get(&self, field: LaunchField) -> LaunchValue {
        match field {
            LaunchField::Memory => LaunchValue::Number(self.memory),
            LaunchField::MemoryMode => LaunchValue::Memory(self.memory_mode),
            LaunchField::WindowMode => LaunchValue::Window(self.window_mode),
            LaunchField::WindowWidth => LaunchValue::Number(self.window_width),
            LaunchField::WindowHeight => LaunchValue::Number(self.window_height),
            LaunchField::QuickPlayMode => LaunchValue::QuickPlay(self.quick_play_mode),
            LaunchField::QuickPlayTarget => LaunchValue::Text(self.quick_play_target.clone()),
            LaunchField::GameArgs => LaunchValue::Text(self.game_args.clone()),
            LaunchField::EnvironmentVariables => {
                LaunchValue::Text(self.environment_variables.clone())
            }
            LaunchField::JavaArgs => LaunchValue::Text(self.java_args.clone()),
            LaunchField::PreLaunchCommand => LaunchValue::Text(self.pre_launch_command.clone()),
            LaunchField::CommandWrapper => LaunchValue::Text(self.command_wrapper.clone()),
            LaunchField::PostExitCommand => LaunchValue::Text(self.post_exit_command.clone()),
            LaunchField::NativesPath => LaunchValue::Text(self.natives_path.clone()),
            LaunchField::GlfwPath => LaunchValue::Text(self.glfw_path.clone()),
            LaunchField::OpenalPath => LaunchValue::Text(self.openal_path.clone()),
            LaunchField::UseDefaultJvmArgs => LaunchValue::Flag(self.use_default_jvm_args),
            LaunchField::UseOptimizedJvmArgs => LaunchValue::Flag(self.use_optimized_jvm_args),
            LaunchField::SkipJvmValidation => LaunchValue::Flag(self.skip_jvm_validation),
            LaunchField::AllowAutoAgent => LaunchValue::Flag(self.allow_auto_agent),
            LaunchField::SkipIntegrityCheck => LaunchValue::Flag(self.skip_integrity_check),
            LaunchField::UseCustomNatives => LaunchValue::Flag(self.use_custom_natives),
            LaunchField::SkipNativePatching => LaunchValue::Flag(self.skip_native_patching),
        }
    }

    /// Applies one field. Values of the wrong shape are ignored, so a caller
    /// cannot silently turn a window mode into a number.
    pub fn set(&mut self, field: LaunchField, value: LaunchValue) {
        match (field, value) {
            (LaunchField::Memory, LaunchValue::Number(value)) => self.memory = value,
            (LaunchField::MemoryMode, LaunchValue::Memory(value)) => self.memory_mode = value,
            (LaunchField::WindowMode, LaunchValue::Window(value)) => self.window_mode = value,
            (LaunchField::WindowWidth, LaunchValue::Number(value)) => self.window_width = value,
            (LaunchField::WindowHeight, LaunchValue::Number(value)) => self.window_height = value,
            (LaunchField::QuickPlayMode, LaunchValue::QuickPlay(value)) => {
                self.quick_play_mode = value
            }
            (LaunchField::QuickPlayTarget, LaunchValue::Text(value)) => {
                self.quick_play_target = value
            }
            (LaunchField::GameArgs, LaunchValue::Text(value)) => self.game_args = value,
            (LaunchField::EnvironmentVariables, LaunchValue::Text(value)) => {
                self.environment_variables = value
            }
            (LaunchField::JavaArgs, LaunchValue::Text(value)) => self.java_args = value,
            (LaunchField::PreLaunchCommand, LaunchValue::Text(value)) => {
                self.pre_launch_command = value
            }
            (LaunchField::CommandWrapper, LaunchValue::Text(value)) => self.command_wrapper = value,
            (LaunchField::PostExitCommand, LaunchValue::Text(value)) => {
                self.post_exit_command = value
            }
            (LaunchField::NativesPath, LaunchValue::Text(value)) => self.natives_path = value,
            (LaunchField::GlfwPath, LaunchValue::Text(value)) => self.glfw_path = value,
            (LaunchField::OpenalPath, LaunchValue::Text(value)) => self.openal_path = value,
            (LaunchField::UseDefaultJvmArgs, LaunchValue::Flag(value)) => {
                self.use_default_jvm_args = value
            }
            (LaunchField::UseOptimizedJvmArgs, LaunchValue::Flag(value)) => {
                self.use_optimized_jvm_args = value
            }
            (LaunchField::SkipJvmValidation, LaunchValue::Flag(value)) => {
                self.skip_jvm_validation = value
            }
            (LaunchField::AllowAutoAgent, LaunchValue::Flag(value)) => {
                self.allow_auto_agent = value
            }
            (LaunchField::SkipIntegrityCheck, LaunchValue::Flag(value)) => {
                self.skip_integrity_check = value
            }
            (LaunchField::UseCustomNatives, LaunchValue::Flag(value)) => {
                self.use_custom_natives = value
            }
            (LaunchField::SkipNativePatching, LaunchValue::Flag(value)) => {
                self.skip_native_patching = value
            }
            _ => {}
        }
    }
}

/// Per-instance overrides of the global launch settings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaunchOverrides(BTreeMap<LaunchField, LaunchValue>);

impl LaunchOverrides {
    pub fn is_overridden(&self, field: LaunchField) -> bool {
        self.0.contains_key(&field)
    }

    pub fn get(&self, field: LaunchField) -> Option<&LaunchValue> {
        self.0.get(&field)
    }

    /// Returns `true` when the stored value actually changed.
    pub fn set(&mut self, field: LaunchField, value: LaunchValue) -> bool {
        if self.0.get(&field) == Some(&value) {
            return false;
        }
        self.0.insert(field, value);
        true
    }

    pub fn clear(&mut self, field: LaunchField) -> bool {
        self.0.remove(&field).is_some()
    }

    /// The override keys, in field order — this is what the crash report shows
    /// as "Launch settings overridden".
    pub fn keys(&self) -> Vec<&'static str> {
        self.0.keys().map(|field| field.key()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Global defaults with this instance's overrides applied on top.
    pub fn resolve(&self, base: &LaunchSettings) -> LaunchSettings {
        let mut resolved = base.clone();
        for (field, value) in &self.0 {
            resolved.set(*field, value.clone());
        }
        resolved
    }
}

/// The stages a launch moves through, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LaunchPhase {
    Preparing,
    Resolving,
    DownloadingAssets,
    DownloadingLibraries,
    ExtractingNatives,
    Launching,
}

impl LaunchPhase {
    pub const ALL: [LaunchPhase; 6] = [
        LaunchPhase::Preparing,
        LaunchPhase::Resolving,
        LaunchPhase::DownloadingAssets,
        LaunchPhase::DownloadingLibraries,
        LaunchPhase::ExtractingNatives,
        LaunchPhase::Launching,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LaunchPhase::Preparing => "Preparing environment",
            LaunchPhase::Resolving => "Resolving versions",
            LaunchPhase::DownloadingAssets => "Downloading assets",
            LaunchPhase::DownloadingLibraries => "Downloading libraries",
            LaunchPhase::ExtractingNatives => "Extracting natives",
            LaunchPhase::Launching => "Launching game",
        }
    }

    /// Upper progress bound of each phase, matching `PHASE_AT` in the prototype.
    pub fn upper_bound(self) -> f32 {
        match self {
            LaunchPhase::Preparing => 10.0,
            LaunchPhase::Resolving => 22.0,
            LaunchPhase::DownloadingAssets => 62.0,
            LaunchPhase::DownloadingLibraries => 84.0,
            LaunchPhase::ExtractingNatives => 94.0,
            LaunchPhase::Launching => 100.0,
        }
    }

    /// The phase a given percentage falls into.
    pub fn for_progress(progress: f32) -> LaunchPhase {
        LaunchPhase::ALL
            .into_iter()
            .find(|phase| progress < phase.upper_bound())
            .unwrap_or(LaunchPhase::Launching)
    }
}

/// One in-flight launch. Multiple jobs may run at once.
#[derive(Clone, Debug, PartialEq)]
pub struct LaunchJob {
    pub instance: InstanceRef,
    pub name: String,
    pub icon_seed: String,
    pub loader: super::Loader,
    /// 0..=100.
    pub progress: f32,
    pub phase: LaunchPhase,
    pub error: Option<String>,
    pub cancellable: bool,
}

impl LaunchJob {
    pub fn new(
        instance: InstanceRef,
        name: impl Into<String>,
        icon_seed: impl Into<String>,
        loader: super::Loader,
    ) -> Self {
        Self {
            instance,
            name: name.into(),
            icon_seed: icon_seed.into(),
            loader,
            progress: 0.0,
            phase: LaunchPhase::Preparing,
            error: None,
            cancellable: true,
        }
    }
}

/// High-frequency launch state. No low-frequency data lives here, and neither
/// the navigation sidebar nor the status bar observes it.
#[derive(Default)]
pub struct LaunchStore {
    jobs: Vec<LaunchJob>,
    revision: u64,
}

impl LaunchStore {
    pub fn all(&self) -> &[LaunchJob] {
        &self.jobs
    }

    pub fn get(&self, instance: &InstanceRef) -> Option<&LaunchJob> {
        self.jobs.iter().find(|job| &job.instance == instance)
    }

    pub fn is_launching(&self, instance: &InstanceRef) -> bool {
        self.get(instance).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Starting the same instance twice is a no-op with a visible existing job.
    pub fn start(&mut self, job: LaunchJob) -> bool {
        if self.is_launching(&job.instance) {
            return false;
        }
        self.jobs.push(job);
        self.revision += 1;
        true
    }

    pub fn update(&mut self, instance: &InstanceRef, phase: LaunchPhase, progress: f32) -> bool {
        if let Some(job) = self.jobs.iter_mut().find(|job| &job.instance == instance) {
            if job.phase == phase && (job.progress - progress).abs() < f32::EPSILON {
                return false;
            }
            job.phase = phase;
            job.progress = progress;
            self.revision += 1;
            return true;
        }
        false
    }

    pub fn fail(&mut self, instance: &InstanceRef, message: impl Into<String>) -> bool {
        let message = message.into();
        if let Some(job) = self.jobs.iter_mut().find(|job| &job.instance == instance) {
            if job.error.as_deref() == Some(message.as_str()) {
                return false;
            }
            job.error = Some(message);
            job.cancellable = false;
            self.revision += 1;
            return true;
        }
        false
    }

    pub fn finish(&mut self, instance: &InstanceRef) -> bool {
        let before = self.jobs.len();
        self.jobs.retain(|job| &job.instance != instance);
        if self.jobs.len() != before {
            self.revision += 1;
            return true;
        }
        false
    }

    /// Drops every job for an instance that no longer exists.
    pub fn remove_instance(&mut self, instance: &InstanceRef) -> bool {
        self.finish(instance)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}
