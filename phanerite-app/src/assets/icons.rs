/// Icon assets, most of which are from Lucide Icons <https://lucide.dev/>.
pub enum PhaIcon {
    Flame,
    Layers,
    PlayFilled,
    Package,
    Trash2,
}

impl From<PhaIcon> for gpui_component::Icon {
    fn from(icon: PhaIcon) -> Self {
        match icon {
            PhaIcon::Flame => Self::default().path("icons/flame.svg"),
            PhaIcon::Layers => Self::default().path("icons/layers.svg"),
            PhaIcon::PlayFilled => Self::default().path("icons/play-filled.svg"),
            PhaIcon::Package => Self::default().path("icons/package.svg"),
            PhaIcon::Trash2 => Self::default().path("icons/trash-2.svg"),
        }
    }
}
