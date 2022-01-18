use clap::StructOpt;

mod cli;

fn main() {
    cli::Cli::parse().execute();
}
