use byteorder::{BigEndian, ReadBytesExt};
use lc3_vm::{
    command_line_parser::parser::{receive_command_line_arguments, receive_file},
    virtual_machine::lc3_virtual_machine::LC3VirtualMachine,
};

fn main() -> Result<(), String> {
    let args = receive_command_line_arguments()?;
    let program_counter_start = 0x3000;

    if let Some(file) = args.get_one::<String>("file") {
        let mut reader = receive_file(file.clone())?;

        let program_counter_start = reader
            .read_u16::<BigEndian>()
            .map_err(|error| error.to_string())?;

        let mut virtual_machine = LC3VirtualMachine::new(program_counter_start);
        let mut memory_address = program_counter_start;
        loop {
            match reader.read_u16::<BigEndian>() {
                Ok(instruction) => {
                    virtual_machine.memory_write(memory_address, instruction);
                    memory_address += 1;
                }
                Err(_) => break,
            }
        }
        loop {
            virtual_machine.next_instruction();
        }
    }

    Ok(())
}
