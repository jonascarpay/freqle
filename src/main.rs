use clap::{arg, command, Command};

fn main() {
    let matches = command!()
        .arg(arg!([name] "Optional name"))
        .arg(arg!(-c --config <FILE> "Sets a config file").required(false))
        .arg(arg!(-d --debug ... "Turn debugging info on"))
        .subcommand(
            Command::new("test")
                .about("tests")
                .arg(arg!(-l --list "does list")),
        )
        .get_matches();
    print!("{matches:?}");
}
