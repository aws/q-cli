use clap::Parser;

#[derive(Debug, Parser)]
#[clap(version, about)]
pub struct Cli {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_app() {
        use clap::IntoApp;
        Cli::command().debug_assert()
    }
}
