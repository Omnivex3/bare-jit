#![cfg_attr(
    not(all(target_os = "linux", target_arch = "x86_64")),
    allow(dead_code)
)]

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
compile_error!("bare-jit-rs currently supports only Linux x86-64");

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

#[derive(Debug)]
#[must_use]
pub struct CompiledExpression {
    code: Vec<u8>,
}

pub fn compile(expression: &str) -> Result<CompiledExpression, CompileError> {
    Ok(CompiledExpression {
        code: parser::Parser::new(expression).compile()?,
    })
}

/// Execute a compiler-generated expression with `x` as its argument.
///
/// The unsafe machine-code boundary is kept internal because callers cannot
/// construct a `CompiledExpression` from arbitrary bytes.
pub fn execute(expression: &CompiledExpression, x: i64) -> io::Result<i64> {
    let memory = executable_memory::ExecutableMemory::new(&expression.code)?;
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
        assert_eq!(run(" - 9223372036854775808 ", 0), i64::MIN);
        assert_eq!(run("1 + + 2 * -3", 0), -5);
        assert_eq!(run("9223372036854775807", 0), i64::MAX);
        assert_eq!(run("-9223372036854775808", 0), i64::MIN);
    }

    #[test]
    fn rejects_programs_that_are_too_large() {
        let expression = std::iter::repeat_n("x + ", 1_000)
            .chain(std::iter::once("x"))
            .collect::<String>();
        assert_eq!(
            compile(&expression).unwrap_err(),
            CompileError::CodeTooLarge
        );
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
        assert_eq!(
            compile("9223372036854775808").unwrap_err(),
            CompileError::IntegerOutOfRange
        );
    }
}
