use clap::{Arg, Command as ClapCommand};

pub struct Config {
    pub bind_address: String,
    pub port: u16,
}

pub fn parse_args() -> Config {
    let matches = ClapCommand::new("MkArchQemu Server")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Runs MkArchQemu build server")
        .arg(
            Arg::new("bind_address")
                .short('b')
                .long("bind")
                .value_parser(clap::value_parser!(String))
                .default_value("127.0.0.1")
                .help("Bind address for the server"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_parser(clap::value_parser!(u16))
                .default_value("8080")
                .help("Port number for the server"),
        )
        .get_matches();

    Config {
        bind_address: matches.get_one::<String>("bind_address").unwrap().to_string(),
        port: *matches.get_one::<u16>("port").unwrap(),
    }
}

