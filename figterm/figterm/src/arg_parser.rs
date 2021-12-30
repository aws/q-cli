use clap::App;

#[derive(Debug)]
pub struct ArgParser {
    app: App<'static>,
}

impl ArgParser {
    pub fn new() -> Self {
        let app = App::new("figterm")
            .version("0.1.0")
            .author("Fig")
            .about("The Fig terminal layer");

        Self { app }
    }

    pub fn parse(self) -> clap::ArgMatches {
        self.app.get_matches()
    }
}
