use tui::components::{
    Disclosure,
    Frame,
    Label,
    TextField,
};
use tui::layouts::Container;
use tui::{
    BorderStyle,
    Color,
    ControlFlow,
    DisplayMode,
    EventLoop,
    Style,
    StyleSheet,
};

fn main() -> Result<(), std::io::Error> {
    // let rounded_border = BorderStyle::Ascii {
    //     top_left: '╭',
    //     top: '─',
    //     top_right: '╮',
    //     left: '│',
    //     right: '│',
    //     bottom_left: '╰',
    //     bottom: '─',
    //     bottom_right: '╯',
    // };

    let thin_border = BorderStyle::Ascii {
        top_left: '┌',
        top: '─',
        top_right: '┐',
        left: '│',
        right: '│',
        bottom_left: '└',
        bottom: '─',
        bottom_right: '┘',
    };

    let stylesheet = StyleSheet::new()
        .with_style(
            "*",
            Style::new(), // .with_color(Color::White)
        )
        .with_style(
            "disclosure",
            Style::new()
            // .with_border_style(rounded_border)
            // .with_border_color(Color::Magenta)
            // .with_border_top_width(1)
            // .with_border_bottom_width(1)
            // .with_border_left_width(1)
            // .with_border_right_width(1)
            .with_margin_top(1)
            .with_padding_left(1)
            .with_padding_right(1),
        );
    // .with_style(
    //     "select",
    //     Style::new()
    // )
    // .with_style("textfield", Style::default())
    // .with_style(".my_border", Style::default());

    let mut label1 = Label::new("These are some details that would normally be hidden...").with_color(Color::DarkGrey);
    let mut label2 = Label::new("These are some details that would normally be hidden...").with_color(Color::DarkGrey);

    let todo1 = &mut Label::new(format!("{} All done!", "✔"))
        .with_color(Color::Green)
        .with_margin_bottom(1);
    let todo2 = &mut Label::new(format!("{} Cancelled", "✘"))
        .with_color(Color::Red)
        .with_margin_bottom(1);
    let todo3 = &mut Label::new(format!("{} In progress", "●"))
        .with_color(Color::Yellow)
        .with_margin_bottom(1);
    let todo4 = &mut Label::new(format!("{} Waiting...", "-"))
        .with_color(Color::DarkGrey)
        .with_margin_bottom(1);

    let mut container = Container::new(vec![todo1, todo2, todo3, todo4]);

    // .with_border_left_width(1)
    // .with_border_top_width(1)
    // .with_border_bottom_width(1)
    // .with_border_right_width(1)
    // .with_border_style(thin_border)
    // .with_border_left_color(Color::Grey)
    // .with_border_bottom_color(Color::Grey)
    // .with_border_top_color(Color::Grey)
    // .with_border_right_color(Color::Grey);
    // .with_background_color(Color::Cyan);
    let mut frame = Frame::new(&mut container)
        .with_title("Todo List")
        .with_title_style(
            Style::new().with_padding_left(2)
                                                        .with_padding_right(2)
                                                        .with_color(Color::Grey)
                                                        .with_border_top_color(Color::Grey)
                                                        .with_border_bottom_color(Color::Grey)
                                                        .with_border_left_color(Color::Grey)
                                                        .with_border_right_color(Color::Grey)
                                                        .with_border_top_width(1)
                                                        // .with_background_color(Color::Cyan)
                                                        .with_border_bottom_width(1)
                                                        .with_margin_left(0)
                                                        .with_border_right_width(1)
                                                        .with_border_left_width(1)
                                                        .with_border_style(BorderStyle::Ascii {
                                                            top_left: '╭',
                                                            top: '─',
                                                            top_right: '╮',
                                                            left: '┤',
                                                            right: '├',
                                                            bottom_left: '└',
                                                            bottom: '─',
                                                            bottom_right: '┘',
                                                        }),
        )
        .with_border_left_width(1)
        .with_border_top_width(1)
        .with_border_bottom_width(1)
        .with_border_right_width(1)
        .with_border_style(thin_border)
        .with_border_left_color(Color::Grey)
        .with_border_bottom_color(Color::Grey)
        .with_border_top_color(Color::Grey)
        .with_border_right_color(Color::Grey)
        .with_padding_left(2)
        .with_margin_bottom(3);

    let mut name = TextField::new()
        .with_hint("user@email.com")
        .with_border_left_width(1)
        .with_border_top_width(1)
        .with_border_bottom_width(1)
        .with_border_right_width(1)
        .with_border_style(thin_border)
        .with_border_left_color(Color::Grey)
        .with_border_bottom_color(Color::Grey)
        .with_border_top_color(Color::Grey)
        .with_border_right_color(Color::Grey)
        .with_padding_left(1)
        .with_margin_top(1);

    EventLoop::new()
        .with_style_sheet(&stylesheet)
        .run::<std::io::Error, _>(
            ControlFlow::Wait,
            DisplayMode::AlternateScreen,
            &mut Disclosure::new(
                "This is the summary!",
                Container::new(vec![&mut label1, &mut label2, &mut frame, &mut name]),
            ),
        )?;

    Ok(())
}
