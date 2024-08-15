pub const AMOUNT_OF_REGISTERS: usize = 10;

pub enum Flag {
    POSITIVE = 1 << 0,
    ZERO = 1 << 1,
    NEGATIVE = 1 << 2,
}

#[derive(Clone, Copy)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    ProgramCounter,
    ConditionFlag,
}

pub struct Registers {
    registers: Vec<u16>,
}

impl Registers {
    pub fn new(program_counter_start: u16) -> Self {
        let mut registers = vec![0; AMOUNT_OF_REGISTERS];
        registers[Register::ProgramCounter as usize] = program_counter_start;
        Self { registers }
    }

    pub fn read_register(&self, source_register: Register) -> u16 {
        self.registers[source_register as usize]
    }

    pub fn update_register(&mut self, destination_register: Register, new_register_value: u16) {
        self.registers[destination_register as usize] = new_register_value;
    }

    pub fn update_flags(&mut self, register: Register) {
        if self.read_register(register) == 0 {
            self.update_register(Register::ConditionFlag, Flag::ZERO as u16)
        } else if (self.read_register(register) >> 15) != 0 {
            self.update_register(Register::ConditionFlag, Flag::NEGATIVE as u16)
        } else {
            self.update_register(Register::ConditionFlag, Flag::POSITIVE as u16)
        }
    }

    pub fn state_of_registers(&mut self) -> String {
        self.registers
            .iter()
            .fold(String::new(), |mut accumulator, register| {
                accumulator.push_str(format!("{register:#018b}::").as_str());
                accumulator
            })
    }
}

impl From<u16> for Register {
    fn from(value: u16) -> Self {
        match value {
            0 => Register::R0,
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            6 => Register::R6,
            7 => Register::R7,
            8 => Register::ProgramCounter,
            9 => Register::ConditionFlag,
            _ => panic!("Wrong Register"),
        }
    }
}
