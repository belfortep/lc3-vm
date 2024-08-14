use std::{fs::File, io::BufReader};

use byteorder::{BigEndian, ReadBytesExt};
use clap::{arg, ArgGroup, ArgMatches, Command};

use crate::virtual_machine::lc3_virtual_machine::LC3VirtualMachine;

pub fn receive_command_line_arguments() -> Result<ArgMatches, String> {
    let args = Command::new("LC3 Virtual Machine")
        .arg(arg!(-f --file <FILE> "file to execute").required(false))
        .group(ArgGroup::new("run program").args(["file"]).required(false))
        .after_help("You need to pass a FILE to execute the program")
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
