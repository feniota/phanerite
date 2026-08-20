use crate::parsers::options::editor::parse_config_line;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid options value")]
    Parse(#[from] std::num::ParseIntError),

    #[error("invalid options float")]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error("invalid options bool")]
    ParseBool(#[from] std::str::ParseBoolError),
}

#[derive(Clone, Debug)]
pub struct Options {
    pub version: u32,

    // Graphics
    pub ao: bool,
    pub biome_blend_radius: u32,
    pub chunk_section_fade_in_time: f32,
    pub cutout_leaves: bool,
    pub enable_vsync: bool,
    pub entity_distance_scaling: f32,
    pub entity_shadows: bool,
    pub force_unicode_font: bool,
    pub japanese_glyph_variants: bool,
    pub fov: f32,
    pub fov_effect_scale: f32,
    pub darkness_effect_scale: f32,
    pub glint_speed: f32,
    pub glint_strength: f32,
    pub preferred_graphics_backend: String,
    pub graphics_preset: String,
    pub prioritize_chunk_updates: u32,
    pub fullscreen: bool,
    pub exclusive_fullscreen: bool,
    pub gamma: f32,
    pub gui_scale: u32,
    pub max_anisotropy_bit: u32,
    pub texture_filtering: u32,
    pub max_fps: u32,
    pub improved_transparency: bool,
    pub inactivity_fps_limit: String,
    pub mipmap_levels: u32,
    pub narrator: u32,
    pub particles: u32,
    pub reduced_debug_info: bool,
    pub render_clouds: String,
    pub cloud_range: u32,
    pub render_distance: u32,
    pub simulation_distance: u32,
    pub screen_effect_scale: f32,
    pub sound_device: String,
    pub vignette: bool,
    pub weather_radius: u32,

    // Controls
    pub auto_jump: bool,
    pub rotate_with_minecart: bool,
    pub operator_items_tab: bool,
    pub auto_suggestions: bool,
    pub chat_colors: bool,
    pub chat_links: bool,
    pub chat_links_prompt: bool,
    pub discrete_mouse_scroll: bool,
    pub invert_x_mouse: bool,
    pub invert_y_mouse: bool,
    pub realms_notifications: bool,
    pub show_subtitles: bool,
    pub directional_audio: bool,
    pub bob_view: bool,
    pub toggle_crouch: bool,
    pub toggle_sprint: bool,
    pub toggle_attack: bool,
    pub toggle_use: bool,
    pub sprint_window: u32,
    pub mouse_sensitivity: f32,
    pub damage_tilt_strength: f32,
    pub raw_mouse_input: bool,
    pub mouse_wheel_sensitivity: f32,
    pub allow_cursor_changes: bool,

    // Accessibility / UI
    pub dark_mojang_studios_background: bool,
    pub hide_lightning_flashes: bool,
    pub hide_splash_texts: bool,
    pub high_contrast: bool,
    pub high_contrast_block_outline: bool,
    pub narrator_hotkey: bool,
    pub menu_background_blurriness: u32,
    pub onboard_accessibility: bool,

    // Chat
    pub lang: String,
    pub chat_visibility: u32,
    pub chat_opacity: f32,
    pub chat_line_spacing: f32,
    pub text_background_opacity: f32,
    pub background_for_chat_only: bool,
    pub hide_server_address: bool,
    pub advanced_item_tooltips: bool,
    pub pause_on_lost_focus: bool,
    pub override_width: u32,
    pub override_height: u32,
    pub chat_height_focused: f32,
    pub chat_delay: f32,
    pub chat_height_unfocused: f32,
    pub chat_scale: f32,
    pub chat_width: f32,
    pub notification_display_time: f32,

    // Misc
    pub use_native_transport: bool,
    pub main_hand: String,
    pub attack_indicator: u32,
    pub tutorial_step: String,
    pub gl_debug_verbosity: u32,
    pub skip_multiplayer_warning: bool,
    pub hide_matched_names: bool,
    pub joined_first_server: bool,
    pub sync_chunk_writes: bool,
    pub show_autosave_indicator: bool,
    pub allow_server_listing: bool,
    pub in_game_notification: bool,
    pub share_presence: String,
    pub only_show_secure_chat: bool,
    pub save_chat_drafts: bool,
    pub panorama_scroll_speed: f32,
    pub telemetry_opt_in_extra: bool,
    pub started_cleanly: bool,
    pub music_toast: String,
    pub music_frequency: String,

    // Dynamic options
    pub key_bindings: HashMap<String, String>,
    pub sound_categories: HashMap<String, f32>,
    pub model_parts: HashMap<String, bool>,

    /// 未识别的配置项。
    ///
    /// 用于兼容未来 Minecraft 版本新增的配置。
    pub other: HashMap<String, String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            version: 0,

            ao: true,
            biome_blend_radius: 2,
            chunk_section_fade_in_time: 0.75,
            cutout_leaves: true,
            enable_vsync: true,
            entity_distance_scaling: 1.0,
            entity_shadows: true,
            force_unicode_font: false,
            japanese_glyph_variants: false,
            fov: 0.0,
            fov_effect_scale: 1.0,
            darkness_effect_scale: 1.0,
            glint_speed: 0.5,
            glint_strength: 0.75,
            preferred_graphics_backend: "default".into(),
            graphics_preset: "fancy".into(),
            prioritize_chunk_updates: 1,
            fullscreen: false,
            exclusive_fullscreen: false,
            gamma: 0.5,
            gui_scale: 0,
            max_anisotropy_bit: 1,
            texture_filtering: 1,
            max_fps: 120,
            improved_transparency: false,
            inactivity_fps_limit: "afk".into(),
            mipmap_levels: 4,
            narrator: 0,
            particles: 0,
            reduced_debug_info: false,
            render_clouds: "true".into(),
            cloud_range: 64,
            render_distance: 16,
            simulation_distance: 12,
            screen_effect_scale: 1.0,
            sound_device: String::new(),
            vignette: true,
            weather_radius: 10,

            auto_jump: false,
            rotate_with_minecart: false,
            operator_items_tab: false,
            auto_suggestions: true,
            chat_colors: true,
            chat_links: true,
            chat_links_prompt: true,
            discrete_mouse_scroll: false,
            invert_x_mouse: false,
            invert_y_mouse: false,
            realms_notifications: true,
            show_subtitles: false,
            directional_audio: false,
            bob_view: true,
            toggle_crouch: false,
            toggle_sprint: false,
            toggle_attack: false,
            toggle_use: false,
            sprint_window: 7,
            mouse_sensitivity: 0.5,
            damage_tilt_strength: 1.0,
            raw_mouse_input: true,
            mouse_wheel_sensitivity: 1.0,
            allow_cursor_changes: true,

            dark_mojang_studios_background: false,
            hide_lightning_flashes: false,
            hide_splash_texts: false,
            high_contrast: false,
            high_contrast_block_outline: false,
            narrator_hotkey: true,
            menu_background_blurriness: 5,
            onboard_accessibility: true,

            lang: "en_us".into(),
            chat_visibility: 0,
            chat_opacity: 1.0,
            chat_line_spacing: 0.0,
            text_background_opacity: 0.5,
            background_for_chat_only: true,
            hide_server_address: false,
            advanced_item_tooltips: false,
            pause_on_lost_focus: true,
            override_width: 0,
            override_height: 0,
            chat_height_focused: 1.0,
            chat_delay: 0.0,
            chat_height_unfocused: 0.4375,
            chat_scale: 1.0,
            chat_width: 1.0,
            notification_display_time: 1.0,

            use_native_transport: true,
            main_hand: "right".into(),
            attack_indicator: 1,
            tutorial_step: "movement".into(),
            gl_debug_verbosity: 1,
            skip_multiplayer_warning: false,
            hide_matched_names: true,
            joined_first_server: false,
            sync_chunk_writes: true,
            show_autosave_indicator: true,
            allow_server_listing: true,
            in_game_notification: false,
            share_presence: "all".into(),
            only_show_secure_chat: false,
            save_chat_drafts: false,
            panorama_scroll_speed: 1.0,
            telemetry_opt_in_extra: false,
            started_cleanly: true,
            music_toast: "never".into(),
            music_frequency: "DEFAULT".into(),

            key_bindings: HashMap::new(),
            sound_categories: HashMap::new(),
            model_parts: HashMap::new(),
            other: HashMap::new(),
        }
    }
}

impl FromStr for Options {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let mut options = Self::default();

        for line in s.lines() {
            let Some((key, value)) = parse_config_line(line) else {
                continue;
            };

            match key {
                "version" => options.version = value.parse()?,

                "ao" => options.ao = value.parse()?,
                "biomeBlendRadius" => options.biome_blend_radius = value.parse()?,
                "chunkSectionFadeInTime" => options.chunk_section_fade_in_time = value.parse()?,
                "cutoutLeaves" => options.cutout_leaves = value.parse()?,
                "enableVsync" => options.enable_vsync = value.parse()?,
                "entityDistanceScaling" => options.entity_distance_scaling = value.parse()?,
                "entityShadows" => options.entity_shadows = value.parse()?,
                "forceUnicodeFont" => options.force_unicode_font = value.parse()?,
                "japaneseGlyphVariants" => options.japanese_glyph_variants = value.parse()?,
                "fov" => options.fov = value.parse()?,
                "fovEffectScale" => options.fov_effect_scale = value.parse()?,
                "darknessEffectScale" => options.darkness_effect_scale = value.parse()?,
                "glintSpeed" => options.glint_speed = value.parse()?,
                "glintStrength" => options.glint_strength = value.parse()?,

                "preferredGraphicsBackend" => {
                    options.preferred_graphics_backend = parse_string(value).to_owned()
                }
                "graphicsPreset" => options.graphics_preset = parse_string(value).to_owned(),
                "prioritizeChunkUpdates" => options.prioritize_chunk_updates = value.parse()?,
                "fullscreen" => options.fullscreen = value.parse()?,
                "exclusiveFullscreen" => options.exclusive_fullscreen = value.parse()?,
                "gamma" => options.gamma = value.parse()?,
                "guiScale" => options.gui_scale = value.parse()?,
                "maxAnisotropyBit" => options.max_anisotropy_bit = value.parse()?,
                "textureFiltering" => options.texture_filtering = value.parse()?,
                "maxFps" => options.max_fps = value.parse()?,
                "improvedTransparency" => options.improved_transparency = value.parse()?,
                "inactivityFpsLimit" => {
                    options.inactivity_fps_limit = parse_string(value).to_owned()
                }
                "mipmapLevels" => options.mipmap_levels = value.parse()?,
                "narrator" => options.narrator = value.parse()?,
                "particles" => options.particles = value.parse()?,
                "reducedDebugInfo" => options.reduced_debug_info = value.parse()?,
                "renderClouds" => options.render_clouds = parse_string(value).to_owned(),
                "cloudRange" => options.cloud_range = value.parse()?,
                "renderDistance" => options.render_distance = value.parse()?,
                "simulationDistance" => options.simulation_distance = value.parse()?,
                "screenEffectScale" => options.screen_effect_scale = value.parse()?,
                "soundDevice" => options.sound_device = parse_string(value).to_owned(),
                "vignette" => options.vignette = value.parse()?,
                "weatherRadius" => options.weather_radius = value.parse()?,

                "autoJump" => options.auto_jump = value.parse()?,
                "rotateWithMinecart" => options.rotate_with_minecart = value.parse()?,
                "operatorItemsTab" => options.operator_items_tab = value.parse()?,
                "autoSuggestions" => options.auto_suggestions = value.parse()?,
                "chatColors" => options.chat_colors = value.parse()?,
                "chatLinks" => options.chat_links = value.parse()?,
                "chatLinksPrompt" => options.chat_links_prompt = value.parse()?,
                "discrete_mouse_scroll" => options.discrete_mouse_scroll = value.parse()?,
                "invertXMouse" => options.invert_x_mouse = value.parse()?,
                "invertYMouse" => options.invert_y_mouse = value.parse()?,
                "realmsNotifications" => options.realms_notifications = value.parse()?,
                "showSubtitles" => options.show_subtitles = value.parse()?,
                "directionalAudio" => options.directional_audio = value.parse()?,
                "bobView" => options.bob_view = value.parse()?,
                "toggleCrouch" => options.toggle_crouch = value.parse()?,
                "toggleSprint" => options.toggle_sprint = value.parse()?,
                "toggleAttack" => options.toggle_attack = value.parse()?,
                "toggleUse" => options.toggle_use = value.parse()?,
                "sprintWindow" => options.sprint_window = value.parse()?,
                "mouseSensitivity" => options.mouse_sensitivity = value.parse()?,
                "damageTiltStrength" => options.damage_tilt_strength = value.parse()?,

                "darkMojangStudiosBackground" => {
                    options.dark_mojang_studios_background = value.parse()?
                }
                "hideLightningFlashes" => options.hide_lightning_flashes = value.parse()?,
                "hideSplashTexts" => options.hide_splash_texts = value.parse()?,
                "highContrast" => options.high_contrast = value.parse()?,
                "highContrastBlockOutline" => {
                    options.high_contrast_block_outline = value.parse()?
                }
                "narratorHotkey" => options.narrator_hotkey = value.parse()?,
                "menuBackgroundBlurriness" => options.menu_background_blurriness = value.parse()?,
                "onboardAccessibility" => options.onboard_accessibility = value.parse()?,

                "lang" => options.lang = parse_string(value).to_owned(),
                "chatVisibility" => options.chat_visibility = value.parse()?,
                "chatOpacity" => options.chat_opacity = value.parse()?,
                "chatLineSpacing" => options.chat_line_spacing = value.parse()?,
                "textBackgroundOpacity" => options.text_background_opacity = value.parse()?,
                "backgroundForChatOnly" => options.background_for_chat_only = value.parse()?,
                "hideServerAddress" => options.hide_server_address = value.parse()?,
                "advancedItemTooltips" => options.advanced_item_tooltips = value.parse()?,
                "pauseOnLostFocus" => options.pause_on_lost_focus = value.parse()?,
                "overrideWidth" => options.override_width = value.parse()?,
                "overrideHeight" => options.override_height = value.parse()?,
                "chatHeightFocused" => options.chat_height_focused = value.parse()?,
                "chatDelay" => options.chat_delay = value.parse()?,
                "chatHeightUnfocused" => options.chat_height_unfocused = value.parse()?,
                "chatScale" => options.chat_scale = value.parse()?,
                "chatWidth" => options.chat_width = value.parse()?,
                "notificationDisplayTime" => options.notification_display_time = value.parse()?,

                "useNativeTransport" => options.use_native_transport = value.parse()?,
                "mainHand" => options.main_hand = parse_string(value).to_owned(),
                "attackIndicator" => options.attack_indicator = value.parse()?,
                "tutorialStep" => options.tutorial_step = value.to_owned(),
                "mouseWheelSensitivity" => options.mouse_wheel_sensitivity = value.parse()?,
                "rawMouseInput" => options.raw_mouse_input = value.parse()?,
                "allowCursorChanges" => options.allow_cursor_changes = value.parse()?,
                "glDebugVerbosity" => options.gl_debug_verbosity = value.parse()?,
                "skipMultiplayerWarning" => options.skip_multiplayer_warning = value.parse()?,
                "hideMatchedNames" => options.hide_matched_names = value.parse()?,
                "joinedFirstServer" => options.joined_first_server = value.parse()?,
                "syncChunkWrites" => options.sync_chunk_writes = value.parse()?,
                "showAutosaveIndicator" => options.show_autosave_indicator = value.parse()?,
                "allowServerListing" => options.allow_server_listing = value.parse()?,
                "inGameNotification" => options.in_game_notification = value.parse()?,

                "sharePresence" => options.share_presence = parse_string(value).to_owned(),
                "onlyShowSecureChat" => options.only_show_secure_chat = value.parse()?,
                "saveChatDrafts" => options.save_chat_drafts = value.parse()?,
                "panoramaScrollSpeed" => options.panorama_scroll_speed = value.parse()?,
                "telemetryOptInExtra" => options.telemetry_opt_in_extra = value.parse()?,
                "startedCleanly" => options.started_cleanly = value.parse()?,
                "musicToast" => options.music_toast = parse_string(value).to_owned(),
                "musicFrequency" => options.music_frequency = parse_string(value).to_owned(),

                key if key.starts_with("key_") => {
                    options
                        .key_bindings
                        .insert(key.to_owned(), parse_string(value).to_owned());
                }

                key if key.starts_with("soundCategory_") => {
                    options
                        .sound_categories
                        .insert(key.to_owned(), value.parse()?);
                }

                key if key.starts_with("modelPart_") => {
                    options.model_parts.insert(key.to_owned(), value.parse()?);
                }

                _ => {
                    options.other.insert(key.to_owned(), value.to_owned());
                }
            }
        }

        Ok(options)
    }
}

impl Options {
    pub fn diff(&self, new: &Self) -> Vec<(String, String)> {
        let mut diff = Vec::new();

        macro_rules! check {
            ($field:ident, $key:literal) => {
                if self.$field != new.$field {
                    diff.push(($key.to_owned(), new.$field.to_string()));
                }
            };
        }

        check!(version, "version");
        check!(ao, "ao");
        check!(biome_blend_radius, "biomeBlendRadius");
        check!(chunk_section_fade_in_time, "chunkSectionFadeInTime");
        check!(cutout_leaves, "cutoutLeaves");
        check!(enable_vsync, "enableVsync");
        check!(entity_distance_scaling, "entityDistanceScaling");
        check!(entity_shadows, "entityShadows");
        check!(force_unicode_font, "forceUnicodeFont");
        check!(japanese_glyph_variants, "japaneseGlyphVariants");
        check!(fov, "fov");
        check!(fov_effect_scale, "fovEffectScale");
        check!(darkness_effect_scale, "darknessEffectScale");
        check!(glint_speed, "glintSpeed");
        check!(glint_strength, "glintStrength");
        check!(preferred_graphics_backend, "preferredGraphicsBackend");
        check!(graphics_preset, "graphicsPreset");
        check!(prioritize_chunk_updates, "prioritizeChunkUpdates");
        check!(fullscreen, "fullscreen");
        check!(exclusive_fullscreen, "exclusiveFullscreen");
        check!(gamma, "gamma");
        check!(gui_scale, "guiScale");
        check!(max_anisotropy_bit, "maxAnisotropyBit");
        check!(texture_filtering, "textureFiltering");
        check!(max_fps, "maxFps");
        check!(improved_transparency, "improvedTransparency");
        check!(inactivity_fps_limit, "inactivityFpsLimit");
        check!(mipmap_levels, "mipmapLevels");
        check!(narrator, "narrator");
        check!(particles, "particles");
        check!(reduced_debug_info, "reducedDebugInfo");
        check!(render_clouds, "renderClouds");
        check!(cloud_range, "cloudRange");
        check!(render_distance, "renderDistance");
        check!(simulation_distance, "simulationDistance");
        check!(screen_effect_scale, "screenEffectScale");
        check!(sound_device, "soundDevice");
        check!(vignette, "vignette");
        check!(weather_radius, "weatherRadius");

        check!(auto_jump, "autoJump");
        check!(rotate_with_minecart, "rotateWithMinecart");
        check!(operator_items_tab, "operatorItemsTab");
        check!(auto_suggestions, "autoSuggestions");
        check!(chat_colors, "chatColors");
        check!(chat_links, "chatLinks");
        check!(chat_links_prompt, "chatLinksPrompt");
        check!(discrete_mouse_scroll, "discrete_mouse_scroll");
        check!(invert_x_mouse, "invertXMouse");
        check!(invert_y_mouse, "invertYMouse");
        check!(realms_notifications, "realmsNotifications");
        check!(show_subtitles, "showSubtitles");
        check!(directional_audio, "directionalAudio");
        check!(bob_view, "bobView");
        check!(toggle_crouch, "toggleCrouch");
        check!(toggle_sprint, "toggleSprint");
        check!(toggle_attack, "toggleAttack");
        check!(toggle_use, "toggleUse");
        check!(sprint_window, "sprintWindow");
        check!(mouse_sensitivity, "mouseSensitivity");
        check!(damage_tilt_strength, "damageTiltStrength");

        check!(
            dark_mojang_studios_background,
            "darkMojangStudiosBackground"
        );
        check!(hide_lightning_flashes, "hideLightningFlashes");
        check!(hide_splash_texts, "hideSplashTexts");
        check!(high_contrast, "highContrast");
        check!(high_contrast_block_outline, "highContrastBlockOutline");
        check!(narrator_hotkey, "narratorHotkey");
        check!(menu_background_blurriness, "menuBackgroundBlurriness");
        check!(onboard_accessibility, "onboardAccessibility");

        check!(lang, "lang");
        check!(chat_visibility, "chatVisibility");
        check!(chat_opacity, "chatOpacity");
        check!(chat_line_spacing, "chatLineSpacing");
        check!(text_background_opacity, "textBackgroundOpacity");
        check!(background_for_chat_only, "backgroundForChatOnly");
        check!(hide_server_address, "hideServerAddress");
        check!(advanced_item_tooltips, "advancedItemTooltips");
        check!(pause_on_lost_focus, "pauseOnLostFocus");
        check!(override_width, "overrideWidth");
        check!(override_height, "overrideHeight");
        check!(chat_height_focused, "chatHeightFocused");
        check!(chat_delay, "chatDelay");
        check!(chat_height_unfocused, "chatHeightUnfocused");
        check!(chat_scale, "chatScale");
        check!(chat_width, "chatWidth");
        check!(notification_display_time, "notificationDisplayTime");

        check!(use_native_transport, "useNativeTransport");
        check!(main_hand, "mainHand");
        check!(attack_indicator, "attackIndicator");
        check!(tutorial_step, "tutorialStep");
        check!(mouse_wheel_sensitivity, "mouseWheelSensitivity");
        check!(raw_mouse_input, "rawMouseInput");
        check!(allow_cursor_changes, "allowCursorChanges");
        check!(gl_debug_verbosity, "glDebugVerbosity");
        check!(skip_multiplayer_warning, "skipMultiplayerWarning");
        check!(hide_matched_names, "hideMatchedNames");
        check!(joined_first_server, "joinedFirstServer");
        check!(sync_chunk_writes, "syncChunkWrites");
        check!(show_autosave_indicator, "showAutosaveIndicator");
        check!(allow_server_listing, "allowServerListing");
        check!(in_game_notification, "inGameNotification");
        check!(share_presence, "sharePresence");
        check!(only_show_secure_chat, "onlyShowSecureChat");
        check!(save_chat_drafts, "saveChatDrafts");
        check!(panorama_scroll_speed, "panoramaScrollSpeed");
        check!(telemetry_opt_in_extra, "telemetryOptInExtra");
        check!(started_cleanly, "startedCleanly");
        check!(music_toast, "musicToast");
        check!(music_frequency, "musicFrequency");

        for (key, value) in &new.key_bindings {
            if self.key_bindings.get(key) != Some(value) {
                diff.push((key.clone(), value.clone()));
            }
        }

        for (key, value) in &new.sound_categories {
            if self.sound_categories.get(key) != Some(value) {
                diff.push((key.clone(), value.to_string()));
            }
        }

        for (key, value) in &new.model_parts {
            if self.model_parts.get(key) != Some(value) {
                diff.push((key.clone(), value.to_string()));
            }
        }

        for (key, value) in &new.other {
            if self.other.get(key) != Some(value) {
                diff.push((key.clone(), value.clone()));
            }
        }

        diff
    }
}

fn parse_string(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}
