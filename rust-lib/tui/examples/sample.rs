use tui::components::{
    Frame,
    Label,
};
use tui::layouts::Form;
use tui::{
    BorderStyle,
    ControlFlow,
    DisplayMode,
    EventLoop,
    Style,
    StyleSheet,
};

fn main() -> Result<(), std::io::Error> {
    let rounded_border = BorderStyle::Ascii {
        top_left: '╭',
        top: '─',
        top_right: '╮',
        left: '│',
        right: '│',
        bottom_left: '╰',
        bottom: '─',
        bottom_right: '╯',
    };

    let style_sheet = StyleSheet::new().with_style(
        "frame",
        Style::new()
            .with_border_style(rounded_border)
            .with_border_bottom_width(1)
            .with_border_top_width(1),
    );
    let context = tui::StyleContext {
        focused: false,
        hover: false,
    };

    let mut label = Label::new("Hello world");
    let mut frame = Frame::new(&mut label);
    println!("{:#?}", style_sheet.get_style_for_component(&frame, context));
    let mut form = Form::new([&mut frame]);

    // println!("{:#?}",s);
    // println!("{:#?} {:#?}",frame.desired_height(&stylesheet, context),
    // frame.desired_width(&stylesheet, context)); println!("{:#?}",frame.style(&stylesheet,
    // context).border_style());

    EventLoop::new()
        .with_style_sheet(&style_sheet)
        .run(ControlFlow::Wait, DisplayMode::AlternateScreen, &mut form)
}
