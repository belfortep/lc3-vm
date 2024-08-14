use super::{lc3_virtual_machine::LC3VirtualMachine, register::Register, trap::Trap};

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

pub fn add(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    source_one_register: Register,
    source_two_register: Register,
) {
    let new_register_value = virtual_machine
        .read_register(source_one_register)
        .wrapping_add(virtual_machine.read_register(source_two_register));

    virtual_machine.update_register(destination_register, new_register_value);

    virtual_machine.update_flags(destination_register);
}

pub fn add_inmediate(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    source_one_register: Register,
    inmediate_value: u16,
) {
    let new_register_value = virtual_machine
        .read_register(source_one_register)
        .wrapping_add(inmediate_value);

    virtual_machine.update_register(destination_register, new_register_value);
    virtual_machine.update_flags(destination_register);
}

pub fn load(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    program_counter_offset: u16,
) {
    let new_register_value = virtual_machine.memory_read(
        virtual_machine
            .read_register(Register::ProgramCounter)
            .wrapping_add(program_counter_offset),
    );

    virtual_machine.update_register(destination_register, new_register_value);
    virtual_machine.update_flags(destination_register);
}

pub fn load_indirect(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    program_counter_offset: u16,
) {
    let memory_address = virtual_machine.memory_read(
        virtual_machine
            .read_register(Register::ProgramCounter)
            .wrapping_add(program_counter_offset),
    );
    let new_register_value = virtual_machine.memory_read(memory_address);
    virtual_machine.update_register(destination_register, new_register_value);

    virtual_machine.update_flags(destination_register);
}

pub fn not(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    source_register: Register,
) {
    let new_register_value = !virtual_machine.read_register(source_register);
    virtual_machine.update_register(destination_register, new_register_value);
}

pub fn branch(
    virtual_machine: &mut LC3VirtualMachine,
    program_counter_offset: u16,
    conditions_flag: u16,
) {
    if (conditions_flag & virtual_machine.read_register(Register::ConditionFlag)) != 0 {
        let new_register_value = virtual_machine
            .read_register(Register::ProgramCounter)
            .wrapping_add(program_counter_offset);
        virtual_machine.update_register(Register::ProgramCounter, new_register_value);
    }
}

pub fn and(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    source_one_register: Register,
    source_two_register: Register,
) {
    let new_register_value = virtual_machine.read_register(source_one_register)
        & virtual_machine.read_register(source_two_register);
    virtual_machine.update_register(destination_register, new_register_value);

    virtual_machine.update_flags(destination_register);
}

pub fn and_inmediate(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    source_one_register: Register,
    inmediate_value: u16,
) {
    let new_register_value = virtual_machine.read_register(source_one_register) & inmediate_value;
    virtual_machine.update_register(destination_register, new_register_value);
}

pub fn trap_instruction(virtual_machine: &mut LC3VirtualMachine, trap: Trap) {
    virtual_machine.update_register(
        Register::R7,
        virtual_machine.read_register(Register::ProgramCounter),
    );

    trap.execute_trap(virtual_machine);
}

pub fn load_base_offset(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    base_register: Register,
    offset: u16,
) {
    let register_value = virtual_machine.read_register(base_register);
    let new_register_value = virtual_machine.memory_read(register_value.wrapping_add(offset));
    virtual_machine.update_register(destination_register, new_register_value);

    virtual_machine.update_flags(destination_register);
}

pub fn load_effective_address(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    program_counter_offset: u16,
) {
    let new_register_value = virtual_machine
        .read_register(Register::ProgramCounter)
        .wrapping_add(program_counter_offset);
    virtual_machine.update_register(destination_register, new_register_value);

    virtual_machine.update_flags(destination_register);
}

pub fn jump(virtual_machine: &mut LC3VirtualMachine, base_register: Register) {
    let new_register_value = virtual_machine.read_register(base_register);
    virtual_machine.update_register(Register::ProgramCounter, new_register_value);
}

pub fn jump_to_subroutine(virtual_machine: &mut LC3VirtualMachine, base_register: Register) {
    virtual_machine.update_register(
        Register::R7,
        virtual_machine.read_register(Register::ProgramCounter),
    );

    virtual_machine.update_register(
        Register::ProgramCounter,
        virtual_machine.read_register(base_register),
    );
}

pub fn jump_to_subroutine_with_offset(
    virtual_machine: &mut LC3VirtualMachine,
    program_counter_offset: u16,
) {
    virtual_machine.update_register(
        Register::R7,
        virtual_machine.read_register(Register::ProgramCounter),
    );

    let new_register_value = virtual_machine
        .read_register(Register::ProgramCounter)
        .wrapping_add(program_counter_offset);

    virtual_machine.update_register(Register::ProgramCounter, new_register_value);
}

pub fn store(
    virtual_machine: &mut LC3VirtualMachine,
    source_register: Register,
    program_counter_offset: u16,
) {
    let value_to_write = virtual_machine.read_register(source_register);
    let memory_address = virtual_machine
        .read_register(Register::ProgramCounter)
        .wrapping_add(program_counter_offset);
    virtual_machine.memory_write(memory_address, value_to_write);
}

pub fn store_indirect(
    virtual_machine: &mut LC3VirtualMachine,
    source_register: Register,
    program_counter_offset: u16,
) {
    let value_to_write = virtual_machine.read_register(source_register);

    let memory_address = virtual_machine.memory_read(
        virtual_machine
            .read_register(Register::ProgramCounter)
            .wrapping_add(program_counter_offset),
    );

    let destination_address = virtual_machine.memory_read(memory_address);

    virtual_machine.memory_write(destination_address, value_to_write);
}

pub fn store_base_offset(
    virtual_machine: &mut LC3VirtualMachine,
    source_register: Register,
    base_register: Register,
    offset: u16,
) {
    let value_to_write = virtual_machine.read_register(source_register);
    let base_register_address = virtual_machine.read_register(base_register);

    virtual_machine.memory_write(base_register_address.wrapping_add(offset), value_to_write)
}

#[cfg(test)]
pub mod test {
    use crate::virtual_machine::register::Flag;

    use super::LC3VirtualMachine;

    #[test]
    fn can_add_two_numbers_in_same_register() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_one_to_register_zero = 0b0001_000_000_1_00001;
        virtual_machine.decode_instruction(add_one_to_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 1);
    }

    #[test]
    fn can_add_two_numbers_in_different_registers() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_one_to_register_zero = 0b0001_000_000_1_00001;
        let add_one_to_register_one = 0b0001_001_001_1_00001;
        virtual_machine.decode_instruction(add_one_to_register_zero);
        virtual_machine.decode_instruction(add_one_to_register_one);
        let add_register_zero_and_one_in_register_two = 0b0001_010_000_0_00001;
        virtual_machine.decode_instruction(add_register_zero_and_one_in_register_two);
        let result = virtual_machine.read_register(super::Register::R2);

        assert_eq!(result, 2);
    }

    #[test]
    fn can_and_two_numbers_in_same_register() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_max_inmediate_value_to_register_zero = 0b0001_000_000_1_11111;
        let and_five_to_register_zero = 0b0101_000_000_1_00101;
        virtual_machine.decode_instruction(add_max_inmediate_value_to_register_zero);
        virtual_machine.decode_instruction(and_five_to_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 0b00101);
    }

    #[test]
    fn can_and_two_numbers_in_differents_registers() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_max_inmediate_value_to_register_zero = 0b0001_000_000_111111;
        let add_five_to_register_one = 0b0001_001_001_1_00101;
        virtual_machine.decode_instruction(add_max_inmediate_value_to_register_zero);
        virtual_machine.decode_instruction(add_five_to_register_one);
        let and_register_zero_and_one_in_register_two = 0b0101_010_000_0_00001;
        virtual_machine.decode_instruction(and_register_zero_and_one_in_register_two);
        let result = virtual_machine.read_register(super::Register::R2);

        assert_eq!(result, 0b00101);
    }

    #[test]
    fn can_negate_the_values_of_two_registers() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_register_zero = 0b0001_000_000_1_00101;
        virtual_machine.decode_instruction(add_five_to_register_zero);
        let negate_register_zero = 0b1001_000_000_1_11111;
        virtual_machine.decode_instruction(negate_register_zero);
        let result = virtual_machine.read_register(super::Register::R0);

        assert_eq!(result, 0b1111111111111010);
    }

    #[test]
    fn can_branch_if_positive_flag() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_one_to_register_zero = 0b0001_000_000_1_00001;
        virtual_machine.decode_instruction(add_one_to_register_zero);
        let branch_positive_flag = 0b0000_0_0_1_000000010;
        virtual_machine.decode_instruction(branch_positive_flag);

        let result = virtual_machine.read_register(super::Register::ConditionFlag);
        assert_eq!(result, Flag::POSITIVE as u16);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_branch_if_negative_flag() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_negative_number_to_register_zero = 0b0001_000_000_110001;
        virtual_machine.decode_instruction(add_negative_number_to_register_zero);
        let branch_negative_flag = 0b0000_1_0_0_000000010;
        virtual_machine.decode_instruction(branch_negative_flag);

        let result = virtual_machine.read_register(super::Register::ConditionFlag);
        assert_eq!(result, Flag::NEGATIVE as u16);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_branch_if_zero_flag() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_zero_to_register_zero = 0b0001_000_000_100000;
        virtual_machine.decode_instruction(add_zero_to_register_zero);
        let branch_zero_flag = 0b0000_0_1_0_000000010;
        virtual_machine.decode_instruction(branch_zero_flag);

        let result = virtual_machine.read_register(super::Register::ConditionFlag);
        assert_eq!(result, Flag::ZERO as u16);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b10);
    }

    #[test]
    fn can_store_and_load_from_memory() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_register_zero = 0b0001_000_000_1_00101;
        virtual_machine.decode_instruction(add_five_to_register_zero);
        let store_register_zero_value_to_memory = 0b0011_000_000000001;
        virtual_machine.decode_instruction(store_register_zero_value_to_memory);
        let load_value_from_memory_to_register_one = 0b0010_001_000000001;
        virtual_machine.decode_instruction(load_value_from_memory_to_register_one);

        let result = virtual_machine.read_register(super::Register::R1);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_jump_to_subroutine_and_return_with_register_seven() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let jump_to_position_four = 0b0100_1_00000000100;
        virtual_machine.decode_instruction(jump_to_position_four);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b100);
        let jump_to_register_zero = 0b0100_0_00_000_000000;
        virtual_machine.decode_instruction(jump_to_register_zero);
        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0);
    }

    #[test]
    fn can_store_and_load_from_memory_with_base_and_offset() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_register_zero = 0b0001_000_000_1_00101;
        virtual_machine.decode_instruction(add_five_to_register_zero);
        let add_five_to_register_one = 0b0001_001_001_1_00101;
        virtual_machine.decode_instruction(add_five_to_register_one);

        let store_register_zero_value_to_memory_from_register_one_and_one_offset =
            0b0111_000_001_000001;
        virtual_machine.decode_instruction(
            store_register_zero_value_to_memory_from_register_one_and_one_offset,
        );
        let load_value_from_memory_from_register_one_and_one_offset_to_register_two =
            0b0110_010_001_000001;
        virtual_machine.decode_instruction(
            load_value_from_memory_from_register_one_and_one_offset_to_register_two,
        );

        let result = virtual_machine.read_register(super::Register::R2);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_unconditionally_jumps() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let add_five_to_register_zero = 0b0001_000_000_1_00101;
        virtual_machine.decode_instruction(add_five_to_register_zero);
        let unconditionally_jump_to_register_zero_value = 0b1100_000_000_000000;
        virtual_machine.decode_instruction(unconditionally_jump_to_register_zero_value);

        let result = virtual_machine.read_register(super::Register::ProgramCounter);
        assert_eq!(result, 0b101);
    }

    #[test]
    fn can_load_effective_address() {
        let mut virtual_machine = LC3VirtualMachine::new(0);
        let load_effective_address_three_to_register_zero = 0b1110_000_000000011;
        virtual_machine.decode_instruction(load_effective_address_three_to_register_zero);

        let result = virtual_machine.read_register(super::Register::R0);
        assert_eq!(result, 0b11);
    }
}
