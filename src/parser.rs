use crate::{codegen::Emitter, CompileError};

pub struct Parser<'a> {
    input: &'a [u8],
    position: usize,
    emitter: Emitter,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            position: 0,
            emitter: Emitter::new(),
        }
    }

    pub fn compile(mut self) -> Result<Vec<u8>, CompileError> {
        self.expression()?;
        self.spaces();
        if self.position != self.input.len() {
            return Err(CompileError::UnexpectedCharacter(
                self.input[self.position] as char,
            ));
        }
        self.emitter.finish()
    }

    fn spaces(&mut self) {
        while self
            .input
            .get(self.position)
            .is_some_and(|c| c.is_ascii_whitespace())
        {
            self.position += 1;
        }
    }

    fn take(&mut self, expected: u8) -> bool {
        self.spaces();
        if self.input.get(self.position) == Some(&expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn expression(&mut self) -> Result<(), CompileError> {
        self.product()?;
        loop {
            self.spaces();
            let Some(&op) = self.input.get(self.position) else {
                break;
            };
            if op != b'+' && op != b'-' {
                break;
            }
            self.position += 1;
            self.product()?;
            self.emitter.bytes(&[0x59, 0x58])?; // pop rcx; pop rax
            self.emitter.bytes(if op == b'+' {
                &[0x48, 0x01, 0xc8]
            } else {
                &[0x48, 0x29, 0xc8]
            })?;
            self.emitter.byte(0x50)?; // push rax
        }
        Ok(())
    }

    fn product(&mut self) -> Result<(), CompileError> {
        self.unary()?;
        loop {
            self.spaces();
            let Some(&op) = self.input.get(self.position) else {
                break;
            };
            if op != b'*' && op != b'/' {
                break;
            }
            self.position += 1;
            self.unary()?;
            self.emitter.bytes(&[0x59, 0x58])?; // pop rcx; pop rax
            if op == b'*' {
                self.emitter.bytes(&[0x48, 0x0f, 0xaf, 0xc1])?; // imul rax, rcx
            } else {
                self.emitter.bytes(&[0x48, 0x99, 0x48, 0xf7, 0xf9])?; // cqo; idiv rcx
            }
            self.emitter.byte(0x50)?; // push rax
        }
        Ok(())
    }

    fn unary(&mut self) -> Result<(), CompileError> {
        if self.take(b'-') {
            // Parse the minimum i64 directly. Its positive magnitude (2^63)
            // cannot be represented as an i64 before the negation.
            self.spaces();
            if self
                .input
                .get(self.position)
                .is_some_and(|c| c.is_ascii_digit())
            {
                self.number(true)?;
            } else {
                self.unary()?;
                self.emitter.byte(0x58)?; // pop rax
                self.emitter.bytes(&[0x48, 0xf7, 0xd8])?; // neg rax
                self.emitter.byte(0x50)?; // push rax
            }
        } else if self.take(b'+') {
            self.unary()?;
        } else {
            self.primary()?;
        }
        Ok(())
    }

    fn primary(&mut self) -> Result<(), CompileError> {
        self.spaces();
        if self.take(b'(') {
            self.expression()?;
            if !self.take(b')') {
                return Err(CompileError::ExpectedClosingParen);
            }
            return Ok(());
        }

        match self.input.get(self.position) {
            Some(b'x') => {
                self.position += 1;
                self.emitter.byte(0x57)?; // push rdi
                Ok(())
            }
            Some(c) if c.is_ascii_digit() => self.number(false),
            Some(c) => Err(CompileError::ExpectedValue(*c as char)),
            None => Err(CompileError::ExpectedValueEnd),
        }
    }

    fn number(&mut self, negative: bool) -> Result<(), CompileError> {
        let start = self.position;
        while self
            .input
            .get(self.position)
            .is_some_and(|c| c.is_ascii_digit())
        {
            self.position += 1;
        }
        let text = std::str::from_utf8(&self.input[start..self.position]).unwrap();
        let magnitude = text
            .parse::<u64>()
            .map_err(|_| CompileError::IntegerOutOfRange)?;
        let value = if negative {
            if magnitude == 1_u64 << 63 {
                i64::MIN
            } else {
                i64::try_from(magnitude)
                    .ok()
                    .and_then(|value| value.checked_neg())
                    .ok_or(CompileError::IntegerOutOfRange)?
            }
        } else {
            i64::try_from(magnitude).map_err(|_| CompileError::IntegerOutOfRange)?
        };
        self.emitter.bytes(&[0x48, 0xb8])?; // mov rax, imm64
        self.emitter.imm64(value)?;
        self.emitter.byte(0x50) // push rax
    }
}
