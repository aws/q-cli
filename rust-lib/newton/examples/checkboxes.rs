use std::collections::HashMap;

use newton::{
    Color,
    ControlFlow,
    DisplayMode,
    Event,
    EventLoop,
    KeyCode,
};

#[derive(Default)]
struct State {
    active: (usize, usize),
    selected: HashMap<(usize, usize), bool>,
}

fn main() {
    let mut event_loop = EventLoop::new();
    let mut state = State::default();

    let a = vec!["Hello", "World"];
    let b = vec!["The", "Carp", "Stands", "Up"];
    let choices = [&a, &b];

    event_loop
        .run::<_, std::io::Error>(
            ControlFlow::Wait,
            DisplayMode::AlternateScreen,
            |event, display_state, control_flow| {
                match event {
                    Event::KeyPressed { code, .. } => match code {
                        KeyCode::Enter => {
                            state
                                .selected
                                .insert(state.active, match state.selected.get(&state.active) {
                                    Some(true) => false,
                                    Some(false) => true,
                                    None => true,
                                });
                        },
                        KeyCode::Up => {
                            state.active.1 =
                                (state.active.1 + choices[state.active.0].len() - 1) % choices[state.active.0].len()
                        },
                        KeyCode::Down => state.active.1 = (state.active.1 + 1) % choices[state.active.0].len(),
                        KeyCode::Tab => {
                            state.active.0 = (state.active.0 + 1) % choices.len();
                            state.active.1 = 0;
                        },
                        KeyCode::BackTab => {
                            state.active.0 = (state.active.0 + choices.len() - 1) % choices.len();
                            state.active.1 = 0;
                        },
                        KeyCode::Esc => *control_flow = ControlFlow::Exit,
                        _ => (),
                    },
                    _ => (),
                }

                display_state.clear();

                for (i, field) in choices.iter().enumerate() {
                    display_state.draw_string(
                        format!("Flags"),
                        0,
                        (i * 4).try_into().unwrap(),
                        Color::Reset,
                        Color::Reset,
                    );
                    for (k, choice) in field.iter().enumerate() {
                        display_state.draw_string(
                            format!(
                                "{}[{}] {choice}",
                                match (i, k) == state.active {
                                    true => "> ",
                                    false => "  ",
                                },
                                match state.selected.get(&(i, k)) {
                                    Some(true) => 'x',
                                    _ => ' ',
                                }
                            ),
                            0,
                            (i * 4 + k + 1).try_into().unwrap(),
                            Color::Reset,
                            Color::Reset,
                        );
                    }
                }

                Ok(())
            },
        )
        .unwrap();
}
