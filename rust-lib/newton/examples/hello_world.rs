use newton::{
    Color,
    ControlFlow,
    DisplayMode,
    Event,
    EventLoop,
    KeyCode,
};

fn main() {
    let mut event_loop = EventLoop::new();

    event_loop
        .run::<_, std::io::Error>(
            ControlFlow::Wait,
            DisplayMode::AlternateScreen,
            move |event, display_state, control_flow| {
                match event {
                    Event::KeyPressed { code: KeyCode::Esc, .. } => *control_flow = ControlFlow::Exit,
                    _ => (),
                }

                display_state.clear();

                display_state
                    .draw_string("hello world! 🧙", 0, 0, Color::Reset, Color::Reset)
                    .draw_string("press escape to exit...", 0, 20, Color::Reset, Color::Reset);

                for fg in 0..16 {
                    for bg in 0..16 {
                        display_state.draw_string(
                            " col ",
                            (fg * 5).into(),
                            (bg + 3).into(),
                            Color::AnsiValue(fg),
                            Color::AnsiValue(bg),
                        );
                    }
                }

                display_state.draw_string(
                    format!(
                        "The terminal size is ({}, {})",
                        display_state.width(),
                        display_state.height()
                    ),
                    0,
                    1,
                    Color::Reset,
                    Color::Reset,
                );

                Ok(())
            },
        )
        .unwrap();
}
