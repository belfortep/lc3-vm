use std::io::Read;

use super::{
    instructions::*,
    register::{Register, Registers},
    trap::Trap,
};

enum MemoryMappedRegister {
    KeyBoardStatusRegister = 0xFE00,
    KeyBoardDataRegister = 0xFE02,
}

pub struct LC3VirtualMachine {
    registers: Registers,
    memory: Vec<u16>,
}

impl LC3VirtualMachine {
    pub fn new(program_counter_start: u16) -> Self {
        Self {
            registers: Registers::new(program_counter_start),
            memory: vec![0; 1 << 16],
        }
    }

    pub fn read_register(&self, source_register: Register) -> u16 {
        self.registers.read_register(source_register)
    }

    pub fn update_register(&mut self, destination_register: Register, new_register_value: u16) {
        self.registers
            .update_register(destination_register, new_register_value)
    }

    pub fn update_flags(&mut self, register: Register) {
        self.registers.update_flags(register)
    }

    fn receive_keyboard_input(&mut self) {
        let mut buffer = [0; 1];
        std::io::stdin()
            .read_exact(&mut buffer)
            .expect("Couldn't read from stdin");
        if buffer[0] != 0 {
            self.memory_write(MemoryMappedRegister::KeyBoardStatusRegister as u16, 1 << 15);
            self.memory_write(
                MemoryMappedRegister::KeyBoardDataRegister as u16,
                buffer[0] as u16,
            );
        } else {
            self.memory_write(MemoryMappedRegister::KeyBoardStatusRegister as u16, 0)
        }
    }

    pub fn memory_read(&mut self, memory_address: u16) -> u16 {
        if memory_address == MemoryMappedRegister::KeyBoardStatusRegister as u16 {
            self.receive_keyboard_input();
        }
        self.memory[memory_address as usize]
    }

    pub fn memory_write(&mut self, memory_address: u16, value_to_write: u16) {
        self.memory[memory_address as usize] = value_to_write;
    }

    pub fn decode_instruction(&mut self, instruction: u16) {
        let instruction_opcode = instruction >> 12;
        match instruction_opcode {
            BR => {
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                let conditions_flag = (instruction >> 9) & 0b111;
                branch(self, program_counter_offset, conditions_flag)
            }
            ADD => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let source_one_register = Register::from((instruction >> 6) & 0b111);
                let inmediate_return_flag = (instruction >> 5) & 0b1;
                if inmediate_return_flag == 1 {
                    let inmediate_value = Self::sign_extend(instruction & 0b11111, 5);
                    add_inmediate(
                        self,
                        destination_register,
                        source_one_register,
                        inmediate_value,
                    )
                } else {
                    let source_two_register = Register::from(instruction & 0b111);
                    add(
                        self,
                        destination_register,
                        source_one_register,
                        source_two_register,
                    )
                }
            }
            LD => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                load(self, destination_register, program_counter_offset)
            }
            ST => {
                let source_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                store(self, source_register, program_counter_offset)
            }
            JSR => {
                let offset_flag = (instruction >> 11) & 0b1;
                if offset_flag == 1 {
                    let program_counter_offset = Self::sign_extend(instruction & 0b11111111111, 11);
                    jump_to_subroutine_with_offset(self, program_counter_offset)
                } else {
                    let base_register = Register::from((instruction >> 6) & 0b111);
                    jump_to_subroutine(self, base_register)
                }
            }
            AND => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let source_one_register = Register::from((instruction >> 6) & 0b111);
                let inmediate_return_flag = (instruction >> 5) & 0b1;
                if inmediate_return_flag == 1 {
                    let inmediate_value = Self::sign_extend(instruction & 0b11111, 5);
                    and_inmediate(
                        self,
                        destination_register,
                        source_one_register,
                        inmediate_value,
                    )
                } else {
                    let source_two_register = Register::from(instruction & 0b111);
                    and(
                        self,
                        destination_register,
                        source_one_register,
                        source_two_register,
                    )
                }
            }
            LDR => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let base_register = Register::from((instruction >> 6) & 0b111);
                let offset = Self::sign_extend(instruction & 0b111111, 6);
                load_base_offset(self, destination_register, base_register, offset)
            }
            STR => {
                let source_register = Register::from((instruction >> 9) & 0b111);
                let base_register = Register::from((instruction >> 6) & 0b111);
                let offset = Self::sign_extend(instruction & 0b111111, 6);
                store_base_offset(self, source_register, base_register, offset)
            }
            NOT => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let source_register = Register::from((instruction >> 6) & 0b111);
                not(self, destination_register, source_register)
            }
            LDI => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                load_indirect(self, destination_register, program_counter_offset)
            }
            STI => {
                let source_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                store_indirect(self, source_register, program_counter_offset)
            }
            JMP => {
                let base_register = Register::from((instruction >> 6) & 0b111);
                jump(self, base_register)
            }
            LEA => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                load_effective_address(self, destination_register, program_counter_offset)
            }
            TRAP => {
                let trap = Trap::from(instruction & 0b11111111);
                trap_instruction(self, trap)
            }
            RTI => panic!("This opcode is not supported"),
            RES => panic!("This opcode is not supported"),
            _ => unreachable!("Getting the last 4 bits of a u16 should never reach here"),
        }
    }

    pub fn next_instruction(&mut self) {
        let instruction = self.memory_read(self.read_register(Register::ProgramCounter));

        let new_register_value = self.read_register(Register::ProgramCounter) + 1;
        self.update_register(Register::ProgramCounter, new_register_value);

        self.decode_instruction(instruction);
    }

    pub fn next_instructions(&mut self, steps: u16) {
        for _ in 0..steps {
            self.next_instruction();
        }
    }

    pub fn state_of_registers(&mut self) -> String {
        self.registers.to_string()
    }

    fn sign_extend(mut value_to_extend: u16, amount_of_bits: u16) -> u16 {
        if (value_to_extend >> (amount_of_bits - 1) & 0b1) == 1 {
            value_to_extend |= 0xFFFF << amount_of_bits;
        }
        value_to_extend
    }
}
