use termwiz::surface::Surface;

use super::{
    Component,
    ComponentData,
};
use crate::event_loop::{
    State,
    TreeElement,
};
use crate::input::{
    InputAction,
    MouseAction,
};
use crate::surface_ext::SurfaceExt;
use crate::Display;

#[derive(Debug)]
pub enum Layout {
    Vertical,
    Horizontal,
}

#[derive(Debug)]
pub struct Div {
    layout: Layout,
    inner: ComponentData,
}

impl Div {
    pub fn new() -> Self {
        Self {
            layout: Layout::Vertical,
            inner: ComponentData::new("div".to_owned(), false),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner.id = Some(id.into());
        self
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.inner.classes.push(class.into());
        self
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn push(mut self, component: impl Component + 'static) -> Self {
        self.inner.children.push(Box::new(component));
        self
    }
}

impl Component for Div {
    fn draw(&self, state: &mut State, surface: &mut Surface, mut x: f64, mut y: f64, width: f64, height: f64) {
        let initial_x = x;
        let initial_y = y;

        let mut previous_siblings = std::collections::LinkedList::new();
        for child in self.inner.children.iter() {
            let style_info = child.inner().style_info();
            state.tree.push(TreeElement {
                inner: style_info.clone(),
                siblings: previous_siblings.clone(),
            });
            previous_siblings.push_front(style_info);
            let style = child.style(state);

            if let Display::None = style.display() {
                state.tree.pop();
                continue;
            }

            let size = child.size(state);

            let content_width = style
                .width()
                .map(|width| width - style.padding_horizontal() - style.border_horizontal())
                .unwrap_or(size.0)
                .min(width - style.spacing_horizontal() - (x - initial_x));

            let content_height = style
                .height()
                .map(|height| (height - style.padding_vertical() - style.border_vertical()))
                .unwrap_or(size.1)
                .min(height - style.spacing_vertical() - (y - initial_y));

            surface.draw_border(
                x + style.margin_left(),
                y + style.margin_top(),
                style.border_horizontal() + style.padding_horizontal() + content_width,
                style.border_vertical() + style.padding_vertical() + content_height,
                &style,
            );

            child.draw(
                state,
                surface,
                x + style.margin_left() + style.border_left_width() + style.padding_left(),
                y + style.margin_top() + style.border_top_width() + style.padding_top(),
                content_width,
                content_height,
            );

            match self.layout {
                Layout::Vertical => y += style.spacing_vertical() + content_height,
                Layout::Horizontal => x += style.spacing_horizontal() + content_width,
            }

            state.tree.pop();
        }
    }

    fn on_input_action(&mut self, state: &mut State, input_action: &InputAction) {
        if let Some(child) = self.inner.focused_child() {
            child.on_input_action(state, input_action);
        }
    }

    fn on_mouse_action(
        &mut self,
        state: &mut State,
        mouse_action: &MouseAction,
        mut x: f64,
        mut y: f64,
        width: f64,
        height: f64,
    ) {
        let mut previous_siblings = std::collections::LinkedList::new();
        for i in 0..self.inner.children.len() {
            self.inner.children[i].inner_mut().hover = false;

            let style_info = self.inner.children[i].inner().style_info();
            state.tree.push(TreeElement {
                inner: style_info.clone(),
                siblings: previous_siblings.clone(),
            });
            previous_siblings.push_front(style_info);
            let style = self.inner.children[i].style(state);

            if let Display::None = style.display() {
                state.tree.pop();
                continue;
            }

            let size = self.inner.children[i].size(state);

            let content_width = style
                .width()
                .map(|width| width - style.padding_horizontal() - style.border_horizontal())
                .unwrap_or(size.0)
                .min(width - style.spacing_horizontal());

            let content_height = style
                .height()
                .map(|height| (height - style.padding_vertical() - style.border_vertical()))
                .unwrap_or(size.1)
                .min(height - style.spacing_vertical());

            if mouse_action.x >= x + style.margin_left()
                && mouse_action.x
                    < style.margin_left() + style.border_horizontal() + style.padding_horizontal() + content_width
                && mouse_action.y >= y + style.margin_top()
                && mouse_action.y
                    < y + style.margin_top() + style.border_vertical() + style.padding_vertical() + content_height
            {
                self.inner.children[i].on_mouse_action(
                    state,
                    mouse_action,
                    x + style.margin_left() + style.border_left_width() + style.padding_left(),
                    y + style.margin_top() + style.border_top_width() + style.padding_top(),
                    content_width,
                    content_height,
                );

                if mouse_action.just_pressed && self.inner.children[i].interactive() {
                    self.inner.focus_child_at_index(state, Some(i));
                    state.tree.pop();
                    return;
                }

                self.inner.children[i].inner_mut().hover = true;
            }

            match self.layout {
                Layout::Vertical => y += style.spacing_vertical() + content_height,
                Layout::Horizontal => x += style.spacing_horizontal() + content_width,
            }

            state.tree.pop();
        }

        if mouse_action.just_pressed && self.interactive() {
            self.on_focus(state, true);
        }
    }

    fn next(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        self.inner
            .focused_child()
            .and_then(|child| child.next(state, false))
            .or_else(|| {
                // If currently focused element doesn't have another interactive element with in it
                // we iterate through the children, wrapping if necessary.
                let next_child_idx = self.inner.find_next_child(
                    |c| c.interactive(),
                    self.inner.focused_child_index.map(|x| x + 1),
                    wrap,
                );

                // Traverse tree to get next id before we focus.
                let next_id = next_child_idx.and_then(|idx| {
                    let child = &mut self.inner.children[idx];
                    child.next(state, false).or_else(|| child.id().to_owned())
                });

                self.inner.focus_child_at_index(state, next_child_idx);

                next_id
            })
    }

    fn prev(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        self.inner
            .focused_child()
            .and_then(|child| child.prev(state, false))
            .or_else(|| {
                // If currently focused element doesn't have another interactive element with in it
                // we iterate through the children, wrapping if necessary.
                let prev_child_idx =
                    self.inner
                        .find_prev_child(|c| c.interactive(), self.inner.focused_child_index, wrap);

                // Traverse tree to get previous id before we focus.
                let prev_id = prev_child_idx.and_then(|idx| {
                    let child = &mut self.inner.children[idx];
                    child.prev(state, false).or_else(|| child.id().to_owned())
                });

                self.inner.focus_child_at_index(state, prev_child_idx);

                prev_id
            })
    }

    fn remove(&mut self, id: &str) -> Option<Box<dyn Component>> {
        for i in 0..self.inner.children.len() {
            if let Some(removed) = self.inner.children[i].remove(id) {
                return Some(removed);
            } else if let Some(child_id) = self.inner.children[i].id() {
                if child_id == id {
                    self.inner.focused_child_index = None;
                    return Some(self.inner.children.remove(i));
                }
            }
        }

        None
    }

    fn insert(&mut self, id: &str, mut component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        for (i, child) in self.inner.children.iter_mut().enumerate() {
            if let Some(child_id) = child.id() {
                if child_id == id {
                    self.inner.focused_child_index = None;
                    self.inner.children.insert(i + 1, component);
                    return None;
                }
            }

            component = child.insert(id, component)?;
        }

        Some(component)
    }

    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner.focus = focus;
        match self.inner.focused_child() {
            Some(child) => {
                child.on_focus(state, focus);
            },
            None => {
                if focus {
                    let focus_index = self.inner.find_next_child(|c| c.interactive(), None, false);
                    self.inner.focus_child_at_index(state, focus_index);
                }
            },
        }
    }

    fn interactive(&self) -> bool {
        self.inner.interactive()
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }

    fn size(&self, state: &mut State) -> (f64, f64) {
        let mut width = 0_f64;
        let mut height = 0_f64;

        let mut previous_siblings = std::collections::LinkedList::new();
        for child in self.inner.children.iter() {
            let style_info = child.inner().style_info();
            state.tree.push(TreeElement {
                inner: style_info.clone(),
                siblings: previous_siblings.clone(),
            });
            previous_siblings.push_front(style_info);
            let style = child.style(state);

            if let Display::None = style.display() {
                state.tree.pop();
                continue;
            }

            let size = child.size(state);

            let content_width = style
                .width()
                .map(|width| width - style.padding_horizontal() - style.border_horizontal())
                .unwrap_or(size.0);

            let content_height = style
                .height()
                .map(|height| height - style.padding_vertical() - style.border_vertical())
                .unwrap_or(size.1);

            match self.layout {
                Layout::Vertical => {
                    width = width.max(content_width + style.spacing_horizontal());
                    height += content_height + style.spacing_vertical();
                },
                Layout::Horizontal => {
                    width += content_width + style.spacing_horizontal();
                    height = height.max(content_height + style.spacing_vertical());
                },
            };

            state.tree.pop();
        }

        (width, height)
    }
}

impl Default for Div {
    fn default() -> Self {
        Self::new()
    }
}
