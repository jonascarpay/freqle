use clap::{arg, command};

fn main() {
    let matches = command!()
        .arg(arg!([name] "Optional name"))
        .arg(arg!(-c --config <FILE> "Sets a config file").required(false))
        .arg(arg!(-d --debug ... "Turn debugging info on"))
        .subcommand(
            clap::Command::new("test")
                .about("tests")
                .arg(arg!(-l --list "does list")),
        )
        .get_matches();
    print!("{matches:?}");
}

struct Weights {
    hour: f64,
    day: f64,
    month: f64,
}

struct FileArgs {
    path: String,
    strict: bool,
}

enum Command {
    Bump {
        file_args: FileArgs,
        query: String,
        thresh: f64,
    },
    Touch {
        file_args: FileArgs,
        thresh: f64,
    },
    Delete {
        file_args: FileArgs,
        query: String,
    },
    View {
        file_args: FileArgs,
        weights: Weights,
        augment: bool,
        restrict: bool,
        verbose: bool,
    },
}
