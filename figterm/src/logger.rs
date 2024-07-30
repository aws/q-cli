use std::fmt::Display;

use fig_log::get_max_fig_log_level;
use tracing::Level;

pub fn stdio_debug_log(s: impl Display) {
    if get_max_fig_log_level() >= Level::DEBUG {
        println!("{s}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_debug_log() {
        stdio_debug_log("test");
    }
}
