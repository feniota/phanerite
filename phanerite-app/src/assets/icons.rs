/// Icon assets, most of which are from Lucide Icons (https://lucide.dev/).
pub enum PhaIcon {
    Flame,
    Layers,
    PlayFilled,
}

impl Into<gpui_component::Icon> for PhaIcon {
    fn into(self) -> gpui_component::Icon {
        match self {
            Self::Flame => gpui_component::Icon::default().path("icons/flame.svg"),
            Self::Layers => gpui_component::Icon::default().path("icons/layers.svg"),
            Self::PlayFilled => gpui_component::Icon::default().path("icons/play-filled.svg"),
        }
    }
}
