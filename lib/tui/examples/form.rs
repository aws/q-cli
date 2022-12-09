use tui::component::{
    CheckBox,
    Container,
    Layout,
    Select,
};
use tui::{
    BorderStyle,
    ColorAttribute,
    ControlFlow,
    EventLoop,
    InputMethod,
};

fn main() {
    let style_sheet = tui::style_sheet! {
        "div" => {
            color: ColorAttribute::PaletteIndex(8);
            border_left_width: 1.0;
            border_top_width: 1.0;
            border_bottom_width: 1.0;
            border_right_width: 1.0;
            border_left_color: ColorAttribute::PaletteIndex(8);
            border_right_color: ColorAttribute::PaletteIndex(8);
            border_top_color: ColorAttribute::PaletteIndex(8);
            border_bottom_color: ColorAttribute::PaletteIndex(8);
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
        "div:focus" => {
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
    };

    EventLoop::new()
        .run(
            &mut Container::new("container3", Layout::Horizontal)
                .push(Container::new("container1", Layout::Vertical).push(CheckBox::new(
                    "check_box",
                    "Are you cool?",
                    false,
                )))
                .push(Container::new("container2", Layout::Vertical).push(Select::new(
                    "select",
                    vec!["hello".to_owned(), "world".to_owned()],
                    false,
                ))),
            InputMethod::Form,
            style_sheet,
            |event, _component, control_flow| match event {
                tui::Event::Quit | tui::Event::Terminate => *control_flow = ControlFlow::Quit,
                _ => (),
            },
        )
        .unwrap();
}
