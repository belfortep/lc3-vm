use lc3_vm::command_line_parser::parser::{
    debug_program_from_file, execute_program_from_file, execute_vm_in_interactive_mode,
    receive_command_line_arguments,
};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

fn disable_input_buffering() -> Result<(), String> {
    let mut termios = Termios::from_fd(0).map_err(|error| error.to_string())?;

    termios.c_lflag &= !ICANON & !ECHO;

    tcsetattr(0, TCSANOW, &termios).map_err(|error| error.to_string())?;

    Ok(())
}

fn main() -> Result<(), String> {
    let args = receive_command_line_arguments()?;

    if let Some(file) = args.get_one::<String>("file") {
        disable_input_buffering()?;
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
