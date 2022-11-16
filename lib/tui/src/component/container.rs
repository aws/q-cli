use termwiz::surface::Surface;

use super::{
    Component,
    ComponentData,
};
use crate::event_loop::State;
use crate::input::InputAction;
use crate::surface_ext::SurfaceExt;

pub struct Container {
    components: Vec<Box<dyn Component + 'static>>,
    active: Option<usize>,
    inner: ComponentData,
}

impl Container {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            components: vec![],
            active: None,
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

            (
                acc.0
                    .max(style.width().unwrap_or_else(|| c.width()) + style.spacing_horizontal()),
                acc.1 + style.height().unwrap_or_else(|| c.height()) + style.spacing_vertical(),
            )
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
        x: f64,
        mut y: f64,
        width: f64,
        _: f64,
        screen_width: f64,
        screen_height: f64,
    ) {
        for component in &self.components {
            let style = component.style(state);
            let mut x = x;
            let mut cy = y;
            let mut width =
                (style.width().unwrap_or_else(|| component.width()) + style.spacing_horizontal()).min(width);
            let mut height = style.height().unwrap_or_else(|| component.height()) + style.spacing_vertical();
            surface.draw_border(&style, &mut x, &mut cy, &mut width, &mut height);
            component.draw(state, surface, x, cy, width, height, screen_width, screen_height);

            y += style.height().unwrap_or_else(|| component.height()) + style.spacing_vertical();
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

    fn next(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        if let Some(active) = self.active {
            match self.components[active].next(state, false) {
                Some(id) => return Some(id),
                None => {
                    self.components[active].on_focus(state, false);
                    self.active = self
                        .components
                        .iter()
                        .enumerate()
                        .skip(active + 1)
                        .find(|(_, c)| c.interactive(state))
                        .map(|(i, _)| i);
                    if let Some(active) = self.active {
                        let active_id = {
                            let active = &mut self.components[active];
                            active.on_focus(state, true);
                            active.id()
                        };
                        self.resize(state);
                        return Some(active_id);
                    }
                },
            }
        }

        if self.interactive(state) && wrap {
            self.active = self
                .components
                .iter()
                .enumerate()
                .find(|(_, c)| c.interactive(state))
                .map(|(i, _)| i);

            let active_id = {
                let active = &mut self.components[self.active.unwrap()];
                active.on_focus(state, true);
                active.id()
            };
            self.resize(state);
            return Some(active_id);
        }

        self.resize(state);
        None
    }

    fn prev(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        if let Some(active) = self.active {
            match self.components[active].prev(state, false) {
                Some(id) => return Some(id),
                None => {
                    self.components[active].on_focus(state, false);
                    self.active = self.components[0..active]
                        .iter()
                        .enumerate()
                        .rev()
                        .find(|(_, c)| c.interactive(state))
                        .map(|(i, _)| i);
                    if let Some(active) = self.active {
                        let active_id = {
                            let active = &mut self.components[active];
                            active.on_focus(state, true);
                            active.id()
                        };
                        self.resize(state);
                        return Some(active_id);
                    }
                },
            }
        }

        if self.interactive(state) && wrap {
            self.active = self
                .components
                .iter()
                .enumerate()
                .rev()
                .find(|(_, c)| c.interactive(state))
                .map(|(i, _)| i);

            let active_id = {
                let active = &mut self.components[self.active.unwrap()];
                active.on_focus(state, true);
                active.id()
            };
            self.resize(state);
            return Some(active_id);
        }

        self.resize(state);
        None
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

    fn on_resize(&mut self, state: &mut State, width: f64, height: f64) {
        for component in &mut self.components {
            component.on_resize(state, width, height);
        }
    }

    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner.focus = focus;

        if focus {
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
