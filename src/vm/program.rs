use core::mem;

pub struct Program<'a> {
    program: &'a [u8],
}

impl<'a> Program<'a> {
    pub fn as_bytes(&self) -> &[u8] {
        self.program
    }
}

impl<'a> From<&'a [u8]> for Program<'a> {
    fn from(program: &'a [u8]) -> Self {
        Self { program }
    }
}

impl<'a> From<&'a [u16]> for Program<'a> {
    fn from(program: &'a [u16]) -> Self {
        Self {
            program: unsafe { core::mem::transmute::<&[u16], &[u8]>(program) },
        }
    }
}
