use lc3_vm::command_line_parser::parser::{
    debug_program_from_file, execute_program_from_file, execute_vm_in_interactive_mode,
    receive_command_line_arguments,
};

fn main() -> Result<(), String> {
    let args = receive_command_line_arguments()?;

    if let Some(file) = args.get_one::<String>("file") {
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
