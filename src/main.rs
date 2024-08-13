use std::{
    io::{stdin, BufRead, BufReader, Write},
    net::TcpListener,
};

use lc3_vm::{
    command_line_parser::parser::{
        load_reader_file_to_vm_memory, receive_command_line_arguments, receive_file,
    },
    virtual_machine::{lc3_virtual_machine::LC3VirtualMachine, register::Register},
};

fn main() -> Result<(), String> {
    let args = receive_command_line_arguments()?;
    let program_counter_start = 0x3000;

    if let Some(file) = args.get_one::<String>("file") {
        let reader = receive_file(file.clone())?;
        let mut virtual_machine = load_reader_file_to_vm_memory(reader)?;
        loop {
            virtual_machine.next_instruction();
        }
    }

    if let Some(file) = args.get_one::<String>("debug") {
        let reader = receive_file(file.clone())?;
        let mut virtual_machine = load_reader_file_to_vm_memory(reader)?;

        let listener = TcpListener::bind("127.0.0.1:3000").map_err(|error| error.to_string())?;
        let (stream, _) = listener.accept().map_err(|error| error.to_string())?;

        let mut reader = BufReader::new(&stream);

        loop {
            let mut command = String::new();
            reader
                .read_line(&mut command)
                .map_err(|error| error.to_string())?;
            if command == "\n" {
                virtual_machine.next_instruction();
                let memory_address = virtual_machine.read_register(Register::ProgramCounter);
                let instruction = virtual_machine.memory_read(memory_address);
                let response = format!("instruction: {instruction:#018b}",);
                writeln!(&stream, "{}", response).map_err(|error| error.to_string())?;
            } else if command == "r\n" {
                let response = virtual_machine.print_registers();

                writeln!(&stream, "{}", response).map_err(|error| error.to_string())?;
            } else {
                let command = command.trim();
                match command.parse::<u16>() {
                    Ok(number) => {
                        for _ in 0..number {
                            virtual_machine.next_instruction();
                        }
                        let response = format!("executed {} instructions", number);
                        writeln!(&stream, "{}", response).map_err(|error| error.to_string())?;
                    }
                    Err(_) => {
                        let response = "Invalid command";

                        writeln!(&stream, "{}", response).map_err(|error| error.to_string())?;
                    }
                }
            }
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
