//! Local crash findings, evidence navigation, and portable report actions.

use super::{back_button, missing_resource, widgets::*};
use crate::{
    assets::PhaIcon,
    components::instance_actions,
    route::CrashRef,
    state::{AppState, CrashFinding, CrashReport, CrashSource, LaunchField, LaunchValue, redact},
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    notification::Notification,
    scroll::{ScrollableElement as _, Scrollbar},
    v_flex,
};
use gpui_kit::{
    App, ClipboardItem, Context, Entity, InteractiveElement as _, IntoElement as _,
    ParentElement as _, Render, ScrollHandle, StatefulInteractiveElement as _, Styled as _, Window,
    div, prelude::FluentBuilder as _, rems,
};

struct CrashPage {
    app: Entity<AppState>,
    reference: CrashRef,
    selected: usize,
    show_mods: bool,
    output_scroll: ScrollHandle,
}

pub fn render(
    reference: &CrashRef,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    if app.read(cx).crashes.read(cx).find(reference).is_none() {
        return missing_resource("crash report", app).into_any_element();
    }
    let key = format!(
        "{}:{}:crash-page",
        reference.storage.root_dir.display(),
        reference.report_id
    );
    let reference = reference.clone();
    window
        .use_keyed_state(key, cx, move |_, _| CrashPage {
            app,
            reference,
            selected: 0,
            show_mods: false,
            output_scroll: ScrollHandle::new(),
        })
        .into_any_element()
}

impl CrashPage {
    fn select_finding(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected = index;
        if let Some(line) = self
            .app
            .read(cx)
            .crashes
            .read(cx)
            .find(&self.reference)
            .and_then(|report| report.findings.get(index))
            .and_then(|finding| finding.evidence_lines.first())
        {
            self.output_scroll.scroll_to_item(*line);
        }
        cx.notify();
    }

    fn apply_finding(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(report) = self
            .app
            .read(cx)
            .crashes
            .read(cx)
            .find(&self.reference)
            .cloned()
        else {
            return;
        };
        let Some(finding) = report.findings.get(index) else {
            return;
        };
        let reference = report.instance();
        self.app.read(cx).instances.clone().update(cx, |store, cx| {
            let mut changed = false;
            // Local rules identify alternatives; the suggested action disables
            // the first implicated mod, matching the named button.
            if let Some(id) = finding.implicated_mod_ids.first() {
                changed |= store.set_mod_enabled(&reference, id, false);
            }
            if let Some(memory) = finding.suggested_memory {
                changed |= store.set_launch_override(
                    &reference,
                    LaunchField::Memory,
                    LaunchValue::Number(memory),
                );
            }
            if changed {
                cx.notify();
            }
        });
        instance_actions::start(&reference, &self.app, window, cx);
        window.push_notification(Notification::success("Changes applied."), cx);
    }

    fn finding(
        &self,
        finding: &CrashFinding,
        index: usize,
        report: &CrashReport,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        let selected = index == self.selected;
        let action_label = finding
            .implicated_mod_ids
            .first()
            .map(|id| {
                let name = self
                    .app
                    .read(cx)
                    .instances
                    .read(cx)
                    .find(&report.instance())
                    .and_then(|instance| instance.mods.iter().find(|item| &item.id == id))
                    .map(|item| item.display_name().to_owned())
                    .unwrap_or_else(|| "mod".into());
                format!("Disable {name} and retry")
            })
            .unwrap_or_else(|| {
                format!(
                    "Raise memory to {} GB and retry",
                    finding.suggested_memory.unwrap_or(4)
                )
            });
        panel(cx)
            .p_4()
            .gap_3()
            .border_color(if selected {
                cx.theme().primary
            } else {
                cx.theme().border
            })
            .child(
                Button::new(format!("crash-finding-{index}"))
                    .ghost()
                    .w_full()
                    .h_auto()
                    .p_0()
                    .selected(selected)
                    .bg(cx.theme().group_box)
                    .accessibility_label(format!("Show evidence for {}", finding.title))
                    .child(
                        h_flex()
                            .w_full()
                            .items_start()
                            .gap_3()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_1()
                                    .text_left()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_medium()
                                            .whitespace_normal()
                                            .child(finding.title.clone()),
                                    )
                                    .child(
                                        muted(finding.explanation.clone(), cx).whitespace_normal(),
                                    ),
                            )
                            .child(badge("Matched")),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| this.select_finding(index, cx))),
            )
            .when(selected && finding.has_action(), |card| {
                card.child(
                    h_flex().child(
                        Button::new(format!("crash-fix-{index}"))
                            .primary()
                            .small()
                            .icon(PhaIcon::Play)
                            .label(action_label)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.apply_finding(index, window, cx)
                            })),
                    ),
                )
            })
    }

    fn output(&self, report: &CrashReport, cx: &App) -> gpui_kit::Div {
        let evidence = report
            .findings
            .get(self.selected)
            .map(|finding| finding.evidence_lines.as_slice())
            .unwrap_or_default();
        div()
            .relative()
            .h_96()
            .min_w_0()
            .bg(crate::theme::terminal())
            .child(
                div()
                    .id("crash-output")
                    .size_full()
                    .p_4()
                    .overflow_scroll()
                    .track_scroll(&self.output_scroll)
                    .font_family(crate::theme::MONO_FONT_FAMILY)
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .children(report.output().iter().enumerate().map(|(index, line)| {
                        div()
                            .min_h(rems(1.5))
                            .whitespace_normal()
                            .when(evidence.contains(&index), |line| {
                                line.bg(cx.theme().primary.opacity(0.15))
                                    .text_color(cx.theme().primary)
                            })
                            .child(line.clone())
                    })),
            )
            .child(Scrollbar::new(&self.output_scroll))
    }
}

impl Render for CrashPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        let Some(report) = self
            .app
            .read(cx)
            .crashes
            .read(cx)
            .find(&self.reference)
            .cloned()
        else {
            return missing_resource("crash report", self.app.clone()).into_any_element();
        };
        let name = self
            .app
            .read(cx)
            .instances
            .read(cx)
            .find(&report.instance())
            .map(|instance| instance.name.clone())
            .unwrap_or_else(|| report.instance_id.clone());
        let header = v_flex()
            .flex_shrink_0()
            .px_6()
            .pt_4()
            .pb_4()
            .gap_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(h_flex().child(back_button(self.app.clone())))
            .child(
                h_flex()
                    .items_start()
                    .gap_3()
                    .child(
                        Icon::new(PhaIcon::TriangleAlert)
                            .size_5()
                            .mt_1()
                            .text_color(cx.theme().danger),
                    )
                    .child(
                        v_flex()
                            .min_w_0()
                            .flex_1()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .truncate()
                                    .child(format!("{name} crashed")),
                            )
                            .child(muted(
                                format!("{} · exit code {}", report.when, report.exit_code),
                                cx,
                            )),
                    )
                    .when(report.environment.source == CrashSource::Aphanite, |row| {
                        row.child(badge("Aphanite"))
                    }),
            );
        let mut body = v_flex().w_full().max_w(rems(64.)).mx_auto().gap_6();
        if report.findings.is_empty() {
            let app = self.app.clone();
            let reference = report.instance();
            body = body.child(panel(cx).p_4().child(h_flex().flex_wrap().gap_4()
                .child(v_flex().flex_1().min_w_0().gap_2()
                    .child(div().text_sm().font_semibold().child("No known crash pattern matched"))
                    .child(muted("No local rule matched this output. The prepared report includes the full output and environment for further investigation.", cx)))
                .child(Button::new("crash-retry").icon(PhaIcon::Play).label("Retry")
                    .on_click(move |_, window, cx| instance_actions::start(&reference, &app, window, cx)))));
        } else {
            let mut findings = v_flex().gap_3();
            for (index, finding) in report.findings.iter().enumerate() {
                findings = findings.child(self.finding(finding, index, &report, cx));
            }
            body = body.child(section(
                "Known crash patterns",
                "These are literal matches from local crash rules.",
                findings,
                cx,
            ));
            if let Some(finding) = report.findings.get(self.selected) {
                let evidence = finding
                    .evidence_lines
                    .iter()
                    .filter_map(|&index| report.output().get(index))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n");
                if !evidence.is_empty() {
                    body = body.child(section(
                        "Evidence",
                        "",
                        div()
                            .font_family(crate::theme::MONO_FONT_FAMILY)
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(evidence),
                        cx,
                    ));
                }
            }
        }
        if report.has_report() {
            let copy_text = redact(&report.source_text());
            body = body.child(
                panel(cx)
                    .overflow_hidden()
                    .child(
                        h_flex()
                            .justify_between()
                            .gap_3()
                            .px_4()
                            .py_2()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(div().text_sm().font_semibold().child("Crash report"))
                            .child(
                                Button::new("crash-copy-output")
                                    .ghost()
                                    .small()
                                    .icon(PhaIcon::Copy)
                                    .label("Copy")
                                    .on_click(move |_, window, cx| {
                                        copy(copy_text.clone(), window, cx)
                                    }),
                            ),
                    )
                    .child(self.output(&report, cx)),
            );
        } else {
            let mut output = panel(cx).p_4().gap_4()
                .child(h_flex().items_start().gap_3()
                    .child(Icon::new(PhaIcon::TriangleAlert).size_5().text_color(cx.theme().danger))
                    .child(v_flex().gap_2().min_w_0().flex_1()
                        .child(div().text_sm().font_semibold().child("No crash report was written"))
                        .child(muted("Minecraft did not write a crash report. The Java VM may have terminated before the game could capture it.", cx))))
                .child(panel(cx).p_4().gap_3().bg(crate::theme::terminal())
                    .child(div().text_sm().font_semibold().child("Process output (last 20 lines)"))
                    .child(div().font_family(crate::theme::MONO_FONT_FAMILY).text_sm().text_color(cx.theme().muted_foreground).child(report.stderr_tail.join("\n"))));
            if let Some(path) = &report.hs_err_path {
                let path = std::path::PathBuf::from(path);
                output = output.child(
                    h_flex()
                        .flex_wrap()
                        .gap_3()
                        .child(muted(format!("JVM error file: {}", path.display()), cx).text_xs())
                        .child(
                            Button::new("crash-reveal-jvm")
                                .small()
                                .icon(PhaIcon::FolderOpen)
                                .label("Reveal in file manager")
                                .on_click(move |_, window, cx| {
                                    if path.exists() {
                                        cx.reveal_path(&path);
                                    } else {
                                        window.push_notification(
                                            Notification::info(
                                                "This preview error file is not on disk.",
                                            ),
                                            cx,
                                        );
                                    }
                                }),
                        ),
                );
            }
            body = body.child(output);
        }
        let environment = &report.environment;
        let row = |label: &str, value: String| {
            h_flex()
                .justify_between()
                .items_start()
                .gap_4()
                .child(muted(label.to_owned(), cx).flex_shrink_0())
                .child(div().text_sm().text_right().child(value))
        };
        let mut details = v_flex()
            .gap_3()
            .child(
                h_flex()
                    .items_start()
                    .gap_6()
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_3()
                            .child(row("Minecraft", environment.mc_version.clone()))
                            .child(row(
                                "Loader",
                                format!(
                                    "{} {}",
                                    environment.loader.label(),
                                    environment.loader_version
                                ),
                            )),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_3()
                            .child(row(
                                "Java",
                                format!("{} {}", environment.java_name, environment.java_version),
                            ))
                            .child(row("Memory", format!("{} GB", environment.memory))),
                    ),
            )
            .child(row(
                "OS / GPU",
                format!("{} · {}", environment.os, environment.gpu),
            ));
        if !environment.active_overrides.is_empty() {
            details = details.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().warning)
                    .child(format!(
                        "Launch settings overridden: {}",
                        environment.active_overrides.join(", ")
                    )),
            );
        }
        details = details.child(
            h_flex().child(
                Button::new("crash-mods-toggle")
                    .ghost()
                    .small()
                    .icon(if self.show_mods {
                        PhaIcon::ChevronDown
                    } else {
                        PhaIcon::ChevronRight
                    })
                    .label(format!("{} enabled mods", environment.enabled_mods.len()))
                    .selected(self.show_mods)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.show_mods = !this.show_mods;
                        cx.notify();
                    })),
            ),
        );
        if self.show_mods {
            details = details.child(
                v_flex().gap_1().children(
                    environment
                        .enabled_mods
                        .iter()
                        .map(|(name, version)| muted(format!("{name} {version}"), cx).text_xs()),
                ),
            );
        }
        body = body.child(section("Environment", "", details, cx));
        let ai = prepared_report(&report, &name, ReportFormat::Ai);
        let post = prepared_report(&report, &name, ReportFormat::Post);
        let save = post.clone();
        let mut footer = h_flex()
            .flex_shrink_0()
            .flex_wrap()
            .gap_2()
            .p_4()
            .border_t_1()
            .border_color(cx.theme().border)
            .child(
                Button::new("crash-copy-ai")
                    .primary()
                    .small()
                    .icon(PhaIcon::Copy)
                    .label("Copy for AI")
                    .on_click(move |_, window, cx| copy(ai.clone(), window, cx)),
            )
            .child(
                Button::new("crash-copy-post")
                    .small()
                    .icon(PhaIcon::FileText)
                    .label("Copy as post")
                    .on_click(move |_, window, cx| copy(post.clone(), window, cx)),
            )
            .child(
                Button::new("crash-save")
                    .small()
                    .icon(PhaIcon::FileDown)
                    .label("Save report…")
                    .on_click(move |_, window, cx| save_report(save.clone(), window, cx)),
            );
        if environment.source == CrashSource::Aphanite {
            let owner = environment
                .aphanite_server
                .clone()
                .unwrap_or_else(|| "the server owner".into());
            let post = prepared_report(&report, &name, ReportFormat::Post);
            footer = footer.child(Button::new("crash-prepare-owner").ml_auto().small().icon(PhaIcon::Send).label("Report to server owner")
                .on_click(move |_, window, cx| {
                    let owner = owner.clone(); let post = post.clone();
                    window.open_dialog(cx, move |dialog, _, _| {
                        let post = post.clone(); let owner = owner.clone();
                        dialog.title("Report to server owner")
                            .content(move |content, _, cx| content.child(muted(format!("Copy this prepared report to share with {owner}. It includes the crash output, local findings and environment."), cx)))
                            .footer(Button::new("crash-owner-copy").primary().icon(PhaIcon::Copy).label("Copy report")
                                .on_click(move |_, window, cx| { copy(post.clone(), window, cx); window.close_dialog(cx); }))
                    });
                }));
        }
        v_flex()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(header)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .child(div().p_6().child(body)),
            )
            .child(footer)
            .into_any_element()
    }
}

#[derive(Clone, Copy)]
enum ReportFormat {
    Ai,
    Post,
}

fn prepared_report(report: &CrashReport, name: &str, format: ReportFormat) -> String {
    let environment = &report.environment;
    let findings = if report.findings.is_empty() {
        "- No local crash signature matched.".into()
    } else {
        report
            .findings
            .iter()
            .map(|finding| {
                format!(
                    "- {} ({}): {}",
                    finding.title, finding.rule, finding.explanation
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mods = if environment.enabled_mods.is_empty() {
        "None".into()
    } else {
        environment
            .enabled_mods
            .iter()
            .map(|(name, version)| format!("- {name} {version}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let overrides = if environment.active_overrides.is_empty() {
        "None".into()
    } else {
        environment.active_overrides.join(", ")
    };
    let intro = match format {
        ReportFormat::Ai => {
            "My Minecraft game crashed while using Phanerite. Help me identify the likely cause from the captured output and environment. Explain the evidence and where to find any additional information you need."
        }
        ReportFormat::Post => "Minecraft crash report captured by Phanerite.",
    };
    // Redact the entire document: paths or credentials may occur in environment
    // values and findings as well as in the captured process output.
    redact(&format!(
        "{intro}\n\nInstance: {name}\nTime: {}\nExit code: {}\n\n## Local crash-signature matches\n{findings}\n\n## Environment\nMinecraft {} · {} {}\nJava {} ({})\nJava path: {}\nMemory {} GB · {} · {}\nOverridden launch settings: {overrides}\n\n## Enabled mods ({})\n{mods}\n\n## {}\n\n~~~text\n{}\n~~~\n",
        report.when,
        report.exit_code,
        environment.mc_version,
        environment.loader.label(),
        environment.loader_version,
        environment.java_version,
        environment.java_name,
        environment.java_path,
        environment.memory,
        environment.os,
        environment.gpu,
        environment.enabled_mods.len(),
        if report.has_report() {
            "Crash report"
        } else {
            "Process output"
        },
        report.source_text()
    ))
}

fn copy(text: String, window: &mut Window, cx: &mut App) {
    let count = text.chars().count();
    cx.write_to_clipboard(ClipboardItem::new_string(text));
    window.push_notification(
        Notification::success(format!("Copied report · {count} characters")),
        cx,
    );
}

fn save_report(text: String, window: &mut Window, cx: &mut App) {
    let directory = dirs::download_dir().unwrap_or_else(std::env::temp_dir);
    let picker = cx.prompt_for_new_path(&directory, Some("phanerite-crash-report.md"));
    window
        .spawn(cx, async move |cx| {
            let result = match picker.await {
                Ok(Ok(Some(path))) => async_fs::write(path, text)
                    .await
                    .map_err(|error| error.to_string()),
                Ok(Ok(None)) => return,
                Ok(Err(error)) => Err(error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = cx.update(|window, cx| {
                let notification = match result {
                    Ok(()) => Notification::success("Report saved."),
                    Err(error) => Notification::error(format!("Could not save report: {error}")),
                };
                window.push_notification(notification, cx);
            });
        })
        .detach();
}
