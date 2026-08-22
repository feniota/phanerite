use std::sync::Arc;

use gpui::prelude::*;
use gpui::*;

pub fn render() -> Img {
    let bytes: Vec<u8> = super::Assets::get("phanerite-logo.svg")
        .unwrap()
        .data
        .to_vec();
    let data = Image::from_bytes(ImageFormat::Svg, bytes);
    img(Arc::new(data))
}
