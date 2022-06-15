use std::collections::HashMap;
use std::fmt;

use crate::{
    Component,
    Style,
    StyleContext,
};

#[macro_export]
macro_rules! style_sheet {
    ($( $class:expr => $val:tt ),*) => {{
        $crate::StyleSheet::new() $( .with_style($class, $crate::style_sheet!( @internal $val )) )*
    }};
    ( @internal { $( $prop:ident: $val:expr; )* } ) => {{
        $crate::paste::paste! {
            $crate::Style::new() $( .[<with_ $prop>]($val) )*
        }
    }};
    ( @internal { ..$parent:expr; $( $prop:ident: $val:expr; )* } ) => {{
        $crate::paste::paste! {
            $parent $( .[<with_ $prop>]($val) )*
        }
    }};
    ( @internal $val:expr ) => {
        $val
    }
}

#[derive(Debug, Default)]
pub struct StyleSheet(HashMap<String, Style>);

#[derive(Debug, Clone, Copy)]
pub enum PseudoClass {
    Focus,
    Hover,
}

impl fmt::Display for PseudoClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PseudoClass::Focus => write!(f, ":focus"),
            PseudoClass::Hover => write!(f, ":hover"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PseudoElement {
    Placeholder,
    Selection,
}

impl fmt::Display for PseudoElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PseudoElement::Placeholder => write!(f, "::placeholder"),
            PseudoElement::Selection => write!(f, "::selection"),
        }
    }
}

impl StyleSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_style(&self, selector: impl AsRef<str>) -> Style {
        match self.0.get(selector.as_ref()) {
            Some(style) => *style,
            None => Style::default(),
        }
    }

    pub fn get_computed_style(&self, selector: impl AsRef<str>, context: StyleContext) -> Style {
        let pseudo_class = if context.focused {
            Some(PseudoClass::Focus)
        } else {
            None
        };
        let computed_selector = Style::selector_for(selector.as_ref(), None, pseudo_class, None);

        match self.0.get(&computed_selector) {
            Some(style) => *style,
            None => Style::default(),
        }
    }

    pub fn with_style(mut self, selector: impl Into<String>, style: Style) -> Self {
        self.0.insert(selector.into(), style);
        self
    }

    // *
    // *:focus
    // element
    // element:focus
    // inline-style
    // element.class
    // element.class:focus
    #[allow(dead_code)]
    fn inherit_style_from(&self, mut elements: Vec<&str>, inline_style: Option<Style>, context: StyleContext) -> Style {
        match elements.pop() {
            Some(elm) => {
                let mut style = self.inherit_style_from(elements, None, context);

                style = style.apply(self.get_style(elm));

                match inline_style {
                    Some(inline_style) => style = style.apply(inline_style),
                    None => (),
                }

                if context.focused {
                    style = style.apply(self.get_style(Style::selector_for(elm, None, Some(PseudoClass::Focus), None)))
                }

                style
            },
            None => Default::default(),
        }
    }

    pub fn get_style_for_element(&self, element: &str, inline_style: Option<Style>, context: StyleContext) -> Style {
        let mut style = self.get_style("*");
        if context.focused {
            style = style.apply(self.get_style("*:focus"));
        }
        style = style.apply(self.get_style(element));
        if let Some(inline_style) = inline_style {
            style = style.apply(inline_style);
        }
        if context.focused {
            style = style.apply(self.get_style(format!("{}:focus", element)));
        }
        style
    }

    pub fn get_style_for_component(&self, component: &impl Component, context: StyleContext) -> Style {
        self.get_style_for_element(component.class(), component.inline_style(), context)
    }

    pub fn get_style_for_component_with_class(
        &self,
        component: &impl Component,
        class: &str,
        context: StyleContext,
    ) -> Style {
        let pseudo_class = if context.focused {
            Some(PseudoClass::Focus)
        } else {
            None
        };
        let class_style = self.get_style(Style::selector_for(class, None, pseudo_class, None));
        self.get_style_for_component(component, context).apply(class_style)
    }
}
