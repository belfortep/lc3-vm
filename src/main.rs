use lc3_vm::{
    command_line_parser::parser::receive_command_line_arguments,
    virtual_machine_start::{
        debug_program_from_file, execute_program_from_file, execute_vm_in_interactive_mode,
    },
};
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
