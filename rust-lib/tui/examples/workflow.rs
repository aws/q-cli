use tui::components::{
    CollapsiblePicker,
    Disclosure,
    FilterablePicker,
    Frame,
    Label,
    Picker,
    PickerComponent,
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

    let selection_border = BorderStyle::Ascii {
        top_left: ' ',
        top: ' ',
        top_right: ' ',
        left: '▸',
        right: ' ',
        bottom_left: ' ',
        bottom: ' ',
        bottom_right: ' ',
    };

    let focus_style = Style::new()
        .with_border_left_color(Color::White)
        .with_border_right_color(Color::White)
        .with_border_top_color(Color::White)
        .with_border_bottom_color(Color::White)
        .with_border_style(thick_border);

    let unfocused_style = Style::new()
        .with_border_left_width(1)
        .with_border_top_width(1)
        .with_border_bottom_width(1)
        .with_border_right_width(1)
        .with_border_left_color(Color::DarkGrey)
        .with_border_right_color(Color::DarkGrey)
        .with_border_top_color(Color::DarkGrey)
        .with_border_bottom_color(Color::DarkGrey)
        .with_border_style(rounded_border);
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
        .with_style("frame", unfocused_style)
        .with_style(
            "frame.title",
            Style::new()
                .with_color(Color::DarkGrey)
                .with_padding_left(1)
                .with_padding_right(1)
                .with_margin_left(1),
        )
        .with_style(
            "frame.title:focus",
            Style::new()
            .with_color(Color::White)
            // .with_background_color(Color::White)
            .with_padding_left(1)
            .with_padding_right(1)
            .with_margin_left(1),
        )
        .with_style("frame:focus", focus_style)
        .with_style("textfield", Style::new().with_padding_left(2).with_color(Color::Grey))
        .with_style("textfield:focus", focus_style.with_color(Color::White))
        .with_style("disclosure.summary:focus", Style::new().with_color(Color::White))
        .with_style("disclosure.summary", Style::new().with_color(Color::Cyan))
        .with_style(
            "picker.item",
            Style::new().with_padding_left(2).with_color(Color::DarkGrey),
        )
        .with_style(
            "picker.item:focus",
            Style::new().with_padding_left(2).with_color(Color::White),
        )
        .with_style(
            "picker.selected",
            Style::new()
                .with_margin_left(2)
                .with_background_color(Color::DarkGrey)
                .with_color(Color::Grey),
        )
        .with_style(
            "picker.selected:focus",
            Style::new()
                .with_margin_left(2)
                .with_background_color(Color::White)
                .with_color(Color::DarkGrey),
        );

    let mut textfield = TextField::new().with_hint("fix: syntax error");
    let mut commit = Frame::new(&mut textfield).with_title("Commit message");

    let mut branch = CollapsiblePicker::<FilterablePicker>::new(vec!["main", "my-happy-branch", "the-carp-stands-up"])
        .with_placeholder("Select a branch...");
    let mut branch_picker = Frame::new(&mut branch).with_title("Branch");

    let mut remote = CollapsiblePicker::<FilterablePicker>::new(vec![
        "origin",
        "heroku",
        "github",
        "aws",
        "node",
        "vercel",
        "git.fig.io",
    ])
    .with_placeholder("Select a remote...");

    let mut remote_picker = Frame::new(&mut remote).with_title("Remote");

    let mut git_author =
        FilterablePicker::new(vec!["matt@fig.io", "matthewschrage@gmail.com"]).with_placeholder("Search...");
    let mut git_author_picker = Frame::new(&mut git_author).with_title("Git Author");

    EventLoop::new().with_style_sheet(&stylesheet).run(
        ControlFlow::Wait,
        DisplayMode::AlternateScreen,
        &mut Form::new([
            &mut commit,
            &mut branch_picker,
            &mut remote_picker,
            &mut git_author_picker,
        ])
        .with_margin_top(1)
        .with_margin_left(2),
    )
}
