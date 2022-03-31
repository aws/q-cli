mod app;
mod config;

use config::Config;
use viuer::ViuError;

pub fn display_gif(path: &str, cleanup_message: &str) -> Result<(), ViuError>{
    
    let mut files = Vec::new();
    files.push(path);

    let conf = Config::new(
        None,
        None,
        Some(files),
        false,
        false,
        false,
        true,
        false,
        false,
        None,
        cleanup_message,
    );


    return app::run(conf);
}