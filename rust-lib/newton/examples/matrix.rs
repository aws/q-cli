use std::time::Instant;

use newton::{
    Color,
    ControlFlow,
    DisplayMode,
    Event,
    EventLoop,
    KeyCode,
};
use noise::{
    NoiseFn,
    Perlin,
};
use rand::distributions::Alphanumeric;
use rand::{
    Rng,
    SeedableRng,
};

fn main() {
    let mut event_loop = EventLoop::new();
    let noise = Perlin::new();
    let start = Instant::now();
    let mut chars = vec![];
    for y in 0..=255_u64 {
        chars.push(vec![]);
        for x in 0..=255_u64 {
            chars[usize::try_from(y).unwrap()].push(char::from(
                rand::rngs::StdRng::seed_from_u64(x << 16 | y).sample(Alphanumeric),
            ));
        }
    }

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

                for x in 0..display_state.width() {
                    for y in 0..display_state.height() {
                        display_state.draw_symbol(
                            match noise.get([
                                f64::from(x) / 16.0,
                                f64::from(y) / 32.0 - start.elapsed().as_secs_f64() + f64::from(x) * 32.0,
                            ]) < 0.15
                            {
                                true => chars[usize::from(y) % 256][usize::from(x) % 256],
                                false => ' ',
                            },
                            x,
                            y,
                            Color::Green,
                            Color::Reset,
                        );
                    }
                }

                Ok(())
            },
        )
        .unwrap();
}
