//! Shared instance commands for the UI preview and its menus.

#[cfg(feature = "seed")]
use crate::state::{SessionId, SessionSummary};
use crate::{
    route::{InstanceRef, Route},
    state::AppState,
};
use gpui_kit::component::{WindowExt as _, notification::Notification};
use gpui_kit::{App, Entity, Window};

/// The seeded app previews process state without spawning Minecraft.
pub(crate) fn play(
    reference: &InstanceRef,
    app: &Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) {
    if app.read(cx).sessions.read(cx).is_running(reference) {
        stop(reference, app, cx);
    } else {
        start(reference, app, window, cx);
    }
}

pub(crate) fn stop(reference: &InstanceRef, app: &Entity<AppState>, cx: &mut App) {
    app.read(cx).sessions.clone().update(cx, |sessions, cx| {
        if sessions.stop_instance(reference, 0) {
            cx.notify();
        }
    });
}

/// Retry and quick-play actions are idempotent while an instance is running.
pub(crate) fn start(
    reference: &InstanceRef,
    app: &Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) {
    #[cfg(not(feature = "seed"))]
    {
        let _ = (reference, app);
        window.push_notification(
            Notification::info("Game launching is not connected yet."),
            cx,
        );
    }
    #[cfg(feature = "seed")]
    {
        let _ = window;
        app.update(cx, |app, cx| {
            if app.instances.read(cx).find(reference).is_none()
                || app.sessions.read(cx).is_running(reference)
            {
                return;
            }
            let id = SessionId(format!(
                "preview-{}-{}",
                reference.instance_id,
                app.sessions.read(cx).revision()
            ));
            app.sessions.update(cx, |sessions, cx| {
                if sessions.start(SessionSummary {
                    id,
                    instance: reference.clone(),
                    started_at: "just now".into(),
                    exit_code: None,
                    running: true,
                }) {
                    cx.notify();
                }
            });
            app.instances.update(cx, |instances, cx| {
                if instances.record_launch(reference) {
                    cx.notify();
                }
            });
        });
    }
}

pub(crate) fn duplicate(reference: &InstanceRef, app: &Entity<AppState>, cx: &mut App) {
    app.update(cx, |app, cx| {
        let created = app.instances.update(cx, |instances, cx| {
            let created = instances.duplicate(reference);
            if created.is_some() {
                cx.notify();
            }
            created
        });
        if let Some(reference) = created {
            app.push(Route::InstanceDetail(reference), cx);
        }
    });
}

pub(crate) fn delete(
    reference: InstanceRef,
    name: String,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) {
    super::confirm_dialog::open(
        window,
        cx,
        format!("Delete “{name}”?"),
        "The instance and its worlds, mods and resource packs are removed.",
        move |_, _, cx| {
            app.update(cx, |app, cx| {
                app.instances.update(cx, |instances, cx| {
                    if instances.remove(&reference) {
                        cx.notify();
                    }
                });
                app.sessions.update(cx, |sessions, cx| {
                    if sessions.remove_instance(&reference) {
                        cx.notify();
                    }
                });
                app.crashes.update(cx, |crashes, cx| {
                    if crashes.remove_instance(&reference) {
                        cx.notify();
                    }
                });
                app.replace(Route::Instances, cx);
            });
            true
        },
    );
}

pub(crate) fn open_folder(reference: &InstanceRef, window: &mut Window, cx: &mut App) {
    let path = reference
        .storage
        .root_dir
        .join("instances")
        .join(&reference.instance_id);
    if path.is_dir() {
        cx.reveal_path(&path);
    } else {
        window.push_notification(
            Notification::info("The instance folder will be available after installation."),
            cx,
        );
    }
}
