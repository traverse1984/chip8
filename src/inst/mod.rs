mod inst;
mod macros;
mod operands;

pub mod bytecode;

pub use inst::{ops, Instruction, Opcode};
pub use operands::Operands;
