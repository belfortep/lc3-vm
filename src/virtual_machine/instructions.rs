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
    let new_register_value = virtual_machine.read_register(source_one_register)
        + virtual_machine.read_register(source_two_register);

    virtual_machine.update_register(destination_register, new_register_value);

    virtual_machine.update_flags(destination_register);
}

pub fn add_inmediate(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    source_one_register: Register,
    inmediate_value: u16,
) {
    let new_register_value = inmediate_value + virtual_machine.read_register(source_one_register);

    virtual_machine.update_register(destination_register, new_register_value);
    virtual_machine.update_flags(destination_register);
}

pub fn load(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    program_counter_offset: u16,
) {
    let new_register_value = virtual_machine.memory_read(
        virtual_machine.read_register(Register::ProgramCounter) + program_counter_offset,
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
        virtual_machine.read_register(Register::ProgramCounter) + program_counter_offset,
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
        let new_register_value =
            virtual_machine.read_register(Register::ProgramCounter) + program_counter_offset;
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
    let new_register_value = virtual_machine.memory_read(register_value + offset);
    virtual_machine.update_register(destination_register, new_register_value);

    virtual_machine.update_flags(destination_register);
}

pub fn load_effective_address(
    virtual_machine: &mut LC3VirtualMachine,
    destination_register: Register,
    program_counter_offset: u16,
) {
    let new_register_value =
        virtual_machine.read_register(Register::ProgramCounter) + program_counter_offset;
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

    let new_register_value =
        virtual_machine.read_register(Register::ProgramCounter) + program_counter_offset;

    virtual_machine.update_register(Register::ProgramCounter, new_register_value);
}

pub fn store(
    virtual_machine: &mut LC3VirtualMachine,
    source_register: Register,
    program_counter_offset: u16,
) {
    let value_to_write = virtual_machine.read_register(source_register);
    let memory_address =
        virtual_machine.read_register(Register::ProgramCounter) + program_counter_offset;
    virtual_machine.memory_write(memory_address, value_to_write);
}

pub fn store_indirect(
    virtual_machine: &mut LC3VirtualMachine,
    source_register: Register,
    program_counter_offset: u16,
) {
    let value_to_write = virtual_machine.read_register(source_register);

    let memory_address = virtual_machine.memory_read(
        virtual_machine.read_register(Register::ProgramCounter) + program_counter_offset,
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

    virtual_machine.memory_write(offset + base_register_address, value_to_write)
}
