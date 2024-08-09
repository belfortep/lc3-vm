pub enum Instruction {
    BR = 0,
    ADD,
    LD,
    ST,
    JSR,
    AND,
    NOT = 0b1001,
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
}

impl LC3VirtualMachine {
    pub fn new() -> Self {
        Self {
            registers: vec![0; Register::AmmountOfRegisters as usize],
        }
    }

    fn sign_extend(mut value_to_extend: u16, ammount_of_bits: u16) -> u16 {
        if (value_to_extend >> (ammount_of_bits - 1) & 0b1) == 1 {
            value_to_extend |= 0xFFFF << ammount_of_bits;
        }
        value_to_extend
    }

    pub fn process_input(&mut self, instruction: u16) {
        let opcode = instruction >> 12;
        match opcode {
            opcode if opcode == Instruction::ADD as u16 => {
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
            }
            opcode if opcode == Instruction::AND as u16 => {
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
            }

            opcode if opcode == Instruction::NOT as u16 => {
                let destination_register = (instruction >> 9) & 0b111;
                let source_register = (instruction >> 6) & 0b111;

                self.registers[destination_register as usize] =
                    !self.registers[source_register as usize];
            }
            _ => {}
        }
    }
    pub fn read_register(&self, register: Register) -> u16 {
        self.registers[register as usize]
    }
}

#[cfg(test)]
pub mod test {
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
}
