use lightningcss::stylesheet::{
    ParserOptions,
    StyleSheet,
};
use tui::component::Div;
use tui::{
    ControlFlow,
    DisplayMode,
    EventLoop,
    InputMethod,
};

fn main() {
    EventLoop::new(
        Div::new().with_id("parent"),
        DisplayMode::AlternateScreen,
        InputMethod::new(),
        StyleSheet::parse(include_str!("form.css"), ParserOptions::default()).unwrap(),
    )
    .run(|event, _component, control_flow| match event {
        tui::Event::Quit | tui::Event::Terminate => *control_flow = ControlFlow::Quit,
        _ => (),
    })
    .unwrap();
}
