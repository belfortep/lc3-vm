use std::io::{Read, Write};

struct Instruction;
impl Instruction {
    pub const BR: u16 = 0;
    pub const ADD: u16 = 1;
    pub const LD: u16 = 2;
    pub const ST: u16 = 3;
    pub const JSR: u16 = 4;
    pub const AND: u16 = 5;
    pub const LDR: u16 = 6;
    pub const STR: u16 = 7;
    pub const RTI: u16 = 8;
    pub const NOT: u16 = 9;
    pub const LDI: u16 = 10;
    pub const STI: u16 = 11;
    pub const JMP: u16 = 12;
    pub const RES: u16 = 13;
    pub const LEA: u16 = 14;
    pub const TRAP: u16 = 15;
}

struct Trap;
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

struct Register;

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
            let inmediate_value = Self::sign_extend(instruction & 0b11111, 5);
            let new_register_value = inmediate_value + self.registers[source_one_register as usize];
            self.registers[destination_register as usize] = new_register_value;
        } else {
            let source_two_register = instruction & 0b111;
            let new_register_value = self.registers[source_one_register as usize]
                + self.registers[source_two_register as usize];
            self.registers[destination_register as usize] = new_register_value;
        }

        self.update_flags(destination_register);
    }

    fn and_instruction(&mut self, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let source_one_register = (instruction >> 6) & 0b111;
        let inmediate_return_flag = (instruction >> 5) & 0b1;

        if inmediate_return_flag == 1 {
            let inmediate_value = Self::sign_extend(instruction & 0b11111, 5);
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
        let conditions_flag = (instruction >> 9) & 0b111;

        if (conditions_flag & self.registers[Register::CONDITION_FLAG]) != 0 {
            let new_programm_counter_value =
                self.registers[Register::PROGRAM_COUNTER] + programm_counter_offset;
            self.registers[Register::PROGRAM_COUNTER] = new_programm_counter_value;
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

        let result_value =
            self.memory_read(self.registers[Register::PROGRAM_COUNTER] + programm_counter_offset);

        self.registers[destination_register as usize] = result_value;
        self.update_flags(destination_register);
    }

    fn load_indirect(&mut self, instruction: u16) {
        let destination_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);

        let memory_address =
            self.memory_read(self.registers[Register::PROGRAM_COUNTER] + programm_counter_offset);
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
            self.registers[Register::PROGRAM_COUNTER] + programm_counter_offset;

        self.update_flags(destination_register);
    }

    fn jump(&mut self, instruction: u16) {
        let base_register = (instruction >> 6) & 0b111;
        self.registers[Register::PROGRAM_COUNTER] = self.registers[base_register as usize];
    }

    fn jump_to_subroutine(&mut self, instruction: u16) {
        self.registers[Register::R7] = self.registers[Register::PROGRAM_COUNTER];

        let offset_flag = (instruction >> 11) & 0b1;

        if offset_flag == 1 {
            let programm_counter_offset = Self::sign_extend(instruction & 0b11111111111, 11);
            let new_programm_counter_value =
                self.registers[Register::PROGRAM_COUNTER] + programm_counter_offset;

            self.registers[Register::PROGRAM_COUNTER] = new_programm_counter_value;
        } else {
            let base_register = (instruction >> 6) & 0b111;
            self.registers[Register::PROGRAM_COUNTER] = self.registers[base_register as usize];
        }
    }

    fn store(&mut self, instruction: u16) {
        let source_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let value_to_write = self.registers[source_register as usize];
        let memory_address = self.registers[Register::PROGRAM_COUNTER] + programm_counter_offset;
        self.memory_write(memory_address, value_to_write);
    }

    fn store_indirect(&mut self, instruction: u16) {
        let source_register = (instruction >> 9) & 0b111;
        let programm_counter_offset = Self::sign_extend(instruction & 0b111111111, 9);
        let value_to_write = self.registers[source_register as usize];

        let memory_address =
            self.memory_read(self.registers[Register::PROGRAM_COUNTER] + programm_counter_offset);
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
        self.registers[Register::R7] = self.registers[Register::PROGRAM_COUNTER];

        let trap = instruction & 0b11111111;

        match trap {
            Trap::GETC => {
                let mut buffer = [0; 1];
                std::io::stdin()
                    .read_exact(&mut buffer)
                    .expect("Couldn't read from stdin");
                self.registers[Register::R0] = buffer[0] as u16;
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
                self.registers[Register::R0] = char;
            }
            Trap::OUT => {
                print!("{}", (self.registers[Register::R0] as u8) as char);
            }
            Trap::PUTS => {
                let mut read_index = self.registers[Register::R0];
                let mut char = self.memory_read(read_index);
                while char != 0 {
                    print!("{}", (char as u8) as char);
                    read_index += 1;
                    char = self.memory_read(read_index);
                }
                std::io::stdout().flush().expect("Couldn't flush");
            }
            Trap::PUTSP => {
                let mut read_index = self.registers[Register::R0];
                let mut char = self.memory_read(read_index);
                while char != 0 {
                    let first_char = char & 0b11111111;
                    print!("{}", (first_char as u8) as char);
                    let second_char = char >> 8;
                    if second_char != 0 {
                        print!("{}", (second_char as u8) as char);
                    }
                    read_index += 1;
                    char = self.memory_read(read_index);
                }
                std::io::stdout().flush().expect("Couldn't flush");
            }

            _ => panic!("Wrong trap directive"),
        }
    }

    pub fn process_input(&mut self, instruction: u16) {
        let opcode = instruction >> 12;
        match opcode {
            Instruction::BR => self.branch_instruction(instruction),
            Instruction::ADD => self.add_instruction(instruction),
            Instruction::LD => self.load(instruction),
            Instruction::ST => self.store(instruction),
            Instruction::JSR => self.jump_to_subroutine(instruction),
            Instruction::AND => self.and_instruction(instruction),
            Instruction::LDR => self.load_base_offset(instruction),
            Instruction::STR => self.store_base_offset(instruction),
            Instruction::NOT => self.not_instruction(instruction),
            Instruction::LDI => self.load_indirect(instruction),
            Instruction::STI => self.store_indirect(instruction),
            Instruction::JMP => self.jump(instruction),
            Instruction::LEA => self.load_effective_address(instruction),
            Instruction::TRAP => self.trap(instruction),
            Instruction::RTI => panic!("This opcode is not supported"),
            Instruction::RES => panic!("This opcode is not supported"),
            _ => panic!("Wrong opcode received"),
        }
    }
    pub fn read_register(&self, register: usize) -> u16 {
        if register > self.registers.len() {
            return 0;
        }
        self.registers[register]
    }

    fn update_flags(&mut self, register: u16) {
        let condition_flag_register = Register::CONDITION_FLAG;
        let register = register as usize;
        if self.registers[register] == 0 {
            self.registers[condition_flag_register] = Flags::ZERO;
        } else if (self.registers[register] >> 15) != 0 {
            self.registers[condition_flag_register] = Flags::NEGATIVE;
        } else {
            self.registers[condition_flag_register] = Flags::POSITIVE;
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
