mod app;
mod config;

use config::Config;
use viuer::ViuError;

pub fn display_gif(path: &str) -> Result<(), ViuError>{
    
    let mut files = Vec::new();
    files.push(path);

    let conf = Config::new(
        None,
        None,
        Some(files),
        false,
        false,
        false,
        false,
        false,
        false,
        None
    );


    return app::run(conf);
}