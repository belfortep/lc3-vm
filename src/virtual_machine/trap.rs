use std::io::{Read, Write};

use crate::virtual_machine::register::Register;

use super::lc3_virtual_machine::LC3VirtualMachine;

pub enum Trap {
    GETC,
    OUT,
    PUTS,
    IN,
    PUTSP,
    HALT,
}

impl From<u16> for Trap {
    fn from(value: u16) -> Self {
        match value {
            0x20 => Trap::GETC,
            0x21 => Trap::OUT,
            0x22 => Trap::PUTS,
            0x23 => Trap::IN,
            0x24 => Trap::PUTSP,
            0x25 => Trap::HALT,
            _ => panic!("Wrong Trap code"),
        }
    }
}

impl Trap {
    fn getc(&self, virtual_machine: &mut LC3VirtualMachine) {
        let mut buffer = [0; 1];
        std::io::stdin()
            .read_exact(&mut buffer)
            .expect("Couldn't read from stdin");
        virtual_machine.update_register(Register::R0, buffer[0] as u16);
    }

    fn halt(&self) {
        std::process::exit(0);
    }

    fn in_trap(&self, virtual_machine: &mut LC3VirtualMachine) {
        println!("Enter a character: ");
        let char = std::io::stdin()
            .bytes()
            .next()
            .and_then(|read_result| read_result.ok())
            .map(|char| char as u16)
            .unwrap();
        virtual_machine.update_register(Register::R0, char);
    }

    fn out(&self, virtual_machine: &mut LC3VirtualMachine) {
        print!(
            "{}",
            (virtual_machine.read_register(Register::R0) as u8) as char
        );
    }

    fn puts(&self, virtual_machine: &mut LC3VirtualMachine) {
        let mut read_index = virtual_machine.read_register(Register::R0);
        let mut char = virtual_machine.memory_read(read_index);
        while char != 0 {
            print!("{}", (char as u8) as char);
            read_index += 1;
            char = virtual_machine.memory_read(read_index);
        }
        std::io::stdout().flush().expect("Couldn't flush");
    }

    fn putsp(&self, virtual_machine: &mut LC3VirtualMachine) {
        let mut read_index = virtual_machine.read_register(Register::R0);
        let mut char = virtual_machine.memory_read(read_index);
        while char != 0 {
            let first_char = char & 0b11111111;
            print!("{}", (first_char as u8) as char);
            let second_char = char >> 8;
            if second_char != 0 {
                print!("{}", (second_char as u8) as char);
            }
            read_index += 1;
            char = virtual_machine.memory_read(read_index);
        }
        std::io::stdout().flush().expect("Couldn't flush");
    }

    pub fn execute_trap(&self, virtual_machine: &mut LC3VirtualMachine) {
        match self {
            Trap::GETC => self.getc(virtual_machine),
            Trap::HALT => self.halt(),
            Trap::IN => self.in_trap(virtual_machine),
            Trap::OUT => self.out(virtual_machine),
            Trap::PUTS => self.puts(virtual_machine),
            Trap::PUTSP => self.putsp(virtual_machine),
        }
    }
}
