#![cfg_attr(not(target_arch = "x86_64"), allow(dead_code))]

mod codegen;
mod executable_memory;
mod parser;

use std::{error::Error, fmt, io};

#[derive(Debug, PartialEq, Eq)]
pub enum CompileError {
    CodeTooLarge,
    ExpectedClosingParen,
    ExpectedValue(char),
    ExpectedValueEnd,
    IntegerOutOfRange,
    UnexpectedCharacter(char),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CodeTooLarge => write!(f, "generated program is too large"),
            Self::ExpectedClosingParen => write!(f, "expected ')'"),
            Self::ExpectedValue(c) => write!(f, "expected an integer, x, or '(', found '{c}'"),
            Self::ExpectedValueEnd => write!(f, "expected an integer, x, or '('"),
            Self::IntegerOutOfRange => write!(f, "integer is outside the signed 64-bit range"),
            Self::UnexpectedCharacter(c) => write!(f, "unexpected character '{c}'"),
        }
    }
}

impl Error for CompileError {}

pub fn compile(expression: &str) -> Result<Vec<u8>, CompileError> {
    parser::Parser::new(expression).compile()
}

pub fn execute(code: &[u8], x: i64) -> io::Result<i64> {
    let memory = executable_memory::ExecutableMemory::new(code)?;
    // The compiler emits exactly one `extern "C" fn(i64) -> i64` for this target.
    Ok(unsafe { memory.call(x) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(expression: &str, x: i64) -> i64 {
        execute(&compile(expression).unwrap(), x).unwrap()
    }

    #[test]
    fn evaluates_examples() {
        assert_eq!(run("2 + 3 * 4", 0), 14);
        assert_eq!(run("(2 + 3) * 4", 0), 20);
        assert_eq!(run("100 / 7", 0), 14);
        assert_eq!(run("100 / -7", 0), -14);
        assert_eq!(run("x * x + 2 * x + 1", 6), 49);
        assert_eq!(run("-(-x + 4)", 10), 6);
        assert_eq!(run("9223372036854775807", 0), i64::MAX);
    }

    #[test]
    fn reports_parse_errors() {
        assert_eq!(compile("1 +").unwrap_err(), CompileError::ExpectedValueEnd);
        assert_eq!(
            compile("(1 + 2").unwrap_err(),
            CompileError::ExpectedClosingParen
        );
        assert_eq!(
            compile("1abc").unwrap_err(),
            CompileError::UnexpectedCharacter('a')
        );
    }
}
