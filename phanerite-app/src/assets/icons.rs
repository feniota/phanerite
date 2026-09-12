/// Icon assets, most of which are from Lucide Icons <https://lucide.dev/>.
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum PhaIcon {
    ArrowLeft,
    ArrowRight,
    ChevronDown,
    ChevronRight,
    Copy,
    EllipsisVertical,
    Flame,
    Folder,
    FolderOpen,
    Layers,
    Palette,
    Play,
    PlayFilled,
    Package,
    Plus,
    Search,
    Settings,
    Star,
    TriangleAlert,
    #[strum(to_string = "trash-2")]
    Trash2,
    WindowClose,
    WindowMaximize,
    WindowMinimize,
    WindowRestore,
}

impl From<PhaIcon> for gpui_kit::component::Icon {
    fn from(icon: PhaIcon) -> Self {
        Self::default().path(format!("icons/{icon}.svg"))
    }
}
