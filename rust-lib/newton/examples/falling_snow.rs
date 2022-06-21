use std::time::Instant;

use newton::{
    Color,
    ControlFlow,
    DisplayMode,
    Event,
    EventLoop,
    KeyCode,
};
use rand::Rng;

struct SnowFlake {
    pub x: u16,
    pub y: f32,
    pub speed: f32,
}

struct State {
    snowflakes: Vec<SnowFlake>,
    instant: Instant,
}

impl Default for State {
    fn default() -> Self {
        let mut snowflakes = vec![];
        for _ in 0..300 {
            snowflakes.push(SnowFlake {
                x: rand::random(),
                y: rand::thread_rng().gen_range(0.0..f32::from(u16::MAX)),
                speed: rand::thread_rng().gen_range(1.0..8.0),
            })
        }

        Self {
            snowflakes,
            instant: Instant::now(),
        }
    }
}

fn main() {
    let mut event_loop = EventLoop::new();
    let mut state = State::default();

    event_loop
        .run::<_, std::io::Error>(
            ControlFlow::Poll,
            DisplayMode::AlternateScreen,
            move |event, display_state, control_flow| {
                match event {
                    Event::KeyPressed { code: KeyCode::Esc, .. } => *control_flow = ControlFlow::Exit,
                    _ => (),
                }

                display_state.clear();
                let delta = state.instant.elapsed().as_secs_f32();
                state.instant = Instant::now();

                for star in &mut state.snowflakes {
                    star.y += star.speed * delta;
                    star.y %= f32::from(display_state.height());
                    display_state.draw_symbol(
                        '*',
                        star.x % display_state.width(),
                        star.y as u16,
                        Color::Reset,
                        Color::Reset,
                    );
                }

                Ok(())
            },
        )
        .unwrap();
}
