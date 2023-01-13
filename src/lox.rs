use std::process::exit;

use anyhow::{Context, Result};

use crate::{interpreter::Interpreter, parser::Parser, scanner};

pub fn run_file(path: &std::path::PathBuf) -> Result<()> {
    let mut lox = Lox::default();
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Could not read file `{:?}`", path))?;
    lox.run(&source);
    if lox.had_error {
        exit(65);
    }
    Ok(())
}

pub fn run_prompt(
    reader: impl std::io::BufRead,
    mut writer: impl std::io::Write,
) -> std::io::Result<()> {
    let mut lox = Lox::default();

    const WELCOME_MESSAGES: &str = "Welcome to Lox prompt";
    const PROMPT: &str = ">>>";

    write!(writer, "{}\n{} ", WELCOME_MESSAGES, PROMPT)?;
    writer.flush()?;

    for line in reader.lines() {
        let line = line?;
        if !line.is_empty() {
            lox.run(&line);
            lox.reset_error();
        }
        write!(writer, "{} ", PROMPT)?;
        writer.flush()?;
    }

    Ok(())
}

#[derive(Debug, Default)]
pub(crate) struct Lox {
    had_error: bool,
}

impl Lox {
    fn reset_error(&mut self) {
        self.had_error = false
    }

    fn run(&mut self, source: &str) {
        let scan_result = scanner::scan(source);

        self.had_error = scan_result.had_error();
        // deal with error
        for error in scan_result.errors() {
            println!("{}", error);
        }

        let mut parser = Parser::from(scan_result);
        let mut interpreter = Interpreter::default();

        match parser.parse() {
            Ok(e) => match interpreter.expr(&e) {
                Ok(object) => println!("{}", object.to_string()),
                Err(e) => {
                    println!("{}", e)
                }
            },
            Err(e) => {
                println!("{}", e)
            }
        }
    }
}
