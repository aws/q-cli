use std::path::PathBuf;
use std::str::FromStr;

use lightningcss::stylesheet::{
    ParserOptions,
    StyleSheet,
};
use tui::component::{
    Div,
    TextField,
};
use tui::{
    ControlFlow,
    DisplayMode,
    EventLoop,
    InputMethod,
};

fn main() {
    EventLoop::new(
        Div::new().with_id("parent").push(TextField::new()),
        DisplayMode::AlternateScreen,
        InputMethod::new(),
        StyleSheet::parse(include_str!("form.css"), ParserOptions::default()).unwrap(),
        ControlFlow::Wait,
    )
    .with_style_sheet_path(PathBuf::from_str("examples/form.css").unwrap())
    .run(|event, _component, control_flow| match event {
        tui::Event::Quit | tui::Event::Terminate => *control_flow = ControlFlow::Quit,
        _ => (),
    })
    .unwrap();
}
