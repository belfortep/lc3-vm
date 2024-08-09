use std::{
    io::{Read, Write},
    process::exit,
};

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

pub enum Trap {
    GETC = 0x20,
    OUT = 0x21,
    PUTS = 0x22,
    IN = 0x23,
    PUTSP = 0x24,
    HALT = 0x25,
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

    fn load(&mut self, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);

        let result_value = self.memory_read(
            self.registers[Register::ProgramCounter as usize] + programm_counter_offset,
        );

        self.registers[destination_register as usize] = result_value;
        self.update_flags(destination_register);
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

    fn load_base_offset(&mut self, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let base_register = (instruction >> 6) & 0b111;
        let offset = Self::sign_extend(instruction & 0b111111, 6);

        let register_value = self.registers[base_register as usize];
        let result_value = self.memory_read(register_value + offset);

        self.registers[destination_register as usize] = result_value;
        self.update_flags(destination_register);
    }

    fn load_effective_address(&mut self, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);

        self.registers[destination_register as usize] =
            self.registers[Register::ProgramCounter as usize] + programm_counter_offset;

        self.update_flags(destination_register);
    }

    fn jump(&mut self, instruction: u16) {
        let base_register = (instruction >> 6) & 0b111;
        self.registers[Register::ProgramCounter as usize] = self.registers[base_register as usize];
    }

    fn jump_to_subroutine(&mut self, instruction: u16) {
        self.registers[Register::R7 as usize] = self.registers[Register::ProgramCounter as usize];

        let offset_flag = (instruction >> 11) & 0b1;

        if offset_flag == 1 {
            let programm_counter_offset = Self::sign_extend(instruction & 0b11111111111, 11);
            let new_programm_counter_value =
                self.registers[Register::ProgramCounter as usize] + programm_counter_offset;

            self.registers[Register::ProgramCounter as usize] = new_programm_counter_value;
        } else {
            let base_register = (instruction >> 6) & 0b111;
            self.registers[Register::ProgramCounter as usize] =
                self.registers[base_register as usize];
        }
    }

    fn store(&mut self, instruction: u16) {
        let source_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let value_to_write = self.registers[source_register as usize];
        let memory_address =
            self.registers[Register::ProgramCounter as usize] + programm_counter_offset;
        self.memory_write(memory_address, value_to_write);
    }

    fn store_indirect(&mut self, instruction: u16) {
        let source_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let value_to_write = self.registers[source_register as usize];

        let memory_address = self.memory_read(
            self.registers[Register::ProgramCounter as usize] + programm_counter_offset,
        );
        let destination_address = self.memory_read(memory_address);

        self.memory_write(destination_address, value_to_write)
    }

    fn store_base_offset(&mut self, instruction: u16) {
        let source_register = (instruction >> 9) & 0b111;
        let base_register = (instruction >> 6) & 0b111;
        let offset = Self::sign_extend(instruction & 0b111111, 6);

        let value_to_write = self.registers[source_register as usize];
        let base_register_address = self.registers[base_register as usize];

        self.memory_write(offset + base_register_address, value_to_write)
    }

    fn trap(&mut self, instruction: u16) {
        self.registers[Register::R7 as usize] = self.registers[Register::ProgramCounter as usize];

        let trap = instruction & 0b11111111;

        match trap {
            trap if trap == Trap::GETC as u16 => {
                let mut buffer = [0; 1];
                std::io::stdin()
                    .read_exact(&mut buffer)
                    .expect("Couldn't read from stdin");
                self.registers[Register::R0 as usize] = buffer[0] as u16;
            }
            trap if trap == Trap::HALT as u16 => {
                std::process::exit(-1);
            }
            trap if trap == Trap::IN as u16 => {
                println!("Enter a character: ");
                let mut buffer = [0; 1];
                std::io::stdin()
                    .read_exact(&mut buffer)
                    .expect("Couldn't read from stdin");
                std::io::stdout()
                    .lock()
                    .write_all(&[buffer[0]])
                    .expect("Couldn't write to stdout");
                self.registers[Register::R0 as usize] = buffer[0] as u16;
                self.update_flags(Register::R0 as u16);
            }
            trap if trap == Trap::OUT as u16 => {
                let mut stdout = std::io::stdout().lock();
                stdout
                    .write_all(&[self.registers[Register::R0 as usize] as u8])
                    .expect("Couldn't write to stdout");
            }
            trap if trap == Trap::PUTS as u16 => {
                let mut character = self.memory[self.registers[Register::R0 as usize] as usize];
                let mut counter = 0;
                while character != 0 {
                    std::io::stdout()
                        .lock()
                        .write_all(&[character as u8])
                        .expect("Couldn't write to stdout");

                    character =
                        self.memory[self.registers[Register::R0 as usize] as usize + counter];
                    counter += 1;
                }
            }
            trap if trap == Trap::PUTSP as u16 => {
                let mut character = self.memory[self.registers[Register::R0 as usize] as usize];
                let mut counter = 0;
                while character != 0 {
                    let char = character & 0xFF;
                    std::io::stdout()
                        .lock()
                        .write_all(&[char as u8])
                        .expect("Couldn't write to stdout");
                    let char = character >> 8;
                    if char == 1 {
                        std::io::stdout()
                            .lock()
                            .write_all(&[char as u8])
                            .expect("Couldn't write to stdout");
                    }

                    character =
                        self.memory[self.registers[Register::R0 as usize] as usize + counter];
                    counter += 1;
                }
            }
            _ => {}
        }
    }

    pub fn process_input(&mut self, instruction: u16) {
        let opcode = instruction >> 12;
        match opcode {
            opcode if opcode == Instruction::BR as u16 => self.branch_instruction(instruction),
            opcode if opcode == Instruction::ADD as u16 => self.add_instruction(instruction),
            opcode if opcode == Instruction::LD as u16 => self.load(instruction),
            opcode if opcode == Instruction::ST as u16 => self.store(instruction),
            opcode if opcode == Instruction::JSR as u16 => self.jump_to_subroutine(instruction),
            opcode if opcode == Instruction::AND as u16 => self.and_instruction(instruction),
            opcode if opcode == Instruction::LDR as u16 => self.load_base_offset(instruction),
            opcode if opcode == Instruction::STR as u16 => self.store_base_offset(instruction),
            opcode if opcode == Instruction::NOT as u16 => self.not_instruction(instruction),
            opcode if opcode == Instruction::LDI as u16 => self.load_indirect(instruction),
            opcode if opcode == Instruction::STI as u16 => self.store_indirect(instruction),
            opcode if opcode == Instruction::JMP as u16 => self.jump(instruction),
            opcode if opcode == Instruction::LEA as u16 => self.load_effective_address(instruction),
            opcode if opcode == Instruction::RTI as u16 => (),
            opcode if opcode == Instruction::RES as u16 => (),
            opcode if opcode == Instruction::TRAP as u16 => self.trap(instruction),
            _ => {
                exit(-1);
            }
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

    #[test]
    fn can_store_and_load_from_memory() {
        let mut virtual_machine = LC3VirtualMachine::new();
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
        let mut virtual_machine = LC3VirtualMachine::new();
        let jump_to_position_four = 0b0100100000000100;
        virtual_machine.process_input(jump_to_position_four);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b100);
        let jump_to_register_zero = 0b0100000000000000;
        virtual_machine.process_input(jump_to_register_zero);
        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0);
    }

    #[test]
    fn can_store_and_load_from_memory_with_base_and_offset() {
        let mut virtual_machine = LC3VirtualMachine::new();
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
        let mut virtual_machine = LC3VirtualMachine::new();
        let add_five_to_regiser_zero = 0b0001000000100101;
        virtual_machine.process_input(add_five_to_regiser_zero);
        let unconditionally_jump_to_register_zero_value = 0b1100000000000000;
        virtual_machine.process_input(unconditionally_jump_to_register_zero_value);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_load_effective_address() {
        let mut virtual_machine = LC3VirtualMachine::new();
        let load_effective_address_three_to_register_zero = 0b1110000000000011;
        virtual_machine.process_input(load_effective_address_three_to_register_zero);

        let result = virtual_machine.read_register(super::Register::R0);
        assert_eq!(result, 0b11);
    }
}
