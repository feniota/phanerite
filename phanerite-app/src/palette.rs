//! Phanerite color tokens, accent presets, and GPUI color conversions.

// Source values come from design/src/routes/layout.css. OKLCH is the source
// of truth; these are offline OKLab -> linear sRGB -> gamma conversions.
pub mod token {
    pub const BACKGROUND: u32 = 0x090B09; // oklch(.145 .005 150)
    pub const FOREGROUND: u32 = 0xEFF1EF; // oklch(.955 .003 150)
    pub const CARD: u32 = 0x121512; // oklch(.19 .006 150)
    pub const POPOVER: u32 = 0x161917; // oklch(.21 .007 150)
    pub const PRIMARY: u32 = 0x349D62; // oklch(.62 .13 155)
    pub const PRIMARY_FOREGROUND: u32 = 0x04130A; // oklch(.17 .03 155)
    pub const LAUNCH: u32 = 0x2B5139; // oklch(.4 .06 155)
    pub const LAUNCH_FOREGROUND: u32 = 0xE6EDE8; // oklch(.94 .01 155)
    pub const SECONDARY: u32 = 0x1D201D; // oklch(.24 .008 150)
    pub const MUTED: u32 = 0x191B19; // oklch(.22 .006 150)
    pub const MUTED_FOREGROUND: u32 = 0x979D98; // oklch(.69 .01 150)
    pub const MUTED_FOREGROUND_SUBTLE: u32 = 0x7C827D; // oklch(.6 .01 150)
    pub const ACCENT: u32 = 0x212522; // oklch(.26 .009 150)
    pub const ACCENT_FOREGROUND: u32 = 0xF4F6F4; // oklch(.97 .003 150)
    pub const DESTRUCTIVE: u32 = 0xF05653; // oklch(.66 .19 25)
    pub const SIDEBAR: u32 = 0x0F110F; // oklch(.175 .005 150)
    pub const CHART_2: u32 = 0x979D98; // oklch(.69 .01 150)
    pub const CHART_3: u32 = 0x3A6343; // oklch(.46 .07 150)
    pub const CHART_4: u32 = 0xDCAF61; // oklch(.78 .11 80)
    pub const CHART_5: u32 = 0x3E5B44; // oklch(.44 .05 150)
    pub const GOLD: u32 = 0xDCAF61; // oklch(.78 .11 80)

    /// `red-500`, used only for the Aphanite flame mark, as in the prototype.
    pub const FLAME: u32 = 0xEF4444;
    /// `yellow-500`, used only for warning copy and warning log lines.
    pub const WARNING: u32 = 0xEAB308;
    /// Terminal-style surface behind log and crash output (`bg-black/40`).
    pub const TERMINAL: u32 = 0x000000;

    /// `--border: oklch(1 0 0 / 9%)`
    pub const BORDER_ALPHA: u8 = 0x17;
    /// `--input: oklch(1 0 0 / 12%)`
    pub const INPUT_ALPHA: u8 = 0x1F;
    /// Scrollbar thumb, `oklch(1 0 0 / 14%)`.
    pub const SCROLLBAR_ALPHA: u8 = 0x24;
}

pub mod mc {
    pub const GRASS: u32 = 0x4A893B; // oklch(.57 .13 140)
    pub const GRASS_DARK: u32 = 0x316A23; // oklch(.47 .12 140)
    pub const DIRT: u32 = 0x613F27; // oklch(.4 .06 55)
    pub const DIRT_DARK: u32 = 0x442916; // oklch(.31 .05 55)
    pub const STONE: u32 = 0x6B6C6A; // oklch(.53 .004 120)
    pub const STONE_DARK: u32 = 0x4D4D4B; // oklch(.42 .004 120)
    pub const SAND: u32 = 0xC2A464; // oklch(.73 .09 85)
    pub const SAND_DARK: u32 = 0x9F8242; // oklch(.62 .09 85)
    pub const WOOD: u32 = 0x754E2E; // oklch(.46 .07 60)
    pub const WOOD_DARK: u32 = 0x513217; // oklch(.35 .06 60)
    pub const NETHER: u32 = 0x6E282B; // oklch(.38 .1 20)
    pub const NETHER_DARK: u32 = 0x4C0F15; // oklch(.28 .09 20)
    pub const LAVA: u32 = 0xF3680F; // oklch(.68 .19 45)
    pub const END: u32 = 0x44374F; // oklch(.36 .045 310)
    pub const END_DARK: u32 = 0x281B32; // oklch(.25 .045 310)
}

/// Per-loader single-hue ramps, darkest first. Index 0..3 is the shading scale
/// used by the pixel crystal artwork; see `design/src/lib/instance-shapes.ts`.
pub mod ramp {
    pub const VANILLA: [u32; 4] = [0x104F29, 0x21763C, 0x3F9C53, 0x77C77E];
    pub const FABRIC: [u32; 4] = [0x47413C, 0x6A625A, 0x8E8479, 0xBAAFA2];
    pub const FORGE: [u32; 4] = [0x303554, 0x47527F, 0x6374AD, 0x8BA0D7];
    pub const NEOFORGE: [u32; 4] = [0x7F4321, 0xAA5D28, 0xD48140, 0xECAF79];
}

/// Accent presets: name, primary, primary foreground, chart 3, chart 5.
pub const ACCENTS: [(&str, u32, u32, u32, u32); 4] = [
    ("emerald", 0x349D62, 0x04130A, 0x3A6343, 0x3E5B44),
    ("gold", 0xDCAF61, 0x1F1401, 0x93744A, 0x7B5C3E),
    ("slate", 0x9BA6B1, 0x070E16, 0x627080, 0x535F6C),
    ("teal", 0x33A6A0, 0x00100F, 0x366E6B, 0x3D5F5D),
];

/// The accent swatch dot rendered by the appearance settings.
pub fn accent_swatch(name: &str) -> u32 {
    ACCENTS
        .iter()
        .find(|(key, ..)| *key == name)
        .map(|(_, primary, ..)| *primary)
        .unwrap_or(token::PRIMARY)
}

/// Pack RGB and alpha as 0xAARRGGBB, the order accepted by GPUI color values.
pub const fn rgba_hex(rgb: u32, alpha: u8) -> u32 {
    ((alpha as u32) << 24) | (rgb & 0x00FF_FFFF)
}

pub fn hsla_from_rgb(rgb: u32, alpha: f32) -> gpui_kit::Hsla {
    let r = ((rgb >> 16) & 0xff) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xff) as f32 / 255.0;
    let b = (rgb & 0xff) as f32 / 255.0;
    gpui_kit::Hsla::from(gpui_kit::Rgba { r, g, b, a: alpha })
}

/// Opaque color for a palette constant.
pub fn color(rgb: u32) -> gpui_kit::Hsla {
    hsla_from_rgb(rgb, 1.0)
}

/// Translucent color for a palette constant, for surfaces only. Text keeps its
/// own explicit token (see `MUTED_FOREGROUND_SUBTLE`) instead of being dimmed.
pub fn color_alpha(rgb: u32, alpha: f32) -> gpui_kit::Hsla {
    hsla_from_rgb(rgb, alpha)
}
