use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Force Mission Control to be open on launch
    #[clap(long, action)]
    pub mission_control: bool,
    /// Kill old instances of `fig_desktop`
    #[clap(long, action)]
    pub kill_old: bool,
    /// Allow launching multiple instances of `fig_desktop`
    #[clap(long, action)]
    pub allow_multiple: bool,
    /// Url to open
    #[clap(value_parser)]
    url_link: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
