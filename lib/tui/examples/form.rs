use tui::component::{
    CheckBox,
    Container,
    Select,
};
use tui::{
    BorderStyle,
    ColorAttribute,
    EventLoop,
    InputMethod,
};

fn main() {
    let style_sheet = tui::style_sheet! {
        "*" => {
            border_left_color: ColorAttribute::PaletteIndex(8);
            border_right_color: ColorAttribute::PaletteIndex(8);
            border_top_color: ColorAttribute::PaletteIndex(8);
            border_bottom_color: ColorAttribute::PaletteIndex(8);
            border_style: BorderStyle::Ascii {
                top_left: '┌',
                top: '─',
                top_right: '┐',
                left: '│',
                right: '│',
                bottom_left: '└',
                bottom: '─',
                bottom_right: '┘',
            };
        },
        "*:focus" => {
            color: ColorAttribute::PaletteIndex(3);
            border_left_color: ColorAttribute::PaletteIndex(3);
            border_right_color: ColorAttribute::PaletteIndex(3);
            border_top_color: ColorAttribute::PaletteIndex(3);
            border_bottom_color: ColorAttribute::PaletteIndex(3);
            border_style: BorderStyle::Ascii {
                top_left: '┏',
                top: '━',
                top_right: '┓',
                left: '┃',
                right: '┃',
                bottom_left: '┗',
                bottom: '━',
                bottom_right: '┛',
            };
        },
        "input:checkbox" => {
            padding_left: 1.0;
            padding_right: 1.0;
        },
        "div" => {
            color: ColorAttribute::PaletteIndex(8);
            width: Some(100.0);
            border_left_width: 1.0;
            border_top_width: 1.0;
            border_bottom_width: 1.0;
            border_right_width: 1.0;
        },
        "h1" => {
            margin_left: 1.0;
            padding_left: 1.0;
            padding_right: 1.0;
        },
        "p" => {
            padding_left: 1.0;
            padding_right: 1.0;
        },
        "select" => {
            padding_left: 1.0;
            padding_right: 1.0;
        },
        "input:text" => {
            width: Some(98.0);
            padding_left: 1.0;
            padding_right: 2.0;
        }
    };

    #[rustfmt::skip]
    let mut view = 
        Container::new("container3")
            .push(Container::new("container1").push(
                CheckBox::new("check_box", "Are you cool?", false))
            )
            .push(Container::new("container2").push(
                Select::new("select", vec!["hello".to_owned(), "world".to_owned()], false))
            );

    EventLoop::new()
        .run(
            &mut view,
            InputMethod::Form,
            style_sheet,
            |_event, _component, _control_flow| {
                // do nothing I guess?
            },
        )
        .unwrap();
}
