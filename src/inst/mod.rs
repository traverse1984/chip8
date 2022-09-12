mod arch;
mod macros;
mod operands;

pub mod bytecode;

pub use arch::{ops, Instruction, Opcode};
pub use operands::Operands;
