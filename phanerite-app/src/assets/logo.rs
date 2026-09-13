use std::sync::Arc;

use gpui_kit::*;

pub fn render() -> Img {
    let bytes: Vec<u8> = super::Assets::get("phanerite-logo.svg")
        .unwrap()
        .data
        .to_vec();
    let data = Image::from_bytes(ImageFormat::Svg, bytes);
    img(Arc::new(data))
}

#[cfg(target_os = "linux")]
pub(crate) fn window_icon() -> Arc<image::RgbaImage> {
    static ICON: std::sync::LazyLock<Arc<image::RgbaImage>> = std::sync::LazyLock::new(|| {
        let image = image::load_from_memory_with_format(
            include_bytes!("../../assets/app-icons/256.png"),
            image::ImageFormat::Png,
        )
        .expect("bundled application icon must be a valid PNG")
        .into_rgba8();
        Arc::new(image)
    });
    Arc::clone(&ICON)
}
