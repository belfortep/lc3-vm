use std::io::{stdin, BufRead};

use byteorder::{BigEndian, ReadBytesExt};
use lc3_vm::{
    command_line_parser::parser::{receive_command_line_arguments, receive_file},
    virtual_machine::{lc3_virtual_machine::LC3VirtualMachine, register::Register},
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
        while let Ok(instruction) = reader.read_u16::<BigEndian>() {
            virtual_machine.memory_write(memory_address, instruction);
            memory_address += 1;
        }

        loop {
            virtual_machine.next_instruction();
        }
    }

    if let Some(file) = args.get_one::<String>("debug") {
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
        for line in stdin().lock().lines() {
            let line = line.map_err(|error| error.to_string())?;
            if line == "n" {
                virtual_machine.next_instruction();
            } else if line == "r" {
                virtual_machine.print_registers();
            }
            let memory_address = virtual_machine.read_register(Register::ProgramCounter);
            let instruction = virtual_machine.memory_read(memory_address);
            println!("instruction: {}", format!("{instruction:#018b}",))
        }
    }

    if args.get_flag("interactive") {
        let mut virtual_machine = LC3VirtualMachine::new(program_counter_start);

        for line in stdin().lock().lines() {
            let line = line.map_err(|error| error.to_string())?;
            if line == "r" {
                virtual_machine.print_registers();
                continue;
            }

            let instruction = u16::from_str_radix(&line, 2).map_err(|error| error.to_string())?;
            virtual_machine.process_input(instruction);
            println!("instruction proccess: {}", format!("{instruction:#018b}",))
        }
    }

    Ok(())
}
