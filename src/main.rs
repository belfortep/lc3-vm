use std::{
    io::{stdin, BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

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
            println!("Dale boca");
        }
    }

    if let Some(file) = args.get_one::<String>("debug") {
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
        let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
        let (stream, _) = listener.accept().unwrap();

        let mut reader = BufReader::new(&stream);

        loop {
            let mut command = String::new();
            reader.read_line(&mut command).unwrap();
            if command == "\n" {
                virtual_machine.next_instruction();
                let memory_address = virtual_machine.read_register(Register::ProgramCounter);
                let instruction = virtual_machine.memory_read(memory_address);
                let response = format!("instruction: {instruction:#018b}",);
                writeln!(&stream, "{}", response).unwrap();
            } else if command == "r\n" {
                let response = virtual_machine.print_registers();

                writeln!(&stream, "{}", response).unwrap();
            } else {
                let command = command.trim();
                match command.parse::<u16>() {
                    Ok(number) => {
                        for _ in 0..number {
                            virtual_machine.next_instruction();
                        }
                        let response = format!("executed {} instructions", number);
                        writeln!(&stream, "{}", response).unwrap();
                    }
                    Err(_) => {
                        let response = "Invalid command";

                        writeln!(&stream, "{}", response).unwrap();
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
