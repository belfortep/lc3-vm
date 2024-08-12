use std::{fs::File, io::BufReader};

use clap::{arg, ArgGroup, ArgMatches, Command};

pub fn receive_command_line_arguments() -> Result<ArgMatches, String> {
    let args = Command::new(" Conway's game of life")
        .arg(arg!(-i --interactive "interactive console").required(false))
        .arg(arg!(-f --file <FILE> "file to execute").required(false))
        .arg(arg!(-d --debug <FILE> "debug file").required(false))
        .group(
            ArgGroup::new("run program")
                .args(["interactive", "file", "debug"])
                .required(false),
        )
        .after_help("Don't use -i, -f or -d at the same time")
        .get_matches();

    Ok(args)
}

pub fn receive_file(arg: String) -> Result<BufReader<File>, String> {
    let file = File::open(arg).map_err(|error| error.to_string())?;
    let file_reader = BufReader::new(file);
    Ok(file_reader)
}
