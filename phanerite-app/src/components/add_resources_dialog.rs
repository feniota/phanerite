//! Resource import dialog; files are represented in the local UI model.

use crate::{
    assets::PhaIcon,
    pages::widgets::{field, muted},
    route::{InstanceRef, Route},
    state::{AppState, ModSummary, ResourcePackSummary, ShaderPackSummary},
};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, PathPromptOptions,
    Render, Styled as _, Window, div,
};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceMode {
    Mods,
    Packs,
    Shaders,
}
impl ResourceMode {
    fn title(self) -> &'static str {
        match self {
            Self::Mods => "Add mods",
            Self::Packs => "Import resource packs",
            Self::Shaders => "Import shader packs",
        }
    }
    fn extension(self) -> &'static str {
        if self == Self::Mods { "jar" } else { "zip" }
    }
}

pub fn open(window: &mut Window, cx: &mut App, app: Entity<AppState>, mode: ResourceMode) {
    let reference = match app.read(cx).route() {
        Route::Mods(reference)
        | Route::Packs(reference)
        | Route::Shaders(reference)
        | Route::InstanceDetail(reference) => Some(reference.clone()),
        _ => None,
    };
    if let Some(reference) = reference {
        open_for_instance(window, cx, app, reference, mode);
    }
}

pub fn open_for_instance(
    window: &mut Window,
    cx: &mut App,
    app: Entity<AppState>,
    reference: InstanceRef,
    mode: ResourceMode,
) {
    let view = cx.new(|_| ImportDialog {
        app,
        reference,
        mode,
        files: Vec::new(),
        error: None,
    });
    window.open_dialog(cx, move |dialog, window, _| {
        let view = view.clone();
        dialog
            .title(mode.title())
            .width(window.rem_size() * 30.)
            .content(move |content, _, _| content.child(view.clone()))
    });
}

struct ImportDialog {
    app: Entity<AppState>,
    reference: InstanceRef,
    mode: ResourceMode,
    files: Vec<PathBuf>,
    error: Option<String>,
}
impl ImportDialog {
    fn browse(&mut self, cx: &mut Context<Self>) {
        let picker = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Choose files to import".into()),
        });
        cx.spawn(async move |this, cx| {
            let result = picker.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(Some(paths))) => {
                        this.files = paths
                            .into_iter()
                            .filter(|path| {
                                path.extension().is_some_and(|ext| {
                                    ext.eq_ignore_ascii_case(this.mode.extension())
                                })
                            })
                            .collect();
                        this.error = this.files.is_empty().then(|| {
                            format!(
                                "Choose .{} files for this collection.",
                                this.mode.extension()
                            )
                        });
                    }
                    Ok(Ok(None)) => {}
                    _ => this.error = Some("Could not open the file picker. Try again.".into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn import(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.files.is_empty() {
            return;
        }
        let mode = self.mode;
        let reference = self.reference.clone();
        let files = self.files.clone();
        let changed = self.app.read(cx).instances.clone().update(cx, |store, cx| {
            let changed = match mode {
                ResourceMode::Mods => store.add_mods(
                    &reference,
                    files
                        .iter()
                        .map(|path| ModSummary {
                            id: String::new(),
                            name: None,
                            version: None,
                            file_name: file_name(path),
                            loader: None,
                            enabled: true,
                        })
                        .collect(),
                ),
                ResourceMode::Packs => store.add_resource_packs(
                    &reference,
                    files
                        .iter()
                        .map(|path| ResourcePackSummary {
                            id: String::new(),
                            name: file_name(path),
                            author: "Local file".into(),
                            version: String::new(),
                            description: "Imported resource pack".into(),
                            size: String::new(),
                            enabled: true,
                        })
                        .collect(),
                ),
                ResourceMode::Shaders => store.add_shader_packs(
                    &reference,
                    files
                        .iter()
                        .map(|path| ShaderPackSummary {
                            id: String::new(),
                            name: file_name(path),
                            author: "Local file".into(),
                            version: String::new(),
                            gpu: "Custom shader".into(),
                            enabled: true,
                        })
                        .collect(),
                ),
            };
            if changed {
                cx.notify();
            }
            changed
        });
        if changed {
            window.close_dialog(cx);
        } else {
            self.error = Some("The instance is no longer available.".into());
            cx.notify();
        }
    }

    #[cfg(feature = "seed")]
    fn import_samples(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let reference = self.reference.clone();
        let changed = self.app.read(cx).instances.clone().update(cx, |store, cx| {
            let changed = match self.mode {
                ResourceMode::Mods => store.add_mods(
                    &reference,
                    crate::seed::importable_mods()
                        .into_iter()
                        .take(3)
                        .map(|item| ModSummary {
                            id: String::new(),
                            name: Some(item.name),
                            version: Some(item.version),
                            file_name: item.file_name,
                            loader: Some(item.loader),
                            enabled: true,
                        })
                        .collect(),
                ),
                ResourceMode::Packs => store.add_resource_packs(
                    &reference,
                    crate::seed::importable_resource_packs()
                        .into_iter()
                        .map(|mut item| {
                            item.id.clear();
                            item
                        })
                        .collect(),
                ),
                ResourceMode::Shaders => store.add_shader_packs(
                    &reference,
                    crate::seed::importable_shader_packs()
                        .into_iter()
                        .map(|mut item| {
                            item.id.clear();
                            item
                        })
                        .collect(),
                ),
            };
            if changed {
                cx.notify();
            }
            changed
        });
        if changed {
            window.close_dialog(cx);
        }
    }
}
fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

impl Render for ImportDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut content = v_flex()
            .gap_4()
            .child(muted(
                "Add resources to the selected instance. This preview keeps imports in memory.",
                cx,
            ))
            .child(
                Button::new("import-browse-area")
                    .ghost()
                    .w_full()
                    .h_auto()
                    .p_6()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        v_flex()
                            .w_full()
                            .items_center()
                            .gap_3()
                            .child(
                                Icon::new(PhaIcon::Upload)
                                    .size_8()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .font_medium()
                                    .child(format!("Choose .{} files", self.mode.extension())),
                            )
                            .child(muted("Browse files on your computer", cx)),
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.browse(cx))),
            );
        if !self.files.is_empty() {
            content = content.child(field(
                format!("{} files selected", self.files.len()),
                v_flex().gap_1().children(
                    self.files
                        .iter()
                        .map(|path| muted(file_name(path), cx).text_xs()),
                ),
            ));
        }
        if let Some(error) = &self.error {
            content = content.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child(error.clone()),
            );
        }
        let footer = h_flex().gap_2();
        #[cfg(feature = "seed")]
        let footer = footer.child(
            Button::new("import-samples")
                .ghost()
                .label("Import samples")
                .on_click(cx.listener(|this, _, window, cx| this.import_samples(window, cx))),
        );
        content.child(
            footer
                .child(div().flex_1())
                .child(
                    Button::new("import-cancel")
                        .label("Cancel")
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                )
                .child(
                    Button::new("import-submit")
                        .primary()
                        .label("Import")
                        .disabled(self.files.is_empty())
                        .on_click(cx.listener(|this, _, window, cx| this.import(window, cx))),
                ),
        )
    }
}
