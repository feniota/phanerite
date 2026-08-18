// Source values come from design/src/routes/layout.css. OKLCH is the source
// of truth; these are offline OKLab -> linear sRGB -> gamma conversions.
pub mod token {
    pub const BACKGROUND: u32 = 0x090B09; // oklch(.145 .005 150)
    pub const FOREGROUND: u32 = 0xEFF1EF; // oklch(.955 .003 150)
    pub const CARD: u32 = 0x121512; // oklch(.19 .006 150)
    pub const POPOVER: u32 = 0x161917; // oklch(.21 .007 150)
    pub const PRIMARY: u32 = 0x349D62; // oklch(.62 .13 155)
    pub const PRIMARY_FOREGROUND: u32 = 0x04130A; // oklch(.17 .03 155)
    pub const SECONDARY: u32 = 0x1D201D; // oklch(.24 .008 150)
    pub const MUTED: u32 = 0x191B19; // oklch(.22 .006 150)
    pub const MUTED_FOREGROUND: u32 = 0x979D98; // oklch(.69 .01 150)
    pub const MUTED_FOREGROUND_SUBTLE: u32 = 0x7C827D; // oklch(.6 .01 150)
    pub const ACCENT: u32 = 0x212522; // oklch(.26 .009 150)
    pub const ACCENT_FOREGROUND: u32 = 0xF4F6F4; // oklch(.97 .003 150)
    pub const DESTRUCTIVE: u32 = 0xF05653; // oklch(.66 .19 25)
    pub const SIDEBAR: u32 = 0x0F110F; // oklch(.175 .005 150)
    pub const CHART_3: u32 = 0x3A6343; // oklch(.46 .07 150)
    pub const CHART_5: u32 = 0x3E5B44; // oklch(.44 .05 150)
    pub const GOLD: u32 = 0xDCAF61; // oklch(.78 .11 80)
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

pub const ACCENTS: [(&str, u32, u32); 4] = [
    ("emerald", 0x349D62, 0x04130A),
    ("gold", 0xDCAF61, 0x1F1401),
    ("slate", 0x9BA6B1, 0x070E16),
    ("teal", 0x33A6A0, 0x00100F),
];

/// Pack RGB and alpha as 0xAARRGGBB, the order accepted by GPUI color values.
pub const fn rgba_hex(rgb: u32, alpha: u8) -> u32 {
    ((alpha as u32) << 24) | (rgb & 0x00FF_FFFF)
}

pub fn hsla_from_rgb(rgb: u32, alpha: f32) -> gpui::Hsla {
    let r = ((rgb >> 16) & 0xff) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xff) as f32 / 255.0;
    let b = (rgb & 0xff) as f32 / 255.0;
    gpui::Hsla::from(gpui::Rgba { r, g, b, a: alpha })
}
