use lc3_vm::command_line_parser::parser::{
    execute_program_from_file, receive_command_line_arguments,
};

fn main() -> Result<(), String> {
    let args = receive_command_line_arguments()?;

    if let Some(file) = args.get_one::<String>("file") {
        execute_program_from_file(file)?;
    }

    Ok(())
}
