mod base_keymap_setting;
mod multibuffer_hint;

use client::telemetry::Telemetry;
use db::kvp::KEY_VALUE_STORE;
use gpui::{
    actions, svg, AppContext, EventEmitter, FocusHandle, FocusableView, InteractiveElement,
    ParentElement, Render, Styled, Subscription, Task, View, ViewContext, VisualContext, WeakView,
    WindowContext,
};
use settings::{Settings, SettingsStore};
use std::sync::Arc;
use ui::prelude::*;
use workspace::{
    dock::DockPosition,
    item::{Item, ItemEvent},
    open_new, AppState, Welcome, Workspace, WorkspaceId,
};

pub use base_keymap_setting::BaseKeymap;
pub use multibuffer_hint::*;

actions!(welcome, [ResetHints]);

pub const FIRST_OPEN: &str = "first_open";

pub fn init(cx: &mut AppContext) {
    BaseKeymap::register(cx);

    cx.observe_new_views(|workspace: &mut Workspace, _cx| {
        workspace.register_action(|workspace, _: &Welcome, cx| {
            let welcome_page = WelcomePage::new(workspace, cx);
            workspace.add_item_to_active_pane(Box::new(welcome_page), None, true, cx)
        });
        workspace
            .register_action(|_workspace, _: &ResetHints, cx| MultibufferHint::set_count(0, cx));
    })
    .detach();
}

pub fn show_welcome_view(
    app_state: Arc<AppState>,
    cx: &mut AppContext,
) -> Task<anyhow::Result<()>> {
    open_new(Default::default(), app_state, cx, |workspace, cx| {
        workspace.toggle_dock(DockPosition::Left, cx);
        let welcome_page = WelcomePage::new(workspace, cx);
        workspace.add_item_to_center(Box::new(welcome_page.clone()), cx);
        cx.focus_view(&welcome_page);
        cx.notify();

        db::write_and_log(cx, || {
            KEY_VALUE_STORE.write_kvp(FIRST_OPEN.to_string(), "false".to_string())
        });
    })
}

pub struct WelcomePage {
    workspace: WeakView<Workspace>,
    focus_handle: FocusHandle,
    telemetry: Arc<Telemetry>,
    _settings_subscription: Subscription,
}

impl Render for WelcomePage {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .track_focus(&self.focus_handle)
            .child(
                v_flex()
                    .w_96()
                    .gap_4()
                    .mx_auto()
                    .child(
                        svg()
                            .path("icons/logo_96.svg")
                            .text_color(gpui::white())
                            .w(px(96.))
                            .h(px(96.))
                            .mx_auto(),
                    )
                    .child(
                        h_flex()
                            .justify_center()
                            .child(Label::new("Code at the speed of thought")),
                    )
                    .child(
                        v_flex().gap_2().child(
                            Button::new("choose-theme", "Choose a theme")
                                .full_width()
                                .on_click(cx.listener(|this, _, cx| {
                                    this.telemetry
                                        .report_app_event("welcome page: change theme".to_string());
                                    this.workspace
                                        .update(cx, |workspace, cx| {
                                            theme_selector::toggle(
                                                workspace,
                                                &Default::default(),
                                                cx,
                                            )
                                        })
                                        .ok();
                                })),
                        ),
                    ),
            )
    }
}

impl WelcomePage {
    pub fn new(workspace: &Workspace, cx: &mut ViewContext<Workspace>) -> View<Self> {
        let this = cx.new_view(|cx| {
            cx.on_release(|this: &mut Self, _, _| {
                this.telemetry
                    .report_app_event("welcome page: close".to_string());
            })
            .detach();

            WelcomePage {
                focus_handle: cx.focus_handle(),
                workspace: workspace.weak_handle(),
                telemetry: workspace.client().telemetry().clone(),
                _settings_subscription: cx
                    .observe_global::<SettingsStore>(move |_, cx| cx.notify()),
            }
        });

        this
    }
}

impl EventEmitter<ItemEvent> for WelcomePage {}

impl FocusableView for WelcomePage {
    fn focus_handle(&self, _: &AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Item for WelcomePage {
    type Event = ItemEvent;

    fn tab_content_text(&self, _cx: &WindowContext) -> Option<SharedString> {
        Some("Welcome to Zed!".into())
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("welcome page")
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<WorkspaceId>,
        cx: &mut ViewContext<Self>,
    ) -> Option<View<Self>> {
        Some(cx.new_view(|cx| WelcomePage {
            focus_handle: cx.focus_handle(),
            workspace: self.workspace.clone(),
            telemetry: self.telemetry.clone(),
            _settings_subscription: cx.observe_global::<SettingsStore>(move |_, cx| cx.notify()),
        }))
    }

    fn to_item_events(event: &Self::Event, mut f: impl FnMut(workspace::item::ItemEvent)) {
        f(*event)
    }
}
