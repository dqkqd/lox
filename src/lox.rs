use std::process::exit;

use anyhow::{Context, Result};

use crate::{ast_printer::AstPrinter, parser::Parser, scanner};

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
        lox.run(&line);
        lox.reset_error();
        write!(writer, "> ")?;
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
            dbg!(error);
        }

        let mut parser = Parser::from(scan_result);
        let result = parser.parse();
        //dbg!(&result.unwrap());

        let mut printer = AstPrinter::default();
        printer.print_expr(&result.unwrap());
    }
}
