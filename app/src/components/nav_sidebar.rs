//! Primary navigation sidebar and its instance navigation entries.

use gpui::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, StyledExt,
    button::ButtonVariants as _,
    sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarMenu, SidebarMenuItem},
    v_flex,
};

use crate::{route::Route, state::AppState};

fn activate(
    app: Entity<AppState>,
    route: Route,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) {
    move |_, _, cx| app.update(cx, |state, cx| state.push(route.clone(), cx))
}

pub fn render(app: Entity<AppState>, cx: &App) -> impl IntoElement {
    let state = app.read(cx);
    let instances = state.instances.read(cx);
    let sessions = state.sessions.read(cx);
    let current = state.route().clone();
    let favorites = instances.favorites().collect::<Vec<_>>();
    let local = instances.local().collect::<Vec<_>>();
    let aphanite = instances.aphanite_unfavorited().collect::<Vec<_>>();
    let account = state
        .accounts
        .read(cx)
        .active()
        .map(|item| item.username.clone());
    let primary = cx.theme().primary;

    let instance_item = |instance: &crate::state::InstanceSummary| {
        let reference = instance.reference();
        SidebarMenuItem::new(instance.name.clone())
            .icon(crate::assets::PhaIcon::Layers)
            .active(matches!(&current, Route::InstanceDetail(active) if active == &reference))
            .suffix({
                let running = sessions.is_running(&reference);
                move |_, _| {
                    if running {
                        div().size_2().rounded_full().bg(primary)
                    } else {
                        div()
                    }
                }
            })
            .on_click(activate(app.clone(), Route::InstanceDetail(reference)))
    };

    v_flex()
        .h_full()
        .w(gpui::px(208.))
        .flex_shrink_0()
        .border_r_1()
        .border_color(cx.theme().sidebar_border)
        .child(
            Sidebar::new("launcher-sidebar")
                .child(
                    SidebarGroup::new("LIBRARY").child(
                        SidebarMenu::new()
                            .child(
                                SidebarMenuItem::new("Quick Play")
                                    .icon(IconName::Play)
                                    .active(matches!(current, Route::Play))
                                    .suffix({
                                        let count = sessions.running_count();
                                        move |_, _| {
                                            if count > 0 {
                                                div()
                                                    .text_xs()
                                                    .text_color(primary)
                                                    .child(count.to_string())
                                            } else {
                                                div()
                                            }
                                        }
                                    })
                                    .on_click(activate(app.clone(), Route::Play)),
                            )
                            .child(
                                SidebarMenuItem::new(format!("Favorites {}", favorites.len()))
                                    .icon(IconName::Star)
                                    .children(favorites.iter().map(|item| instance_item(item))),
                            )
                            .child(
                                SidebarMenuItem::new(format!("Instances {}", local.len()))
                                    .icon(IconName::Folder)
                                    .active(matches!(current, Route::Instances))
                                    .click_to_toggle(true)
                                    .children(local.iter().map(|item| instance_item(item)))
                                    .on_click(activate(app.clone(), Route::Instances)),
                            )
                            .child(
                                SidebarMenuItem::new(format!("Aphanite {}", aphanite.len()))
                                    .icon(crate::assets::PhaIcon::Layers)
                                    .active(matches!(current, Route::Aphanite))
                                    .click_to_toggle(true)
                                    .children(aphanite.iter().map(|item| instance_item(item)))
                                    .on_click(activate(app.clone(), Route::Aphanite)),
                            ),
                    ),
                )
                .footer(
                    SidebarFooter::new().child(
                        v_flex()
                            .gap_2()
                            .child(
                                gpui_component::button::Button::new("sidebar-account")
                                    .ghost()
                                    .label(account.unwrap_or_else(|| "Offline".into()))
                                    .on_click({
                                        let app = app.clone();
                                        move |_, _, cx| {
                                            app.update(cx, |state, cx| {
                                                state.push(Route::Accounts, cx)
                                            })
                                        }
                                    }),
                            )
                            .child(
                                gpui_component::button::Button::new("sidebar-settings")
                                    .ghost()
                                    .icon(IconName::Settings)
                                    .label("Settings")
                                    .on_click(move |_, _, cx| {
                                        app.update(cx, |state, cx| state.push(Route::Settings, cx))
                                    }),
                            ),
                    ),
                ),
        )
}
