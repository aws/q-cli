use lightningcss::stylesheet::{
    ParserOptions,
    StyleSheet,
};
use tui::component::{
    CheckBox,
    Div,
    FilePicker,
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
        Div::new()
            .push(FilePicker::new(true, false, vec![]))
            .push(CheckBox::new("hi", false))
            .push(TextField::new()),
        DisplayMode::Inline,
        InputMethod::default(),
        StyleSheet::parse(include_str!("form.css"), ParserOptions::default()).unwrap(),
        ControlFlow::Wait,
    )
    .run(|event, _component, control_flow| match event {
        tui::Event::Quit | tui::Event::Terminate => *control_flow = ControlFlow::Quit,
        _ => (),
    })
    .unwrap();
}
