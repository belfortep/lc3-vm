use std::io::{Read, Write};

use super::lc3_virtual_machine::{LC3VirtualMachine, Register, Trap};

pub enum Instruction {
    BR = 0,
    ADD,
    LD,
    ST,
    JSR,
    AND,
    LDR,
    STR,
    RTI,
    NOT,
    LDI,
    STI,
    JMP,
    RES,
    LEA,
    TRAP,
}

impl From<u16> for Instruction {
    fn from(value: u16) -> Self {
        match value {
            0 => Instruction::BR,
            1 => Instruction::ADD,
            2 => Instruction::LD,
            3 => Instruction::ST,
            4 => Instruction::JSR,
            5 => Instruction::AND,
            6 => Instruction::LDR,
            7 => Instruction::STR,
            8 => Instruction::RTI,
            9 => Instruction::NOT,
            10 => Instruction::LDI,
            11 => Instruction::STI,
            12 => Instruction::JMP,
            13 => Instruction::RES,
            14 => Instruction::LEA,
            15 => Instruction::TRAP,
            _ => panic!("Wrong Instruction code"),
        }
    }
}

impl Instruction {
    fn sign_extend(mut value_to_extend: u16, ammount_of_bits: u16) -> u16 {
        if (value_to_extend >> (ammount_of_bits - 1) & 0b1) == 1 {
            value_to_extend |= 0xFFFF << ammount_of_bits;
        }
        value_to_extend
    }

    fn add(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let source_one_register = (instruction >> 6) & 0b111;
        let inmediate_return_flag = (instruction >> 5) & 0b1;

        if inmediate_return_flag == 1 {
            let inmediate_value = Self::sign_extend(instruction & 0b11111, 5);

            let new_register_value =
                inmediate_value + virtual_machine.read_register(source_one_register as usize);

            virtual_machine.update_register(destination_register, new_register_value);
        } else {
            let source_two_register = instruction & 0b111;
            let new_register_value = virtual_machine.read_register(source_one_register as usize)
                + virtual_machine.read_register(source_two_register as usize);

            virtual_machine.update_register(destination_register, new_register_value);
        }

        virtual_machine.update_flags(destination_register);
    }

    fn load(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);

        let new_register_value = virtual_machine.memory_read(
            virtual_machine.read_register(Register::PROGRAM_COUNTER) + programm_counter_offset,
        );

        virtual_machine.update_register(destination_register, new_register_value);
        virtual_machine.update_flags(destination_register);
    }

    fn load_indirect(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);

        let memory_address = virtual_machine.memory_read(
            virtual_machine.read_register(Register::PROGRAM_COUNTER) + programm_counter_offset,
        );
        let new_register_value = virtual_machine.memory_read(memory_address);
        virtual_machine.update_register(destination_register, new_register_value);

        virtual_machine.update_flags(destination_register);
    }

    fn not(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let source_register = (instruction >> 6) & 0b111;
        let new_register_value = !virtual_machine.read_register(source_register as usize);
        virtual_machine.update_register(destination_register, new_register_value);
    }

    fn branch(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let conditions_flag = (instruction >> 9) & 0b111;
        if (conditions_flag & virtual_machine.read_register(Register::CONDITION_FLAG)) != 0 {
            let new_register_value =
                virtual_machine.read_register(Register::PROGRAM_COUNTER) + programm_counter_offset;
            virtual_machine.update_register(Register::PROGRAM_COUNTER as u16, new_register_value);
        }
    }

    fn and(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let source_one_register = (instruction >> 6) & 0b111;
        let inmediate_return_flag = (instruction >> 5) & 0b1;

        if inmediate_return_flag == 1 {
            let inmediate_value = Self::sign_extend(instruction & 0b11111, 5);
            let new_register_value =
                virtual_machine.read_register(source_one_register as usize) & inmediate_value;
            virtual_machine.update_register(destination_register, new_register_value);
        } else {
            let source_two_register = instruction & 0b111;
            let new_register_value = virtual_machine.read_register(source_one_register as usize)
                & virtual_machine.read_register(source_two_register as usize);
            virtual_machine.update_register(destination_register, new_register_value);
        }
        virtual_machine.update_flags(destination_register);
    }

    fn trap(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        virtual_machine.update_register(
            Register::R7 as u16,
            virtual_machine.read_register(Register::PROGRAM_COUNTER),
        );

        let trap = instruction & 0b11111111;

        match trap {
            Trap::GETC => {
                let mut buffer = [0; 1];
                std::io::stdin()
                    .read_exact(&mut buffer)
                    .expect("Couldn't read from stdin");
                virtual_machine.update_register(Register::R0 as u16, buffer[0] as u16);
            }
            Trap::HALT => {
                std::process::exit(-1);
            }
            Trap::IN => {
                println!("Enter a character: ");
                let char = std::io::stdin()
                    .bytes()
                    .next()
                    .and_then(|read_result| read_result.ok())
                    .map(|char| char as u16)
                    .unwrap();
                virtual_machine.update_register(Register::R0 as u16, char);
            }
            Trap::OUT => {
                print!(
                    "{}",
                    (virtual_machine.read_register(Register::R0) as u8) as char
                );
            }
            Trap::PUTS => {
                let mut read_index = virtual_machine.read_register(Register::R0);
                let mut char = virtual_machine.memory_read(read_index);
                while char != 0 {
                    print!("{}", (char as u8) as char);
                    read_index += 1;
                    char = virtual_machine.memory_read(read_index);
                }
                std::io::stdout().flush().expect("Couldn't flush");
            }
            Trap::PUTSP => {
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

            _ => panic!("Wrong trap directive"),
        }
    }

    fn load_base_offset(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let base_register = (instruction >> 6) & 0b111;
        let offset = Self::sign_extend(instruction & 0b111111, 6);

        let register_value = virtual_machine.read_register(base_register as usize);
        let new_register_value = virtual_machine.memory_read(register_value + offset);
        virtual_machine.update_register(destination_register, new_register_value);

        virtual_machine.update_flags(destination_register);
    }

    fn load_effective_address(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let new_register_value =
            virtual_machine.read_register(Register::PROGRAM_COUNTER) + programm_counter_offset;
        virtual_machine.update_register(destination_register, new_register_value);

        virtual_machine.update_flags(destination_register);
    }

    fn jump(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let base_register = (instruction >> 6) & 0b111;
        let new_register_value = virtual_machine.read_register(base_register as usize);
        virtual_machine.update_register(Register::PROGRAM_COUNTER as u16, new_register_value);
    }

    fn jump_to_subroutine(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        virtual_machine.update_register(
            Register::R7 as u16,
            virtual_machine.read_register(Register::PROGRAM_COUNTER),
        );

        let offset_flag = (instruction >> 11) & 0b1;

        if offset_flag == 1 {
            let programm_counter_offset = Self::sign_extend(instruction & 0b11111111111, 11);
            let new_register_value =
                virtual_machine.read_register(Register::PROGRAM_COUNTER) + programm_counter_offset;

            virtual_machine.update_register(Register::PROGRAM_COUNTER as u16, new_register_value);
        } else {
            let base_register = (instruction >> 6) & 0b111;
            virtual_machine.update_register(
                Register::PROGRAM_COUNTER as u16,
                virtual_machine.read_register(base_register as usize),
            );
        }
    }

    fn store(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let source_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let value_to_write = virtual_machine.read_register(source_register as usize);
        let memory_address =
            virtual_machine.read_register(Register::PROGRAM_COUNTER) + programm_counter_offset;
        virtual_machine.memory_write(memory_address, value_to_write);
    }

    fn store_indirect(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let source_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let value_to_write = virtual_machine.read_register(source_register as usize);

        let memory_address = virtual_machine.memory_read(
            virtual_machine.read_register(Register::PROGRAM_COUNTER) + programm_counter_offset,
        );

        let destination_address = virtual_machine.memory_read(memory_address);

        virtual_machine.memory_write(destination_address, value_to_write);
    }

    fn store_base_offset(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        let source_register = (instruction >> 9) & 0b111;
        let base_register = (instruction >> 6) & 0b111;
        let offset = Self::sign_extend(instruction & 0b111111, 6);

        let value_to_write = virtual_machine.read_register(source_register as usize);
        let base_register_address = virtual_machine.read_register(base_register as usize);

        virtual_machine.memory_write(offset + base_register_address, value_to_write)
    }

    pub fn execute_instruction(&self, virtual_machine: &mut LC3VirtualMachine, instruction: u16) {
        match self {
            Instruction::BR => self.branch(virtual_machine, instruction),
            Instruction::ADD => self.add(virtual_machine, instruction),
            Instruction::LD => self.load(virtual_machine, instruction),
            Instruction::ST => self.store(virtual_machine, instruction),
            Instruction::JSR => self.jump_to_subroutine(virtual_machine, instruction),
            Instruction::AND => self.and(virtual_machine, instruction),
            Instruction::LDR => self.load_base_offset(virtual_machine, instruction),
            Instruction::STR => self.store_base_offset(virtual_machine, instruction),
            Instruction::NOT => self.not(virtual_machine, instruction),
            Instruction::LDI => self.load_indirect(virtual_machine, instruction),
            Instruction::STI => self.store_indirect(virtual_machine, instruction),
            Instruction::JMP => self.jump(virtual_machine, instruction),
            Instruction::LEA => self.load_effective_address(virtual_machine, instruction),
            Instruction::TRAP => self.trap(virtual_machine, instruction),
            Instruction::RTI => panic!("This opcode is not supported"),
            Instruction::RES => panic!("This opcode is not supported"),
        }
    }
}
