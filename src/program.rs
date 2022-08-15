#[derive(Debug, Clone, Copy)]
pub struct Program<'a> {
    program: &'a [u8],
}

impl<'a> Program<'a> {
    pub fn bytes(&self) -> &'a [u8] {
        &self.program
    }
}

impl<'a> From<&'a [u8]> for Program<'a> {
    fn from(program: &'a [u8]) -> Self {
        Self { program }
    }
}

impl<'a> From<&'a mut [u16]> for Program<'a> {
    fn from(program: &'a mut [u16]) -> Self {
        #[cfg(target_endian = "little")]
        program.iter_mut().for_each(|word| *word = word.to_be());

        let len = program.len() * 2;
        let ptr: *const u8 = program.as_ptr().cast();

        Self {
            program: unsafe { core::slice::from_raw_parts(ptr, len) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Program;

    #[test]
    fn from_u8() {
        let program = &[1u8, 2, 3, 4][..];
        let program: Program = program.into();

        assert_eq!(program.bytes(), &[1, 2, 3, 4]);
    }

    #[test]
    fn from_u16() {
        let program = &mut [0x0102u16, 0x0304][..];
        let program: Program = program.into();

        assert_eq!(program.bytes(), &[1, 2, 3, 4]);
    }
}
