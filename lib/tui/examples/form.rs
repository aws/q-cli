use fig_log::{
    set_fig_log_level,
    Logger,
};
use lightningcss::stylesheet::{
    ParserOptions,
    StyleSheet,
};
use tui::component::{
    CheckBox,
    Container,
    Layout,
    Paragraph,
    Select,
    TextField,
};
use tui::{
    ControlFlow,
    EventLoop,
    InputMethod,
};

fn main() {
    let logger = Logger::new().with_file("test.log");
    let _logger_guard = logger.init().expect("Failed to init logger");
    set_fig_log_level("error".to_string()).ok();

    EventLoop::new()
        .run(
            Container::new("parent", Layout::Vertical)
                .push(
                    Container::new("inner-1", Layout::Horizontal)
                        .push(CheckBox::new("check_box", "Are you cool?", false))
                        .push(TextField::new("text-field").with_text("hi there"))
                        .push(Paragraph::new("").push_text("hello world!"))
                        .push(TextField::new("text-field").with_text("hi there 2"))
                        .push(Select::new(
                            "select",
                            vec!["hello".to_owned(), "world".to_owned()],
                            false,
                        )),
                )
                .push(
                    Container::new("inner-2", Layout::Horizontal)
                        .push(TextField::new("text-field").with_text("hi there a"))
                        .push(TextField::new("text-field").with_text("hi there b"))
                        .push(Select::new(
                            "select",
                            vec!["hello".to_owned(), "world".to_owned()],
                            false,
                        )),
                )
                .push(
                    Container::new("inner-3", Layout::Horizontal)
                        .push(Paragraph::new("").push_text("upper level"))
                        .push(Container::new("nested", Layout::Horizontal).push(Paragraph::new("").push_text("3rd p"))),
                ),
            &InputMethod::new(),
            StyleSheet::parse(include_str!("form.css"), ParserOptions::default()).unwrap(),
            |event, _component, control_flow| match event {
                tui::Event::Quit | tui::Event::Terminate => *control_flow = ControlFlow::Quit,
                _ => (),
            },
        )
        .unwrap();
}
