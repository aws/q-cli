mod check_box;
mod div;
mod file_picker;
mod p;
mod select;
mod text_field;
pub(self) mod text_state;

pub use check_box::{
    CheckBox,
    CheckBoxEvent,
};
pub use div::{
    Div,
    Layout,
};
pub use file_picker::{
    FilePicker,
    FilePickerEvent,
};
pub use p::P;
pub use select::{
    Select,
    SelectEvent,
};
use termwiz::surface::Surface;
pub use text_field::{
    TextField,
    TextFieldEvent,
};

use crate::event_loop::State;
use crate::input::{
    InputAction,
    MouseAction,
};
use crate::style_sheet_ext::StyleSheetExt;
use crate::Style;

type Child = Box<dyn Component + 'static>;

#[derive(Debug, Default)]
pub struct ComponentData {
    ///
    pub type_selector: String,
    ///
    pub id: Option<String>,
    ///
    pub classes: Vec<String>,
    ///
    pub interactive: bool,
    ///
    pub hover: bool,
    ///
    pub focus: bool,
    ///
    pub active: bool,
    //
    pub focused_child_index: Option<usize>,
    //
    pub children: Vec<Child>,
}

#[derive(Clone, Debug, Default)]
pub struct StyleInfo {
    ///
    pub type_selector: String,
    ///
    pub id: Option<String>,
    ///
    pub classes: Vec<String>,
    ///
    pub hover: bool,
    ///
    pub focus: bool,
    ///
    pub active: bool,
}

impl ComponentData {
    pub fn new(type_selector: String, interactive: bool) -> Self {
        Self {
            type_selector,
            id: None,
            interactive,
            ..Default::default()
        }
    }

    pub fn style_info(&self) -> StyleInfo {
        StyleInfo {
            type_selector: self.type_selector.clone(),
            classes: self.classes.clone(),
            id: self.id.clone(),
            hover: self.hover,
            focus: self.focus,
            active: self.active,
        }
    }

    pub fn interactive(&self) -> bool {
        self.children.iter().any(|c| c.interactive())
    }

    pub fn focused_leaf_id(&self) -> &Option<String> {
        if let Some(child) = self.focused_child_index.and_then(|idx| self.children.get(idx)) {
            child.inner().focused_leaf_id()
        } else {
            &self.id
        }
    }

    pub fn focused_child(&mut self) -> Option<&mut Child> {
        self.children.get_mut(self.focused_child_index?)
    }

    pub fn focus_child_at_index(&mut self, state: &mut State, index: Option<usize>) {
        let old_focus_index = self.focused_child_index;
        let new_focus_index = index.and_then(|i| if i >= self.children.len() { None } else { Some(i) });

        if old_focus_index != new_focus_index {
            if let Some(old_index) = old_focus_index {
                self.children[old_index].on_focus(state, false);
            }

            if let Some(new_index) = new_focus_index {
                self.children[new_index].on_focus(state, true);
            }
        }

        self.focused_child_index = new_focus_index;
    }

    pub fn find_next_child<P>(&mut self, mut predicate: P, start_index: Option<usize>, wrap: bool) -> Option<usize>
    where
        P: FnMut(&Child) -> bool,
    {
        let child = self
            .children
            .iter()
            .enumerate()
            .skip(start_index.unwrap_or(0))
            .find(|(_, c)| predicate(c))
            .map(|(i, _)| i);

        if child.is_some() || !wrap {
            return child;
        }

        self.children
            .iter()
            .enumerate()
            .take(start_index.unwrap_or(0))
            .find(|(_, c)| predicate(c))
            .map(|(i, _)| i)
    }

    pub fn find_prev_child<P>(&mut self, mut predicate: P, start_index: Option<usize>, wrap: bool) -> Option<usize>
    where
        P: FnMut(&Child) -> bool,
    {
        let child = self
            .children
            .iter()
            .enumerate()
            .take(start_index.unwrap_or(self.children.len()))
            .rfind(|(_, c)| predicate(c))
            .map(|(i, _)| i);

        if child.is_some() || !wrap {
            return child;
        }

        self.children
            .iter()
            .enumerate()
            .skip(start_index.unwrap_or(self.children.len()))
            .rfind(|(_, c)| predicate(c))
            .map(|(i, _)| i)
    }
}

pub trait Component: std::fmt::Debug {
    /// Initialize the component with information not available at creation
    ///
    /// You will likely want to initialize the size of your inner content here, if not possible to
    /// do otherwise.
    // fn initialize(&mut self, state: &mut State);

    /// Draw the component
    ///
    /// This function assumes that borders, margin, and padding are handled by container
    /// components. This makes the implementation of simple interactive components easier, but
    /// container types more difficult.
    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64);

    /// How the component handles input actions such as next or prev
    #[allow(unused_variables)]
    fn on_input_action(&mut self, state: &mut State, input_action: &InputAction) {}

    /// How the component handles mouse related events including scroll, movement, and click
    #[allow(unused_variables)]
    fn on_mouse_action(
        &mut self,
        state: &mut State,
        mouse_action: &MouseAction,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) {
    }

    /// Navigate focus to the next interactive element in the tree
    #[allow(unused_variables)]
    fn next(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        None
    }

    /// Navigate focus to the previous interactive element in the tree
    #[allow(unused_variables)]
    fn prev(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        None
    }

    /// Removes an element with the given id from the tree and returns it if found
    #[allow(unused_variables)]
    fn remove(&mut self, id: &str) -> Option<Box<dyn Component>> {
        None
    }

    /// Insert a component after the given ui element id
    ///
    /// This function returns None on success, but returns ownership of the component in the event
    /// that an element of id does not exist
    #[allow(unused_variables)]
    fn insert(&mut self, id: &str, component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        Some(component)
    }

    /// Replace an existing component in the current ui element or its children with the given ui
    /// element
    ///
    /// This function returns the element it replaced on success, but otherwise returns ownership of
    /// the original element on failure
    #[allow(unused_variables)]
    fn replace(&mut self, id: &str, component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        Some(component)
    }

    /// The logic ran when the user focuses a ui element, or more specifically, when the ui element
    /// receives input
    #[allow(unused_variables)]
    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner_mut().focus = focus;
    }

    /// Whether or not the component is capable of receiving input
    #[allow(unused_variables)]
    fn interactive(&self) -> bool {
        self.inner().interactive
    }

    /// The style of the ui element
    fn style(&self, state: &State) -> Style {
        state.style_sheet.get_style(state)
    }

    /// The width of the inner content regardless of border and padding
    fn size(&self, state: &mut State) -> (f64, f64);

    /// The id of the ui element used for event handling and styling
    fn id(&self) -> &Option<String> {
        &self.inner().id
    }

    /// A string identifying the type selector of the ui element e.g. 'div', 'p', 'body'
    fn type_selector(&self) -> &str {
        &self.inner().type_selector
    }

    /// Retrieve a reference to the inner component data of the ui element
    fn inner(&self) -> &ComponentData;

    /// Retrieve a mutable reference to the inner component data of the ui element
    fn inner_mut(&mut self) -> &mut ComponentData;
}
