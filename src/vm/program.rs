use core::slice;

use crate::vm::{mem::Ram, status::Status};

pub struct Program<'a> {
    program: &'a [u8],
    data: Option<(u16, &'a [u8])>,
}

impl<'a> Program<'a> {
    pub fn copy_to(&self, ram: &mut Ram) -> Status {
        ram.load(0x200, self.program)?;
        if let Some((offset, data)) = self.data {
            ram.load(offset, data)?;
        }

        Ok(())
    }

    pub fn data(&mut self, offset: u16, data: &'a [u8]) {
        self.data = Some((offset, data));
    }
}

impl<'a> From<&'a [u8]> for Program<'a> {
    fn from(program: &'a [u8]) -> Self {
        Self {
            program,
            data: None,
        }
    }
}

impl<'a> From<&'a mut [u16]> for Program<'a> {
    fn from(program: &'a mut [u16]) -> Self {
        if cfg!(target_endian = "little") {
            program.iter_mut().for_each(|word| *word = word.to_be());
        }

        let len = program.len() * 2;
        let ptr: *const u8 = program.as_ptr().cast();

        Self {
            program: unsafe { slice::from_raw_parts(ptr, len) },
            data: None,
        }
    }
}

mod tests {
    use super::Program;
    use crate::vm::mem::Ram;

    #[test]
    fn from_u8() {
        let program = &[1u8, 2, 3, 4][..];
        let program: Program = program.into();

        assert_eq!(program.program, &[1, 2, 3, 4]);
    }

    #[test]
    fn from_u16() {
        let program = &mut [0x0102u16, 0x0304][..];
        let program: Program = program.into();

        assert_eq!(program.program, &[1, 2, 3, 4]);
    }

    #[test]
    fn copy_to() {
        let program = &mut [0x0102u16, 0x0304][..];
        let mut program: Program = program.into();

        program.data(0x300, &[5, 6, 7, 8]);

        let mut ram = Ram::new();
        program.copy_to(&mut ram).unwrap();

        assert_eq!(ram.read_bytes(0x200, 4).unwrap(), &[1, 2, 3, 4]);
        assert_eq!(ram.read_bytes(0x300, 4).unwrap(), &[5, 6, 7, 8]);
    }
}
