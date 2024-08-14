use std::io::Read;

use super::{
    instructions::*,
    register::{Register, AMOUNT_OF_REGISTERS},
    trap::Trap,
};

pub enum Flag {
    POSITIVE = 1 << 0,
    ZERO = 1 << 1,
    NEGATIVE = 1 << 2,
}

enum MemoryMappedRegister {
    KeyBoardStatusRegister = 0xFE00,
    KeyBoardDataRegister = 0xFE02,
}

pub struct LC3VirtualMachine {
    registers: Vec<u16>,
    memory: Vec<u16>,
}

impl LC3VirtualMachine {
    pub fn new(program_counter_start: u16) -> Self {
        let mut registers = vec![0; AMOUNT_OF_REGISTERS];
        registers[Register::ProgramCounter as usize] = program_counter_start;
        Self {
            registers,
            memory: vec![0; 1 << 16],
        }
    }

    pub fn read_register(&self, source_register: Register) -> u16 {
        self.registers[source_register as usize]
    }

    pub fn update_register(&mut self, destination_register: Register, new_register_value: u16) {
        self.registers[destination_register as usize] = new_register_value;
    }

    pub fn update_flags(&mut self, register: Register) {
        let register = register as usize;
        if self.registers[register] == 0 {
            self.update_register(Register::ConditionFlag, Flag::ZERO as u16)
        } else if (self.registers[register] >> 15) != 0 {
            self.update_register(Register::ConditionFlag, Flag::NEGATIVE as u16)
        } else {
            self.update_register(Register::ConditionFlag, Flag::POSITIVE as u16)
        }
    }

    fn receive_keyboard_input(&mut self) {
        let mut buffer = [0; 1];
        std::io::stdin().read_exact(&mut buffer).unwrap();
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
        let instruction_opcode = Instruction::from(instruction >> 12);
        match instruction_opcode {
            Instruction::BR => {
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                let conditions_flag = (instruction >> 9) & 0b111;
                branch(self, program_counter_offset, conditions_flag)
            }
            Instruction::ADD => {
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
            Instruction::LD => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                load(self, destination_register, program_counter_offset)
            }
            Instruction::ST => {
                let source_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                store(self, source_register, program_counter_offset)
            }
            Instruction::JSR => {
                let offset_flag = (instruction >> 11) & 0b1;
                if offset_flag == 1 {
                    let program_counter_offset = Self::sign_extend(instruction & 0b11111111111, 11);
                    jump_to_subroutine_with_offset(self, program_counter_offset)
                } else {
                    let base_register = Register::from((instruction >> 6) & 0b111);
                    jump_to_subroutine(self, base_register)
                }
            }
            Instruction::AND => {
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
            Instruction::LDR => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let base_register = Register::from((instruction >> 6) & 0b111);
                let offset = Self::sign_extend(instruction & 0b111111, 6);
                load_base_offset(self, destination_register, base_register, offset)
            }
            Instruction::STR => {
                let source_register = Register::from((instruction >> 9) & 0b111);
                let base_register = Register::from((instruction >> 6) & 0b111);
                let offset = Self::sign_extend(instruction & 0b111111, 6);
                store_base_offset(self, source_register, base_register, offset)
            }
            Instruction::NOT => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let source_register = Register::from((instruction >> 6) & 0b111);
                not(self, destination_register, source_register)
            }
            Instruction::LDI => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                load_indirect(self, destination_register, program_counter_offset)
            }
            Instruction::STI => {
                let source_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                store_indirect(self, source_register, program_counter_offset)
            }
            Instruction::JMP => {
                let base_register = Register::from((instruction >> 6) & 0b111);
                jump(self, base_register)
            }
            Instruction::LEA => {
                let destination_register = Register::from((instruction >> 9) & 0b111);
                let program_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
                load_effective_address(self, destination_register, program_counter_offset)
            }
            Instruction::TRAP => {
                let trap = Trap::from(instruction & 0b11111111);
                trap_instruction(self, trap)
            }
            Instruction::RTI => panic!("This opcode is not supported"),
            Instruction::RES => panic!("This opcode is not supported"),
        }
    }

    fn sign_extend(mut value_to_extend: u16, ammount_of_bits: u16) -> u16 {
        if (value_to_extend >> (ammount_of_bits - 1) & 0b1) == 1 {
            value_to_extend |= 0xFFFF << ammount_of_bits;
        }
        value_to_extend
    }
}
