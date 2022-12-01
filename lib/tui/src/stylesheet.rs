use std::collections::HashMap;

use crate::Style;

#[macro_export]
macro_rules! style_sheet {
    ($( $( $class:expr ),* => $val:tt ),*) => {{
        $crate::StyleSheet::new() $( $( .with_style($class, $crate::style_sheet!( @internal $val )) )* )*
    }};
    ($( $( $class:expr ),* => $val:tt, )*) => {{
        $crate::StyleSheet::new() $( $( .with_style($class, $crate::style_sheet!( @internal $val )) )* )*
    }};
    ( @internal { ..$parent:expr; $( $prop:ident: $val:expr; )* } ) => {{
        $crate::style! {
            $( ..$parent; $prop: $val; )*
        }
    }};
    ( @internal { $( $prop:ident: $val:expr; )* } ) => {{
        $crate::style! {
            $( $prop: $val; )*
        }
    }};
    ( @internal $val:expr ) => {
        $val
    }
}

#[derive(Debug, Default)]
pub struct StyleSheet(HashMap<String, Style>);

impl StyleSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_style(mut self, selector: impl Into<String>, style: Style) -> Self {
        let selector = selector.into();
        match self.0.get_mut(&selector) {
            Some(base) => {
                base.apply(&style);
            },
            None => {
                self.0.insert(selector, style);
            },
        }

        self
    }

    // *
    // *:focus
    // element
    // element:focus
    // inline-style
    // element.class
    // element.class:focus
    // element#id
    // element#id:focus
    pub(crate) fn get_style(
        &self,
        selector: impl AsRef<str>,
        id: impl AsRef<str>,
        hover: bool,
        focus: bool,
        active: bool,
    ) -> Style {
        let mut style = Style::default();

        if let Some(all) = self.0.get("*") {
            style.apply(all);
        }
        if hover {
            if let Some(all_hover) = self.0.get("*:hover") {
                style.apply(all_hover);
            }
        }
        if focus {
            if let Some(all_focus) = self.0.get("*:focus") {
                style.apply(all_focus);
            }
        }
        if active {
            if let Some(all_active) = self.0.get("*:active") {
                style.apply(all_active);
            }
        }

        if let Some(all) = self.0.get(&selector.as_ref().to_string()) {
            style.apply(all);
        }
        if hover {
            if let Some(all_hover) = self.0.get(&format!("{}:hover", selector.as_ref())) {
                style.apply(all_hover);
            }
        }
        if focus {
            if let Some(all_focus) = self.0.get(&format!("{}:focus", selector.as_ref())) {
                style.apply(all_focus);
            }
        }
        if active {
            if let Some(all_active) = self.0.get(&format!("{}:active", selector.as_ref())) {
                style.apply(all_active);
            }
        }

        if let Some(all) = self.0.get(&format!("#{}", id.as_ref())) {
            style.apply(all);
        }
        if hover {
            if let Some(all_hover) = self.0.get(&format!("#{}:hover", id.as_ref())) {
                style.apply(all_hover);
            }
        }
        if focus {
            if let Some(all_focus) = self.0.get(&format!("#{}:focus", id.as_ref())) {
                style.apply(all_focus);
            }
        }
        if active {
            if let Some(all_active) = self.0.get(&format!("#{}:active", id.as_ref())) {
                style.apply(all_active);
            }
        }

        style
    }
}
