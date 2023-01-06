use lightningcss::properties::border::BorderSideWidth;
use lightningcss::properties::custom::{
    Function,
    Token,
    TokenOrValue,
    UnparsedProperty,
};
use lightningcss::properties::display::{
    Display,
    DisplayKeyword,
};
use lightningcss::properties::size::Size;
use lightningcss::properties::ui::ColorOrAuto;
use lightningcss::properties::{
    Property,
    PropertyId,
};
use lightningcss::rules::CssRule;
use lightningcss::selector::{
    Combinator,
    Component,
    PseudoClass,
    Selector,
};
use lightningcss::stylesheet::StyleSheet;
use lightningcss::values::color::CssColor;
use lightningcss::values::length::{
    Length,
    LengthPercentage,
    LengthPercentageOrAuto,
    LengthValue,
};
use lightningcss::values::percentage::DimensionPercentage;
use termwiz::color::{
    ColorAttribute,
    SrgbaTuple,
};

use crate::event_loop::TreeElement;
use crate::{
    BorderStyle,
    State,
    Style,
};

pub trait StyleSheetExt {
    fn get_style(&self, state: &State) -> Style;
}

impl StyleSheetExt for StyleSheet<'_, '_> {
    fn get_style(&self, state: &State) -> Style {
        let mut out = Style::default();
        for rule in &self.rules.0 {
            if let CssRule::Style(style) = rule {
                // iterate through comma separated selector list
                for selector in &style.selectors.0 {
                    if selector_matches(selector, state) {
                        for property in &style.declarations.declarations {
                            match property {
                                Property::Display(display) => {
                                    if let Display::Keyword(keyword) = display {
                                        match keyword {
                                            DisplayKeyword::None => out.with_display(crate::Display::None),
                                            _ => out.with_display(crate::Display::Block),
                                        };
                                    } else {
                                        out.with_display(crate::Display::Block);
                                    }
                                },
                                Property::BackgroundColor(color) => {
                                    if let Some(color) = color_convert(color) {
                                        out.with_background_color(color);
                                    }
                                },
                                Property::Color(color) => {
                                    if let Some(color) = color_convert(color) {
                                        out.with_color(color);
                                    }
                                },
                                Property::Width(width) => {
                                    if let Size::LengthPercentage(DimensionPercentage::Dimension(LengthValue::Px(
                                        cols,
                                    ))) = width
                                    {
                                        out.with_width(Some((*cols).into()));
                                    }
                                },
                                Property::Height(height) => {
                                    if let Size::LengthPercentage(DimensionPercentage::Dimension(LengthValue::Px(
                                        rows,
                                    ))) = height
                                    {
                                        out.with_height(Some((*rows).into()));
                                    }
                                },
                                // Property::MinWidth(_) => todo!(),
                                // Property::MinHeight(_) => todo!(),
                                // Property::MaxWidth(_) => todo!(),
                                // Property::MaxHeight(_) => todo!(),
                                // Property::BorderSpacing(_) => todo!(),
                                // Property::BorderTopColor(_) => todo!(),
                                // Property::BorderBottomColor(_) => todo!(),
                                // Property::BorderLeftColor(_) => todo!(),
                                // Property::BorderRightColor(_) => todo!(),
                                // Property::BorderTopStyle(_) => todo!(),
                                // Property::BorderBottomStyle(_) => todo!(),
                                // Property::BorderLeftStyle(_) => todo!(),
                                // Property::BorderRightStyle(_) => todo!(),
                                // Property::BorderTopWidth(_) => todo!(),
                                // Property::BorderBottomWidth(_) => todo!(),
                                // Property::BorderLeftWidth(_) => todo!(),
                                // Property::BorderRightWidth(_) => todo!(),
                                Property::BorderColor(color) => {
                                    if let Some(color) = color_convert(&color.top) {
                                        out.with_border_color(color);
                                    }
                                },
                                // Property::BorderStyle(_) => todo!(),
                                Property::BorderWidth(width) => {
                                    if let BorderSideWidth::Length(Length::Value(LengthValue::Px(width))) = width.top {
                                        out.with_border_width(width.into());
                                    }
                                },
                                Property::MarginTop(margin) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(margin),
                                    )) = margin
                                    {
                                        out.with_margin_top((*margin).into());
                                    }
                                },
                                Property::MarginBottom(margin) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(margin),
                                    )) = margin
                                    {
                                        out.with_margin_bottom((*margin).into());
                                    }
                                },
                                Property::MarginLeft(margin) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(margin),
                                    )) = margin
                                    {
                                        out.with_margin_left((*margin).into());
                                    }
                                },
                                Property::MarginRight(margin) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(margin),
                                    )) = margin
                                    {
                                        out.with_margin_right((*margin).into());
                                    }
                                },
                                Property::Margin(margin) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(margin),
                                    )) = margin.top
                                    {
                                        out.with_margin(margin.into());
                                    }
                                },
                                Property::PaddingTop(padding) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(padding),
                                    )) = padding
                                    {
                                        out.with_padding_top((*padding).into());
                                    }
                                },
                                Property::PaddingBottom(padding) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(padding),
                                    )) = padding
                                    {
                                        out.with_padding_bottom((*padding).into());
                                    }
                                },
                                Property::PaddingLeft(padding) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(padding),
                                    )) = padding
                                    {
                                        out.with_padding_left((*padding).into());
                                    }
                                },
                                Property::PaddingRight(padding) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(padding),
                                    )) = padding
                                    {
                                        out.with_padding_right((*padding).into());
                                    }
                                },
                                Property::Padding(padding) => {
                                    if let LengthPercentageOrAuto::LengthPercentage(LengthPercentage::Dimension(
                                        LengthValue::Px(padding),
                                    )) = padding.top
                                    {
                                        out.with_padding(padding.into());
                                    }
                                },
                                Property::CaretColor(color) => {
                                    if let ColorOrAuto::Color(color) = color {
                                        if let Some(color) = color_convert(color) {
                                            out.with_caret_color(color);
                                        }
                                    }
                                },
                                // Property::CaretShape(_) => todo!(),
                                // custom properties have an unknown name
                                // Property::Custom(CustomProperty { name, value }) => {},
                                // unparsed properties have a known name, but an unknown value
                                Property::Unparsed(UnparsedProperty { property_id, value }) => {
                                    let color = match value.0.get(0) {
                                        Some(token) => {
                                            if let TokenOrValue::Function(Function { name, arguments }) = token {
                                                if name.as_ref() == "ansi" && arguments.0.len() == 1 {
                                                    if let TokenOrValue::Token(Token::Number {
                                                        has_sign: false,
                                                        int_value,
                                                        ..
                                                    }) = arguments.0[0]
                                                    {
                                                        u8::try_from(int_value.unwrap()).ok()
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        },
                                        None => None,
                                    };

                                    match property_id {
                                        PropertyId::BackgroundColor => {
                                            if let Some(color) = color {
                                                out.with_background_color(ColorAttribute::PaletteIndex(color));
                                            }
                                        },
                                        PropertyId::Color => {
                                            if let Some(color) = color {
                                                out.with_color(ColorAttribute::PaletteIndex(color));
                                            }
                                        },
                                        PropertyId::BorderStyle => {
                                            out.with_border_style(BorderStyle::Ascii {
                                                top_left: '┌',
                                                top: '─',
                                                top_right: '┐',
                                                left: '│',
                                                right: '│',
                                                bottom_left: '└',
                                                bottom: '─',
                                                bottom_right: '┘',
                                            });
                                        },
                                        PropertyId::BorderColor => {
                                            if let Some(color) = color {
                                                out.with_border_color(ColorAttribute::PaletteIndex(color));
                                            }
                                        },
                                        PropertyId::CaretColor => {
                                            if let Some(color) = color {
                                                out.with_caret_color(ColorAttribute::PaletteIndex(color));
                                            }
                                        },
                                        _ => tracing::warn!("unparsed css property: {property_id:?}: {value:?}"),
                                    }
                                },
                                property => tracing::warn!("unhandled css property: {property:?}"),
                            };
                        }
                    }
                }
            }
        }

        out
    }
}

fn selector_component_matches(component: &Component, element: &TreeElement) -> bool {
    match component {
        Component::Combinator(_) => unreachable!("combinator is stored here for improved alignment and padding"),
        Component::ExplicitUniversalType => true,
        Component::LocalName(local_name) => element.inner.type_selector == local_name.name.as_ref(),
        Component::ID(id) => match &element.inner.id {
            Some(inner_id) => inner_id == id.as_ref(),
            None => false,
        },
        Component::Class(_) => false,
        Component::NonTSPseudoClass(class) => match class {
            PseudoClass::Hover => element.inner.hover,
            PseudoClass::Focus => element.inner.focus,
            _ => {
                tracing::warn!("unhandled nts pseudo class");
                false
            },
        },
        // Component::Negation(negation) => (),
        component => {
            tracing::warn!("unhandled selector component: {component:?}");
            false
        },
    }
}

/// This function determines whether or not a single selector matches
fn selector_matches(selector: &Selector, state: &State) -> bool {
    let mut components = state.tree.iter().cloned().rev();
    let mut current_element = components.next();
    let mut selector_iter = selector.iter();

    let mut combinator = None;

    loop {
        match current_element {
            Some(element) => {
                let component_matches = selector_iter.clone().all(|c| selector_component_matches(c, &element));
                if component_matches {
                    // Must clear selector_iter before calling next_sequence.
                    selector_iter.all(|_| true);
                    combinator = selector_iter.next_sequence();
                }

                match (component_matches, combinator) {
                    (_, None) => {
                        return component_matches;
                    },
                    (true, Some(Combinator::Child)) | (_, Some(Combinator::Descendant)) => {
                        current_element = components.next();
                    },
                    (true, Some(Combinator::NextSibling)) | (_, Some(Combinator::LaterSibling)) => {
                        current_element = element.next_sibling();
                    },
                    (false, Some(Combinator::NextSibling)) | (false, Some(Combinator::Child)) => {
                        return false;
                    },
                    _ => {
                        tracing::warn!("Unhandled combinator");
                        return false;
                    },
                }
            },
            None => {
                return selector_iter.next().is_none() && selector_iter.next_sequence().is_none();
            },
        }
    }
}

fn color_convert(color: &CssColor) -> Option<ColorAttribute> {
    let color = match color {
        CssColor::CurrentColor => None,
        CssColor::RGBA(color) => Some(SrgbaTuple(
            color.red_f32(),
            color.green_f32(),
            color.blue_f32(),
            color.alpha_f32(),
        )),
        _ => {
            let color = color.to_rgb();
            if let CssColor::RGBA(color) = color {
                Some(SrgbaTuple(
                    color.red_f32(),
                    color.green_f32(),
                    color.blue_f32(),
                    color.alpha_f32(),
                ))
            } else {
                unreachable!();
            }
        },
    }?;

    Some(ColorAttribute::TrueColorWithDefaultFallback(color))
}
