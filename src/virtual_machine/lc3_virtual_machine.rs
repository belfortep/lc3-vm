use std::io::{Read, Write};

use super::instructions::Instruction;

pub struct Trap;
impl Trap {
    pub const GETC: u16 = 0x20;
    pub const OUT: u16 = 0x21;
    pub const PUTS: u16 = 0x22;
    pub const IN: u16 = 0x23;
    pub const PUTSP: u16 = 0x24;
    pub const HALT: u16 = 0x25;
}

struct Flags;
impl Flags {
    pub const POSITIVE: u16 = 1 << 0;
    pub const ZERO: u16 = 1 << 1;
    pub const NEGATIVE: u16 = 1 << 2;
}

pub struct Register;
impl Register {
    pub const R0: usize = 0;
    pub const R1: usize = 1;
    pub const R2: usize = 2;
    pub const R3: usize = 3;
    pub const R4: usize = 4;
    pub const R5: usize = 5;
    pub const R6: usize = 6;
    pub const R7: usize = 7;
    pub const PROGRAM_COUNTER: usize = 8;
    pub const CONDITION_FLAG: usize = 9;
    pub const AMMOUNT_OF_REGISTERS: usize = 10;
}

pub struct LC3VirtualMachine {
    registers: Vec<u16>,
    memory: Vec<u16>,
}

impl LC3VirtualMachine {
    pub fn new(programm_counter_start: u16) -> Self {
        let mut registers = vec![0; Register::AMMOUNT_OF_REGISTERS];
        registers[Register::PROGRAM_COUNTER] = programm_counter_start;
        Self {
            registers,
            memory: vec![0; 1 << 16],
        }
    }

    pub fn memory_read(&mut self, memory_address: u16) -> u16 {
        self.memory[memory_address as usize]
    }

    pub fn memory_write(&mut self, memory_address: u16, value_to_write: u16) {
        self.memory[memory_address as usize] = value_to_write;
    }

    pub fn process_input(&mut self, instruction: u16) {
        let instruction_opcode = Instruction::from(instruction >> 12);
        instruction_opcode.execute_instruction(self, instruction);
    }
    pub fn read_register(&self, source_register: usize) -> u16 {
        if source_register > self.registers.len() {
            return 0;
        }
        self.registers[source_register]
    }

    pub fn update_register(&mut self, destination_register: u16, new_register_value: u16) {
        self.registers[destination_register as usize] = new_register_value;
    }

    pub fn update_flags(&mut self, register: u16) {
        let register = register as usize;
        if self.registers[register] == 0 {
            self.registers[Register::CONDITION_FLAG] = Flags::ZERO;
        } else if (self.registers[register] >> 15) != 0 {
            self.registers[Register::CONDITION_FLAG] = Flags::NEGATIVE;
        } else {
            self.registers[Register::CONDITION_FLAG] = Flags::POSITIVE;
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::virtual_machine::lc3_virtual_machine::Flags;

    use super::LC3VirtualMachine;

    #[test]
    fn can_add_two_numbers_in_same_register() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_one_to_register_zero = 0b0001000000100001;
        virtual_machine.process_input(add_one_to_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 1);
    }

    #[test]
    fn can_add_two_numbers_in_differents_registers() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_one_to_register_zero = 0b0001000000100001;
        let add_one_to_register_one = 0b0001001001100001;
        virtual_machine.process_input(add_one_to_register_zero);
        virtual_machine.process_input(add_one_to_register_one);
        let add_register_zero_and_one_in_register_two = 0b0001010000000001;
        virtual_machine.process_input(add_register_zero_and_one_in_register_two);
        let result = virtual_machine.read_register(super::Register::R2);

        assert_eq!(result, 2);
    }

    #[test]
    fn can_and_two_numbers_in_same_register() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_max_inmediate_value_to_register_zero = 0b0001000000111111;
        let and_five_to_register_zero = 0b0101000000100101;
        virtual_machine.process_input(add_max_inmediate_value_to_register_zero);
        virtual_machine.process_input(and_five_to_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 0b00101);
    }

    #[test]
    fn can_and_two_numbers_in_differents_registers() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_max_inmediate_value_to_register_zero = 0b0001000000111111;
        let add_five_to_regiser_one = 0b0001001001100101;
        virtual_machine.process_input(add_max_inmediate_value_to_register_zero);
        virtual_machine.process_input(add_five_to_regiser_one);
        let and_register_zero_and_one_in_register_two = 0b0101010000000001;
        virtual_machine.process_input(and_register_zero_and_one_in_register_two);
        let result = virtual_machine.read_register(super::Register::R2);

        assert_eq!(result, 0b00101);
    }

    #[test]
    fn can_negate_the_values_of_two_registers() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_regiser_zero = 0b0001000000100101;
        virtual_machine.process_input(add_five_to_regiser_zero);
        let negate_register_zero = 0b1001000000111111;
        virtual_machine.process_input(negate_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 0b1111111111111010);
    }

    #[test]
    fn can_branch_if_positive_flag() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_one_to_regiser_zero = 0b0001000000100001;
        virtual_machine.process_input(add_one_to_regiser_zero);
        let branch_positive_flag = 0b0000001000000010;
        virtual_machine.process_input(branch_positive_flag);

        let result = virtual_machine.read_register(super::Register::CONDITION_FLAG);
        assert_eq!(result, Flags::POSITIVE);

        let result = virtual_machine.read_register(super::Register::PROGRAM_COUNTER);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_branch_if_negative_flag() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_negative_number_to_regiser_zero = 0b0001000000110001;
        virtual_machine.process_input(add_negative_number_to_regiser_zero);
        let branch_negative_flag = 0b0000100000000010;
        virtual_machine.process_input(branch_negative_flag);

        let result = virtual_machine.read_register(super::Register::CONDITION_FLAG);
        assert_eq!(result, Flags::NEGATIVE);

        let result = virtual_machine.read_register(super::Register::PROGRAM_COUNTER);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_branch_if_zero_flag() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_zero_to_regiser_zero = 0b0001000000100000;
        virtual_machine.process_input(add_zero_to_regiser_zero);
        let branch_positive_flag = 0b0000010000000010;
        virtual_machine.process_input(branch_positive_flag);

        let result = virtual_machine.read_register(super::Register::CONDITION_FLAG);
        assert_eq!(result, Flags::ZERO);

        let result = virtual_machine.read_register(super::Register::PROGRAM_COUNTER);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_store_and_load_from_memory() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_regiser_zero = 0b0001000000100101;
        virtual_machine.process_input(add_five_to_regiser_zero);
        let store_register_zero_value_to_memory = 0b0011000000000001;
        virtual_machine.process_input(store_register_zero_value_to_memory);
        let load_value_from_memory_to_register_one = 0b0010001000000001;
        virtual_machine.process_input(load_value_from_memory_to_register_one);

        let result = virtual_machine.read_register(super::Register::R1);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_jump_to_subroutine_and_return_with_register_seven() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let jump_to_position_four = 0b0100100000000100;
        virtual_machine.process_input(jump_to_position_four);

        let result = virtual_machine.read_register(super::Register::PROGRAM_COUNTER);
        assert_eq!(result, 0b100);
        let jump_to_register_zero = 0b0100000000000000;
        virtual_machine.process_input(jump_to_register_zero);
        let result = virtual_machine.read_register(super::Register::PROGRAM_COUNTER);
        assert_eq!(result, 0);
    }

    #[test]
    fn can_store_and_load_from_memory_with_base_and_offset() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_regiser_zero = 0b0001000000100101;
        virtual_machine.process_input(add_five_to_regiser_zero);
        let add_five_to_regiser_one = 0b0001001001100101;
        virtual_machine.process_input(add_five_to_regiser_one);

        let store_register_zero_value_to_memory_from_register_one_and_one_offset =
            0b0111000001000001;
        virtual_machine
            .process_input(store_register_zero_value_to_memory_from_register_one_and_one_offset);
        let load_value_from_memory_from_register_one_and_one_offset_to_register_two =
            0b0110010001000001;
        virtual_machine
            .process_input(load_value_from_memory_from_register_one_and_one_offset_to_register_two);

        let result = virtual_machine.read_register(super::Register::R2);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_unconditionally_jumps() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_regiser_zero = 0b0001000000100101;
        virtual_machine.process_input(add_five_to_regiser_zero);
        let unconditionally_jump_to_register_zero_value = 0b1100000000000000;
        virtual_machine.process_input(unconditionally_jump_to_register_zero_value);

        let result = virtual_machine.read_register(super::Register::PROGRAM_COUNTER);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_load_effective_address() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let load_effective_address_three_to_register_zero = 0b1110000000000011;
        virtual_machine.process_input(load_effective_address_three_to_register_zero);

        let result = virtual_machine.read_register(super::Register::R0);
        assert_eq!(result, 0b11);
    }
}
