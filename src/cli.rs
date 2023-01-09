use clap::Parser;

use crate::lox;

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    file_path: Option<std::path::PathBuf>,
}

impl Cli {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.file_path {
            Some(path) => lox::run_file(path)?,
            None => lox::run_prompt(std::io::stdin().lock(), std::io::stdout().lock())?,
        }
        Ok(())
    }
}

pub fn exec() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    cli.run()?;
    Ok(())
}
