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
