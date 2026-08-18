use gpui::*;
use gpui_component::{h_flex, searchable_list::SearchableListItem};

/// Avatar + Username
/// good for showing a single user as an selection item
#[derive(Clone)]
pub struct UserItem {
    /// The avatar image URL of the user. If None, the `name` will be used as the avatar placeholder.
    pub avatar: Option<SharedString>,
    /// The display name of the user.
    pub name: SharedString,
    /// The unique identifier of the user.
    pub value: SharedString,
    /// Whether the user is selected.
    selected: bool,
}

impl UserItem {
    fn new(
        avatar: Option<impl Into<SharedString>>,
        name: impl Into<SharedString>,
        value: impl Into<SharedString>,
    ) -> Self {
        Self {
            avatar: avatar.map(|a| a.into()),
            name: name.into(),
            value: value.into(),
            selected: false,
        }
    }
}

impl PartialEq for UserItem {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl SearchableListItem for UserItem {
    type Value = SharedString;
    fn value(&self) -> &SharedString {
        &self.value
    }

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn render(&self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let avatar = {
            let a = gpui_component::avatar::Avatar::new().name(&self.name);
            if let Some(avatar_url) = &self.avatar {
                a.src(avatar_url.clone())
            } else {
                a
            }
        };

        h_flex()
            .items_center()
            .child(avatar)
            .child(self.name.clone())
    }
}
