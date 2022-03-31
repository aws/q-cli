use std::time::Duration;
use viuer::Config as ViuerConfig;

pub struct Config<'a> {
    pub files: Vec<&'a str>,
    pub loop_gif: bool,
    pub name: bool,
    pub recursive: bool,
    pub static_gif: bool,
    pub viuer_config: ViuerConfig,
    pub frame_duration: Option<Duration>,
    pub loading_message: &'a str,
    pub cleanup_message: &'a str,
}

impl<'a> Config<'a> {
    pub fn new(width: Option<u32>,
        height: Option<u32>,
        files: Option<Vec<&'a str>>,
        once: bool,
        static_gif: bool,
        transparent: bool,
        use_blocks: bool,
        name: bool,
        recursive: bool,
        frames_per_second: Option<i32>,
        loading_message: &'a str,
        cleanup_message: &'a str,
    ) -> Config<'a> {
        
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

        let frame_duration = if !frames_per_second.is_none() {
            Some(Duration::from_secs_f32(1.0 / frames_per_second.unwrap() as f32))
        } else {
            None
        };

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
    pub fn test_config() -> Config<'a> {
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
            cleanup_message: ""
        }
    }
}
