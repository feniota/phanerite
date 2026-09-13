//! First-run storage selection using the native directory picker.

use super::widgets::{muted, panel};
use crate::{
    assets::{self, PhaIcon},
    route::Route,
    state::AppState,
};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    App, Context, Entity, IntoElement, ParentElement as _, PathPromptOptions, Render, Styled as _,
    Window, div, rems,
};
use std::path::PathBuf;

struct SetupPage {
    app: Entity<AppState>,
    directory: Option<PathBuf>,
    busy: bool,
    error: Option<String>,
}

pub fn render(app: Entity<AppState>, window: &mut Window, cx: &mut App) -> impl IntoElement {
    window.use_keyed_state("setup-page", cx, move |_, cx| {
        let directory = app.read(cx).storage().map(|storage| storage.root_dir);
        SetupPage {
            app,
            directory,
            busy: false,
            error: None,
        }
    })
}

impl SetupPage {
    fn browse(&mut self, cx: &mut Context<Self>) {
        let picker = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Choose game directory".into()),
        });
        cx.spawn(async move |this, cx| {
            let result = picker.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(Some(paths))) => {
                        this.directory = paths.into_iter().next();
                        this.error = None;
                    }
                    Ok(Ok(None)) => {}
                    _ => {
                        this.error = Some("Could not open the directory picker. Try again.".into())
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn continue_setup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(directory) = self.directory.clone() else {
            return;
        };
        if self
            .app
            .read(cx)
            .storage()
            .is_some_and(|storage| storage.root_dir == directory)
        {
            self.app.update(cx, |app, cx| app.replace(Route::Play, cx));
            return;
        }
        self.busy = true;
        self.error = None;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = phanerite_core::storage::Storage::new(&directory).await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(storage) => {
                        let ready = this.app.update(cx, |app, cx| {
                            let key = phanerite_core::storage::StorageIdent::from(&storage);
                            if !app.storages.contains(&key)
                                && app.add_storage(storage, cx).is_none()
                            {
                                return false;
                            }
                            app.set_default_storage(key.clone(), cx);
                            app.instances.update(cx, |store, cx| {
                                store.set_storage_context(key.clone());
                                cx.notify();
                            });
                            app.crashes.update(cx, |store, cx| {
                                store.set_storage_context(key);
                                cx.notify();
                            });
                            app.replace(Route::Play, cx);
                            true
                        });
                        if !ready {
                            this.error = Some("Could not register the game directory.".into());
                        }
                    }
                    Err(error) => {
                        this.error = Some(format!("Could not open this directory: {error}"))
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for SetupPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut content = v_flex()
            .w_full()
            .max_w(rems(32.))
            .gap_6()
            .child(
                v_flex()
                    .items_center()
                    .gap_3()
                    .child(assets::phanerite_logo().size(rems(4.)))
                    .child(
                        div()
                            .text_2xl()
                            .font_semibold()
                            .child("Welcome to Phanerite"),
                    )
                    .child(
                        muted(
                            "Choose where your Minecraft instances, mods and worlds will live.",
                            cx,
                        )
                        .text_center(),
                    ),
            )
            .child(
                panel(cx)
                    .p_4()
                    .gap_4()
                    .child(
                        h_flex()
                            .gap_3()
                            .child(
                                Icon::new(PhaIcon::FolderOpen)
                                    .size_6()
                                    .text_color(cx.theme().primary),
                            )
                            .child(
                                v_flex()
                                    .min_w_0()
                                    .flex_1()
                                    .gap_1()
                                    .child(div().text_sm().font_medium().child("Game directory"))
                                    .child(
                                        muted(
                                            self.directory
                                                .as_ref()
                                                .map(|path| path.display().to_string())
                                                .unwrap_or_else(|| "No directory selected".into()),
                                            cx,
                                        )
                                        .text_xs(),
                                    ),
                            ),
                    )
                    .child(
                        Button::new("setup-browse")
                            .icon(PhaIcon::FolderOpen)
                            .label("Choose game directory")
                            .disabled(self.busy)
                            .on_click(cx.listener(|this, _, _, cx| this.browse(cx))),
                    )
                    .child(
                        muted(
                            "Each instance keeps its own game files and configuration.",
                            cx,
                        )
                        .text_xs(),
                    ),
            );
        if let Some(error) = &self.error {
            content = content.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child(error.clone()),
            );
        }
        content = content.child(
            Button::new("setup-continue")
                .primary()
                .label("Open launcher")
                .loading(self.busy)
                .disabled(self.directory.is_none() || self.busy)
                .on_click(cx.listener(|this, _, window, cx| this.continue_setup(window, cx))),
        );
        v_flex()
            .size_full()
            .p_6()
            .items_center()
            .justify_center()
            .child(content)
    }
}
