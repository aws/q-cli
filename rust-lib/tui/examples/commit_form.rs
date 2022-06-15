use tui::components::{
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
    // let double_border = BorderStyle::Ascii {
    //     top_left: '╔',
    //     top: '═',
    //     top_right: '╗',
    //     left: '║',
    //     right: '║',
    //     bottom_left: '╚',
    //     bottom: '═',
    //     bottom_right: '╝',
    // };

    // let thin_border = BorderStyle::Ascii {
    //     top_left: '┌',
    //     top: '─',
    //     top_right: '┐',
    //     left: '│',
    //     right: '│',
    //     bottom_left: '└',
    //     bottom: '─',
    //     bottom_right: '┘',
    // };

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

    let style_sheet = StyleSheet::new()
        .with_style("*", Style::new().with_width(50))
        .with_style(
            "div",
            Style::new()
                .with_border_top_width(1)
                .with_border_bottom_width(1)
                .with_border_left_width(1)
                .with_border_right_width(1)
                .with_border_style(rounded_border)
                .with_border_color(Color::DarkGrey)
                .with_padding_left(1)
                .with_padding_right(1),
        )
        .with_style("label", Style::new().with_color(Color::Cyan))
        .with_style("select", Style::new())
        .with_style("textfield", Style::default())
        .with_style(".my_border", Style::default());

    let mut message = TextField::new().with_hint("message");
    let mut remote = Select::new(&["origin"]);
    let mut branch = Select::new(&["origin/main", "origin/my-happy-branch", "origin/the-carp-stands-up"]);

    EventLoop::new()
        .with_style_sheet(&style_sheet)
        .run::<std::io::Error, _>(
            ControlFlow::Wait,
            DisplayMode::AlternateScreen,
            &mut Form::new(vec![
                &mut Container::new(vec![&mut Label::new("commit message:"), &mut message]),
                &mut Container::new(vec![&mut Label::new("remote:"), &mut remote]),
                &mut Container::new(vec![&mut Label::new("branch:"), &mut branch]),
            ])
            .with_height(40),
        )?;

    println!(
        "git commit -m '{}'\ngit push {} {}",
        message.text,
        remote.value_of_selected(),
        branch.value_of_selected()
    );

    Ok(())
}
