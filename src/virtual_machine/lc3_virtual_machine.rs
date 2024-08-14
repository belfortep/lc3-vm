use std::io::Read;

use super::{instructions::*, register::Register, trap::Trap};

const AMOUNT_OF_REGISTERS: usize = 10;
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
        let mut registers = vec![0; AMMOUNT_OF_REGISTERS];
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

    pub fn process_input(&mut self, instruction: u16) {
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

#[cfg(test)]
pub mod test {
    use crate::virtual_machine::lc3_virtual_machine::Flag;

    use super::LC3VirtualMachine;

    #[test]
    fn can_add_two_numbers_in_same_register() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_one_to_register_zero = 0b0001_000_000_1_00001;
        virtual_machine.process_input(add_one_to_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 1);
    }

    #[test]
    fn can_add_two_numbers_in_different_registers() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_one_to_register_zero = 0b0001_000_000_1_00001;
        let add_one_to_register_one = 0b0001_001_001_1_00001;
        virtual_machine.process_input(add_one_to_register_zero);
        virtual_machine.process_input(add_one_to_register_one);
        let add_register_zero_and_one_in_register_two = 0b0001_010_000_0_00001;
        virtual_machine.process_input(add_register_zero_and_one_in_register_two);
        let result = virtual_machine.read_register(super::Register::R2);

        assert_eq!(result, 2);
    }

    #[test]
    fn can_and_two_numbers_in_same_register() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_max_inmediate_value_to_register_zero = 0b0001_000_000_1_11111;
        let and_five_to_register_zero = 0b0101_000_000_1_00101;
        virtual_machine.process_input(add_max_inmediate_value_to_register_zero);
        virtual_machine.process_input(and_five_to_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 0b00101);
    }

    #[test]
    fn can_and_two_numbers_in_differents_registers() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_max_inmediate_value_to_register_zero = 0b0001_000_000_111111;
        let add_five_to_register_one = 0b0001_001_001_1_00101;
        virtual_machine.process_input(add_max_inmediate_value_to_register_zero);
        virtual_machine.process_input(add_five_to_register_one);
        let and_register_zero_and_one_in_register_two = 0b0101_010_000_0_00001;
        virtual_machine.process_input(and_register_zero_and_one_in_register_two);
        let result = virtual_machine.read_register(super::Register::R2);

        assert_eq!(result, 0b00101);
    }

    #[test]
    fn can_negate_the_values_of_two_registers() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_register_zero = 0b0001_000_000_1_00101;
        virtual_machine.process_input(add_five_to_register_zero);
        let negate_register_zero = 0b1001_000_000_1_11111;
        virtual_machine.process_input(negate_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 0b1111111111111010);
    }

    #[test]
    fn can_branch_if_positive_flag() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_one_to_register_zero = 0b0001_000_000_1_00001;
        virtual_machine.process_input(add_one_to_register_zero);
        let branch_positive_flag = 0b0000_0_0_1_000000010;
        virtual_machine.process_input(branch_positive_flag);

        let result = virtual_machine.read_register(super::Register::ConditionFlag);
        assert_eq!(result, Flag::POSITIVE as u16);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_branch_if_negative_flag() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_negative_number_to_register_zero = 0b0001_000_000_110001;
        virtual_machine.process_input(add_negative_number_to_register_zero);
        let branch_negative_flag = 0b0000_1_0_0_000000010;
        virtual_machine.process_input(branch_negative_flag);

        let result = virtual_machine.read_register(super::Register::ConditionFlag);
        assert_eq!(result, Flag::NEGATIVE as u16);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_branch_if_zero_flag() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_zero_to_register_zero = 0b0001_000_000_100000;
        virtual_machine.process_input(add_zero_to_register_zero);
        let branch_zero_flag = 0b0000_0_1_0_000000010;
        virtual_machine.process_input(branch_zero_flag);

        let result = virtual_machine.read_register(super::Register::ConditionFlag);
        assert_eq!(result, Flag::ZERO as u16);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_store_and_load_from_memory() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_register_zero = 0b0001_000_000_1_00101;
        virtual_machine.process_input(add_five_to_register_zero);
        let store_register_zero_value_to_memory = 0b0011_000_000000001;
        virtual_machine.process_input(store_register_zero_value_to_memory);
        let load_value_from_memory_to_register_one = 0b0010_001_000000001;
        virtual_machine.process_input(load_value_from_memory_to_register_one);

        let result = virtual_machine.read_register(super::Register::R1);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_jump_to_subroutine_and_return_with_register_seven() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let jump_to_position_four = 0b0100_1_00000000100;
        virtual_machine.process_input(jump_to_position_four);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b100);
        let jump_to_register_zero = 0b0100_0_00_000_000000;
        virtual_machine.process_input(jump_to_register_zero);
        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0);
    }

    #[test]
    fn can_store_and_load_from_memory_with_base_and_offset() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_register_zero = 0b0001_000_000_1_00101;
        virtual_machine.process_input(add_five_to_register_zero);
        let add_five_to_register_one = 0b0001_001_001_1_00101;
        virtual_machine.process_input(add_five_to_register_one);

        let store_register_zero_value_to_memory_from_register_one_and_one_offset =
            0b0111_000_001_000001;
        virtual_machine
            .process_input(store_register_zero_value_to_memory_from_register_one_and_one_offset);
        let load_value_from_memory_from_register_one_and_one_offset_to_register_two =
            0b0110_010_001_000001;
        virtual_machine
            .process_input(load_value_from_memory_from_register_one_and_one_offset_to_register_two);

        let result = virtual_machine.read_register(super::Register::R2);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_unconditionally_jumps() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_register_zero = 0b0001_000_000_1_00101;
        virtual_machine.process_input(add_five_to_register_zero);
        let unconditionally_jump_to_register_zero_value = 0b1100_000_000_000000;
        virtual_machine.process_input(unconditionally_jump_to_register_zero_value);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_load_effective_address() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let load_effective_address_three_to_register_zero = 0b1110_000_000000011;
        virtual_machine.process_input(load_effective_address_three_to_register_zero);

        let result = virtual_machine.read_register(super::Register::R0);
        assert_eq!(result, 0b11);
    }
}
