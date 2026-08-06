use crate::CompileError;

pub const MAX_CODE_SIZE: usize = 4096;

#[derive(Debug, Default)]
pub struct Emitter {
    code: Vec<u8>,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            code: Vec::with_capacity(MAX_CODE_SIZE),
        }
    }

    pub fn byte(&mut self, value: u8) -> Result<(), CompileError> {
        if self.code.len() >= MAX_CODE_SIZE {
            return Err(CompileError::CodeTooLarge);
        }
        self.code.push(value);
        Ok(())
    }

    pub fn bytes(&mut self, values: &[u8]) -> Result<(), CompileError> {
        for &value in values {
            self.byte(value)?;
        }
        Ok(())
    }

    pub fn imm64(&mut self, value: i64) -> Result<(), CompileError> {
        let bits = value as u64;
        for shift in (0..64).step_by(8) {
            self.byte((bits >> shift) as u8)?;
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<Vec<u8>, CompileError> {
        self.byte(0x58)?; // pop rax
        self.byte(0xc3)?; // ret
        Ok(self.code)
    }
}
