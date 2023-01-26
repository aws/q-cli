use lightningcss::stylesheet::{
    ParserOptions,
    StyleSheet,
};
use tui::component::SegmentedControl;
use tui::{
    ControlFlow,
    DisplayMode,
    EventLoop,
    InputMethod,
};

fn main() {
    EventLoop::new(
        SegmentedControl::new(vec![
            "👨‍👩‍👦‍👦 family".to_owned(),
            "🐱 cat".to_owned(),
            "🐁 mouse".to_owned(),
            "🦤 dodo".to_owned(),
            "👨‍👩‍👦‍👦 family".to_owned(),
            "👩‍🔬 scientist".to_owned(),
        ]),
        DisplayMode::Inline,
        InputMethod::default(),
        StyleSheet::parse(include_str!("form.css"), ParserOptions::default()).unwrap(),
        ControlFlow::Wait,
    )
    .with_style_sheet_path("./examples/form.css")
    .run(|event, _component, control_flow| match event {
        tui::Event::Quit | tui::Event::Terminate => *control_flow = ControlFlow::Quit,
        _ => (),
    })
    .unwrap();
}
