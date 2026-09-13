//! Source selection and a bounded, virtualized game-output viewer.

use std::collections::VecDeque;

use gpui_kit::component::{
    ActiveTheme as _, Icon, IndexPath, h_flex,
    input::{Input, InputEvent, InputState},
    scroll::Scrollbar,
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement as _,
    ListHorizontalSizingBehavior, ParentElement as _, Render, SharedString, Styled as _,
    Subscription, UniformListScrollHandle, Window, div, rems, uniform_list,
};

use super::{missing_resource, widgets::*};
use crate::{
    assets::PhaIcon,
    route::InstanceRef,
    state::{AppState, LogLevel, SessionStore},
};

const SOURCES: [&str; 3] = ["Live output", "latest.log", "debug.log"];
const LEVELS: [&str; 5] = ["All levels", "Info", "Warning", "Error", "Debug"];

// Only the holder is observed by the route renderer. Log changes notify the
// nested view, so streaming output does not invalidate the sidebar and shell.
struct LogsView {
    view: Entity<LogsPage>,
}

struct LogsPage {
    app: Entity<AppState>,
    reference: InstanceRef,
    sessions: Entity<SessionStore>,
    source: Entity<SelectState<Vec<&'static str>>>,
    level: Entity<SelectState<Vec<&'static str>>>,
    query: Entity<InputState>,
    live: VecDeque<SharedString>,
    latest: Vec<SharedString>,
    debug: Vec<SharedString>,
    scroll: UniformListScrollHandle,
    _subscriptions: Vec<Subscription>,
    #[cfg(feature = "seed")]
    _stream: gpui_kit::Task<()>,
}

pub fn render(
    reference: &InstanceRef,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    if app.read(cx).instances.read(cx).find(reference).is_none() {
        return missing_resource("instance", app).into_any_element();
    }
    let reference = reference.clone();
    let holder = window.use_keyed_state(
        instance_key(&reference, "logs-page"),
        cx,
        move |window, cx| LogsView {
            view: cx.new(|cx| LogsPage::new(reference, app, window, cx)),
        },
    );
    holder.read(cx).view.clone().into_any_element()
}

impl LogsPage {
    fn new(
        reference: InstanceRef,
        app: Entity<AppState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let source =
            cx.new(|cx| SelectState::new(SOURCES.to_vec(), Some(IndexPath::new(0)), window, cx));
        let level =
            cx.new(|cx| SelectState::new(LEVELS.to_vec(), Some(IndexPath::new(0)), window, cx));
        let query = cx.new(|cx| InputState::new(window, cx).placeholder("Search log entries…"));
        let sessions = app.read(cx).sessions.clone();
        let subscriptions = vec![
            cx.subscribe(&source, |this: &mut Self, _, event, cx| {
                if matches!(event, SelectEvent::Confirm(_)) {
                    this.reset_scroll();
                    cx.notify();
                }
            }),
            cx.subscribe(&level, |this: &mut Self, _, event, cx| {
                if matches!(event, SelectEvent::Confirm(_)) {
                    this.reset_scroll();
                    cx.notify();
                }
            }),
            cx.subscribe(&query, |this: &mut Self, _, event, cx| {
                if matches!(event, InputEvent::Change) {
                    this.reset_scroll();
                    cx.notify();
                }
            }),
            cx.observe(&sessions, |_, _, cx| cx.notify()),
        ];
        #[cfg(feature = "seed")]
        let (live, latest, debug) = {
            let name = &app
                .read(cx)
                .instances
                .read(cx)
                .find(&reference)
                .unwrap()
                .name;
            (
                crate::seed::seed_instance_log(name)
                    .into_iter()
                    .map(|line| line.text.into())
                    .collect(),
                crate::seed::seed_latest_log()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                crate::seed::seed_debug_log()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            )
        };
        #[cfg(not(feature = "seed"))]
        let (live, latest, debug) = (VecDeque::new(), Vec::new(), Vec::new());
        #[cfg(feature = "seed")]
        let stream = cx.spawn(async move |this, cx| {
            let mut sample = 0;
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(1500))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        if !this.sessions.read(cx).is_running(&this.reference) {
                            return;
                        }
                        let follow = {
                            let handle = &this.scroll.0.borrow().base_handle;
                            handle.max_offset().y + handle.offset().y <= gpui_kit::px(24.)
                        };
                        this.live.push_back(
                            format!(
                                "[{}] {}",
                                chrono::Local::now().format("%H:%M:%S"),
                                crate::seed::live_output_samples()[sample % 4]
                            )
                            .into(),
                        );
                        sample += 1;
                        while this.live.len() > crate::state::LiveLogStore::MAX_LINES {
                            this.live.pop_front();
                        }
                        if this.source.read(cx).selected_value() == Some(&SOURCES[0]) {
                            if follow {
                                this.scroll.scroll_to_bottom();
                            }
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            app,
            reference,
            sessions,
            source,
            level,
            query,
            live,
            latest,
            debug,
            scroll: UniformListScrollHandle::new(),
            _subscriptions: subscriptions,
            #[cfg(feature = "seed")]
            _stream: stream,
        }
    }

    fn reset_scroll(&self) {
        self.scroll
            .scroll_to_item_strict(0, gpui_kit::ScrollStrategy::Top);
    }

    fn filtered_lines(&self, cx: &App) -> Vec<SharedString> {
        let source = self
            .source
            .read(cx)
            .selected_value()
            .copied()
            .unwrap_or(SOURCES[0]);
        let level = self
            .level
            .read(cx)
            .selected_value()
            .copied()
            .unwrap_or(LEVELS[0]);
        let query = self.query.read(cx).value().trim().to_lowercase();
        let lines: Box<dyn Iterator<Item = &SharedString> + '_> = match source {
            "latest.log" => Box::new(self.latest.iter()),
            "debug.log" => Box::new(self.debug.iter()),
            _ => Box::new(self.live.iter()),
        };
        lines
            .filter(|line| {
                (level == LEVELS[0] || LogLevel::of(line).label() == level)
                    && (query.is_empty() || line.to_lowercase().contains(&query))
            })
            .cloned()
            .collect()
    }
}

impl Render for LogsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        let Some(instance) = self
            .app
            .read(cx)
            .instances
            .read(cx)
            .find(&self.reference)
            .cloned()
        else {
            return missing_resource("instance", self.app.clone()).into_any_element();
        };
        let live = self.source.read(cx).selected_value() == Some(&SOURCES[0])
            && self.sessions.read(cx).is_running(&self.reference);
        let mut title = heading(PhaIcon::ScrollText, "Game logs", cx);
        if live {
            title = title.child(badge("● Live"));
        }
        let header = instance_header(
            &instance,
            title,
            if live {
                "Streaming the game process output."
            } else {
                "Inspect output captured for this instance."
            },
            div(),
            self.app.clone(),
            cx,
        );
        let lines = self.filtered_lines(cx);
        let terminal = if lines.is_empty() {
            v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .gap_2()
                .child(
                    Icon::new(PhaIcon::FileText)
                        .size_6()
                        .text_color(cx.theme().muted_foreground),
                )
                .child(muted("No matching log entries.", cx))
                .into_any_element()
        } else {
            div()
                .relative()
                .size_full()
                .child(
                    uniform_list("log-lines", lines.len(), move |range, _, cx| {
                        range
                            .map(|ix| {
                                let color = match LogLevel::of(&lines[ix]) {
                                    LogLevel::Info => cx.theme().primary.opacity(0.8),
                                    LogLevel::Warn => cx.theme().warning,
                                    LogLevel::Error => cx.theme().danger,
                                    LogLevel::Debug => cx.theme().muted_foreground,
                                };
                                div()
                                    .h(rems(1.5))
                                    .font_family(crate::theme::MONO_FONT_FAMILY)
                                    .text_sm()
                                    .text_color(color)
                                    .whitespace_nowrap()
                                    .child(lines[ix].clone())
                            })
                            .collect::<Vec<_>>()
                    })
                    .size_full()
                    .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
                    .track_scroll(&self.scroll),
                )
                .child(Scrollbar::new(&self.scroll))
                .into_any_element()
        };
        v_flex()
            .size_full()
            .min_h_0()
            .min_w_0()
            .child(header)
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .p_6()
                    .gap_4()
                    .child(
                        h_flex()
                            .debug_selector(|| "log-filters".into())
                            .flex_shrink_0()
                            .flex_wrap()
                            .gap_3()
                            .child(
                                control_slot(
                                    Select::new(&self.source)
                                        .w_full()
                                        .accessibility_label("Log source"),
                                )
                                .w(rems(11.)),
                            )
                            .child(
                                control_slot(
                                    Select::new(&self.level)
                                        .w_full()
                                        .accessibility_label("Log level"),
                                )
                                .w(rems(9.)),
                            )
                            .child(
                                div().h_8().flex_1().min_w_48().child(
                                    Input::new(&self.query)
                                        .prefix(Icon::new(PhaIcon::Search).size_4())
                                        .cleanable(true),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .debug_selector(|| "log-terminal".into())
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .p_4()
                            .rounded(cx.theme().radius)
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(crate::theme::terminal())
                            .child(terminal),
                    ),
            )
            .into_any_element()
    }
}
