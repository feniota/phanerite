//! Running games, with direct access to output and Stop controls.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    popover::Popover,
    scroll::ScrollableElement as _,
    v_flex,
};
use gpui_kit::{
    Anchor, App, DismissEvent, Entity, IntoElement, ParentElement as _, Styled as _, div,
};

use super::{instance_actions, instance_icon};
use crate::{assets::PhaIcon, pages::widgets::muted, route::Route, state::AppState};

pub fn render(app: Entity<AppState>, cx: &App) -> impl IntoElement {
    let count = app.read(cx).sessions.read(cx).running_count();
    Popover::new("running-instances-popover")
        .anchor(Anchor::BottomRight)
        .trigger(
            Button::new("running-instances-trigger")
                .ghost()
                .xsmall()
                .h_5()
                .label(format!("● {count} running"))
                .text_color(crate::theme::launch_foreground()),
        )
        .content(move |_, window, cx| {
            let state = app.read(cx);
            let running: Vec<_> = state
                .sessions
                .read(cx)
                .running()
                .filter_map(|session| state.instances.read(cx).find(&session.instance).cloned())
                .collect();
            let mut list = v_flex().gap_1();
            for instance in running {
                let app_logs = app.clone();
                let logs = instance.reference();
                let app_stop = app.clone();
                let stop = instance.reference();
                list = list.child(
                    h_flex()
                        .gap_2()
                        .p_2()
                        .rounded(cx.theme().radius)
                        .child(instance_icon::render_sized(
                            &instance,
                            window.rem_size() * 2.,
                            cx,
                        ))
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_medium()
                                        .truncate()
                                        .child(instance.name.clone()),
                                )
                                .child(
                                    muted(
                                        format!(
                                            "MC {} · {}",
                                            instance.mc_version,
                                            instance.loader.label()
                                        ),
                                        cx,
                                    )
                                    .text_xs()
                                    .truncate(),
                                ),
                        )
                        .child(
                            Button::new(format!("running-logs-{}", instance.id))
                                .ghost()
                                .small()
                                .icon(PhaIcon::ScrollText)
                                .tooltip("Game logs")
                                .accessibility_label(format!("Logs for {}", instance.name))
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    app_logs.update(cx, |app, cx| {
                                        app.push(Route::Logs(logs.clone()), cx)
                                    });
                                    cx.emit(DismissEvent);
                                })),
                        )
                        .child(
                            Button::new(format!("running-stop-{}", instance.id))
                                .danger()
                                .small()
                                .icon(PhaIcon::Square)
                                .tooltip("Stop game")
                                .accessibility_label(format!("Stop {}", instance.name))
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    instance_actions::stop(&stop, &app_stop, cx);
                                    cx.notify();
                                })),
                        ),
                );
            }
            v_flex()
                .w_80()
                .gap_3()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().text_sm().font_semibold().child("Running instances"))
                        .child(muted("Stop a game or inspect its live output.", cx).text_xs()),
                )
                .child(div().max_h_80().overflow_y_scrollbar().child(list))
        })
}
