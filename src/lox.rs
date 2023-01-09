use std::process::exit;

use anyhow::{Context, Result};

use crate::scanner;

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

    write!(writer, "Welcome to Lox prompt\n> ")?;
    writer.flush()?;

    for line in reader.lines() {
        let line = line?;
        write!(writer, "> ")?;
        writer.flush()?;
        lox.run(&line);
        lox.reset_error();
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
        let result = scanner::scan(source);
        for token in result.tokens() {
            dbg!(token);
        }

        self.had_error = result.had_error();
        for error in result.errors() {
            dbg!(error);
        }
    }
}
