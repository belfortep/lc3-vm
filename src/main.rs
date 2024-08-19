use lc3_vm::virtual_machine_start::{
    debug_program_from_file, execute_program_from_file, execute_vm_in_interactive_mode,
};

use clap::{arg, ArgGroup, ArgMatches, Command};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

const STDIN: i32 = 0;

struct TermiosWrapper {
    termios: Termios,
}

impl TermiosWrapper {
    pub fn new() -> Result<Self, String> {
        let termios = Termios::from_fd(STDIN).map_err(|error| error.to_string())?;
        let mut new_termios = termios;
        new_termios.c_lflag &= !ICANON & !ECHO;

        tcsetattr(STDIN, TCSANOW, &new_termios).map_err(|error| error.to_string())?;
        Ok(Self {
            termios: new_termios,
        })
    }
}

impl Drop for TermiosWrapper {
    fn drop(&mut self) {
        tcsetattr(STDIN, TCSANOW, &self.termios).expect("Couldn't return terminal to normal");
    }
}

pub fn receive_command_line_arguments() -> Result<ArgMatches, String> {
    let args = Command::new("LC3 Virtual Machine")
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

fn main() -> Result<(), String> {
    let args = receive_command_line_arguments()?;
    if let Some(file) = args.get_one::<String>("file") {
        TermiosWrapper::new()?;
        execute_program_from_file(file)?;
    }

    if args.get_flag("interactive") {
        execute_vm_in_interactive_mode()?;
    }

    if let Some(file) = args.get_one::<String>("debug") {
        debug_program_from_file(file)?;
    }

    Ok(())
}
