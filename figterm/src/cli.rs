use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
