use gpui::{prelude::*, AppContext, FocusHandle, FocusableView, View};
use ui::prelude::*;

/// The head of a [`Picker`](crate::Picker).
pub(crate) enum Head {
    /// Picker has no head, it's just a list of items.
    Empty(View<EmptyHead>),
}

impl Head {
    pub fn empty<V: 'static>(
        blur_handler: impl FnMut(&mut V, &mut ViewContext<'_, V>) + 'static,
        cx: &mut ViewContext<V>,
    ) -> Self {
        let head = cx.new_view(|cx| EmptyHead::new(cx));
        cx.on_blur(&head.focus_handle(cx), blur_handler).detach();
        Self::Empty(head)
    }
}

/// An invisible element that can hold focus.
pub(crate) struct EmptyHead {
    focus_handle: FocusHandle,
}

impl EmptyHead {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for EmptyHead {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        div().track_focus(&self.focus_handle)
    }
}

impl FocusableView for EmptyHead {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}
