use phanerite::palette;

#[test]
fn token_hex_is_rgb_in_gpui_order() {
    assert_eq!(palette::token::BACKGROUND, 0x090B09);
    assert_eq!(palette::token::PRIMARY, 0x349D62);
}

#[test]
fn transparent_border_uses_expected_alpha() {
    assert_eq!(palette::rgba_hex(0xFFFFFF, 0x17), 0x17FFFFFF);
}
