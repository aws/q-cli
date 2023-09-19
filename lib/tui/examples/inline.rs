use lightningcss::stylesheet::{
    ParserOptions,
    StyleSheet,
};
use tui::component::{
    Div,
    Hr,
    Multiselect,
    SegmentedControl,
};
use tui::{
    ControlFlow,
    DisplayMode,
    EventLoop,
    InputMethod,
};

fn main() {
    EventLoop::new(
        Div::new()
            .push(SegmentedControl::new(vec![
                "👨‍👩‍👦‍👦 family".to_owned(),
                "🐱 cat".to_owned(),
                "🐁 mouse".to_owned(),
                "🦤 dodo".to_owned(),
                "👨‍👩‍👦‍👦 family".to_owned(),
                "👩‍🔬 scientist".to_owned(),
            ]))
            .push(Hr::new())
            .push(Multiselect::new(vec![
                "a".to_owned(),
                "b".to_owned(),
                "c".to_owned(),
                "d".to_owned(),
            ])),
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
