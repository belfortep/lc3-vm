pub enum Instruction {
    BR = 0,
    ADD,
    LD,
    ST,
    JSR,
    AND,
    NOT = 0b1001,
    LDI = 0b1010,
}

pub enum Flags {
    ZERO,
    NEGATIVE,
    POSITIVE,
}
pub enum Register {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    ProgramCounter,
    ConditionFlag,
    AmmountOfRegisters,
}

pub struct LC3VirtualMachine {
    registers: Vec<u16>,
    memory: Vec<u16>,
}

impl LC3VirtualMachine {
    pub fn new() -> Self {
        Self {
            registers: vec![0; Register::AmmountOfRegisters as usize],
            memory: vec![0; 1 << 16],
        }
    }

    fn sign_extend(mut value_to_extend: u16, ammount_of_bits: u16) -> u16 {
        if (value_to_extend >> (ammount_of_bits - 1) & 0b1) == 1 {
            value_to_extend |= 0xFFFF << ammount_of_bits;
        }
        value_to_extend
    }

    fn add_instruction(&mut self, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let source_one_register = (instruction >> 6) & 0b111;
        let inmediate_return_flag = (instruction >> 5) & 0b1;

        if inmediate_return_flag == 1 {
            let inmediate_value = Self::sign_extend(instruction & 0b00011111, 5);
            self.registers[destination_register as usize] =
                self.registers[source_one_register as usize] + inmediate_value;
        } else {
            let source_two_register = instruction & 0b111;
            self.registers[destination_register as usize] = self.registers
                [source_one_register as usize]
                + self.registers[source_two_register as usize];
        }

        self.update_flags(destination_register);
    }

    fn and_instruction(&mut self, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let source_one_register = (instruction >> 6) & 0b111;
        let inmediate_return_flag = (instruction >> 5) & 0b1;

        if inmediate_return_flag == 1 {
            let inmediate_value = Self::sign_extend(instruction & 0b00011111, 5);
            self.registers[destination_register as usize] =
                self.registers[source_one_register as usize] & inmediate_value;
        } else {
            let source_two_register = instruction & 0b111;
            self.registers[destination_register as usize] = self.registers
                [source_one_register as usize]
                & self.registers[source_two_register as usize];
        }
        self.update_flags(destination_register);
    }

    fn not_instruction(&mut self, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let source_register = (instruction >> 6) & 0b111;

        self.registers[destination_register as usize] = !self.registers[source_register as usize];
    }

    fn branch_instruction(&mut self, instruction: u16) {
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let negative_flag = instruction >> 11 & 0b1;
        let zero_flag = instruction >> 10 & 0b1;

        if (negative_flag == 1)
            && (self.registers[Register::ConditionFlag as usize] == Flags::NEGATIVE as u16)
        {
            self.registers[Register::ProgramCounter as usize] += programm_counter_offset;
        } else if (zero_flag == 1)
            && (self.registers[Register::ConditionFlag as usize] == Flags::ZERO as u16)
        {
            self.registers[Register::ProgramCounter as usize] += programm_counter_offset;
        } else {
            self.registers[Register::ProgramCounter as usize] += programm_counter_offset;
        }
    }
    fn memory_read(&mut self, memory_address: u16) -> u16 {
        self.memory[memory_address as usize]
    }

    fn memory_write(&mut self, memory_address: u16, value_to_write: u16) {
        self.memory[memory_address as usize] = value_to_write;
    }

    fn load_indirect(&mut self, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);

        let memory_address = self.memory_read(
            self.registers[Register::ProgramCounter as usize] + programm_counter_offset,
        );
        self.registers[destination_register as usize] = self.memory_read(memory_address);

        self.update_flags(destination_register);
    }

    fn store(&mut self, instruction: u16) {
        let source_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let value_to_write = self.registers[source_register as usize];
        let memory_address =
            self.registers[Register::ProgramCounter as usize] + programm_counter_offset;
        self.memory_write(memory_address, value_to_write);
    }

    pub fn process_input(&mut self, instruction: u16) {
        let opcode = instruction >> 12;
        match opcode {
            opcode if opcode == Instruction::ADD as u16 => self.add_instruction(instruction),
            opcode if opcode == Instruction::AND as u16 => self.and_instruction(instruction),
            opcode if opcode == Instruction::NOT as u16 => self.not_instruction(instruction),
            opcode if opcode == Instruction::BR as u16 => self.branch_instruction(instruction),
            opcode if opcode == Instruction::LDI as u16 => self.load_indirect(instruction),
            opcode if opcode == Instruction::ST as u16 => self.store(instruction),
            _ => {}
        }
    }
    pub fn read_register(&self, register: Register) -> u16 {
        self.registers[register as usize]
    }

    fn update_flags(&mut self, register: u16) {
        let condition_flag_register = Register::ConditionFlag as usize;
        let register = register as usize;
        if self.registers[register] == 0 {
            self.registers[condition_flag_register] = Flags::ZERO as u16;
        } else if self.registers[register] >> 15 == 1 {
            self.registers[condition_flag_register] = Flags::NEGATIVE as u16;
        } else {
            self.registers[condition_flag_register] = Flags::POSITIVE as u16;
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::virtual_machine::lc3_virtual_machine::Flags;

    use super::LC3VirtualMachine;

    #[test]
    fn can_add_two_numbers_in_same_register() {
        let mut virtual_machine = LC3VirtualMachine::new();
        let add_one_to_register_zero = 0b0001000000100001;
        virtual_machine.process_input(add_one_to_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 1);
    }

    #[test]
    fn can_add_two_numbers_in_differents_registers() {
        let mut virtual_machine = LC3VirtualMachine::new();
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
        let mut virtual_machine = LC3VirtualMachine::new();
        let add_max_inmediate_value_to_register_zero = 0b0001000000111111;
        let and_five_to_register_zero = 0b0101000000100101;
        virtual_machine.process_input(add_max_inmediate_value_to_register_zero);
        virtual_machine.process_input(and_five_to_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 0b00101);
    }

    #[test]
    fn can_and_two_numbers_in_differents_registers() {
        let mut virtual_machine = LC3VirtualMachine::new();
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
        let mut virtual_machine = LC3VirtualMachine::new();
        let add_five_to_regiser_zero = 0b0001000000100101;
        virtual_machine.process_input(add_five_to_regiser_zero);
        let negate_register_zero = 0b1001000000111111;
        virtual_machine.process_input(negate_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 0b1111111111111010);
    }

    #[test]
    fn can_branch_if_positive_flag() {
        let mut virtual_machine = LC3VirtualMachine::new();
        let add_one_to_regiser_zero = 0b0001000000100001;
        virtual_machine.process_input(add_one_to_regiser_zero);
        let branch_positive_flag = 0b0000001000000010;
        virtual_machine.process_input(branch_positive_flag);

        let result = virtual_machine.read_register(super::Register::ConditionFlag);
        assert_eq!(result, Flags::POSITIVE as u16);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_branch_if_negative_flag() {
        let mut virtual_machine = LC3VirtualMachine::new();
        let add_negative_number_to_regiser_zero = 0b0001000000110001;
        virtual_machine.process_input(add_negative_number_to_regiser_zero);
        let branch_negative_flag = 0b0000100000000010;
        virtual_machine.process_input(branch_negative_flag);

        let result = virtual_machine.read_register(super::Register::ConditionFlag);
        assert_eq!(result, Flags::NEGATIVE as u16);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_branch_if_zero_flag() {
        let mut virtual_machine = LC3VirtualMachine::new();
        let add_zero_to_regiser_zero = 0b0001000000100000;
        virtual_machine.process_input(add_zero_to_regiser_zero);
        let branch_positive_flag = 0b0000010000000010;
        virtual_machine.process_input(branch_positive_flag);

        let result = virtual_machine.read_register(super::Register::ConditionFlag);
        assert_eq!(result, Flags::ZERO as u16);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b10);
    }
}
