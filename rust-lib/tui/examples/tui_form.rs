use tui::components::{
    Disclosure,
    Frame,
    Label,
    Select,
    TextField,
};
use tui::layouts::{
    Container,
    Form,
};
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

    let double_border = BorderStyle::Ascii {
        top_left: '╔',
        top: '═',
        top_right: '╗',
        left: '║',
        right: '║',
        bottom_left: '╚',
        bottom: '═',
        bottom_right: '╝',
    };

    let thick_border = BorderStyle::Ascii {
        top_left: '┏',
        top: '━',
        top_right: '┓',
        left: '┃',
        right: '┃',
        bottom_left: '┗',
        bottom: '━',
        bottom_right: '┛',
    };

    let focus_style = Style::new()
        .with_border_left_color(Color::White)
        .with_border_right_color(Color::White)
        .with_border_top_color(Color::White)
        .with_border_bottom_color(Color::White)
        .with_border_style(thick_border);
    let stylesheet = StyleSheet::new()
        .with_style(
            "*",
            Style::new()
                .with_border_left_color(Color::Grey)
                .with_border_right_color(Color::Grey)
                .with_border_top_color(Color::Grey)
                .with_border_bottom_color(Color::Grey),
        )
        .with_style("*:focus", focus_style)
        .with_style(
            "frame:focus",
            focus_style, // .with_background_color(Color::Cyan)
        )
        .with_style("textfield:focus", focus_style)
        .with_style("disclosure.summary:focus", Style::new().with_color(Color::White))
        .with_style("disclosure.summary", Style::new().with_color(Color::Grey));
    // .with_style("frame.title:focus", style);

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

    let mut container = Container::new([todo1, todo2, todo3, todo4]);

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

    // let mut details =
    // Disclosure::new("This is the summary!", &mut detail_view);
    // let mut name = TextField::new()
    //     .with_title(" Email ")
    //     .with_hint("user@email.com")
    //     .with_border_left_width(1)
    //     .with_border_top_width(1)
    //     .with_border_bottom_width(1)
    //     .with_border_right_width(1)
    //     .with_border_style(rounded_border)
    //     .with_border_left_color(Color::Grey)
    //     .with_border_bottom_color(Color::Grey)
    //     .with_border_top_color(Color::Grey)
    //     .with_border_right_color(Color::Grey)
    //     .with_padding_left(1);

    // let mut title = Label::new("Please enter a
    // username...").with_color(Color::White).with_margin_left(1);

    // let mut user = TextField::new()
    //     .with_title(" Username ")
    //     .with_hint("@username")
    //     .with_border_left_width(1)
    //     .with_border_top_width(1)
    //     .with_border_bottom_width(1)
    //     .with_border_right_width(1)
    //     .with_border_style(rounded_border)
    //     .with_border_left_color(Color::Grey)
    //     .with_border_bottom_color(Color::Grey)
    //     .with_border_top_color(Color::Grey)
    //     .with_border_right_color(Color::Grey)
    //     .with_padding_left(1);
    let mut commit = TextField::new()
        .with_hint("fix: syntax error")
        .with_border_left_width(1)
        .with_border_top_width(1)
        .with_border_bottom_width(1)
        .with_border_right_width(1)
        .with_border_style(rounded_border)
        .with_border_left_color(Color::DarkGrey)
        .with_border_bottom_color(Color::DarkGrey)
        .with_border_top_color(Color::DarkGrey)
        .with_border_right_color(Color::DarkGrey)
        .with_padding_left(1);

    let mut branch = Select::new(&["origin/main", "origin/my-happy-branch", "origin/the-carp-stands-up"])
                            // .with_background_color(Color::Cyan)    
    ;

    EventLoop::new().with_style_sheet(&stylesheet).run(
        ControlFlow::Wait,
        DisplayMode::AlternateScreen,
        &mut Container::new([
            // &mut name,
            // &mut title,
            // &mut user,
            &mut commit,
            &mut Disclosure::new("Commit style guide", label2)
                .with_margin_bottom(2)
                .with_margin_left(1),
            // &mut Disclosure::new("Remote branch", &mut container).with_margin_bottom(2).with_margin_left(1),
            &mut Frame::new(&mut branch)
            .with_title(" Branch ")
            // .with_title_style(
            //     Style::new().with_padding_left(2)
            //                 .with_padding_right(2)
            //                 .with_color(Color::Grey)
            //                 .with_border_top_color(Color::Grey)
            //                 .with_border_bottom_color(Color::Grey)
            //                 .with_border_left_color(Color::Grey)
            //                 .with_border_right_color(Color::Grey)
            //                 .with_border_top_width(1)
            //                 // .with_background_color(Color::Cyan)
            //                 .with_border_bottom_width(0)
            //                 .with_margin_left(0)
            //                 .with_border_right_width(1)
            //                 .with_border_left_width(1)
            //                 .with_border_style(BorderStyle::Ascii {
            //                     top_left: '╭',
            //                     top: '─',
            //                     top_right: '╮',
            //                     left: '┤',
            //                     right: '├',
            //                     bottom_left: '└',
            //                     bottom: '─',
            //                     bottom_right: '┘',
            //                 })
            // )                                 
            .with_border_left_width(1)
            .with_border_top_width(1)
            .with_border_bottom_width(1)
            .with_border_right_width(1)
            .with_border_style(thin_border)
            .with_border_left_color(Color::Grey)
            .with_border_bottom_color(Color::Grey)
            .with_border_top_color(Color::Grey)
            .with_border_right_color(Color::Grey)
            .with_padding_left(2), // &mut frame
        ])
        .with_margin_left(6)
        .with_margin_top(3),
    )
}
