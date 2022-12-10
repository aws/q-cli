use termwiz::input::{
    MouseButtons,
    MouseEvent,
};
use termwiz::surface::Surface;

use super::{
    Component,
    ComponentData,
};
use crate::event_loop::State;
use crate::input::InputAction;
use crate::surface_ext::SurfaceExt;

#[derive(Debug)]
pub enum Layout {
    Vertical,
    Horizontal,
}

#[derive(Debug)]
pub struct Container {
    components: Vec<Box<dyn Component + 'static>>,
    layout: Layout,
    active: Option<usize>,
    inner: ComponentData,
}

impl Container {
    pub fn new(id: impl Into<String>, layout: Layout) -> Self {
        Self {
            components: vec![],
            active: None,
            layout,
            inner: ComponentData::new(id.into(), false),
        }
    }

    pub fn push(mut self, component: impl Component + 'static) -> Self {
        self.components.push(Box::new(component));
        self
    }

    fn resize(&mut self, state: &mut State) {
        (self.inner.width, self.inner.height) = self.components.iter().fold((0.0_f64, 0.0_f64), |acc, c| {
            let style = c.style(state);

            match self.layout {
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
            }
        });
    }
}

impl Component for Container {
    fn initialize(&mut self, state: &mut State) {
        for component in &mut self.components {
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
        for component in &self.components {
            let style = component.style(state);
            let mut cx = x;
            let mut cy = y;

            let mut width =
                (style.width().unwrap_or_else(|| component.width()) + style.spacing_horizontal()).min(width);
            let mut height =
                (style.height().unwrap_or_else(|| component.height()) + style.spacing_vertical()).min(height);

            surface.draw_border(&mut cx, &mut cy, &mut width, &mut height, &style);
            component.draw(state, surface, cx, cy, width, height, screen_width, screen_height);

            match self.layout {
                Layout::Vertical => {
                    y += style.height().unwrap_or_else(|| component.height()) + style.spacing_vertical()
                },
                Layout::Horizontal => {
                    x += style.width().unwrap_or_else(|| component.width()) + style.spacing_horizontal()
                },
            }
        }
    }

    fn on_input_action(&mut self, state: &mut State, input_action: InputAction) -> bool {
        let mut no_consume = true;
        if let Some(active) = self.active {
            no_consume &= self.components[active].on_input_action(state, input_action);
        }

        self.resize(state);

        no_consume
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

        for i in 0..self.components.len() {
            let style = self.components[i].style(state);
            let mut cx = x;
            let mut cy = y;

            let mut width =
                (style.width().unwrap_or_else(|| self.components[i].width()) + style.spacing_horizontal()).min(width);
            let mut height =
                (style.height().unwrap_or_else(|| self.components[i].height()) + style.spacing_vertical()).min(height);

            cx += style.margin_left();
            cy += style.margin_top();
            width -= style.margin_horizontal();
            height -= style.margin_vertical();

            if mouse_event.x as f64 >= cx
                && mouse_event.x as f64 <= cx + width
                && mouse_event.y as f64 >= cy
                && mouse_event.y as f64 <= cy + height
            {
                if let Some(active) = self.active {
                    self.components[active].on_focus(state, false);
                }

                self.active = Some(i);
                self.components[i].on_mouse_event(state, mouse_event, x, y, width, height);
            }

            match self.layout {
                Layout::Vertical => {
                    y += style.height().unwrap_or_else(|| self.components[i].height()) + style.spacing_vertical()
                },
                Layout::Horizontal => {
                    x += style.width().unwrap_or_else(|| self.components[i].width()) + style.spacing_horizontal()
                },
            }
        }
    }

    fn next(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        let active_old = self.active;

        let next = match self.active {
            Some(active) => {
                let component = &mut self.components[active];
                match component.next(state, false) {
                    Some(active) => Some(active),
                    None => {
                        self.active = self
                            .components
                            .iter()
                            .enumerate()
                            .skip(active + 1)
                            .find(|(_, c)| c.interactive(state))
                            .map(|(i, _)| i);

                        match self.active {
                            Some(active) => {
                                let component = &mut self.components[active];
                                match component.next(state, false) {
                                    Some(active) => Some(active),
                                    None => Some(component.id()),
                                }
                            },
                            None => match wrap {
                                true => return self.next(state, false),
                                false => None,
                            },
                        }
                    },
                }
            },
            None => {
                self.active = self
                    .components
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c.interactive(state))
                    .map(|(i, _)| i);

                match self.active {
                    Some(active) => {
                        let component = &mut self.components[active];
                        match component.next(state, wrap) {
                            Some(active) => Some(active),
                            None => Some(component.id()),
                        }
                    },
                    None => None,
                }
            },
        };

        if active_old != self.active {
            if let Some(active) = active_old {
                self.components[active].on_focus(state, false);
            }

            if let Some(active) = self.active {
                self.components[active].on_focus(state, true);
            }
        }

        self.resize(state);
        next
    }

    fn prev(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        let active_old = self.active;

        let prev = match self.active {
            Some(active) => {
                let component = &mut self.components[active];
                match component.prev(state, false) {
                    Some(active) => Some(active),
                    None => {
                        self.active = self.components[0..active]
                            .iter()
                            .enumerate()
                            .rev()
                            .find(|(_, c)| c.interactive(state))
                            .map(|(i, _)| i);

                        match self.active {
                            Some(active) => {
                                let component = &mut self.components[active];
                                match component.prev(state, false) {
                                    Some(active) => Some(active),
                                    None => Some(component.id()),
                                }
                            },
                            None => match wrap {
                                true => return self.prev(state, false),
                                false => None,
                            },
                        }
                    },
                }
            },
            None => {
                self.active = self
                    .components
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, c)| c.interactive(state))
                    .map(|(i, _)| i);

                match self.active {
                    Some(active) => {
                        let component = &mut self.components[active];
                        match component.prev(state, wrap) {
                            Some(active) => Some(active),
                            None => Some(component.id()),
                        }
                    },
                    None => None,
                }
            },
        };

        if active_old != self.active {
            if let Some(active) = active_old {
                self.components[active].on_focus(state, false);
            }

            if let Some(active) = self.active {
                self.components[active].on_focus(state, true);
            }
        }

        self.resize(state);
        prev
    }

    fn remove(&mut self, id: &str) -> Option<Box<dyn Component>> {
        for i in 0..self.components.len() {
            if self.components[i].id() == id {
                self.active = None;
                return Some(self.components.remove(i));
            }
        }

        None
    }

    fn insert(&mut self, id: &str, mut component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        for (i, child) in self.components.iter_mut().enumerate() {
            if child.id() == id {
                self.active = None;
                self.components.insert(i + 1, component);
                return None;
            }

            component = child.insert(id, component)?;
        }

        Some(component)
    }

    fn replace(&mut self, id: &str, mut component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        for (i, child) in self.components.iter_mut().enumerate() {
            if child.id() == id {
                if let Some(active) = self.active {
                    if active == i {
                        self.active = None;
                    }
                }

                let removed = self.components.remove(i);
                self.components.insert(i, component);
                return Some(removed);
            }

            component = child.insert(id, component)?;
        }

        Some(component)
    }

    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner.focus = focus;

        if focus && self.active.is_none() {
            self.active = self
                .components
                .iter()
                .enumerate()
                .find(|(_, c)| c.interactive(state))
                .map(|(i, _)| i);
        }

        if let Some(active) = self.active {
            self.components[active].on_focus(state, focus);
        }

        self.resize(state);
    }

    fn on_paste(&mut self, state: &mut State, clipboard: &str) {
        if let Some(active) = self.active {
            self.components[active].on_paste(state, clipboard)
        }
    }

    fn interactive(&self, state: &mut State) -> bool {
        self.components.iter().any(|c| c.interactive(state))
    }

    fn class(&self) -> &'static str {
        "div"
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }
}
