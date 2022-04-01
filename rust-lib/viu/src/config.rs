use std::{fmt::Display, time::Duration};
use viuer::Config as ViuerConfig;

pub struct Config<'a, D>
where
    D: Display,
{
    pub files: Vec<&'a str>,
    pub loop_gif: bool,
    pub name: bool,
    pub recursive: bool,
    pub static_gif: bool,
    pub viuer_config: ViuerConfig,
    pub frame_duration: Option<Duration>,
    pub loading_message: &'a D,
    pub cleanup_message: &'a D,
}

impl<'a, D> Config<'a, D>
where
    D: Display,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        width: Option<u32>,
        height: Option<u32>,
        files: Option<Vec<&'a str>>,
        once: bool,
        static_gif: bool,
        transparent: bool,
        use_blocks: bool,
        name: bool,
        recursive: bool,
        frames_per_second: Option<i32>,
        loading_message: &'a D,
        cleanup_message: &'a D,
    ) -> Config<'a, D> {
        let files = match files {
            None => Vec::new(),
            Some(values) => values,
        };

        let loop_gif = files.len() <= 1 && !once;

        let viuer_config = ViuerConfig {
            transparent,
            width,
            height,
            absolute_offset: false,
            use_kitty: !use_blocks,
            use_iterm: !use_blocks,
            #[cfg(feature = "sixel")]
            use_sixel: !use_blocks,
            ..Default::default()
        };

        let frame_duration = frames_per_second
            .map(|frames_per_second| Duration::from_secs_f32(1.0 / frames_per_second as f32));

        Config {
            files,
            loop_gif,
            name,
            recursive,
            static_gif,
            viuer_config,
            frame_duration,
            loading_message,
            cleanup_message,
        }
    }

    #[cfg(test)]
    pub fn test_config() -> Config<'a, &'static str> {
        Config {
            files: vec![],
            loop_gif: true,
            name: false,
            recursive: false,
            static_gif: false,
            viuer_config: ViuerConfig {
                absolute_offset: false,
                use_kitty: false,
                ..Default::default()
            },
            frame_duration: None,
            loading_message: &"",
            cleanup_message: &"",
        }
    }
}
