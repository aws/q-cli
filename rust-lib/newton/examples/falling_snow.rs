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
    pub x: i32,
    pub y: f64,
    pub speed: f64,
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
                y: rand::thread_rng().gen_range(0.0..f64::from(u16::MAX)),
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
    let mut event_loop = EventLoop::new(DisplayMode::AlternateScreen).unwrap();
    let mut state = State::default();

    event_loop
        .run::<_, std::io::Error>(ControlFlow::Poll, move |event, display_state, control_flow| {
            match event {
                Event::KeyPressed { code: KeyCode::Esc, .. } => *control_flow = ControlFlow::Exit(0),
                _ => (),
            }

            display_state.clear();
            let delta = state.instant.elapsed().as_secs_f64();
            state.instant = Instant::now();

            for star in &mut state.snowflakes {
                star.y += star.speed * delta;
                star.y %= f64::from(display_state.height());
                display_state.draw_symbol(
                    '*',
                    star.x % display_state.width(),
                    star.y as i32,
                    Color::Reset,
                    Color::Reset,
                    false,
                );
            }

            Ok(())
        })
        .unwrap();
}
