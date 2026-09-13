//! Provider tabs and local account creation, with isolated online preview profiles.
#[cfg(feature = "seed")]
use crate::state::PlayerProfileSummary;
use crate::{
    assets::PhaIcon,
    pages::widgets::{field, muted, panel},
    state::{AccountType, AppState},
};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Selectable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    tab::{Tab, TabBar},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, Window, div,
};

const PROVIDERS: [AccountType; 4] = [
    AccountType::Microsoft,
    AccountType::Aphanite,
    AccountType::Yggdrasil,
    AccountType::Offline,
];
struct AccountDialog {
    app: Entity<AppState>,
    provider: usize,
    profile: usize,
    name: Entity<InputState>,
    server: Entity<InputState>,
    error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

pub fn open(window: &mut Window, cx: &mut App, app: Entity<AppState>) {
    let view = cx.new(|cx| {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. Steve"));
        let server =
            cx.new(|cx| InputState::new(window, cx).placeholder("https://auth.example.com/"));
        let subscriptions = vec![
            cx.observe(&name, |_: &mut AccountDialog, _, cx| cx.notify()),
            cx.observe(&server, |_: &mut AccountDialog, _, cx| cx.notify()),
        ];
        AccountDialog {
            app,
            provider: 0,
            profile: 0,
            name,
            server,
            error: None,
            _subscriptions: subscriptions,
        }
    });
    window.open_dialog(cx, move |dialog, window, _| {
        let view = view.clone();
        dialog
            .title("Add account")
            .width(window.rem_size() * 34.)
            .content(move |content, _, _| content.child(view.clone()))
    });
}

impl AccountDialog {
    fn valid(&self, cx: &App) -> bool {
        let provider = PROVIDERS[self.provider];
        if provider == AccountType::Offline {
            let name = self.name.read(cx).value();
            return !name.is_empty()
                && name.len() <= 16
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        }
        if !cfg!(feature = "seed") {
            return false;
        }
        if provider == AccountType::Yggdrasil {
            let server = self.server.read(cx).value();
            return !self.name.read(cx).value().trim().is_empty()
                && (server.starts_with("https://") || server.starts_with("http://"))
                && !server.chars().any(char::is_whitespace);
        }
        true
    }
    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.valid(cx) {
            return;
        }
        let provider = PROVIDERS[self.provider];
        let name = self.name.read(cx).value().trim().to_owned();
        let mut added = None;
        if provider == AccountType::Offline {
            added = self.app.read(cx).accounts.clone().update(cx, |store, cx| {
                let id = store.add(name.clone(), provider, None, vec![], None);
                if id.is_some() {
                    cx.notify();
                }
                id
            });
        }
        #[cfg(feature = "seed")]
        if provider != AccountType::Offline {
            let names = match provider {
                AccountType::Microsoft => vec!["enita".to_owned()],
                AccountType::Aphanite => vec!["enita".to_owned(), "Builder".to_owned()],
                _ => vec![name, "Alex".to_owned()],
            };
            let profiles: Vec<_> = names
                .into_iter()
                .map(|name| PlayerProfileSummary {
                    id: format!("preview-profile-{}-{name}", provider.key()),
                    skin_url: format!("https://mc-heads.net/skin/{name}"),
                    is_slim: name == "Alex",
                    name,
                })
                .collect();
            let selected = profiles[self.profile.min(profiles.len() - 1)].clone();
            let server = match provider {
                AccountType::Aphanite => Some(
                    self.app
                        .read(cx)
                        .settings
                        .read(cx)
                        .preferences()
                        .aphanite_server
                        .clone(),
                ),
                AccountType::Yggdrasil => Some(self.server.read(cx).value().to_string()),
                _ => None,
            };
            added = self.app.read(cx).accounts.clone().update(cx, |store, cx| {
                let id = store.add_preview(selected.name, provider, server, profiles, selected.id);
                if id.is_some() {
                    cx.notify();
                }
                id
            });
        }
        if added.is_some() {
            window.close_dialog(cx);
        } else {
            self.error = Some("This account has already been added.".into());
            cx.notify();
        }
    }
}

impl Render for AccountDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let provider = PROVIDERS[self.provider];
        let (icon, title, help) = match provider {
            AccountType::Microsoft => (
                PhaIcon::KeyRound,
                "Microsoft account",
                "Use your Minecraft Java player profile from Microsoft.",
            ),
            AccountType::Aphanite => (
                PhaIcon::Flame,
                "Aphanite Yggdrasil",
                "Use the Aphanite server configured in Settings.",
            ),
            AccountType::Yggdrasil => (
                PhaIcon::Server,
                "Custom Yggdrasil server",
                "Use a third-party Yggdrasil authentication endpoint.",
            ),
            AccountType::Offline => (
                PhaIcon::Monitor,
                "Single-player only",
                "Offline accounts skip authentication. Use a name with up to 16 letters, digits or underscores.",
            ),
        };
        let mut content = v_flex()
            .gap_4()
            .child(muted(
                "Sign in with a provider, or add an offline player profile.",
                cx,
            ))
            .child(
                TabBar::new("account-provider")
                    .segmented()
                    .w_full()
                    .selected_index(self.provider)
                    .children(
                        PROVIDERS
                            .iter()
                            .map(|provider| Tab::new().label(provider.label()).flex_1()),
                    )
                    .on_click(cx.listener(|this, ix, _, cx| {
                        this.provider = *ix;
                        this.profile = 0;
                        this.error = None;
                        cx.notify();
                    })),
            )
            .child(
                panel(cx)
                    .p_3()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Icon::new(icon).size_4())
                            .child(div().text_sm().font_medium().child(title)),
                    )
                    .child(muted(help, cx).text_xs()),
            );
        match provider {
            AccountType::Offline => {
                content = content.child(field("Player name", Input::new(&self.name)));
            }
            AccountType::Yggdrasil => {
                content = content
                    .child(field("Server URL", Input::new(&self.server)))
                    .child(field("Player name", Input::new(&self.name)));
            }
            AccountType::Aphanite => {
                content = content.child(
                    muted(
                        self.app
                            .read(cx)
                            .settings
                            .read(cx)
                            .preferences()
                            .aphanite_server
                            .clone(),
                        cx,
                    )
                    .font_family(crate::theme::MONO_FONT_FAMILY)
                    .text_xs(),
                );
            }
            AccountType::Microsoft => {}
        }
        if matches!(provider, AccountType::Aphanite | AccountType::Yggdrasil) {
            let labels = if provider == AccountType::Aphanite {
                ["enita", "Builder"]
            } else {
                ["Player", "Alex"]
            };
            content = content.child(field(
                "Player profile",
                h_flex()
                    .gap_2()
                    .children(labels.into_iter().enumerate().map(|(ix, name)| {
                        Button::new(format!("provider-profile-{ix}"))
                            .label(name)
                            .selected(self.profile == ix)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.profile = ix;
                                cx.notify();
                            }))
                    })),
            ));
        }
        if provider != AccountType::Offline {
            content = content.child(muted(if cfg!(feature = "seed") { "This preview adds a sample profile without signing in or storing credentials." } else { "Online sign-in is not connected in this build." }, cx).text_xs());
        }
        if let Some(error) = &self.error {
            content = content.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child(error.clone()),
            );
        }
        content.child(
            h_flex()
                .justify_end()
                .gap_2()
                .child(
                    Button::new("account-dialog-close")
                        .label("Cancel")
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                )
                .child(
                    Button::new("account-dialog-submit")
                        .primary()
                        .label(if provider == AccountType::Offline {
                            "Add offline account"
                        } else {
                            "Add preview account"
                        })
                        .disabled(!self.valid(cx))
                        .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                ),
        )
    }
}
