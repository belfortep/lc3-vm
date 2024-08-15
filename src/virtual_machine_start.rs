use crate::{
    constants::{CLIENT_PATH, SERVER_PATH, STREAM_DATA_SEPARATOR},
    virtual_machine::{lc3_virtual_machine::LC3VirtualMachine, register::Register},
};
use byteorder::{BigEndian, ReadBytesExt};

use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
    os::unix::net::UnixDatagram,
    path::Path,
};

pub fn execute_program_from_file(file: &str) -> Result<(), String> {
    let reader = receive_file(file)?;
    let mut virtual_machine = load_reader_file_to_vm_memory(reader)?;
    loop {
        virtual_machine.next_instruction();
    }
}

fn print_instructions_for_debugger(file: &str) {
    println!("Starting debugging of the program {}", file);
    println!("Remember to open the debugger from another terminal with make debugger or cargo run --bin debugger");
}

pub fn debug_program_from_file(file: &str) -> Result<(), String> {
    let reader = receive_file(file)?;
    let mut virtual_machine = load_reader_file_to_vm_memory(reader)?;
    let socket: UnixDatagram =
        UnixDatagram::bind(SERVER_PATH).map_err(|error| error.to_string())?;
    print_instructions_for_debugger(file);
    loop {
        let mut buffer = [0; 1024];
        match socket.recv_from(&mut buffer) {
            Ok((size, addr)) => {
                let command = String::from_utf8_lossy(&buffer[..size]);
                let command = command.trim();
                match command {
                    "n" => {
                        virtual_machine.next_instruction();
                        let memory_address =
                            virtual_machine.read_register(Register::ProgramCounter);
                        let instruction = virtual_machine.memory_read(memory_address);
                        let response = format!("instruction: {instruction:#018b}",);
                        socket
                            .send_to_addr(response.as_bytes(), &addr)
                            .map_err(|error| error.to_string())?;
                    }
                    "r" => {
                        let response = virtual_machine.state_of_registers();
                        socket
                            .send_to_addr(response.as_bytes(), &addr)
                            .map_err(|error| error.to_string())?;
                    }
                    _ => match command.parse::<u16>() {
                        Ok(amount_of_instructions) => {
                            virtual_machine.next_instructions(amount_of_instructions);
                            let response =
                                format!("executed {} instructions", amount_of_instructions);
                            socket
                                .send_to_addr(response.as_bytes(), &addr)
                                .map_err(|error| error.to_string())?;
                        }
                        Err(_) => {
                            socket
                                .send_to_addr("Invalid Command".as_bytes(), &addr)
                                .map_err(|error| error.to_string())?;
                        }
                    },
                }
            }
            Err(_) => {
                socket
                    .send_to("Couldn't receive error".as_bytes(), CLIENT_PATH)
                    .map_err(|error| error.to_string())?;
            }
        }
    }
}

fn print_instructions_for_interactive_console() {
    println!("Starting interactive console");
    println!("Instructions: ");
    println!("<r> to print the state of the registers at the moment");
    println!("<an instruction in binary> to instantly execute that instruction");
}

pub fn execute_vm_in_interactive_mode() -> Result<(), String> {
    let program_counter_start = 0x3000;
    let mut virtual_machine = LC3VirtualMachine::new(program_counter_start);
    print_instructions_for_interactive_console();
    for line in stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        match line.as_str() {
            "r" => {
                let registers = virtual_machine.state_of_registers();
                let registers = registers.split(STREAM_DATA_SEPARATOR);
                for register in registers {
                    println!("{}", register);
                }
            }
            _ => match u16::from_str_radix(&line, 2) {
                Ok(instruction) => {
                    virtual_machine.decode_instruction(instruction);
                    let instruction = format!("{instruction:#018b}",);
                    println!("instruction proccess: {}", instruction);
                }
                Err(_) => println!("Wrong instruction format"),
            },
        }
    }
    Ok(())
}

fn receive_file(path: impl AsRef<Path>) -> Result<BufReader<File>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
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
