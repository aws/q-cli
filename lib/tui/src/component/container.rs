use termwiz::input::{
    MouseButtons,
    MouseEvent,
};
use termwiz::surface::Surface;

use super::{
    Component,
    ComponentData,
};
use crate::event_loop::{
    State,
    TreeElement,
};
use crate::input::InputAction;
use crate::surface_ext::SurfaceExt;
use crate::Display;

#[derive(Debug)]
pub enum Layout {
    Vertical,
    Horizontal,
}

#[derive(Debug)]
pub struct Container {
    layout: Layout,
    inner: ComponentData,
}

impl Container {
    pub fn new(id: impl Into<String>, layout: Layout) -> Self {
        Self {
            layout,
            inner: ComponentData::new("div".to_owned(), id.into(), false),
        }
    }

    pub fn push(mut self, component: impl Component + 'static) -> Self {
        self.inner.children.push(Box::new(component));
        self
    }

    fn resize(&mut self, state: &mut State) {
        let mut previous_siblings = std::collections::LinkedList::new();
        (self.inner.width, self.inner.height) = self.inner.children.iter().fold((0.0_f64, 0.0_f64), |acc, c| {
            let style_info = c.inner().style_info();
            state.tree.push(TreeElement {
                inner: style_info.clone(),
                siblings: previous_siblings.clone(),
            });
            previous_siblings.push_front(style_info);
            let style = c.style(state);

            if let Display::None = style.display() {
                return acc;
            }

            let acc = match self.layout {
                Layout::Vertical => (
                    acc.0
                        .max(style.width().unwrap_or_else(|| c.width()) + style.spacing_horizontal()),
                    acc.1 + style.height().unwrap_or_else(|| c.height()) + style.spacing_vertical(),
                ),
                Layout::Horizontal => (
                    acc.0 + style.width().unwrap_or_else(|| c.width()) + style.spacing_horizontal(),
                    acc.1
                        .max(style.height().unwrap_or_else(|| c.height()) + style.spacing_vertical()),
                ),
            };

            state.tree.pop();
            acc
        });
    }
}

impl Component for Container {
    fn initialize(&mut self, state: &mut State) {
        for component in &mut self.inner.children {
            component.initialize(state);
        }

        self.resize(state);
    }

    fn draw(
        &self,
        state: &mut State,
        surface: &mut Surface,
        mut x: f64,
        mut y: f64,
        width: f64,
        height: f64,
        screen_width: f64,
        screen_height: f64,
    ) {
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

            let mut cx = x + style.margin_left();
            let mut cy = y + style.margin_top();

            let mut width = (style.width().unwrap_or_else(|| child.width())
                + style.border_horizontal()
                + style.padding_horizontal())
            .min(width);
            let mut height =
                (style.height().unwrap_or_else(|| child.height()) + style.border_vertical() + style.padding_vertical())
                    .min(height);

            surface.draw_border(&mut cx, &mut cy, &mut width, &mut height, &style);
            child.draw(state, surface, cx, cy, width, height, screen_width, screen_height);

            match self.layout {
                Layout::Vertical => y += style.height().unwrap_or_else(|| child.height()) + style.spacing_vertical(),
                Layout::Horizontal => x += style.width().unwrap_or_else(|| child.width()) + style.spacing_horizontal(),
            }

            state.tree.pop();
        }
    }

    fn on_input_action(&mut self, state: &mut State, input_action: InputAction) -> Option<bool> {
        let child_did_consume_enter = self
            .inner
            .focused_child()
            .and_then(|child| child.on_input_action(state, input_action))
            .unwrap_or(false);

        self.resize(state);

        Some(child_did_consume_enter)
    }

    fn on_mouse_event(
        &mut self,
        state: &mut State,
        mouse_event: &MouseEvent,
        mut x: f64,
        mut y: f64,
        width: f64,
        height: f64,
    ) {
        if !mouse_event.mouse_buttons.contains(MouseButtons::LEFT) {
            return;
        }

        for i in 0..self.inner.children.len() {
            let style = self.inner.children[i].style(state);
            let mut cx = x;
            let mut cy = y;

            let mut width = (style.width().unwrap_or_else(|| self.inner.children[i].width())
                + style.spacing_horizontal())
            .min(width);
            let mut height = (style.height().unwrap_or_else(|| self.inner.children[i].height())
                + style.spacing_vertical())
            .min(height);

            cx += style.margin_left();
            cy += style.margin_top();
            width -= style.margin_horizontal();
            height -= style.margin_vertical();

            if mouse_event.x as f64 >= cx
                && mouse_event.x as f64 <= cx + width
                && mouse_event.y as f64 >= cy
                && mouse_event.y as f64 <= cy + height
            {
                self.inner.focus_child_at_index(state, Some(i));
                self.inner.children[i].on_mouse_event(state, mouse_event, x, y, width, height);
            }

            match self.layout {
                Layout::Vertical => y += height,
                Layout::Horizontal => x += width,
            }
        }
    }

    fn next(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        let next = self
            .inner
            .focused_child()
            .and_then(|child| child.next(state, false))
            .or_else(|| {
                // If currently focused element doesn't have another interactive element with in it
                // we iterate through the children, wrapping if necessary.
                let next_child_idx = self.inner.find_next_child(
                    |c| c.interactive(state),
                    self.inner.focused_child_index.map(|x| x + 1),
                    wrap,
                );

                // Traverse tree to get next id before we focus.
                let next_id = next_child_idx.map(|idx| {
                    let child = &mut self.inner.children[idx];
                    child.next(state, false).unwrap_or_else(|| child.id().to_owned())
                });
                self.inner.focus_child_at_index(state, next_child_idx);

                next_id
            });

        self.resize(state);
        next
    }

    fn prev(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        let prev = self
            .inner
            .focused_child()
            .and_then(|child| child.prev(state, false))
            .or_else(|| {
                // If currently focused element doesn't have another interactive element with in it
                // we iterate through the children, wrapping if necessary.
                let prev_child_idx =
                    self.inner
                        .find_prev_child(|c| c.interactive(state), self.inner.focused_child_index, wrap);

                // Traverse tree to get previous id before we focus.
                let prev_id = prev_child_idx.map(|idx| {
                    let child = &mut self.inner.children[idx];
                    child.prev(state, false).unwrap_or_else(|| child.id().to_owned())
                });

                self.inner.focus_child_at_index(state, prev_child_idx);

                prev_id
            });

        self.resize(state);
        prev
    }

    fn remove(&mut self, id: &str) -> Option<Box<dyn Component>> {
        for i in 0..self.inner.children.len() {
            if self.inner.children[i].id() == id {
                self.inner.focused_child_index = None;
                return Some(self.inner.children.remove(i));
            }
        }

        None
    }

    fn insert(&mut self, id: &str, mut component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        for (i, child) in self.inner.children.iter_mut().enumerate() {
            if child.id() == id {
                self.inner.focused_child_index = None;
                self.inner.children.insert(i + 1, component);
                return None;
            }

            component = child.insert(id, component)?;
        }

        Some(component)
    }

    fn replace(&mut self, id: &str, mut component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        for (i, child) in self.inner.children.iter_mut().enumerate() {
            if child.id() == id {
                if let Some(focused_child_index) = self.inner.focused_child_index {
                    if focused_child_index == i {
                        self.inner.focused_child_index = None;
                    }
                }

                let removed = self.inner.children.remove(i);
                self.inner.children.insert(i, component);
                return Some(removed);
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
                    let focus_index = self.inner.find_next_child(|c| c.interactive(state), None, false);
                    self.inner.focus_child_at_index(state, focus_index);
                }
            },
        }

        self.resize(state);
    }

    fn on_paste(&mut self, state: &mut State, clipboard: &str) {
        if let Some(child) = self.inner.focused_child() {
            child.on_paste(state, clipboard)
        }
    }

    fn interactive(&self, state: &mut State) -> bool {
        self.inner.interactive(state)
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }
}
