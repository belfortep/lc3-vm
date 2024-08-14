use std::{
    fs::File,
    io::{stdin, BufRead, BufReader, Write},
    net::TcpListener,
};

use byteorder::{BigEndian, ReadBytesExt};
use clap::{arg, ArgGroup, ArgMatches, Command};

use crate::virtual_machine::{lc3_virtual_machine::LC3VirtualMachine, register::Register};

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

pub fn execute_program_from_file(file: &str) -> Result<(), String> {
    let reader = receive_file(file.to_owned())?;
    let mut virtual_machine = load_reader_file_to_vm_memory(reader)?;
    loop {
        virtual_machine.next_instruction();
    }
}

pub fn debug_program_from_file(file: &str) -> Result<(), String> {
    let reader = receive_file(file.to_owned())?;
    let mut virtual_machine = load_reader_file_to_vm_memory(reader)?;
    let listener = TcpListener::bind("127.0.0.1:3000").map_err(|error| error.to_string())?;
    let (stream, _) = listener.accept().map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(&stream);

    loop {
        let mut command = String::new();
        reader
            .read_line(&mut command)
            .map_err(|error| error.to_string())?;
        let command = command.trim();
        if command == "n" {
            virtual_machine.next_instruction();
            let memory_address = virtual_machine.read_register(Register::ProgramCounter);
            let instruction = virtual_machine.memory_read(memory_address);
            let response = format!("instruction: {instruction:#018b}",);
            writeln!(&stream, "{}", response).map_err(|error| error.to_string())?;
        } else if command == "r" {
            let response = virtual_machine.state_of_registers();
            writeln!(&stream, "{}", response).map_err(|error| error.to_string())?;
        } else {
            match command.parse::<u16>() {
                Ok(number) => {
                    virtual_machine.next_instructions(number);
                    let response = format!("executed {} instructions", number);
                    writeln!(&stream, "{}", response).map_err(|error| error.to_string())?;
                }
                Err(_) => {
                    writeln!(&stream, "Invalid Command").map_err(|error| error.to_string())?
                }
            }
        }
    }
}

pub fn execute_vm_in_interactive_mode() -> Result<(), String> {
    let program_counter_start = 0x3000;

    let mut virtual_machine = LC3VirtualMachine::new(program_counter_start);

    for line in stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line == "r" {
            let registers = virtual_machine.state_of_registers();
            let registers = registers.split("::");
            for register in registers {
                println!("{}", register);
            }
            continue;
        }
        match u16::from_str_radix(&line, 2) {
            Ok(instruction) => {
                virtual_machine.decode_instruction(instruction);
                let instruction = format!("{instruction:#018b}",);
                println!("instruction proccess: {}", instruction);
            }
            Err(_) => println!("Wrong instruction format"),
        }
    }
    Ok(())
}

fn receive_file(arg: String) -> Result<BufReader<File>, String> {
    let file = File::open(arg).map_err(|error| error.to_string())?;
    let file_reader = BufReader::new(file);
    Ok(file_reader)
}

fn load_reader_file_to_vm_memory(mut reader: BufReader<File>) -> Result<LC3VirtualMachine, String> {
    let program_counter_start = reader
        .read_u16::<BigEndian>()
        .map_err(|error| error.to_string())?;

    let mut virtual_machine = LC3VirtualMachine::new(program_counter_start);
    let mut memory_address = program_counter_start;
    while let Ok(instruction) = reader.read_u16::<BigEndian>() {
        virtual_machine.memory_write(memory_address, instruction);
        memory_address += 1;
    }
    Ok(virtual_machine)
}
