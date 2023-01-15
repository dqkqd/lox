use std::{io::StdoutLock, process::exit};

use anyhow::{Context, Result};

use crate::{interpreter::Interpreter, parser::Parser, scanner::Scanner};

pub fn run_file(path: &std::path::PathBuf) -> Result<()> {
    let mut lox = Lox::default();
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Could not read file `{:?}`", path))?;
    lox.run(&source)?;
    if lox.had_scan_error || lox.had_parse_error {
        exit(65);
    } else if lox.had_runtime_error {
        exit(70);
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
            lox.run(&line)?;
            lox.reset_error();
        }
        write!(writer, "{} ", PROMPT)?;
        writer.flush()?;
    }

    Ok(())
}

pub(crate) struct Lox<W>
where
    W: std::io::Write,
{
    interpreter: Interpreter<W>,
    had_scan_error: bool,
    had_parse_error: bool,
    had_runtime_error: bool,
}

impl<W> Lox<W>
where
    W: std::io::Write,
{
    fn reset_error(&mut self) {
        self.had_scan_error = false;
        self.had_parse_error = false;
        self.had_runtime_error = false;
    }

    fn run(&mut self, source: &str) -> Result<(), std::io::Error> {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        self.had_scan_error = scanner.had_error();

        if self.had_scan_error {
            for error in scanner.errors() {
                self.interpreter.write(&error.to_string())?
            }
            return Ok(());
        }

        let mut parser = Parser::from(&scanner);
        let statements = parser.parse();
        self.had_parse_error = parser.had_error();
        if self.had_parse_error {
            for error in parser.errors() {
                self.interpreter.write(&error.to_string())?
            }
            return Ok(());
        }

        let result = self.interpreter.interpret(&statements);
        if result.is_err() {
            self.had_runtime_error = true;
            self.interpreter.write(&result.unwrap_err().to_string())?;
        }

        Ok(())
    }
}

impl<'a> Default for Lox<StdoutLock<'a>> {
    fn default() -> Self {
        Self {
            interpreter: Interpreter::default(),
            had_parse_error: false,
            had_runtime_error: false,
            had_scan_error: false,
        }
    }
}
