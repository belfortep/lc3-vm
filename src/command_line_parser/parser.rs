use std::{fs::File, io::BufReader};

use byteorder::{BigEndian, ReadBytesExt};
use clap::{arg, ArgGroup, ArgMatches, Command};

use crate::virtual_machine::lc3_virtual_machine::LC3VirtualMachine;

pub fn receive_command_line_arguments() -> Result<ArgMatches, String> {
    let args = Command::new(" Conway's game of life")
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

pub fn receive_file(arg: String) -> Result<BufReader<File>, String> {
    let file = File::open(arg).map_err(|error| error.to_string())?;
    let file_reader = BufReader::new(file);
    Ok(file_reader)
}

pub fn load_reader_file_to_vm_memory(
    mut reader: BufReader<File>,
) -> Result<LC3VirtualMachine, String> {
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
