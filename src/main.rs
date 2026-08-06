use std::{env, process};

fn usage(program: &str) {
    eprintln!("usage: {program} 'EXPRESSION' [x]");
    eprintln!("example: {program} '(x + 3) * 7 - 2' 10");
}

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "bare-jit-rs".to_string());
    let Some(expression) = args.next() else {
        usage(&program);
        process::exit(2);
    };
    let x_text = match args.next() {
        Some(value) if args.next().is_none() => value,
        None => "0".to_string(),
        Some(_) => {
            usage(&program);
            process::exit(2);
        }
    };
    let x = match x_text.parse::<i64>() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("invalid x value: {x_text}");
            process::exit(2);
        }
    };

    let code = match bare_jit_rs::compile(&expression) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("compile error: {error}");
            process::exit(1);
        }
    };
    match bare_jit_rs::execute(&code, x) {
        Ok(result) => println!("{result}"),
        Err(error) => {
            eprintln!("could not create executable memory: {error}");
            process::exit(1);
        }
    }
}
