use std::{io::StdoutLock, process::exit};

use anyhow::{Context, Result};

use crate::{
    error::reporter::{ErrorReporter, Reporter},
    interpreter::Interpreter,
    object::Object,
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
    source::SourcePos,
};

pub fn run_file(path: &std::path::PathBuf) -> Result<()> {
    let mut lox = Lox::default();
    let source =
        std::fs::read_to_string(path).with_context(|| format!("Could not read file `{path:?}`"))?;
    lox.run(&source)?;
    if lox.had_scan_error || lox.had_parse_error || lox.had_resolve_error {
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

    write!(writer, "{WELCOME_MESSAGES}\n{PROMPT} ")?;
    writer.flush()?;

    for line in reader.lines() {
        let mut line = line?;

        if !line.ends_with(';') {
            line.push(';');
        }

        let object = if !line.is_empty() {
            let object = lox.run(&line)?;
            lox.reset_error();
            object
        } else {
            Object::Null
        };

        match object {
            Object::Null => (),
            object => writeln!(writer, "{}", object.to_string())?,
        };

        write!(writer, "{PROMPT} ")?;
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
    had_resolve_error: bool,
    had_runtime_error: bool,
}

impl<W> Lox<W>
where
    W: std::io::Write,
{
    fn reset_error(&mut self) {
        self.had_scan_error = false;
        self.had_parse_error = false;
        self.had_resolve_error = false;
        self.had_runtime_error = false;
        self.interpreter.flush_error();
    }

    fn run(&mut self, source: &str) -> Result<Object, std::io::Error> {
        let source_pos = SourcePos::new(source);
        let reporter = Reporter::new(&source_pos);

        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        self.had_scan_error = scanner.had_error();

        if self.had_scan_error {
            self.interpreter.write(&scanner.error_msg(&reporter))?;
            return Ok(Object::Null);
        }

        let mut parser = Parser::from(&scanner);
        let statements = parser.parse();
        self.had_parse_error = parser.had_error();
        if self.had_parse_error {
            self.interpreter.write(&parser.error_msg(&reporter))?;
            return Ok(Object::Null);
        }

        let mut resolver = Resolver::new(&mut self.interpreter);
        resolver.resolve(&statements);
        self.had_resolve_error = resolver.had_error();
        if self.had_resolve_error {
            let error_msg = resolver.error_msg(&reporter);
            self.interpreter.write(&error_msg)?;
            return Ok(Object::Null);
        }

        let object = self.interpreter.interpret(&statements);
        if self.interpreter.had_error() {
            let error_msg = self.interpreter.error_msg(&reporter);
            self.interpreter.write(&error_msg)?;
        }

        Ok(object)
    }
}

impl<'a> Default for Lox<StdoutLock<'a>> {
    fn default() -> Self {
        Self {
            interpreter: Interpreter::default(),
            had_parse_error: false,
            had_runtime_error: false,
            had_scan_error: false,
            had_resolve_error: false,
        }
    }
}
