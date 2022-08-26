mod instructions;
mod macros;
mod operands;

pub mod bytecode;

pub use instructions::{ops, Instruction, Opcode};
pub use operands::Operands;
