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
        }
    };

    let mut container1 = Container::new("container1", Layout::Vertical);
    container1.push(CheckBox::new("check_box", "Are you cool?", false));

    let mut container2 = Container::new("container2", Layout::Vertical);
    container2.push(Select::new(
        "select",
        vec!["hello".to_owned(), "world".to_owned()],
        false,
    ));

    EventLoop::new()
        .run(
            Container::new("container3", Layout::Horizontal)
                .push(container1)
                .push(container2),
            InputMethod::Form,
            style_sheet,
            |event, _component, control_flow| match event {
                tui::Event::Quit | tui::Event::Terminate => *control_flow = ControlFlow::Quit,
                _ => (),
            },
        )
        .unwrap();
}
